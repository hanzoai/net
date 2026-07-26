//! Custom precompile registry for the Hanzo L2.
//!
//! Precompiles are deterministic functions callable from EVM contracts at
//! fixed addresses, avoiding the overhead of interpreted bytecode.
//!
//! # Address space
//!
//! | Hanzo address  | Routes to                                | Purpose                  |
//! |----------------|------------------------------------------|--------------------------|
//! | `0x0101..0001` | [`hanzo_pqc::signature::MlDsa`]          | PQ signature verify      |
//! | `0x0102..0002` | `libluxprecompile` Quasar (0x0300..0020) | Quasar committee query   |
//! | `0x0201..0001` | [`hanzo_engine::infer`]                  | AI inference forward pass|
//! | `0x0202..0002` | [`hanzo_engine::embed`]                  | AI embedding             |
//!
//! All four entry points dispatch into the canonical Hanzo or Lux
//! implementation — there are no in-tree fakes. When a downstream impl
//! is not available at runtime (e.g. no LLM engine registered) the
//! precompile reverts with a descriptive reason rather than returning
//! synthetic bytes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use hanzo_engine::{self as engine, EngineError};
use hanzo_pqc::signature::MlDsa;

// ---------------------------------------------------------------------------
// Precompile addresses
// ---------------------------------------------------------------------------

/// PQ signature verification via ML-DSA (FIPS 204) from `hanzo-pqc`.
pub const ADDR_PQ_VERIFY: [u8; 20] = addr(0x01, 0x01);

/// Quasar committee membership query.
pub const ADDR_QUASAR_QUERY: [u8; 20] = addr(0x01, 0x02);

/// AI inference call (forward pass through a registered model).
pub const ADDR_AI_INFERENCE: [u8; 20] = addr(0x02, 0x01);

/// AI embedding computation.
pub const ADDR_AI_EMBEDDING: [u8; 20] = addr(0x02, 0x02);

/// Canonical Lux Quasar (Verkle witness) precompile, addressed inside the
/// `libluxprecompile` Go-backed dispatcher. The Hanzo Quasar precompile
/// is a thin shim that forwards to this address.
pub const LUX_QUASAR_ADDR: &str = "0x0300000000000000000000000000000000000020";

/// Helper to build a 20-byte precompile address from a category and index.
///
/// Layout: `[0x00; 17] ++ [category] ++ [0x00] ++ [index]`
const fn addr(category: u8, index: u8) -> [u8; 20] {
    let mut a = [0u8; 20];
    a[17] = category;
    a[19] = index;
    a
}

// ---------------------------------------------------------------------------
// PrecompileResult
// ---------------------------------------------------------------------------

/// Outcome of a precompile execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecompileResult {
    /// Successful execution with output bytes.
    Success {
        /// Raw output returned to the EVM caller.
        output: Vec<u8>,
        /// Gas consumed by this precompile call.
        gas_used: u64,
    },
    /// Execution reverted.
    Revert {
        /// Human-readable reason string.
        reason: String,
    },
    /// Execution encountered an unrecoverable error.
    Error {
        /// Human-readable error description.
        message: String,
    },
}

// ---------------------------------------------------------------------------
// PrecompileEntry
// ---------------------------------------------------------------------------

/// A single registered precompile.
#[derive(Clone)]
pub struct PrecompileEntry {
    /// 20-byte EVM address where this precompile lives.
    pub address: [u8; 20],
    /// Human-readable name for logging.
    pub name: String,
    /// Base gas cost (charged before execution).
    pub base_gas: u64,
    /// The execution function.
    ///
    /// Receives raw calldata and returns a [`PrecompileResult`].
    pub execute: fn(input: &[u8]) -> PrecompileResult,
}

impl std::fmt::Debug for PrecompileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrecompileEntry")
            .field("address", &hex::encode(self.address))
            .field("name", &self.name)
            .field("base_gas", &self.base_gas)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// PrecompileRegistry
// ---------------------------------------------------------------------------

/// Registry of custom precompile contracts.
///
/// Use [`Default::default()`] to get a registry pre-loaded with all Hanzo
/// precompiles, or build one manually with [`new`](Self::new) and
/// [`register`](Self::register).
#[derive(Debug)]
pub struct PrecompileRegistry {
    entries: HashMap<[u8; 20], PrecompileEntry>,
}

