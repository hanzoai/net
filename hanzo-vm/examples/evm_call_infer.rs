//! Proof: a deployed EVM contract's `CALL(0x0201)` runs native LLM inference
//! during real bytecode execution — using the production `hanzo_vm::evm` path.
//!
//! Run:
//!   ZEN5_WEIGHTS_DIR=/tmp/zen5-weights ZEN_TOK_DIR=/tmp/zen-nano-fused \
//!   cargo run -p hanzo-vm --example evm_call_infer --release --features accelerate -- \
//!     "Who are you and who made you?"

mod common;

use hanzo_vm::evm::{execute_contract_call, passthrough_bytecode};
use hanzo_vm::precompiles::ADDR_AI_INFERENCE;
use revm::context_interface::result::{ExecutionResult, Output};
use revm::primitives::Address;

fn main() -> anyhow::Result<()> {
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Who are you and who made you? Answer in one sentence.".into());

    // Load the native engine into the process-wide bridge.
    common::load_engine()?;

    // Deploy a passthrough contract that CALLs the AI inference precompile.
    let lo3 = [ADDR_AI_INFERENCE[17], ADDR_AI_INFERENCE[18], ADDR_AI_INFERENCE[19]];
    let code = passthrough_bytecode(lo3);
    let contract = Address::from([0x11u8; 20]);
    let caller = Address::from([0x22u8; 20]);

    // Calldata: [selector(4)][model_id(32)][prompt].
    let mut calldata = vec![0u8; 36];
    calldata.extend_from_slice(prompt.as_bytes());

    println!("executing contract {contract} -> CALL(0x020001) with prompt: {prompt:?}\n");
    let started = std::time::Instant::now();
    match execute_contract_call(caller, contract, &code, calldata, 30_000_000)? {
        ExecutionResult::Success { output, gas, .. } => {
            let data = match output {
                Output::Call(d) => d,
                Output::Create(d, _) => d,
            };
            println!(
                "CONTRACT SUCCESS  evm_gas_used={}  elapsed={:.2}s  return={} bytes",
                gas.used(),
                started.elapsed().as_secs_f64(),
                data.len()
            );
            println!("CONTRACT RETURN (LLM output): {}", String::from_utf8_lossy(&data));
        }
        ExecutionResult::Revert { gas, output, .. } => {
            println!("CONTRACT REVERT gas={} data={}", gas.used(), String::from_utf8_lossy(&output));
        }
        ExecutionResult::Halt { reason, gas, .. } => {
            println!("CONTRACT HALT {reason:?} gas={}", gas.used());
        }
    }
    Ok(())
}
