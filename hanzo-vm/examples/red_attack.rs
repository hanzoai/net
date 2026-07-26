//! RED-A adversarial harness for the AIPay pay+infer e2e (Part 1 attack vectors).
//!
//! Drives `execute_payable_call` / the AI precompile directly with hostile inputs
//! to verify BLUE-A's claims:
//!   V1  state readback is genuinely post-commit (value=0 -> slot0==0 from a REAL
//!       read; value=N -> slot0==N; a nonzero->zero transition proves it tracks
//!       committed state, not a constant-zero fallback).
//!   V2  value/gas accounting: with gas_price=0 the only debit is `value`; a
//!       value > balance call must revert with the contract NOT credited.
//!   V3  determinism is from the model (same prompt -> identical bytes; different
//!       prompt -> different bytes), i.e. greedy decode, not a cache echo.
//!   V4  the inference precompile is pure (no state, cannot mutate slot0/replay).
//!
//! Run:
//!   HANZO_FFI_MODELS="zen-nano=gguf:/tmp/zen5-weights/zen-5-flash.gguf" \
//!   HANZO_FFI_TOK_DIR=/tmp/zen-nano-fused \
//!   cargo run -p hanzo-vm --example red_attack --release --features accelerate

mod common;

use hanzo_vm::evm::{aipay_bytecode, execute_payable_call, AiPrecompiles};
use hanzo_vm::precompiles::{PrecompileRegistry, PrecompileResult, ADDR_AI_INFERENCE};
use revm::bytecode::Bytecode;
use revm::context::{Context, Evm, TxEnv};
use revm::context_interface::result::{ExecutionResult, Output};
use revm::database::{CacheDB, EmptyDB};
use revm::handler::instructions::EthInstructions;
use revm::primitives::{hardfork::SpecId, keccak256, Address, Bytes, TxKind, U256};
use revm::state::AccountInfo;
use revm::{Database, ExecuteCommitEvm, MainContext};
use sha2::{Digest, Sha256};

const SPEC: SpecId = SpecId::CANCUN;

const GAS_LIMIT: u64 = 30_000_000;

fn model_field(name: &str) -> [u8; 32] {
    let mut f = [0u8; 32];
    let b = name.as_bytes();
    let n = b.len().min(32);
    f[..n].copy_from_slice(&b[..n]);
    f
}

fn aipay_calldata(model: &str, prompt: &[u8]) -> Vec<u8> {
    let mut c = Vec::with_capacity(32 + prompt.len());
    c.extend_from_slice(&model_field(model));
    c.extend_from_slice(prompt);
    c
}

fn sha_hex(b: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b);
    hex::encode(h.finalize())
}

/// Run one payable call; return Ok((returndata, slot0, caller_after)) on Success,
/// or Err describing the revert/halt (so the caller can assert clean reverts).
fn pay(
    code: &[u8],
    contract: Address,
    caller: Address,
    model: &str,
    prompt: &[u8],
    value: U256,
) -> anyhow::Result<(Vec<u8>, U256, U256)> {
    let (result, slot0, caller_after) = execute_payable_call(
        caller,
        contract,
        code,
        aipay_calldata(model, prompt),
        value,
        GAS_LIMIT,
    )?;
    match result {
        ExecutionResult::Success { output, .. } => {
            let d = match output {
                Output::Call(d) | Output::Create(d, _) => d.to_vec(),
            };
            Ok((d, slot0, caller_after))
        }
        ExecutionResult::Revert { output, gas, .. } => anyhow::bail!(
            "REVERT gas={} slot0={slot0} caller_after={caller_after} data={}",
            gas.used(),
            String::from_utf8_lossy(&output)
        ),
        ExecutionResult::Halt { reason, gas, .. } => {
            anyhow::bail!("HALT {reason:?} gas={} slot0={slot0}", gas.used())
        }
    }
}

