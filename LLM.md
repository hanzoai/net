# LLM.md — hanzoai/net

Guidance for AI agents working in this repo.

## What this is
**Hanzo Net** — run your own distributed AI cluster in Rust across everyday
devices (iPhone, iPad, Mac, NVIDIA, Raspberry Pi). Peer-to-peer over libp2p,
on-device inference/agents/tools, post-quantum identity, embedded L2 settlement.

## Canonical role
This is the **canonical Rust workspace** for the Hanzo Net node — a self-hostable
distributed AI cluster. It is a first-party Rust product (alongside engine, ml,
node, router, pqc, evm), NOT one of the language SDKs. The full cloud SDK and the
AI/agents lib live elsewhere (see cross-links / SDK-ARCHITECTURE.md).

## Layout
- Cargo workspace, 36 member crates (`hanzo-*`); Rust lib name is always `hanzo_*`.
- `hanzonet-*` package names = network-internal impls that collide with the Rust
  consumer SDK (`hanzonet-pqc`, `hanzonet-did`, `hanzonet-config`, `hanzonet-mcp`);
  import via `package = "hanzonet-…"`.
- Key entry points: `hanzo-libp2p` (mesh), `hanzo-runtime`/`hanzo-vm` (execution),
  `hanzo-agentic`/`hanzo-runner`/`hanzo-mcp` (agents+tools), `hanzo-http-api` (`/v1`),
  `hanzo-consensus`/`hanzo-l2` (settlement). Older node binary under `_archived-from-zoo`.

## Build / run
- `cargo build --release` · `cargo test` · `cargo test -p <crate>`.

## Brand rules (hard)
- Never call this an "LLM gateway"; never position vs LiteLLM. Hanzo is a full
  AI cloud, not a proxy.
- Zen models are our own family — never name upstream models.
- HTTP paths are `/v1/…`, never `/api/…`.
- Voice: "Hanzo — the Open AI Cloud." Modern, crisp, developer-first.

## More
Canonical SDK/discoverability model: `~/work/hanzo/SDK-ARCHITECTURE.md`.
