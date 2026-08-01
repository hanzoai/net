<p align="center"><img src=".github/hero.svg" alt="Hanzo Net" width="880"></p>

# Hanzo Net

Hanzo Net is a Rust workspace of 36 crates for running an AI cluster on machines you
already own — peer-to-peer over libp2p, with on-device inference, an agent and tool
runtime, post-quantum identity, and an embedded L2 that meters and settles compute.

<p>
  <img src="https://img.shields.io/badge/license-MIT-black" alt="MIT">
  <img src="https://img.shields.io/badge/rust-2021-black" alt="Rust 2021">
  <img src="https://img.shields.io/badge/crates-36-black" alt="36 crates">
</p>

## Status — read this before you clone

This workspace is a set of libraries under active development. Three things you need to
know up front, because none of them are obvious from the crate list:

- **It does not build from a public clone.** `hanzo-vm` takes path dependencies on
  `../../engine/hanzo-engine` and `../../engine/hanzo-server-core`, expecting a sibling
  checkout named `engine/`. The repository that provides those crates is not public, so
  `cargo build` fails at workspace-manifest resolution — and because that failure happens
  before member selection, `cargo build -p <crate>` fails the same way. Fixing this is
  tracked work, not a configuration you can supply.
- **Nothing here is published to crates.io.** The crate names below are the names in this
  workspace; they are not package names you can `cargo add`. Depend on them by path.
- **There is no node binary in this workspace.** These are libraries. The older
  standalone node lives under `_archived-from-zoo/` and is not maintained.

If you want to run Hanzo models today, use the [Hanzo CLI](https://github.com/hanzoai/cli)
(`curl -fsSL https://hanzo.sh | sh`) against the hosted API. This repository is where the
self-hosted cluster is being built.

## What is in here

| Layer | Crates | What it does |
|-------|--------|--------------|
| **Networking** | `hanzo-libp2p`, `hanzo-libp2p-relayer`, `hanzo-messages` | Peer-to-peer mesh, relaying for NAT traversal, typed node messaging. |
| **Identity & crypto** | `hanzo-identity`, `hanzo-did`, `hanzo-pqc`, `hanzo-zap` | Post-quantum node identity (ML-KEM, ML-DSA, SLH-DSA), DIDs, authorization. |
| **Runtime & inference** | `hanzo-runtime`, `hanzo-wasm`, `hanzo-wasm-runtime`, `hanzo-embed`, `hanzo-models`, `hanzo-model-discovery` | On-device model execution, WASM sandboxing, embeddings, model discovery. |
| **Agents & tools** | `hanzo-agentic`, `hanzo-tools`, `hanzo-tools-runner`, `hanzo-runner`, `hanzo-mcp`, `hanzo-jobs`, `hanzo-job-queue-manager` | Agent orchestration, sandboxed tool execution, Model Context Protocol, the job queue. |
| **Data & state** | `hanzo-database`, `hanzo-db-sqlite`, `hanzo-fs`, `hanzo-api`, `hanzo-http-api` | Storage, filesystem, and the node's local `/v1` HTTP surface. |
| **Cluster economics** | `hanzo-compute`, `hanzo-machine`, `hanzo-mining`, `hanzo-hmm`, `hanzo-brain` | Compute accounting, VM lifecycle, and Hamiltonian market-maker pricing across heterogeneous hardware. |
| **Settlement** | `hanzo-consensus`, `hanzo-vm`, `hanzo-l2` | Quasar BFT consensus, an EVM with post-quantum and inference precompiles, and an L2 bridge on Lux Network. |

Plus `hanzo-ai-format`, `hanzo-config` and `hanzo-runtime-tests`. The full member list is
in the root `Cargo.toml`.

## Crate naming

The directory name is `hanzo-*` for every crate. Four of them declare a `hanzonet-*`
package name instead, because the plain name belongs to the consumer SDK in
[`hanzo-rs/sdk`](https://github.com/hanzo-rs/sdk): `hanzo-pqc`, `hanzo-did`,
`hanzo-config` and `hanzo-mcp` are packaged as `hanzonet-pqc`, `hanzonet-did`,
`hanzonet-config` and `hanzonet-mcp`.

The Rust library name is always `hanzo_*`, so `use hanzo_vm::…;` reads the same either way.

## Working in the tree

```bash
git clone https://github.com/hanzoai/net
cd net
cargo test -p <crate>        # once the workspace resolves; see Status above
```

Each crate owns its own tests. `LLM.md` has the layout notes and the conventions that
apply inside this repo.

## License

MIT © Hanzo AI, Inc. See [LICENSE](LICENSE).

---

Hanzo — the open AI cloud. [hanzo.ai](https://hanzo.ai) · [docs.hanzo.ai](https://docs.hanzo.ai)

SDKs: [Python](https://github.com/hanzoai/python-sdk) · [TypeScript](https://github.com/hanzo-js/sdk) · [Go](https://github.com/hanzo-go/sdk) · [Rust](https://github.com/hanzo-rs/sdk) · [C++](https://github.com/hanzo-cpp/sdk) · [Swift](https://github.com/hanzo-swift/sdk) · [Kotlin](https://github.com/hanzo-kt/sdk) · [umbrella](https://github.com/hanzoai/sdk)
