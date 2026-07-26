//! Proof: EVM precompile `0x0201` runs real native LLM inference.
//!
//! Loads the zen-nano GGUF into the native engine (the same `HanzoForServerBuilder`
//! path `hanzo-node` uses at startup), installs it into the process-global engine
//! bridge, then calls the **production** `PrecompileRegistry` entry for
//! `ADDR_AI_INFERENCE` with EVM calldata laid out exactly as a Solidity contract
//! would produce — `[selector(4)][model_id(32)][prompt..]`. This is the precise
//! function the EVM invokes when contract bytecode does `CALL(0x0201, ...)`.
//!
//! Run:
//!   ZEN5_WEIGHTS_DIR=/tmp/zen5-weights ZEN_TOK_DIR=/tmp/zen-nano-fused \
//!   cargo run -p hanzo-vm --example precompile_infer --release --features accelerate -- \
//!     "Who are you and who made you?"

use hanzo_engine::{AutoDeviceMapParams, ModelDType, ModelSelected};
use hanzo_server_core::hanzo_for_server_builder::HanzoForServerBuilder;
use hanzo_vm::precompiles::{PrecompileRegistry, PrecompileResult, ADDR_AI_INFERENCE};

fn main() -> anyhow::Result<()> {
    let weights_dir = std::env::var("ZEN5_WEIGHTS_DIR").unwrap_or_else(|_| "/tmp/zen5-weights".into());
    let tok_dir = std::env::var("ZEN_TOK_DIR").unwrap_or_else(|_| "/tmp/zen-nano-fused".into());
    let gguf = "zen-5-flash.gguf";
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Who are you and who made you? Answer in one sentence.".into());

    let model = ModelSelected::GGUF {
        tok_model_id: Some(tok_dir),
        quantized_model_id: weights_dir.clone(),
        quantized_filename: gguf.to_string(),
        dtype: ModelDType::Auto,
        topology: None,
        max_seq_len: AutoDeviceMapParams::DEFAULT_MAX_SEQ_LEN,
        max_batch_size: AutoDeviceMapParams::DEFAULT_MAX_BATCH_SIZE,
    };

    // Build the engine (async) on a throwaway runtime; the engine keeps its own
    // worker threads after construction.
    let rt = tokio::runtime::Runtime::new()?;
    let hanzo = rt.block_on(async {
        HanzoForServerBuilder::new()
            .with_model(model)
            .with_model_id_override("zen-5-flash")
            .build()
            .await
    })?;

    let installed = hanzo_engine::register_engine(hanzo);
    println!(
        "engine registered={installed}  inference_engine_registered={}",
        hanzo_engine::inference_engine_registered()
    );

    // EVM calldata: [0:4] selector (reserved 0) | [4:36] model id | [36:] prompt.
    let mut calldata = vec![0u8; 36];
    calldata.extend_from_slice(prompt.as_bytes());

    let registry = PrecompileRegistry::default();
    println!("precompiles registered: {}", registry.len());
    println!("calling 0x0201 (ai_inference) with prompt: {prompt:?}\n");

    let started = std::time::Instant::now();
    match registry.call(&ADDR_AI_INFERENCE, &calldata) {
        Some(PrecompileResult::Success { output, gas_used }) => {
            let secs = started.elapsed().as_secs_f64();
            println!("0x0201 SUCCESS  gas_used={gas_used}  elapsed={secs:.2}s");
            println!("OUTPUT: {}", String::from_utf8_lossy(&output));
        }
        Some(PrecompileResult::Revert { reason }) => println!("0x0201 REVERT: {reason}"),
        Some(PrecompileResult::Error { message }) => println!("0x0201 ERROR: {message}"),
        None => println!("no precompile registered at 0x0201"),
    }
    Ok(())
}