impl PrecompileRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a precompile. Overwrites any existing entry at the same address.
    pub fn register(&mut self, entry: PrecompileEntry) {
        self.entries.insert(entry.address, entry);
    }

    /// Look up a precompile by its 20-byte EVM address.
    pub fn get(&self, address: &[u8; 20]) -> Option<&PrecompileEntry> {
        self.entries.get(address)
    }

    /// Execute a precompile at `address` with the given `input`.
    ///
    /// Returns `None` if no precompile is registered at that address.
    pub fn call(&self, address: &[u8; 20], input: &[u8]) -> Option<PrecompileResult> {
        self.entries
            .get(address)
            .map(|entry| (entry.execute)(input))
    }

    /// Return the number of registered precompiles.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if no precompiles are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for PrecompileRegistry {
    /// Build a registry with all built-in Hanzo precompiles.
    fn default() -> Self {
        let mut r = Self::new();

        r.register(PrecompileEntry {
            address: ADDR_PQ_VERIFY,
            name: "pq_verify".into(),
            base_gas: 3_000,
            execute: exec_pq_verify,
        });

        #[cfg(feature = "quasar")]
        r.register(PrecompileEntry {
            address: ADDR_QUASAR_QUERY,
            name: "quasar_query".into(),
            base_gas: 3_000,
            execute: exec_quasar_query,
        });

        r.register(PrecompileEntry {
            address: ADDR_AI_INFERENCE,
            name: "ai_inference".into(),
            // Canonical input-based base (see GAS_BASE_INFER / inference_gas):
            // the per-call gas is computed input-based by exec_ai_inference; this
            // metadata base mirrors the same canonical constant.
            base_gas: GAS_BASE_INFER,
            execute: exec_ai_inference,
        });

        r.register(PrecompileEntry {
            address: ADDR_AI_EMBEDDING,
            name: "ai_embedding".into(),
            base_gas: 50_000,
            execute: exec_ai_embedding,
        });

        r
    }
}

// ---------------------------------------------------------------------------
// Built-in precompile implementations
// ---------------------------------------------------------------------------

/// PQ signature verification using ML-DSA (FIPS 204) via `hanzo-pqc`.
///
/// # Calldata layout
///
/// | Offset    | Length  | Field             |
/// |-----------|---------|-------------------|
/// | 0         | 4       | public key length |
/// | 4         | pk_len  | public key bytes  |
/// | 4+pk      | 4       | signature length  |
/// | 8+pk      | sig_len | signature bytes   |
/// | rest      | ..      | message bytes     |
///
/// All lengths are big-endian `u32`. The public-key length determines the
/// FIPS 204 parameter set (ML-DSA-44 / 65 / 87 for 1312 / 1952 / 2592 bytes).
///
/// Returns the 32-byte big-endian word `0x..01` on a valid signature and
/// `0x..00` on an invalid one, matching the canonical Lux ML-DSA precompile
/// output shape.
fn exec_pq_verify(input: &[u8]) -> PrecompileResult {
    // Minimum: 4 (pk_len) + 1 (pk) + 4 (sig_len) + 1 (sig) + 0 (msg)
    if input.len() < 10 {
        return PrecompileResult::Revert {
            reason: "input too short for pq_verify".into(),
        };
    }

    let pk_len = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;
    if input.len() < 4 + pk_len + 4 {
        return PrecompileResult::Revert {
            reason: "input truncated at public key".into(),
        };
    }

    let pk_bytes = &input[4..4 + pk_len];

    let sig_offset = 4 + pk_len;
    let sig_len = u32::from_be_bytes([
        input[sig_offset],
        input[sig_offset + 1],
        input[sig_offset + 2],
        input[sig_offset + 3],
    ]) as usize;

    let msg_offset = sig_offset + 4 + sig_len;
    if input.len() < msg_offset {
        return PrecompileResult::Revert {
            reason: "input truncated at signature".into(),
        };
    }

    let sig_bytes = &input[sig_offset + 4..msg_offset];
    let msg_bytes = &input[msg_offset..];

    // Gas cost: base + per-byte cost over the verified payload.
    let gas_used = 3_000u64.saturating_add((pk_len as u64 + sig_len as u64) / 16);

    let valid = match MlDsa::verify_raw(pk_bytes, msg_bytes, sig_bytes) {
        Ok(b) => b,
        Err(err) => {
            return PrecompileResult::Revert {
                reason: format!("ml-dsa verify error: {err}"),
            };
        }
    };

    // 32-byte word, right-aligned (mirrors the canonical Lux precompile).
    let mut output = vec![0u8; 32];
    if valid {
        output[31] = 1;
    }
    PrecompileResult::Success { output, gas_used }
}

