# Hanzo Network — Rust workspace

The chain layer of [Hanzo Network](https://hanzo.network) — VM, consensus,
marketmaker, machine lifecycle, identity, post-quantum crypto, runtime,
tools. 36 crates published on [crates.io](https://crates.io), all building
from this one workspace.

- Repository: `github.com/hanzonet/network`
- Org: `github.com/hanzonet`
- License: MIT
- Rust: edition 2021

## Naming

Two distinct namespaces, one canonical implementation each:

| Prefix       | Org & repo                          | Purpose |
|--------------|-------------------------------------|---------|
| `hanzonet-*` | `hanzonet/network` (this repo)      | Network-internal primitives that diverged from the SDK |
| `hanzo-*`    | `hanzoai/rust-sdk` and this repo    | Consumer-facing SDK crates **plus** the network-only crates that don't conflict with the SDK |

The four names that collided (`pqc`, `did`, `config`, `mcp`) live under
`hanzonet-*` on crates.io but keep `hanzo_*` as their Rust library name so
consumer code reads unchanged:

```toml
[dependencies]
# crates.io serves hanzonet-pqc, your code imports it as hanzo_pqc
hanzo-pqc = { version = "1.1.21", package = "hanzonet-pqc" }
```

```rust
use hanzo_pqc::kem::MlKem768;
```

## Crates

### Chain layer

- [`hanzo-vm`](https://crates.io/crates/hanzo-vm) 1.1.21 — EVM with precompiles for PQ verify (`0x0101`), Quasar (`0x0102`), AI inference (`0x0201`), AI embedding (`0x0202`)
- [`hanzo-consensus`](https://crates.io/crates/hanzo-consensus) 1.1.21 — Quasar BFT consensus engine for L2 (wraps `lux-consensus`)
- [`hanzo-l2`](https://crates.io/crates/hanzo-l2) 1.1.21 — L2 bridge and sequencing on Lux Network
- [`hanzo-mining`](https://crates.io/crates/hanzo-mining) 1.1.21 — mining / staking primitives
- [`hanzo-machine`](https://crates.io/crates/hanzo-machine) 1.1.21 — VM lifecycle wrapper (Apple Virtualization.framework via vfkit on macOS; KVM on Linux)

### Cryptography & identity

- [`hanzonet-pqc`](https://crates.io/crates/hanzonet-pqc) 1.1.21 — post-quantum cryptography (ML-KEM, ML-DSA, SLH-DSA, hybrid, privacy tiers, attestation)
- [`hanzonet-did`](https://crates.io/crates/hanzonet-did) 1.1.21 — W3C DID library for the network node
- [`hanzo-identity`](https://crates.io/crates/hanzo-identity) 1.1.13 — identity primitives

### Marketmaker / pricing

- [`hanzo-hmm`](https://crates.io/crates/hanzo-hmm) 0.1.2 — Hidden Markov Model primitives + Hamiltonian MarketMaker for pricing heterogeneous compute

### Runtime, tools, agents

- [`hanzo-runtime`](https://crates.io/crates/hanzo-runtime) 1.1.13 — tool execution runtime
- [`hanzo-runtime-tests`](https://crates.io/crates/hanzo-runtime-tests) 0.1.2 — runtime test harness
- [`hanzo-runner`](https://crates.io/crates/hanzo-runner) 1.1.13 — tool runner
- [`hanzo-tools`](https://crates.io/crates/hanzo-tools) 1.1.21 — tool primitives
- [`hanzo-tools-runner`](https://crates.io/crates/hanzo-tools-runner) 1.0.2 — tool execution
- [`hanzo-wasm`](https://crates.io/crates/hanzo-wasm) 0.1.2 — WASM module primitives
- [`hanzo-wasm-runtime`](https://crates.io/crates/hanzo-wasm-runtime) 1.1.21 — WASM execution runtime
- [`hanzo-jobs`](https://crates.io/crates/hanzo-jobs) 1.1.13 — job orchestration
- [`hanzo-job-queue-manager`](https://crates.io/crates/hanzo-job-queue-manager) 1.1.21 — job queue manager
- [`hanzo-brain`](https://crates.io/crates/hanzo-brain) 0.1.0 — agent brain
- [`hanzo-agentic`](https://crates.io/crates/hanzo-agentic) 1.1.21 — agentic network integration with post-quantum privacy

### Networking & API

- [`hanzo-libp2p`](https://crates.io/crates/hanzo-libp2p) 1.1.13 — libp2p wrapper
- [`hanzo-libp2p-relayer`](https://crates.io/crates/hanzo-libp2p-relayer) 1.1.13 — libp2p relay node
- [`hanzo-api`](https://crates.io/crates/hanzo-api) 1.1.13 — API surface
- [`hanzo-http-api`](https://crates.io/crates/hanzo-http-api) 1.1.21 — HTTP API
- [`hanzonet-mcp`](https://crates.io/crates/hanzonet-mcp) 1.1.21 — node-internal MCP client/server adapter
- [`hanzo-zap`](https://crates.io/crates/hanzo-zap) 0.6.75 — zero-allocation P2P wire helper

### Configuration, storage, messaging

- [`hanzonet-config`](https://crates.io/crates/hanzonet-config) 1.1.21 — node configuration library
- [`hanzo-database`](https://crates.io/crates/hanzo-database) 1.1.13 — database primitives
- [`hanzo-db-sqlite`](https://crates.io/crates/hanzo-db-sqlite) 1.1.13 — SQLite backend
- [`hanzo-fs`](https://crates.io/crates/hanzo-fs) 1.1.21 — filesystem primitives
- [`hanzo-messages`](https://crates.io/crates/hanzo-messages) 1.1.13 — message schemas
- [`hanzo-embed`](https://crates.io/crates/hanzo-embed) 1.1.13 — embedding primitives

### Models & AI

- [`hanzo-models`](https://crates.io/crates/hanzo-models) 1.1.13 — model registry
- [`hanzo-model-discovery`](https://crates.io/crates/hanzo-model-discovery) 1.1.13 — model discovery
- [`hanzo-ai-format`](https://crates.io/crates/hanzo-ai-format) 1.1.21 — AI artifact format
- [`hanzo-compute`](https://crates.io/crates/hanzo-compute) 1.1.21 — compute primitives

## Related published crates

| Repo                | Crates |
|---------------------|--------|
| `hanzoai/engine`    | [`hanzo-engine`](https://crates.io/crates/hanzo-engine) 0.6.0 — canonical LLM inference + embedding engine (mistral.rs-based) |
| `hanzoai/rust-sdk`  | 14 SDK crates at 1.1.21 / 0.1.x — `hanzo-pqc`, `hanzo-did`, `hanzo-config`, `hanzo-mcp`/`-core`/`-client`/`-server`, `hanzo-message-primitives`, `hanzo-agent`, `hanzo-agent-proxy`, `hanzo-agents`, `hanzo-guard`, `hanzo-crypto`, `hanzo-extract` |
| `luxfi/precompile`  | [`luxprecompile-sys`](https://crates.io/crates/luxprecompile-sys) 0.1.0 — FFI to `libluxprecompile` Go shared library |
| `luxfi/consensus`   | [`lux-consensus`](https://crates.io/crates/lux-consensus) 1.22.0 — Quasar BFT consensus SDK |

## Build

```sh
cargo build --workspace
```

The `hanzo-vm` tests need the canonical Lux precompile dylib at runtime:

```sh
DYLD_LIBRARY_PATH=/Users/z/work/lux/precompile/dist \
  cargo test -p hanzo-vm --lib precompiles
```

## License

MIT. See `LICENSE`.
