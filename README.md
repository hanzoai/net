<p align="center"><img src=".github/hero.svg" alt="Hanzo Net" width="880"></p>

# Hanzo Net

**Run your own distributed AI cluster — in Rust, across every device you own.**

Hanzo Net turns the machines you already have — iPhone, iPad, Mac, an NVIDIA box,
a Raspberry Pi — into one private, peer-to-peer AI cluster. Models, agents, tools,
and inference run on your hardware, meshed together over libp2p, with post-quantum
identity and on-chain settlement built in. No datacenter, no proxy, no lock-in.

This repository is the **canonical Rust workspace** behind Hanzo Net: a set of
focused crates that compose into a self-hostable node.

<p>
  <img src="https://img.shields.io/badge/license-MIT-black" alt="MIT">
  <img src="https://img.shields.io/badge/rust-2021-black" alt="Rust 2021">
  <img src="https://img.shields.io/badge/crates-36-black" alt="36 crates">
  <img src="https://img.shields.io/badge/post--quantum-native-black" alt="Post-quantum native">
</p>

## Why Hanzo Net

- **Your hardware, your cluster.** Pool everyday devices into one inference fabric —
  each node contributes compute, storage, and models.
- **Peer-to-peer by design.** libp2p mesh with a relayer for NAT traversal; no
  central coordinator to trust or pay.
- **Agents, tools, and MCP on-device.** A full agentic runtime — job queue, tool
  runner, and Model Context Protocol — executes locally and safely.
- **Post-quantum from the identity up.** ML-KEM, ML-DSA, and SLH-DSA underpin node
  identity, messaging, and attestation.
- **On-chain settlement.** An embedded L2 (Quasar BFT, EVM with AI precompiles)
  meters and settles heterogeneous compute.
- **Zen models, natively.** Serve the Zen model family — Hanzo's own architecture —
  alongside your local weights.

## Quickstart

Build the whole workspace from source:

```bash
git clone https://github.com/hanzoai/net
cd net
cargo build --release
cargo test
```

Or depend on an individual crate from crates.io:

```toml
[dependencies]
hanzo-vm      = "1.1"
hanzo-runtime = "1.1"
# crates whose name collides with the consumer SDK ship under a hanzonet-* package:
hanzo-pqc     = { version = "1.1", package = "hanzonet-pqc" }
```

The Rust library name is always `hanzo_*`, so `use hanzo_vm::…;` works regardless of
which package on crates.io served the crate.

A running node exposes a local HTTP API under `/v1` for models, jobs, and tools.

## Architecture

Hanzo Net is composed, not monolithic. The core pieces:

| Layer | Crates | What it does |
|-------|--------|--------------|
| **Networking** | `hanzo-libp2p`, `hanzo-libp2p-relayer`, `hanzo-messages` | Peer-to-peer mesh, relaying, and typed node messaging. |
| **Identity & crypto** | `hanzo-identity`, `hanzo-did`, `hanzo-pqc`, `hanzo-zap` | Post-quantum node identity, DIDs, and authorization. |
| **Runtime & inference** | `hanzo-runtime`, `hanzo-wasm`, `hanzo-wasm-runtime`, `hanzo-embed`, `hanzo-models`, `hanzo-model-discovery` | On-device model execution, WASM sandboxing, embeddings, and model discovery. |
| **Agents & tools** | `hanzo-agentic`, `hanzo-tools`, `hanzo-tools-runner`, `hanzo-runner`, `hanzo-mcp`, `hanzo-jobs`, `hanzo-job-queue-manager` | Agentic orchestration, safe tool execution, and the Model Context Protocol. |
| **Data & state** | `hanzo-database`, `hanzo-db-sqlite`, `hanzo-fs`, `hanzo-api`, `hanzo-http-api` | Storage, filesystem, and the local `/v1` HTTP surface. |
| **Cluster economics** | `hanzo-compute`, `hanzo-machine`, `hanzo-mining`, `hanzo-hmm`, `hanzo-brain` | Compute accounting, VM lifecycle, and Hamiltonian market-maker pricing for heterogeneous hardware. |
| **On-chain settlement** | `hanzo-consensus`, `hanzo-vm`, `hanzo-l2` | Quasar BFT consensus, an EVM with PQ / Quasar / AI-inference precompiles, and an L2 bridge on Lux Network. |

### All crates

Every workspace member is also published to crates.io and mirrored as a standalone
repository for focused issues and CI.

<details>
<summary><strong>36 crates → standalone repos</strong></summary>