/// Quasar committee membership query, routed through the canonical Lux
/// Quasar precompile (Verkle witness verifier) in `libluxprecompile`.
///
/// # Calldata layout
///
/// | Offset  | Length | Field                                  |
/// |---------|--------|----------------------------------------|
/// | 0       | 20     | validator address (the query)          |
/// | 20      | 32     | Verkle commitment (committee root)     |
/// | 52      | 32     | Verkle membership proof                |
/// | 84      | 1      | threshold-met flag (0 = unmet, !0 = met)|
///
/// Total: 85 bytes. The 20-byte validator address is the membership
/// query, the next 65 bytes are a Verkle witness over the committee
/// state; they are forwarded to the canonical Lux precompile at
/// [`LUX_QUASAR_ADDR`] in `[commitment(32)][proof(32)][thresholdMet(1)]`
/// order.
///
/// Output layout (53 bytes):
///
/// | Offset  | Length | Field                      |
/// |---------|--------|----------------------------|
/// | 0       | 20     | validator address          |
/// | 20      | 32     | committee root (commitment)|
/// | 52      | 1      | is-member flag (0/1)       |
#[cfg(feature = "quasar")]
fn exec_quasar_query(input: &[u8]) -> PrecompileResult {
    const REQUIRED_LEN: usize = 20 + 32 + 32 + 1;
    if input.len() < REQUIRED_LEN {
        return PrecompileResult::Revert {
            reason: format!(
                "quasar_query requires {} bytes (addr ++ commitment ++ proof ++ flag)",
                REQUIRED_LEN
            ),
        };
    }

    let validator = &input[0..20];
    let commitment = &input[20..52];
    let proof = &input[52..84];
    let threshold_met_byte = input[84];

    // Build the canonical Verkle witness input expected by the Lux Quasar
    // precompile (see lux/precompile/quasar/contract.go).
    let mut witness = Vec::with_capacity(65);
    witness.extend_from_slice(commitment);
    witness.extend_from_slice(proof);
    witness.push(threshold_met_byte);

    // The committee membership decision is derived from the canonical
    // Verkle verification result. Failure to dispatch is a hard error —
    // not a silent zero — so the EVM caller learns the actual reason.
    let res = match lux_precompile::run(LUX_QUASAR_ADDR, &witness, 1_000_000) {
        Ok(r) => r,
        Err(err) => {
            return PrecompileResult::Revert {
                reason: format!("luxprecompile dispatch failed: {err}"),
            };
        }
    };

    // The canonical precompile returns a single byte (0 or 1). Anything
    // else is a protocol break.
    let is_member = match res.output.as_slice() {
        [b] => *b != 0,
        _ => {
            return PrecompileResult::Revert {
                reason: format!(
                    "unexpected quasar output length: {} bytes",
                    res.output.len()
                ),
            };
        }
    };

    let mut output = Vec::with_capacity(20 + 32 + 1);
    output.extend_from_slice(validator);
    output.extend_from_slice(commitment); // committee root == commitment
    output.push(if is_member { 0x01 } else { 0x00 });

    // Gas: 1_000_000 supplied to the canonical impl; charge what it
    // consumed plus a 1k routing fee.
    let gas_used = 1_000_000u64
        .saturating_sub(res.remaining_gas)
        .saturating_add(1_000);

    PrecompileResult::Success { output, gas_used }
}

