//! Proof: ONE EVM transaction does on-chain LLM inference AND payment, atomically.
//!
//! The shared **AIPay** contract (byte-identical across the Rust and Go VMs)
//! accrues `msg.value` to storage slot 0, then `CALL`s the AI inference
//! precompile `0x020001` and returns the LLM output — all in a single
//! transaction. This example deploys it on real revm, funds a caller, sends one
//! payable call, and proves from the **committed** post-state that:
//!
//!   1. the returndata is the LLM output (contains "Zen Nano"),
//!   2. the payment was recorded on-chain (slot 0 == value),
//!   3. the caller's balance decreased by exactly `value`.
//!
//! Then it proves determinism (two identical calls → byte-identical output) and
//! benchmarks the end-to-end pay+infer transaction.
//!
//! Run:
//!   HANZO_FFI_MODELS="zen-nano=gguf:/tmp/zen5-weights/zen-5-flash.gguf" \
//!   HANZO_FFI_TOK_DIR=/tmp/zen-nano-fused BENCH_ITERS=5 \
//!   cargo run -p hanzo-vm --example evm_pay_infer --release --features accelerate -- \
//!     "Who are you and who made you? Answer in one sentence."

mod common;

use hanzo_vm::evm::{aipay_bytecode, execute_payable_call};
use revm::context_interface::result::{ExecutionResult, Output};
use revm::primitives::{Address, U256};
use std::time::Instant;

/// Payment carried by the transaction, in wei.
const PAYMENT_WEI: u128 = 1_000_000;
/// Gas limit for the pay+infer transaction.
const GAS_LIMIT: u64 = 30_000_000;

/// 32-byte calldata model field: name bytes, NUL-padded (matches the precompile's
/// `[selector(4)][model(32)][prompt]` layout, which the contract synthesizes).
fn model_field(name: &str) -> [u8; 32] {
    let mut f = [0u8; 32];
    let b = name.as_bytes();
    let n = b.len().min(32);
    f[..n].copy_from_slice(&b[..n]);
    f
}

/// AIPay contract calldata: `[model(32 NUL-padded)][prompt]` (no selector — the
/// contract writes the 4-byte zero selector itself before the precompile CALL).
fn aipay_calldata(model: &str, prompt: &[u8]) -> Vec<u8> {
    let mut c = Vec::with_capacity(32 + prompt.len());
    c.extend_from_slice(&model_field(model));
    c.extend_from_slice(prompt);
    c
}

/// Run one payable pay+infer transaction; return (returndata, slot0, caller_after).
fn pay_infer(
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
    let data = match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(d) | Output::Create(d, _) => d.to_vec(),
        },
        ExecutionResult::Revert { output, gas, .. } => {
            anyhow::bail!(
                "contract REVERT gas={} data={}",
                gas.used(),
                String::from_utf8_lossy(&output)
            )
        }
        ExecutionResult::Halt { reason, gas, .. } => {
            anyhow::bail!("contract HALT {reason:?} gas={}", gas.used())
        }
    };
    Ok((data, slot0, caller_after))
}

fn mean(s: &[f64]) -> f64 {
    s.iter().sum::<f64>() / s.len().max(1) as f64
}
fn min(s: &[f64]) -> f64 {
    s.iter().cloned().fold(f64::INFINITY, f64::min)
}

