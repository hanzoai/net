//! Real EVM execution (revm) with the AI inference/embedding precompiles wired
//! in, as a first-class library feature.
//!
//! [`AiPrecompiles`] is a [`PrecompileProvider`] that serves the stock Ethereum
//! precompiles plus the Hanzo AI precompiles ([`ADDR_AI_INFERENCE`] /
//! [`ADDR_AI_EMBEDDING`]), routing the AI addresses through the production
//! [`PrecompileRegistry`] → engine bridge → native model. [`execute_contract_call`]
//! runs a single transaction against a contract that may `CALL` those addresses.
//!
//! The engine must be installed process-wide first (binaries do this at startup
//! via `hanzo_engine::register_engine`); when none is installed the AI precompile
//! reverts ("fail open").

use std::iter::once;

use crate::precompiles::{PrecompileRegistry, PrecompileResult, ADDR_AI_EMBEDDING, ADDR_AI_INFERENCE};

use revm::bytecode::Bytecode;
use revm::context::{Context, Evm, TxEnv};
use revm::context_interface::result::ExecutionResult;
use revm::context_interface::{Cfg, ContextTr, LocalContextTr};
use revm::database::{CacheDB, EmptyDB};
use revm::handler::instructions::EthInstructions;
use revm::handler::{EthPrecompiles, PrecompileProvider};
use revm::interpreter::{CallInput, CallInputs, Gas, InstructionResult, InterpreterResult};
use revm::primitives::{hardfork::SpecId, keccak256, Address, Bytes, TxKind, U256};
use revm::state::AccountInfo;
use revm::{Database, ExecuteCommitEvm, MainContext};

/// EVM spec the AI VM executes against.
const SPEC: SpecId = SpecId::CANCUN;

/// Custom precompile set: the stock Ethereum precompiles plus Hanzo AI inference
/// (`0x…020001`) and embedding (`0x…020002`), routed through the production
/// [`PrecompileRegistry`].
pub struct AiPrecompiles {
    inner: EthPrecompiles,
    inference: Address,
    embedding: Address,
}

impl AiPrecompiles {
    /// Build the AI precompile set for `spec`.
    pub fn new(spec: SpecId) -> Self {
        Self {
            inner: EthPrecompiles::new(spec),
            inference: Address::from(ADDR_AI_INFERENCE),
            embedding: Address::from(ADDR_AI_EMBEDDING),
        }
    }

