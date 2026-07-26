//! Shared helper for hanzo-vm examples: load zen / zen-embedding models into the
//! engine bridge using the shared `HANZO_FFI_MODELS` spec (one config format
//! across the FFI and the VM).
#![allow(dead_code)]

use hanzo_server_core::hanzo_for_server_builder::{HanzoForServerBuilder, ModelConfig};

/// Build the multi-model engine from `HANZO_FFI_MODELS` and install it into the
/// bridge. When the spec is unset, defaults to a single zen-5-flash GGUF from
/// `ZEN5_WEIGHTS_DIR` (tokenizer from `ZEN_TOK_DIR`).
pub fn load_engine() -> anyhow::Result<()> {
    let spec = std::env::var("HANZO_FFI_MODELS").unwrap_or_else(|_| {
        let dir = std::env::var("ZEN5_WEIGHTS_DIR").unwrap_or_else(|_| "/tmp/zen5-weights".into());
        format!("zen-nano=gguf:{dir}/zen-5-flash.gguf")
    });
    let tok_dir = std::env::var("HANZO_FFI_TOK_DIR")
        .or_else(|_| std::env::var("ZEN_TOK_DIR"))
        .ok();
    let configs =
        hanzo_engine::parse_model_spec(&spec, tok_dir.as_deref()).map_err(|e| anyhow::anyhow!(e))?;
    let default_id = configs[0].0.clone();
    let mut builder = HanzoForServerBuilder::new();
    for (name, model) in configs {
        // alias == name makes `name` the routable model id (get_sender).
        builder = builder.add_model_config(ModelConfig::new(name.clone(), model).with_alias(name));
    }
    builder = builder.with_default_model_id(default_id);
    let rt = tokio::runtime::Runtime::new()?;
    let hanzo = rt.block_on(builder.build())?;
    hanzo_engine::register_engine(hanzo);
    Ok(())
}