fn main() -> anyhow::Result<()> {
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Who are you and who made you? Answer in one sentence.".into());
    let model = std::env::var("INFER_MODEL").unwrap_or_else(|_| "zen-nano".into());
    let iters: usize = std::env::var("BENCH_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    common::load_engine()?;

    // Deploy the shared AIPay contract; it accrues payment then CALLs 0x020001.
    let code = aipay_bytecode();
    let contract = Address::from([0x11u8; 20]);
    let caller = Address::from([0x22u8; 20]);
    let value = U256::from(PAYMENT_WEI);

    // The caller is funded with value + headroom; gas_price is 0, so the only
    // balance debit is the transferred value. The initial balance is therefore
    // value + headroom, and we assert the exact debit against it.
    let headroom = U256::from(1_000_000_000_000_000_000u128);
    let initial_balance = value + headroom;

    println!(
        "AIPay contract {contract}  code={} bytes\n  caller={caller}  payment={PAYMENT_WEI} wei  model={model:?}\n  prompt: {prompt:?}\n",
        code.len()
    );

    // ---- Single tx: inference + payment, atomically -----------------------
    let t0 = Instant::now();
    let (data, slot0, caller_after) =
        pay_infer(&code, contract, caller, &model, prompt.as_bytes(), value)?;
    let first_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let out = String::from_utf8_lossy(&data);
    println!("CONTRACT RETURN (LLM output, {} bytes):\n{out}\n", data.len());

    // Assertion 1: returndata is the LLM output.
    assert!(!data.is_empty(), "returndata must be non-empty (the LLM output)");
    let identity_ok = out.contains("Zen Nano");
    println!(
        "  [1] LLM output present: true  (contains \"Zen Nano\": {identity_ok})"
    );
    assert!(
        identity_ok,
        "expected the model's identity (\"Zen Nano\") in the output; got: {out:?}"
    );

    // Assertion 2: payment recorded on-chain (committed storage slot 0).
    println!(
        "  [2] on-chain payment  slot0 = {slot0}  (expected {PAYMENT_WEI})"
    );
    assert_eq!(
        slot0,
        value,
        "contract storage slot 0 must equal the payment"
    );

    // Assertion 3: caller balance decreased by exactly the payment.
    let debited = initial_balance - caller_after;
    println!(
        "  [3] caller balance: {initial_balance} -> {caller_after}  (debited {debited}, expected {PAYMENT_WEI})"
    );
    assert_eq!(debited, value, "caller must be debited exactly the payment");

    // ---- Determinism: same call twice -> byte-identical returndata --------
    let (data2, slot0_2, _) =
        pay_infer(&code, contract, caller, &model, prompt.as_bytes(), value)?;
    let deterministic = data == data2;
    // Slot 0 must also be identical: each call runs against a fresh state, so the
    // single payment accrues to exactly `value` every time (not cumulative).
    assert_eq!(slot0_2, value, "each fresh-state call records exactly one payment");
    println!("\n  determinism: {deterministic}  (two identical calls, greedy decode)");
    assert!(
        deterministic,
        "non-deterministic returndata: greedy decode must be reproducible"
    );

    // ---- Benchmark: end-to-end pay+infer transaction ---------------------
    let mut e2e = Vec::with_capacity(iters);
    let mut out_bytes = data.len();
    for _ in 0..iters {
        let t = Instant::now();
        let (d, s0, c_after) =
            pay_infer(&code, contract, caller, &model, prompt.as_bytes(), value)?;
        e2e.push(t.elapsed().as_secs_f64() * 1000.0);
        // Re-assert the invariants every iteration — the benchmark is also a
        // 5x repeat of the full proof, not just a timing loop.
        assert_eq!(s0, value, "payment must be recorded every iteration");
        assert_eq!(initial_balance - c_after, value, "exact debit every iteration");
        assert_eq!(d, data, "returndata must be stable every iteration");
        out_bytes = d.len();
    }

    println!(
        "\n==== RUST VM — pay+infer benchmark ({iters} iters, warm, CPU+Accelerate) ====\n  first (cold-ish): {first_ms:.1} ms  | e2e mean {:.1} ms  min {:.1} ms  | out {out_bytes} B",
        mean(&e2e),
        min(&e2e)
    );
    println!(
        "\nBENCH_JSON {{\"lang\":\"rust\",\"op\":\"pay_infer\",\"e2e_ms\":{:.3},\"out_bytes\":{out_bytes},\"payment_recorded\":true,\"deterministic\":{deterministic}}}",
        mean(&e2e)
    );

    Ok(())
}