fn main() -> anyhow::Result<()> {
    common::load_engine()?;
    let model = std::env::var("INFER_MODEL").unwrap_or_else(|_| "zen-nano".into());
    let contract = Address::from([0x11u8; 20]);
    let caller = Address::from([0x22u8; 20]);
    let code = aipay_bytecode();
    let headroom = U256::from(1_000_000_000_000_000_000u128);

    println!("======================================================================");
    println!("RED-A ADVERSARIAL HARNESS — AIPay pay+infer");
    println!("======================================================================\n");

    // ---- V1: post-commit state readback (value=0 then value=N) ------------
    println!("[V1] STATE READBACK: prove slot0 is a REAL post-commit read");
    let (_d0, slot0_zero, after_zero) = pay(&code, contract, caller, &model, b"Who are you?", U256::ZERO)?;
    println!("  value=0      -> slot0={slot0_zero}  caller_after={after_zero}");
    let (_dn, slot0_big, after_big) = pay(
        &code,
        contract,
        caller,
        &model,
        b"Who are you?",
        U256::from(777_777u64),
    )?;
    println!("  value=777777 -> slot0={slot0_big}  caller_after={after_big}");
    // A constant-zero fallback would read 0 for BOTH. A real committed read sees
    // 0 then 777777. The nonzero->value transition is the discriminator.
    assert_eq!(slot0_zero, U256::ZERO, "value=0 must leave slot0 == 0 (real read)");
    assert_eq!(slot0_big, U256::from(777_777u64), "value=N must write slot0 == N");
    assert_ne!(slot0_zero, slot0_big, "slot0 must TRACK committed state, not be constant");
    // And the value=0 path must not debit the caller (no payment). With
    // execute_payable_call the caller is funded value+headroom; value=0 funds
    // exactly headroom, so caller_after==headroom proves zero debit. For value=N
    // the caller is funded N+headroom and debited N, so caller_after==headroom too.
    assert_eq!(after_zero, headroom, "value=0 must not debit caller (after==headroom)");
    assert_eq!(after_big, headroom, "value=N debit must equal N -> after==headroom (gas_price=0)");
    println!("  PASS: slot0 tracks committed state (0 -> 777777); value=0 debits nothing.\n");

    // ---- V2: value/gas accounting -----------------------------------------
    // V2a: TRUE under-funded caller (value > balance). execute_payable_call
    // auto-funds value+headroom so it CANNOT express this; we drive revm
    // directly with caller.balance = value-1 and assert a clean revert with the
    // contract NOT credited (slot0 == 0, no partial transfer).
    println!("[V2a] VALUE > BALANCE (direct revm, balance=value-1): clean revert, no credit");
    {
        let value = U256::from(1_000_000u64);
        let balance = value - U256::from(1u64); // cannot afford the transfer

        let mut db = CacheDB::new(EmptyDB::default());
        let mut info = AccountInfo::default();
        info.code_hash = keccak256(&code);
        info.code = Some(Bytecode::new_raw(Bytes::copy_from_slice(&code)));
        db.insert_account_info(contract, info);
        let mut caller_info = AccountInfo::default();
        caller_info.balance = balance;
        db.insert_account_info(caller, caller_info);

        let tx = TxEnv::builder()
            .caller(caller)
            .kind(TxKind::Call(contract))
            .data(Bytes::from(aipay_calldata(&model, b"hi")))
            .value(value)
            .gas_limit(GAS_LIMIT)
            .gas_price(0)
            .nonce(0)
            .build_fill();
        let ctx = Context::mainnet().with_db(db).modify_cfg_chained(|c| c.spec = SPEC);
        let mut evm = Evm::new(ctx, EthInstructions::new_mainnet_with_spec(SPEC), AiPrecompiles::new(SPEC));
        let outcome = evm.transact_commit(tx);
        match &outcome {
            Ok(res) => {
                let db = &mut evm.ctx.journaled_state.database;
                let slot0 = db.storage(contract, U256::ZERO).unwrap_or(U256::ZERO);
                let cbal = db.basic(caller).ok().flatten().map(|a| a.balance).unwrap_or_default();
                println!("  committed result={res:?}\n  slot0={slot0}  caller_balance={cbal}");
                anyhow::bail!("value>balance should NOT commit a successful transfer");
            }
            Err(e) => {
                // The state-transition is rejected entirely (lack of funds is a
                // tx-validity error in revm); nothing is committed.
                let estr = format!("{e:?}").to_lowercase();
                println!("  transact_commit rejected (no state change): {e:?}");
                assert!(
                    estr.contains("fund") || estr.contains("balance") || estr.contains("overflow"),
                    "expected an insufficient-funds rejection, got: {e:?}"
                );
            }
        }
    }
    println!("  PASS: under-funded value transfer is rejected; contract not credited.\n");

    // V2b: nonzero gas_price — Blue's "exact debit == value" is gas_price=0
    // specific. With gas_price>0 the debit MUST include gas, proving Blue isn't
    // hiding a value/gas-accounting bug behind gas_price=0.
    println!("[V2b] NONZERO GAS_PRICE (direct revm): debit == value + gas_used*gas_price");
    {
        let value = U256::from(1_000_000u64);
        let gas_price: u128 = 7; // arbitrary nonzero
        let headroom = U256::from(1_000_000_000_000_000_000u128);
        let initial = value + headroom;

        let mut db = CacheDB::new(EmptyDB::default());
        let mut info = AccountInfo::default();
        info.code_hash = keccak256(&code);
        info.code = Some(Bytecode::new_raw(Bytes::copy_from_slice(&code)));
        db.insert_account_info(contract, info);
        let mut caller_info = AccountInfo::default();
        caller_info.balance = initial;
        db.insert_account_info(caller, caller_info);

        let tx = TxEnv::builder()
            .caller(caller)
            .kind(TxKind::Call(contract))
            .data(Bytes::from(aipay_calldata(&model, b"Who are you?")))
            .value(value)
            .gas_limit(GAS_LIMIT)
            .gas_price(gas_price)
            .nonce(0)
            .build_fill();
        let ctx = Context::mainnet().with_db(db).modify_cfg_chained(|c| c.spec = SPEC);
        let mut evm = Evm::new(ctx, EthInstructions::new_mainnet_with_spec(SPEC), AiPrecompiles::new(SPEC));
        let res = evm.transact_commit(tx).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let gas_used = match &res {
            ExecutionResult::Success { gas, .. } => gas.used(),
            other => anyhow::bail!("expected success with nonzero gas_price, got {other:?}"),
        };
        let db = &mut evm.ctx.journaled_state.database;
        let slot0 = db.storage(contract, U256::ZERO).unwrap_or(U256::ZERO);
        let cbal = db.basic(caller).ok().flatten().map(|a| a.balance).unwrap_or_default();
        let debit = initial - cbal;
        let expected_gas_fee = U256::from(gas_used) * U256::from(gas_price);
        let expected_debit = value + expected_gas_fee;
        println!("  gas_used={gas_used} gas_price={gas_price}  slot0={slot0}");
        println!("  debit={debit}  expected(value+gasfee)={expected_debit}");
        assert_eq!(slot0, value, "payment still recorded exactly once");
        assert_eq!(debit, expected_debit, "debit MUST include gas when gas_price>0");
        assert!(debit > value, "with gas_price>0 the debit must exceed `value` alone");
    }
    println!("  PASS: with gas_price>0 the debit = value + gas_fee (Blue's exact-debit claim is gas_price=0-specific, not a bug).\n");

    // ---- V3: determinism is the MODEL's (prompt sensitivity) --------------
    println!("[V3] DETERMINISM PROVENANCE: same prompt == identical; diff prompt != ");
    let v = U256::from(1_000_000u64);
    let (a1, _, _) = pay(&code, contract, caller, &model, b"Who are you?", v)?;
    let (a2, _, _) = pay(&code, contract, caller, &model, b"Who are you?", v)?;
    let (b1, _, _) = pay(&code, contract, caller, &model, b"What is 2+2? Answer with the number only.", v)?;
    println!("  prompt A sha256 = {}  ({} B)", sha_hex(&a1), a1.len());
    println!("  prompt A' sha256= {}  ({} B)", sha_hex(&a2), a2.len());
    println!("  prompt B sha256 = {}  ({} B)", sha_hex(&b1), b1.len());
    assert_eq!(a1, a2, "same prompt must be byte-identical (greedy)");
    assert_ne!(a1, b1, "different prompt MUST change output (proves not a cache echo)");
    println!("  PASS: identical on repeat, differs on a different prompt -> real greedy decode.\n");

    // ---- V4: precompile purity (no state; cannot mutate slot0 / replay) ----
    println!("[V4] PRECOMPILE PURITY: 0x020001 is a pure fn (no EVM state access)");
    // Call the precompile registry DIRECTLY (no EVM, no DB) twice; a pure fn
    // returns identical bytes and has no way to touch contract storage.
    let mut input = vec![0u8; 4];
    input.extend_from_slice(&model_field(&model));
    input.extend_from_slice(b"Who are you?");
    let r1 = PrecompileRegistry::default().call(&ADDR_AI_INFERENCE, &input);
    let r2 = PrecompileRegistry::default().call(&ADDR_AI_INFERENCE, &input);
    match (&r1, &r2) {
        (Some(PrecompileResult::Success { output: o1, gas_used: g1 }),
         Some(PrecompileResult::Success { output: o2, gas_used: g2 })) => {
            println!("  direct call #1 sha256 = {} gas={g1}", sha_hex(o1));
            println!("  direct call #2 sha256 = {} gas={g2}", sha_hex(o2));
            assert_eq!(o1, o2, "pure precompile must be referentially transparent");
            assert_eq!(g1, g2, "gas must be a pure function of the call");
            // The precompile signature is `fn(&[u8]) -> PrecompileResult` — it has
            // NO &mut state, NO DB handle, NO journal. It physically cannot mutate
            // slot0 or replay a payment; the SSTORE is the contract's, pre-CALL.
            println!("  precompile fn type = fn(&[u8]) -> PrecompileResult (no state arg)");
        }
        other => anyhow::bail!("expected two Success results from the precompile, got {other:?}"),
    }
    println!("  PASS: precompile is pure; payment SSTORE is the contract's, not reentrant.\n");

    println!("======================================================================");
    println!("RED-A HARNESS: all vectors exercised.");
    println!("======================================================================");
    Ok(())
}
