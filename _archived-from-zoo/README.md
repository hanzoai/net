# Hanzo Rust content archived from zoo/node

Provenance: All content here was misplaced under `~/work/zoo/node/` (the
canonical Zoo node — which is and always was meant to be Go, a
luxfi/node fork like `~/work/liquidity/node`). The Rust workspace
predated the Go rewrite and was kept dormant in the working tree.
On 2026-05-21 it was lifted into this archive directory inside
`~/work/hanzo/net/_archived-from-zoo/` so the Zoo node can be a clean
Go-only repository.

## Layout

| Subdir | Contents |
|--------|----------|
| `hanzo-libs/` | 29 Hanzo `*-libs` crates that ALSO live in `../hanzo-*/` (the live workspace). Older snapshot. Kept for diffing if anything was lost. |
| `unique-crates/` | 8 crates that were ONLY in zoo/node, never imported into hanzo/net. See table below. |
| `hanzo-bin/` | `hanzo-node`, `hanzoai`, `hanzo-migrate` binary crates (older fork). Live versions in `~/work/hanzo/node-go/hanzo-bin/`. |
| `hanzo-test-framework/` | Older test framework. Live: `~/work/hanzo/node-go/hanzo-test-framework/` |
| `hanzo-test-macro/` | Older proc-macro for tests. Live: `~/work/hanzo/node-go/hanzo-test-macro/` |
| `hanzod/` | `hanzod` config TypeScript (single file). Hanzo daemon config probe. |
| `zoo-flavored/` | `zoo-libs`, `zoo-bin`, `zoo-test-{framework,macro}` — incomplete rebrand attempt of hanzo-libs. Already being reverted upstream (`784c990c2`). Not wired into any workspace. |

## unique-crates/

| Crate | Status | Notes |
|-------|--------|-------|
| `hanzo-baml` | exploratory | BAML (Boundary AI Markup Language) integration probe |
| `hanzo-db` | not-published | Old DB layer superseded by hanzo-database / hanzo-db-sqlite |
| `hanzo-kbs` | incomplete | Knowledge Base Service skeleton, missing types |
| `hanzo-llm` | superseded | Old LLM client, now in hanzo-engine |
| `hanzo-sheet` | depends-on-missing | Pulled `hanzo-sheet` module that doesn't exist |
| `hanzo-simulation` | exploratory | Sim harness probe |
| `hanzo-sovereign` | superseded | L1 sovereign scaffold, now in hanzo-l2 + hanzo-consensus |
| `hanzo-tests` | test-only | Compilation errors, replaced by hanzo-test-framework |

If any of these get resurrected, promote them into the main `hanzo-libs/`
workspace and re-add to the workspace `members` list in `Cargo.toml`.
Otherwise this directory is reference material only — NOT compiled by
the workspace.

Provenance: moved 2026-05-21 from `~/work/zoo/node/hanzo-libs/` working
tree (originated from prior monorepo when Zoo and Hanzo nodes shared
the same Rust workspace). The remaining 29 crates that lived in
`zoo/node/hanzo-libs/` already exist in this workspace, so they were
deleted from `zoo/node/` rather than copied.

## zoo-flavored/

A parallel `zoo-{libs,bin,test-framework,test-macro}/` Rust hierarchy
also lived under `~/work/zoo/node/`. It was a partial rebrand of the
Hanzo workspace (e.g. `zoo-mcp` was just `hanzo-mcp` renamed) and was
NOT wired into the Cargo workspace. Initial commit `f92a727ab`. Last
real work `784c990c2 refactor: replace zoo-embedding with hanzo-embed
(DRY)` — the rebrand was being reverted in the same direction as this
cleanup (Zoo abandons Rust, uses Hanzo Rust libs directly via Go FFI
or HTTP).

Preserved under `zoo-flavored/` as historical reference only. NOT
compiled. NOT to be revived — Zoo went all-Go in 2026-Q2.
