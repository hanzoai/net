//! Benchmark the Rust VM's LLM + embedding precompile integration overhead.
//!
//! Warm, in-process, load-once engine. Measures, for both ops:
//!   - baseline: raw `hanzo_engine::{infer,embed}` (bridge, no EVM),
//!   - rust-vm: a deployed contract's `CALL(0x0201/0x0202)` via real revm bytecode,
//!   - evm-floor: passthrough → IDENTITY precompile `0x04` (no model) — pure EVM cost.
//! Emits BENCH_JSON for the Go comparison.
//!
//! Run:
//!   HANZO_FFI_MODELS="zen-nano=gguf:/tmp/zen5-weights/zen-5-flash.gguf;zen-embed=embedding:/tmp/zen-embedding-0.6B" \
//!   ZEN_TOK_DIR=/tmp/zen-nano-fused INFER_MODEL=zen-nano EMBED_MODEL=zen-embed BENCH_ITERS=8 \
//!   cargo run -p hanzo-vm --example bench_precompile --release --features accelerate

mod common;

use hanzo_vm::evm::{execute_contract_call, passthrough_bytecode};
use hanzo_vm::precompiles::{ADDR_AI_EMBEDDING, ADDR_AI_INFERENCE};
use revm::context_interface::result::{ExecutionResult, Output};
use revm::primitives::Address;
use std::time::Instant;

fn mean(s: &[f64]) -> f64 { s.iter().sum::<f64>() / s.len().max(1) as f64 }
fn min(s: &[f64]) -> f64 { s.iter().cloned().fold(f64::INFINITY, f64::min) }

/// 32-byte calldata model field: name bytes, NUL-padded.
fn model_field(name: &str) -> [u8; 32] {
    let mut f = [0u8; 32];
    let b = name.as_bytes();
    let n = b.len().min(32);
    f[..n].copy_from_slice(&b[..n]);
    f
}

/// Build [selector(4)][model(32)][payload] calldata.
fn calldata(name: &str, payload: &[u8]) -> Vec<u8> {
    let mut c = Vec::with_capacity(36 + payload.len());
    c.extend_from_slice(&[0u8; 4]);
    c.extend_from_slice(&model_field(name));
    c.extend_from_slice(payload);
    c
}

/// Run a precompile through real EVM bytecode; return (ms, output bytes).
fn vm_call(addr: [u8; 20], name: &str, payload: &[u8]) -> (f64, Vec<u8>) {
    let lo3 = [addr[17], addr[18], addr[19]];
    let code = passthrough_bytecode(lo3);
    let contract = Address::from([0x11u8; 20]);
    let caller = Address::from([0x22u8; 20]);
    let t = Instant::now();
    let res = execute_contract_call(caller, contract, &code, calldata(name, payload), 30_000_000).unwrap();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let out = match res {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(d) | Output::Create(d, _) => d.to_vec(),
        },
        _ => Vec::new(),
    };
    (ms, out)
}

fn main() -> anyhow::Result<()> {
    let iters: usize = std::env::var("BENCH_ITERS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let infer_model = std::env::var("INFER_MODEL").unwrap_or_default();
    let embed_model = std::env::var("EMBED_MODEL").unwrap_or_default();
    let prompt = "Who are you and who made you? Answer in one sentence.";
    let text = "The quick brown fox jumps over the lazy dog.";

    common::load_engine()?;

    // Warm + capability probe.
    let _ = hanzo_engine::infer(&infer_model, prompt.as_bytes());
    let embed_ok = hanzo_engine::embed(&embed_model, text.as_bytes()).map(|v| v.len());

    let (mut raw_i, mut vm_i, mut raw_e, mut vm_e, mut floor) =
        (vec![], vec![], vec![], vec![], vec![]);
    let mut out_i = 0usize;
    let mut emb_dim = 0usize;
    for _ in 0..iters {
        let t = Instant::now();
        let o = hanzo_engine::infer(&infer_model, prompt.as_bytes()).unwrap();
        raw_i.push(t.elapsed().as_secs_f64() * 1000.0);
        out_i = o.len();

        let (ms, o) = vm_call(ADDR_AI_INFERENCE, &infer_model, prompt.as_bytes());
        vm_i.push(ms);
        let _ = o;

        if embed_ok.is_ok() {
            let t = Instant::now();
            let v = hanzo_engine::embed(&embed_model, text.as_bytes()).unwrap();
            raw_e.push(t.elapsed().as_secs_f64() * 1000.0);
            emb_dim = v.len();
            let (ms, o) = vm_call(ADDR_AI_EMBEDDING, &embed_model, text.as_bytes());
            vm_e.push(ms);
            emb_dim = emb_dim.max(o.len() / 4);
        }

        // EVM floor: passthrough -> identity precompile 0x04.
        let mut identity = [0u8; 20];
        identity[19] = 0x04;
        let (ms, _) = vm_call(identity, "", &[0xABu8; 64]);
        floor.push(ms * 1000.0);
    }

    println!("\n==== RUST VM — infer + embed precompile benchmark ({iters} iters, warm, CPU+Accelerate) ====");
    println!("[infer model={:?}]  raw {:.1} ms (min {:.1})  | vm 0x0201 {:.1} ms (min {:.1})  | out {} B",
        if infer_model.is_empty() {"default"} else {&infer_model}, mean(&raw_i), min(&raw_i), mean(&vm_i), min(&vm_i), out_i);
    match &embed_ok {
        Ok(_) => println!("[embed model={:?}]  raw {:.1} ms (min {:.1})  | vm 0x0202 {:.1} ms (min {:.1})  | dim {}",
            if embed_model.is_empty() {"default"} else {&embed_model}, mean(&raw_e), min(&raw_e), mean(&vm_e), min(&vm_e), emb_dim),
        Err(e) => println!("[embed]  unavailable: {e:?}"),
    }
    println!("pure EVM floor (0x04): {:.1} us  (interpreter + dispatch, no model)", mean(&floor));
    let infer_tok = (out_i as f64 / 4.0).max(1.0);
    println!("infer rate ~{:.1} tok/s", infer_tok / (min(&raw_i) / 1000.0));

    let emb_json = if embed_ok.is_ok() {
        format!(",\"embed_raw_ms\":{:.3},\"embed_vm_ms\":{:.3},\"embed_dim\":{}", mean(&raw_e), mean(&vm_e), emb_dim)
    } else { String::new() };
    println!("\nBENCH_JSON {{\"lang\":\"rust\",\"iters\":{iters},\"infer_raw_ms\":{:.3},\"infer_vm_ms\":{:.3},\"evm_floor_us\":{:.2},\"infer_out_bytes\":{}{}}}",
        mean(&raw_i), mean(&vm_i), mean(&floor), out_i, emb_json);
    Ok(())
}