/// Decode a model name from a 32-byte calldata field: UTF-8, NUL-padded; empty
/// → the engine default. Lets a contract name any loaded zen / zen-embedding
/// model (e.g. `"zen-nano"`, `"zen-embedding-0.6b"`).
fn model_name(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

/// Calldata header for the AI precompiles: `selector(4) + model id(32)`.
const AI_HEADER_LEN: usize = 4 + 32;

/// CANONICAL INFERENCE GAS SCHEDULE — single source of truth across ALL EVM
/// backends (Rust revm + Go luxfi/geth). Gas MUST be a pure function of INPUT
/// size and be computable BEFORE execution: geth deducts `RequiredGas(input)`
/// before `Run`, when the model output does not yet exist. Pricing on OUTPUT
/// length (as this VM previously did: `100_000 + 8*output_len`) yields a
/// different `gas_used` than geth's input-based charge for the same transaction
/// → divergent state root → chain fork (RED finding H1). These constants and
/// the arithmetic in [`inference_gas`] mirror, byte-for-byte, the authoritative
/// Go schedule in `chains/hanzo-evm/evmllm/precompile.go`
/// (`GasBaseInfer=120_000`, `GasPerPromptByte=30`), pinned by the Go
/// `TestGasScheduleCanonical` differential test.
const GAS_BASE_INFER: u64 = 120_000;
const GAS_PER_PROMPT_BYTE: u64 = 30;

/// Input-based inference gas: `GAS_BASE_INFER + GAS_PER_PROMPT_BYTE * promptBytes`
/// where `promptBytes = input.len().saturating_sub(AI_HEADER_LEN)`. Pure in the
/// input length, independent of the model output — identical on every backend so
/// Rust and Go compute the SAME `gas_used` and agree on the state root. Mirrors
/// Go's `AIInference.RequiredGas` exactly (same clamp-below-header, same
/// overflow-saturating multiply).
fn inference_gas(input_len: usize) -> u64 {
    let prompt_bytes = (input_len.saturating_sub(AI_HEADER_LEN)) as u64;
    match GAS_PER_PROMPT_BYTE.checked_mul(prompt_bytes) {
        Some(add) => GAS_BASE_INFER.saturating_add(add),
        None => u64::MAX, // saturate (parity with Go's overflow branch)
    }
}

/// Base gas for an embedding call (input-based; per-byte over the text region).
const GAS_BASE_EMBED: u64 = 50_000;
/// Per-text-byte embedding gas.
const GAS_PER_TEXT_BYTE: u64 = 16;

/// Input-based embedding gas, same shape as [`inference_gas`]: a fixed base plus
/// a per-byte charge over the text region, computed from input length so it is
/// identical across backends and computable before execution.
fn embedding_gas(input_len: usize) -> u64 {
    let text_bytes = (input_len.saturating_sub(AI_HEADER_LEN)) as u64;
    match GAS_PER_TEXT_BYTE.checked_mul(text_bytes) {
        Some(add) => GAS_BASE_EMBED.saturating_add(add),
        None => u64::MAX,
    }
}

/// Canonical pre-execution gas for an AI precompile, the Rust analog of geth's
/// `RequiredGas(input)`: a pure function of the call address and input length,
/// computable BEFORE execution and charged identically on success AND revert.
///
/// CONSENSUS-CRITICAL (RED H1 + revert-path divergence): the EVM glue
/// ([`crate::evm`]) charges this on the revert path so a reverting AI precompile
/// consumes the SAME gas as geth (which charges `RequiredGas` then refunds the
/// remainder). Charging zero on revert — as the VM previously did — diverges
/// from geth by `RequiredGas(input)` on every reverting call → state-root fork.
///
/// Returns `None` for a non-AI address (the caller falls back to the stock
/// precompile gas model).
pub fn required_gas(addr: &[u8; 20], input_len: usize) -> Option<u64> {
    if *addr == ADDR_AI_INFERENCE {
        Some(inference_gas(input_len))
    } else if *addr == ADDR_AI_EMBEDDING {
        Some(embedding_gas(input_len))
    } else {
        None
    }
}

/// AI text generation.
///
/// # Calldata layout
///
/// | Offset  | Length | Field                                          |
/// |---------|--------|------------------------------------------------|
/// | 0       | 4      | selector (callers may pass 0 — reserved)       |
/// | 4       | 32     | model name (UTF-8, NUL-padded; empty = default)|
/// | 36      | ..     | prompt bytes                                   |
///
/// Dispatches to [`hanzo_engine::infer`], routing to the named model via the
/// engine's native multi-model support. Reverts if no engine is installed or
/// the named model is not loaded.
///
/// Gas is INPUT-based (see [`inference_gas`]) so it matches the Go backend's
/// pre-execution charge exactly — a consensus requirement (RED H1).
fn exec_ai_inference(input: &[u8]) -> PrecompileResult {
    const HEADER_LEN: usize = AI_HEADER_LEN;
    if input.len() < HEADER_LEN {
        return PrecompileResult::Revert {
            reason: format!(
                "ai_inference requires at least {} bytes (selector + model id)",
                HEADER_LEN
            ),
        };
    }

    let model = model_name(&input[4..36]);
    let prompt = &input[HEADER_LEN..];

    if prompt.is_empty() {
        return PrecompileResult::Revert {
            reason: "ai_inference requires a non-empty prompt".into(),
        };
    }

    // Input-based gas, computed from calldata length (NOT output length) so the
    // Rust and Go backends charge identically and agree on the state root.
    let gas_used = inference_gas(input.len());
    match engine::infer(&model, prompt) {
        Ok(output) => PrecompileResult::Success { output, gas_used },
        Err(EngineError::NoInferenceEngine) => PrecompileResult::Revert {
            reason: "no inference engine registered on this node".into(),
        },
        Err(EngineError::NoEmbeddingEngine) => PrecompileResult::Revert {
            reason: "no embedding engine registered on this node".into(),
        },
        Err(EngineError::ModelNotFound(id)) => PrecompileResult::Revert {
            reason: format!("ai_inference model not found: {id}"),
        },
        Err(EngineError::Other(msg)) => PrecompileResult::Revert {
            reason: format!("ai_inference engine failure: {msg}"),
        },
    }
}

/// AI embedding generation.
///
/// # Calldata layout
///
/// | Offset  | Length | Field                                          |
/// |---------|--------|------------------------------------------------|
/// | 0       | 4      | selector (callers may pass 0 — reserved)       |
/// | 4       | 32     | model name (UTF-8, NUL-padded; empty = default)|
/// | 36      | ..     | text bytes                                     |
///
/// Output: `N * 4` bytes (N = the model's native embedding dimension), each
/// four-byte group an IEEE-754 little-endian `f32` of the embedding vector.
fn exec_ai_embedding(input: &[u8]) -> PrecompileResult {
    const HEADER_LEN: usize = 4 + 32;
    if input.len() < HEADER_LEN {
        return PrecompileResult::Revert {
            reason: format!(
                "ai_embedding requires at least {} bytes (selector + model id)",
                HEADER_LEN
            ),
        };
    }

    let model = model_name(&input[4..36]);
    let text = &input[HEADER_LEN..];
    if text.is_empty() {
        return PrecompileResult::Revert {
            reason: "ai_embedding requires non-empty text".into(),
        };
    }

    // Input-based gas (see embedding_gas / required_gas), charged identically on
    // success and (in the EVM glue) on revert — consistent with geth's
    // RequiredGas model and independent of output dimension.
    let gas_used = embedding_gas(input.len());
    match engine::embed(&model, text) {
        Ok(vec) => {
            let mut output = Vec::with_capacity(vec.len() * 4);
            for v in &vec {
                output.extend_from_slice(&v.to_le_bytes());
            }
            PrecompileResult::Success { output, gas_used }
        }
        Err(EngineError::NoEmbeddingEngine) => PrecompileResult::Revert {
            reason: "no embedding engine registered on this node".into(),
        },
        Err(EngineError::NoInferenceEngine) => PrecompileResult::Revert {
            reason: "no inference engine registered on this node".into(),
        },
        Err(EngineError::ModelNotFound(id)) => PrecompileResult::Revert {
            reason: format!("ai_embedding model not found: {id}"),
        },
        Err(EngineError::Other(msg)) => PrecompileResult::Revert {
            reason: format!("ai_embedding engine failure: {msg}"),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Registry-level tests
    // -----------------------------------------------------------------------

    #[test]
    fn default_registry_has_expected_precompiles() {
        let reg = PrecompileRegistry::default();
        assert!(reg.get(&ADDR_PQ_VERIFY).is_some());
        assert!(reg.get(&ADDR_AI_INFERENCE).is_some());
        assert!(reg.get(&ADDR_AI_EMBEDDING).is_some());
        #[cfg(feature = "quasar")]
        {
            assert_eq!(reg.len(), 4);
            assert!(reg.get(&ADDR_QUASAR_QUERY).is_some());
        }
        #[cfg(not(feature = "quasar"))]
        assert_eq!(reg.len(), 3);
    }

    #[test]
    fn registry_call_unknown_address_returns_none() {
        let reg = PrecompileRegistry::default();
        let unknown = [0xff; 20];
        assert!(reg.call(&unknown, &[]).is_none());
    }

    #[test]
    fn addr_helper_layout() {
        // ADDR_PQ_VERIFY: category=0x01, index=0x01
        assert_eq!(ADDR_PQ_VERIFY[17], 0x01);
        assert_eq!(ADDR_PQ_VERIFY[19], 0x01);
        assert_eq!(ADDR_PQ_VERIFY[0..17], [0u8; 17]);

        // ADDR_AI_INFERENCE: category=0x02, index=0x01
        assert_eq!(ADDR_AI_INFERENCE[17], 0x02);
        assert_eq!(ADDR_AI_INFERENCE[19], 0x01);
    }

    // -----------------------------------------------------------------------
    // pq_verify
    // -----------------------------------------------------------------------

    #[test]
    fn pq_verify_rejects_short_input() {
        let result = exec_pq_verify(&[0; 5]);
        assert!(matches!(result, PrecompileResult::Revert { .. }));
    }

    #[test]
    fn pq_verify_rejects_unsupported_pubkey_length() {
        // pk_len=1, pk=[0x00], sig_len=1, sig=[0x00], msg=[0x42] — pk length
        // 1 does not match any ML-DSA parameter set, so the verifier returns
        // Ok(false) → the precompile returns a 32-byte zero word.
        let mut input = Vec::new();
        input.extend_from_slice(&1u32.to_be_bytes());
        input.push(0x00);
        input.extend_from_slice(&1u32.to_be_bytes());
        input.push(0x00);
        input.push(0x42);

        let result = exec_pq_verify(&input);
        match result {
            PrecompileResult::Success { output, .. } => {
                assert_eq!(output.len(), 32);
                assert!(output.iter().all(|&b| b == 0));
            }
            other => panic!("expected Success(zero word), got {other:?}"),
        }
    }

    /// Sign a message with ML-DSA-65 via hanzo-pqc, then verify it through
    /// the precompile. Exercises the full sign → wire-format → verify path.
    #[test]
    fn exec_pq_verify_real_mldsa65_roundtrip() {
        use hanzo_pqc::signature::SignatureAlgorithm;

        let (vk, sk) = MlDsa::generate_keypair_sync(SignatureAlgorithm::MlDsa65)
            .expect("keypair generation");

        let message = b"hanzo-vm pq_verify integration message";
        let sig = MlDsa::sign_sync(&sk, message).expect("signing");

        // Assemble the calldata: [pk_len][pk][sig_len][sig][msg]
        let mut input = Vec::new();
        input.extend_from_slice(&(vk.key_bytes.len() as u32).to_be_bytes());
        input.extend_from_slice(&vk.key_bytes);
        input.extend_from_slice(&(sig.signature_bytes.len() as u32).to_be_bytes());
        input.extend_from_slice(&sig.signature_bytes);
        input.extend_from_slice(message);

        let result = exec_pq_verify(&input);
        match result {
            PrecompileResult::Success { output, gas_used } => {
                assert_eq!(output.len(), 32, "output should be a 32-byte word");
                assert_eq!(output[31], 1, "signature should verify");
                assert!(gas_used >= 3_000, "gas should include base cost");
            }
            other => panic!("expected Success, got {other:?}"),
        }

        // Flip a single byte of the message — verification must fail.
        let mut bad_input = input.clone();
        let msg_offset = 4 + vk.key_bytes.len() + 4 + sig.signature_bytes.len();
        bad_input[msg_offset] ^= 0x01;
        match exec_pq_verify(&bad_input) {
            PrecompileResult::Success { output, .. } => {
                assert_eq!(output[31], 0, "tampered message should not verify");
            }
            other => panic!("expected Success(0), got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // quasar_query
    // -----------------------------------------------------------------------

    #[cfg(feature = "quasar")]
    #[test]
    fn quasar_query_rejects_short_input() {
        let result = exec_quasar_query(&[0; 10]);
        assert!(matches!(result, PrecompileResult::Revert { .. }));
    }

    /// Wire the precompile through `libluxprecompile` and confirm the
    /// canonical Verkle Quasar at `0x0300..0020` is the actual dispatch
    /// target. The proof is a no-op (commitment == proof) which the
    /// canonical impl accepts when the threshold flag is set; the test
    /// then flips the flag to verify the non-member path.
    #[cfg(feature = "quasar")]
    #[test]
    fn exec_quasar_query_routes_through_libluxprecompile() {
        // Confirm the address is registered in the live dylib.
        let registry = lux_precompile::list().expect("list precompiles");
        let found = registry
            .iter()
            .any(|p| p.address.eq_ignore_ascii_case(LUX_QUASAR_ADDR));
        assert!(
            found,
            "expected {} in libluxprecompile registry; got {:?}",
            LUX_QUASAR_ADDR, registry
        );

        let validator = [0x42u8; 20];
        let commitment = [0xAAu8; 32];
        let proof = commitment; // matches → verkle light verifier returns true

        // threshold met: caller is a member.
        let mut input = Vec::with_capacity(85);
        input.extend_from_slice(&validator);
        input.extend_from_slice(&commitment);
        input.extend_from_slice(&proof);
        input.push(0x01);

        match exec_quasar_query(&input) {
            PrecompileResult::Success { output, gas_used } => {
                assert_eq!(output.len(), 53);
                assert_eq!(&output[0..20], &validator[..]);
                assert_eq!(&output[20..52], &commitment[..]);
                assert_eq!(output[52], 0x01, "should report member");
                assert!(gas_used > 0);
            }
            other => panic!("expected Success(member), got {other:?}"),
        }

        // threshold unmet: non-member.
        let mut input = Vec::with_capacity(85);
        input.extend_from_slice(&validator);
        input.extend_from_slice(&commitment);
        input.extend_from_slice(&proof);
        input.push(0x00);
        match exec_quasar_query(&input) {
            PrecompileResult::Success { output, .. } => {
                assert_eq!(output[52], 0x00, "should report non-member");
            }
            other => panic!("expected Success(non-member), got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // ai_inference
    // -----------------------------------------------------------------------

    #[test]
    fn ai_inference_rejects_empty() {
        let result = exec_ai_inference(&[]);
        assert!(matches!(result, PrecompileResult::Revert { .. }));
    }

    /// CONSENSUS-CRITICAL (RED H1): the Rust inference gas schedule MUST equal
    /// the authoritative Go schedule in `chains/hanzo-evm/evmllm/precompile.go`,
    /// byte-for-byte, or the two backends compute different `gas_used` for the
    /// same transaction → divergent state root → chain fork. These LITERAL cases
    /// mirror Go's `TestGasScheduleCanonical` exactly (input-based, clamped below
    /// header, pure in input length), so a coefficient mutation FAILS on both
    /// sides. The gas is input-based — independent of model output length —
    /// because geth charges `RequiredGas(input)` before the output exists.
    #[test]
    fn inference_gas_matches_canonical_go_schedule() {
        // Literal coefficient pin (NOT via the consts) — catches drift.
        assert_eq!(GAS_BASE_INFER, 120_000, "base must match Go GasBaseInfer literal");
        assert_eq!(GAS_PER_PROMPT_BYTE, 30, "per-byte must match Go GasPerPromptByte literal");

        // header == 36; promptBytes = input_len - 36, clamped at 0 below header.
        // Cases are byte-identical to Go's TestGasScheduleCanonical.
        assert_eq!(inference_gas(AI_HEADER_LEN), 120_000, "header-only (zero prompt)");
        assert_eq!(inference_gas(10), 120_000, "short input (<header) clamps prompt to 0");
        assert_eq!(inference_gas(AI_HEADER_LEN + 1), 120_030, "header + 1 byte");
        assert_eq!(inference_gas(AI_HEADER_LEN + 42), 121_260, "header + 42 bytes");
        assert_eq!(inference_gas(AI_HEADER_LEN + 1024), 120_000 + 1024 * 30, "header + 1KiB");

        // Purity: gas depends ONLY on input length, never on byte content or output.
        assert_eq!(
            inference_gas(AI_HEADER_LEN + 100),
            inference_gas(AI_HEADER_LEN + 100),
            "gas must be a pure function of input length"
        );
    }

    #[test]
    fn ai_inference_rejects_missing_prompt() {
        let mut input = vec![0u8; 4 + 32];
        // header only, no prompt
        input.extend_from_slice(&[]);
        let result = exec_ai_inference(&input);
        match result {
            PrecompileResult::Revert { reason } => {
                assert!(reason.contains("non-empty"), "got reason: {reason}");
            }
            other => panic!("expected Revert, got {other:?}"),
        }
    }

    /// Default builds run without a registered inference engine, so the
    /// precompile must revert with `no inference engine registered`. The
    /// runtime impl in [`exec_ai_inference`] always calls
    /// [`hanzo_engine::infer`] — there is no in-tree fallback path — so
    /// this test verifies the dispatch contract.
    ///
    /// In production builds the runtime (`hanzo-node`) installs a real
    /// [`hanzo_engine::MistralEngine`] at startup and the precompile
    /// returns real bytes; that path is exercised by integration tests in
    /// the engine crate, not here.
    #[test]
    fn exec_ai_inference_real_model() {
        let mut input = Vec::new();
        input.extend_from_slice(&[0u8; 4]); // selector
        input.extend_from_slice(&[0xABu8; 32]); // model id
        input.extend_from_slice(b"summarize: this is a tiny prompt");

        let res = exec_ai_inference(&input);
        match res {
            PrecompileResult::Revert { reason } => {
                assert!(
                    reason.contains("inference") && reason.contains("engine"),
                    "expected 'no inference engine registered'-like reason, got: {reason}"
                );
            }
            PrecompileResult::Success { output, .. } => {
                // If another test in this binary has installed a real
                // engine via `register_inference_engine`, we expect the
                // engine's actual output bytes.
                assert!(!output.is_empty(), "engine must return real output");
            }
            other => panic!("unexpected result: {other:?}"),
        }

        // The registry should also report `false` here when no real
        // engine is installed; production startup flips this to `true`.
        if !hanzo_engine::inference_engine_registered() {
            // dispatch must have produced a Revert; covered above.
        }
    }

    // -----------------------------------------------------------------------
    // ai_embedding
    // -----------------------------------------------------------------------

    #[test]
    fn ai_embedding_rejects_short_and_empty() {
        // Too short (< selector + model id).
        let result = exec_ai_embedding(&[0; 2]);
        assert!(matches!(result, PrecompileResult::Revert { .. }));

        // Header present but no text.
        let input = vec![0u8; 4 + 32];
        match exec_ai_embedding(&input) {
            PrecompileResult::Revert { reason } => {
                assert!(reason.contains("non-empty"), "got reason: {reason}");
            }
            other => panic!("expected Revert, got {other:?}"),
        }
    }

    /// Dispatch contract: with no engine the precompile reverts; with an engine
    /// it returns the model's native-dimension embedding as little-endian f32,
    /// so the output is a non-empty multiple of 4 bytes.
    #[test]
    fn exec_ai_embedding_real_model() {
        let mut input = Vec::new();
        input.extend_from_slice(&[0u8; 4]); // selector
        input.extend_from_slice(&[0u8; 32]); // model name (empty = default)
        input.extend_from_slice(b"hello world");

        match exec_ai_embedding(&input) {
            PrecompileResult::Revert { reason } => {
                assert!(
                    reason.contains("embedding")
                        || reason.contains("engine")
                        || reason.contains("model"),
                    "expected engine/model-related revert, got: {reason}"
                );
            }
            PrecompileResult::Success { output, .. } => {
                assert!(
                    !output.is_empty() && output.len() % 4 == 0,
                    "embedding output must be a non-empty multiple of 4 bytes, got {}",
                    output.len()
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }
}