    /// Route an AI precompile call through the production registry and shape the
    /// result for the interpreter.
    fn run_ai(&self, addr: &[u8; 20], input: &[u8], gas_limit: u64) -> InterpreterResult {
        let mut result = InterpreterResult {
            result: InstructionResult::Return,
            gas: Gas::new(gas_limit),
            output: Bytes::new(),
        };
        match PrecompileRegistry::default().call(addr, input) {
            Some(PrecompileResult::Success { output, gas_used }) => {
                if result.gas.record_cost(gas_used) {
                    result.output = Bytes::from(output);
                } else {
                    result.result = InstructionResult::PrecompileOOG;
                }
            }
            Some(PrecompileResult::Revert { reason }) => {
                // CONSENSUS PARITY (RED revert-path divergence): geth charges
                // `RequiredGas(input)` BEFORE Run and, on a precompile revert,
                // refunds only the remainder — so a reverting AI precompile
                // consumes `RequiredGas(input)` gas. Charge the SAME canonical
                // input-based amount here; recording zero (as before) would
                // under-charge by `required_gas` on every revert vs the Go
                // co-validator → divergent gas_used → state-root fork.
                if let Some(req) = crate::precompiles::required_gas(addr, input.len()) {
                    // If even the required gas exceeds the limit, it's OOG (geth
                    // would have failed the pre-charge); otherwise revert with the
                    // required gas consumed and the rest refunded.
                    if result.gas.record_cost(req) {
                        result.result = InstructionResult::Revert;
                        result.output = Bytes::from(reason.into_bytes());
                    } else {
                        result.result = InstructionResult::PrecompileOOG;
                    }
                } else {
                    // Non-AI address routed here (shouldn't happen): preserve the
                    // prior refund-all behavior.
                    result.result = InstructionResult::Revert;
                    result.output = Bytes::from(reason.into_bytes());
                }
            }
            Some(PrecompileResult::Error { .. }) | None => {
                result.result = InstructionResult::PrecompileError;
            }
        }
        result
    }
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for AiPrecompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        <EthPrecompiles as PrecompileProvider<CTX>>::set_spec(&mut self.inner, spec)
    }

    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> Result<Option<InterpreterResult>, String> {
        let addr = inputs.bytecode_address;
        let which = if addr == self.inference {
            Some(ADDR_AI_INFERENCE)
        } else if addr == self.embedding {
            Some(ADDR_AI_EMBEDDING)
        } else {
            None
        };
        let Some(ai_addr) = which else {
            return self.inner.run(context, inputs);
        };

        // Resolve call input bytes (mirrors EthPrecompiles::run).
        let input_bytes: Vec<u8> = match &inputs.input {
            CallInput::SharedBuffer(range) => context
                .local()
                .shared_memory_buffer_slice(range.clone())
                .map(|s| s.to_vec())
                .unwrap_or_default(),
            CallInput::Bytes(b) => b.as_ref().to_vec(),
        };
        Ok(Some(self.run_ai(&ai_addr, &input_bytes, inputs.gas_limit)))
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        Box::new(
            self.inner
                .warm_addresses()
                .chain(once(self.inference))
                .chain(once(self.embedding))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    fn contains(&self, address: &Address) -> bool {
        *address == self.inference || *address == self.embedding || self.inner.contains(address)
    }
}

/// Execute a single call transaction against `contract` (whose runtime is
/// `code`), in a fresh in-memory state with the AI precompiles installed.
/// Returns the raw [`ExecutionResult`] — `output.into_data()` on success is the
/// contract's return value.
pub fn execute_contract_call(
    caller: Address,
    contract: Address,
    code: &[u8],
    calldata: Vec<u8>,
    gas_limit: u64,
) -> anyhow::Result<ExecutionResult> {
    let mut db = CacheDB::new(EmptyDB::default());

    let mut info = AccountInfo::default();
    info.code_hash = keccak256(code);
    info.code = Some(Bytecode::new_raw(Bytes::copy_from_slice(code)));
    db.insert_account_info(contract, info);

    let mut caller_info = AccountInfo::default();
    caller_info.balance = U256::from(1_000_000_000_000_000_000u128);
    db.insert_account_info(caller, caller_info);

    let tx = TxEnv::builder()
        .caller(caller)
        .kind(TxKind::Call(contract))
        .data(Bytes::from(calldata))
        .gas_limit(gas_limit)
        .gas_price(0)
        .nonce(0)
        .build_fill();

    let ctx = Context::mainnet().with_db(db).modify_cfg_chained(|c| c.spec = SPEC);
    let mut evm = Evm::new(
        ctx,
        EthInstructions::new_mainnet_with_spec(SPEC),
        AiPrecompiles::new(SPEC),
    );
    evm.transact_commit(tx)
        .map_err(|e| anyhow::anyhow!("evm execution failed: {e:?}"))
}

/// Headroom funded to the caller above `value`, so the post-state balance check
/// is exact: with `gas_price = 0` the only balance debit is the transferred
/// `value`, hence `caller_balance_after == initial_balance - value`.
const PAYABLE_FUND_HEADROOM: u128 = 1_000_000_000_000_000_000;

/// Execute a single **value-bearing** call against `contract` (runtime `code`)
/// with the AI precompiles installed, then read the committed post-state.
///
/// Funds the caller with `value + PAYABLE_FUND_HEADROOM`, sets `TxEnv.value` (so
/// `CALLVALUE` inside the contract observes the payment), commits the
/// transaction, and reads back from the committed [`CacheDB`]:
///
/// * the contract's storage slot 0 (where the AIPay contract accrues payment),
/// * the caller's balance after the transfer.
///
/// Returns `(result, storage_slot0, caller_balance_after)`.
///
/// `gas_price` is 0, so the only balance movement is the `value` transfer:
/// `caller_balance_after == value + PAYABLE_FUND_HEADROOM - value ==
/// PAYABLE_FUND_HEADROOM` on success, letting callers assert an exact debit.
///
/// State-readback note: [`ExecuteCommitEvm::transact_commit`] takes `&mut self`
/// and commits into the journal's database — it does **not** consume the EVM. The
/// committed `CacheDB` therefore lives on at `evm.ctx.journaled_state.database`
/// after the call and is read directly via the [`Database`] trait.
pub fn execute_payable_call(
    caller: Address,
    contract: Address,
    code: &[u8],
    calldata: Vec<u8>,
    value: U256,
    gas_limit: u64,
) -> anyhow::Result<(ExecutionResult, U256, U256)> {
    let mut db = CacheDB::new(EmptyDB::default());

    let mut info = AccountInfo::default();
    info.code_hash = keccak256(code);
    info.code = Some(Bytecode::new_raw(Bytes::copy_from_slice(code)));
    db.insert_account_info(contract, info);

    let mut caller_info = AccountInfo::default();
    caller_info.balance = value
        .checked_add(U256::from(PAYABLE_FUND_HEADROOM))
        .ok_or_else(|| anyhow::anyhow!("caller funding overflow"))?;
    db.insert_account_info(caller, caller_info);

    let tx = TxEnv::builder()
        .caller(caller)
        .kind(TxKind::Call(contract))
        .data(Bytes::from(calldata))
        .value(value)
        .gas_limit(gas_limit)
        .gas_price(0)
        .nonce(0)
        .build_fill();

    let ctx = Context::mainnet().with_db(db).modify_cfg_chained(|c| c.spec = SPEC);
    let mut evm = Evm::new(
        ctx,
        EthInstructions::new_mainnet_with_spec(SPEC),
        AiPrecompiles::new(SPEC),
    );
    let result = evm
        .transact_commit(tx)
        .map_err(|e| anyhow::anyhow!("evm execution failed: {e:?}"))?;

    // Recover the committed state. `transact_commit` borrowed the evm mutably and
    // committed into this very `CacheDB`; read the post-state straight off it.
    let db = &mut evm.ctx.journaled_state.database;
    let slot0 = db
        .storage(contract, U256::ZERO)
        .map_err(|e| anyhow::anyhow!("post-state storage read failed: {e:?}"))?;
    let caller_balance_after = db
        .basic(caller)
        .map_err(|e| anyhow::anyhow!("post-state balance read failed: {e:?}"))?
        .map(|a| a.balance)
        .unwrap_or_default();

    Ok((result, slot0, caller_balance_after))
}

/// Runtime bytecode of a passthrough contract that forwards its calldata to a
/// precompile `addr` (low 3 bytes `b2 b1 b0`) via `CALL` and returns the
/// returndata. Used by examples/benches to drive a precompile from real EVM
/// bytecode.
pub fn passthrough_bytecode(addr_lo3: [u8; 3]) -> Vec<u8> {
    vec![
        0x36, // CALLDATASIZE
        0x60, 0x00, // PUSH1 0  destOffset
        0x60, 0x00, // PUSH1 0  offset
        0x37, // CALLDATACOPY -> mem[0:cds]=calldata
        0x60, 0x00, // PUSH1 0  retLength
        0x60, 0x00, // PUSH1 0  retOffset
        0x36, // CALLDATASIZE argsLength
        0x60, 0x00, // PUSH1 0  argsOffset
        0x60, 0x00, // PUSH1 0  value
        0x62, addr_lo3[0], addr_lo3[1], addr_lo3[2], // PUSH3 address
        0x5a, // GAS
        0xf1, // CALL
        0x50, // POP
        0x3d, // RETURNDATASIZE
        0x60, 0x00, // PUSH1 0
        0x60, 0x00, // PUSH1 0
        0x3e, // RETURNDATACOPY
        0x3d, // RETURNDATASIZE
        0x60, 0x00, // PUSH1 0
        0xf3, // RETURN
    ]
}

/// Runtime bytecode of the shared **AIPay** contract — byte-for-byte identical
/// across the Rust and Go VMs (the cross-VM determinism fixture).
///
/// Contract calldata is `[model(32, UTF-8 NUL-padded)][prompt]`; the transaction
/// carries `msg.value` as the payment. The contract:
///
/// 1. accrues the payment on-chain: `sstore(0, sload(0) + callvalue)`,
/// 2. lays out precompile calldata in memory as `[selector(4)=0][model(32)][prompt]`
///    by copying the call's calldata to `mem[4..]` (leaving `mem[0..4]` zero),
/// 3. `CALL`s the AI inference precompile `0x020001` with `argsOffset=0`,
///    `argsLength = 4 + calldatasize`,
/// 4. returns the precompile's returndata (the LLM output).
///
/// Canonical hex (verified to disassemble to the opcodes above):
/// `3460005401600055366000600437600060006004360160006000620200015af1503d600060003e3d6000f3`
pub fn aipay_bytecode() -> Vec<u8> {
    vec![
        0x34, // CALLVALUE
        0x60, 0x00, // PUSH1 0
        0x54, // SLOAD            -> sload(0)
        0x01, // ADD              -> sload(0) + callvalue
        0x60, 0x00, // PUSH1 0
        0x55, // SSTORE           -> sstore(0, sum)  [payment accrued on-chain]
        0x36, // CALLDATASIZE
        0x60, 0x00, // PUSH1 0    src offset
        0x60, 0x04, // PUSH1 4    dest offset
        0x37, // CALLDATACOPY     -> mem[4 : 4+cds] = calldata; mem[0..4] = 0 (selector)
        0x60, 0x00, // PUSH1 0    retLength
        0x60, 0x00, // PUSH1 0    retOffset
        0x60, 0x04, // PUSH1 4
        0x36, // CALLDATASIZE
        0x01, // ADD              -> argsLength = 4 + cds
        0x60, 0x00, // PUSH1 0    argsOffset
        0x60, 0x00, // PUSH1 0    value (inner CALL carries none)
        0x62, 0x02, 0x00, 0x01, // PUSH3 0x020001  ADDR_AI_INFERENCE low 3 bytes
        0x5a, // GAS
        0xf1, // CALL            -> precompile input = mem[0 : 4+cds] = [selector][model][prompt]
        0x50, // POP             -> drop success flag
        0x3d, // RETURNDATASIZE
        0x60, 0x00, // PUSH1 0
        0x60, 0x00, // PUSH1 0
        0x3e, // RETURNDATACOPY   -> mem[0 : rds] = returndata
        0x3d, // RETURNDATASIZE
        0x60, 0x00, // PUSH1 0
        0xf3, // RETURN           -> return mem[0 : rds] (the LLM output)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The AIPay runtime must be byte-for-byte the canonical shared bytecode —
    /// the same bytes the Go VM executes. Pinning the hex here makes "hex matches
    /// the authoritative opcodes" a machine-checked invariant.
    #[test]
    fn aipay_bytecode_matches_canonical_hex() {
        const CANONICAL: &str =
            "3460005401600055366000600437600060006004360160006000620200015af1503d600060003e3d6000f3";
        assert_eq!(hex::encode(aipay_bytecode()), CANONICAL);
    }

    /// CONSENSUS PARITY (RED revert-path divergence): a reverting AI inference
    /// precompile call must consume the canonical `required_gas(input)` — the
    /// SAME amount geth charges via `RequiredGas` before refunding the remainder
    /// (Go `TestRevertRefundsGas`) — NOT zero. This drives `run_ai` directly with
    /// a short input (reverts before any engine call, so no model is needed) and
    /// asserts both the Revert result and the exact gas spent. Charging zero here
    /// (the prior behavior) would diverge from the Go co-validator by
    /// `required_gas` on every reverting call → state-root fork.
    #[test]
    fn ai_inference_revert_charges_canonical_required_gas() {
        use crate::precompiles::{required_gas, ADDR_AI_INFERENCE};

        let pre = AiPrecompiles::new(SPEC);
        // < 36 bytes => exec_ai_inference returns Revert (short input) with no
        // engine touched. required_gas for <header clamps prompt to 0 => base.
        let short = [0u8; 8];
        let gas_limit = 5_000_000u64;
        let res = pre.run_ai(&ADDR_AI_INFERENCE, &short, gas_limit);

        let want = required_gas(&ADDR_AI_INFERENCE, short.len()).expect("AI addr has required_gas");
        assert_eq!(res.result, InstructionResult::Revert, "short input must REVERT");
        assert_eq!(
            res.gas.spent(),
            want,
            "revert must consume canonical required_gas ({want}), not zero (Go-parity, anti-fork)"
        );
        assert_eq!(res.gas.remaining(), gas_limit - want, "the remainder must be refunded");
        assert_eq!(want, 120_000, "base required_gas for sub-header input is GAS_BASE_INFER");
        // Returndata carries the reason (parity with Go's reason-in-returndata).
        assert!(!res.output.is_empty(), "revert returndata must carry the reason string");
    }

    /// Gas the precompile reports on a SUCCESS path equals the canonical
    /// `required_gas(input)` (input-based), so the success-path gas matches geth's
    /// `RequiredGas` too. We can't run the engine here, but we can assert the
    /// inference precompile's own gas == required_gas via the registry on a
    /// (reverting, no-engine) call is covered above; this pins the success
    /// arithmetic equality of the two gas entry points for a representative
    /// non-trivial input length.
    #[test]
    fn ai_required_gas_is_input_based_and_addr_routed() {
        use crate::precompiles::{required_gas, ADDR_AI_EMBEDDING, ADDR_AI_INFERENCE};
        // Inference: 120000 + 30*prompt.
        assert_eq!(required_gas(&ADDR_AI_INFERENCE, 36 + 100), Some(120_000 + 30 * 100));
        // Embedding: 50000 + 16*text.
        assert_eq!(required_gas(&ADDR_AI_EMBEDDING, 36 + 100), Some(50_000 + 16 * 100));
        // Non-AI address: None (caller falls back to stock gas).
        assert_eq!(required_gas(&[0xffu8; 20], 100), None);
    }
}