| Crate (crates.io)        | Repo                                                        |
|--------------------------|-------------------------------------------------------------|
| `hanzo-agentic`          | [`hanzonet/agentic`](https://github.com/hanzonet/agentic)             |
| `hanzo-ai-format`        | [`hanzonet/ai-format`](https://github.com/hanzonet/ai-format)         |
| `hanzo-api`              | [`hanzonet/api`](https://github.com/hanzonet/api)                     |
| `hanzo-brain`            | [`hanzonet/brain`](https://github.com/hanzonet/brain)                 |
| `hanzo-compute`          | [`hanzonet/compute`](https://github.com/hanzonet/compute)             |
| `hanzonet-config`        | [`hanzonet/config`](https://github.com/hanzonet/config)               |
| `hanzo-consensus`        | [`hanzonet/consensus`](https://github.com/hanzonet/consensus)         |
| `hanzo-database`         | [`hanzonet/database`](https://github.com/hanzonet/database)           |
| `hanzo-db-sqlite`        | [`hanzonet/db-sqlite`](https://github.com/hanzonet/db-sqlite)         |
| `hanzonet-did`           | [`hanzonet/did`](https://github.com/hanzonet/did)                     |
| `hanzo-embed`            | [`hanzonet/embed`](https://github.com/hanzonet/embed)                 |
| `hanzo-fs`               | [`hanzonet/fs`](https://github.com/hanzonet/fs)                       |
| `hanzo-hmm`              | [`hanzonet/hmm`](https://github.com/hanzonet/hmm)                     |
| `hanzo-http-api`         | [`hanzonet/http-api`](https://github.com/hanzonet/http-api)           |
| `hanzo-identity`         | [`hanzonet/identity`](https://github.com/hanzonet/identity)           |
| `hanzo-job-queue-manager`| [`hanzonet/job-queue-manager`](https://github.com/hanzonet/job-queue-manager) |
| `hanzo-jobs`             | [`hanzonet/jobs`](https://github.com/hanzonet/jobs)                   |
| `hanzo-l2`               | [`hanzonet/l2`](https://github.com/hanzonet/l2)                       |
| `hanzo-libp2p`           | [`hanzonet/libp2p`](https://github.com/hanzonet/libp2p)               |
| `hanzo-libp2p-relayer`   | [`hanzonet/libp2p-relayer`](https://github.com/hanzonet/libp2p-relayer) |
| `hanzo-machine`          | [`hanzonet/machine`](https://github.com/hanzonet/machine)             |
| `hanzonet-mcp`           | [`hanzonet/mcp`](https://github.com/hanzonet/mcp)                     |
| `hanzo-messages`         | [`hanzonet/messages`](https://github.com/hanzonet/messages)           |
| `hanzo-mining`           | [`hanzonet/mining`](https://github.com/hanzonet/mining)               |
| `hanzo-model-discovery`  | [`hanzonet/model-discovery`](https://github.com/hanzonet/model-discovery) |
| `hanzo-models`           | [`hanzonet/models`](https://github.com/hanzonet/models)               |
| `hanzonet-pqc`           | [`hanzonet/pqc`](https://github.com/hanzonet/pqc)                     |
| `hanzo-runner`           | [`hanzonet/runner`](https://github.com/hanzonet/runner)               |
| `hanzo-runtime`          | [`hanzonet/runtime`](https://github.com/hanzonet/runtime)             |
| `hanzo-runtime-tests`    | [`hanzonet/runtime-tests`](https://github.com/hanzonet/runtime-tests) |
| `hanzo-tools`            | [`hanzonet/tools`](https://github.com/hanzonet/tools)                 |
| `hanzo-tools-runner`     | [`hanzonet/tools-runner`](https://github.com/hanzonet/tools-runner)   |
| `hanzo-vm`               | [`hanzonet/vm`](https://github.com/hanzonet/vm)                       |
| `hanzo-wasm`             | [`hanzonet/wasm`](https://github.com/hanzonet/wasm)                   |
| `hanzo-wasm-runtime`     | [`hanzonet/wasm-runtime`](https://github.com/hanzonet/wasm-runtime)   |
| `hanzo-zap`              | [`hanzonet/zap`](https://github.com/hanzonet/zap)                     |

</details>

## Naming convention

- **`hanzonet-*`** = network-internal implementations of names that collide with
  the consumer SDK in [`hanzo-rs/sdk`](https://github.com/hanzo-rs/sdk). Crates:
  `hanzonet-pqc`, `hanzonet-did`, `hanzonet-config`, `hanzonet-mcp`. Import them
  under their original name via a package rename:

  ```toml
  hanzo-pqc = { version = "1.1", package = "hanzonet-pqc" }
  ```

- **`hanzo-*`** = everything else. The Rust library name is always `hanzo_*`, so
  `use hanzo_vm::…;` works regardless of which package on crates.io served the crate.

## Contributing

One repo per focused concern, one workspace to build them together. Each crate has
a clear scope and its own tests. `cargo test` from the root runs the full suite;
`cargo test -p <crate>` targets one.

## License

MIT © Hanzo AI, Inc. See [LICENSE](LICENSE).

## Hanzo — the Open AI Cloud

Open source · every language · on-chain settlement. [hanzo.ai](https://hanzo.ai) · [docs.hanzo.ai](https://docs.hanzo.ai)

**SDKs in every language** — [Python](https://github.com/hanzoai/python-sdk) (flagship) · [TypeScript](https://github.com/hanzo-js/sdk) · [Go](https://github.com/hanzo-go/sdk) · [Rust](https://github.com/hanzo-rs/sdk) · [C++](https://github.com/hanzo-cpp/sdk) · [Swift](https://github.com/hanzo-swift/sdk) · [Kotlin](https://github.com/hanzo-kt/sdk) · [umbrella](https://github.com/hanzoai/sdk)
