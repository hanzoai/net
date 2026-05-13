# hanzonet/network — archived

> **This monorepo has been split into 36 focused repositories** under the
> [`hanzonet`](https://github.com/hanzonet) GitHub organization. Each crate
> now lives in its own repo with its own CI and ship cadence.

## Where each crate lives now

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

## Naming convention

- **`hanzonet-*`** = network-internal implementations of names that collide with
  the consumer SDK in [`hanzoai/rust-sdk`](https://github.com/hanzoai/rust-sdk).
  Crates: `hanzonet-pqc`, `hanzonet-did`, `hanzonet-config`, `hanzonet-mcp`.

  To use them with their original import name:
  ```toml
  hanzo-pqc = { version = "1.1.21", package = "hanzonet-pqc" }
  ```

- **`hanzo-*`** = everything else. The Rust library name is always `hanzo_*`,
  so `use hanzo_vm::...` works regardless of which package on crates.io
  served the crate.

## Why split?

One repo per focused concern. Each repo has its own CI, its own ship cadence,
its own issues, and a clear scope. Easier to grep, easier to depend on,
easier to evolve independently.

## License

MIT.
