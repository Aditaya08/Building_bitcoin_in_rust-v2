# Building Bitcoin in Rust v2

This repository implements the project from `Building bitcoin in Rust` as a Cargo workspace.

## Workspace

- `lib`: shared `btclib` primitives, blockchain validation, serialization, and networking.
- `miner`: CPU miner that asks a node for templates and submits mined blocks.
- `node`: toy bitcoin-like node with a mempool, chain state, peer messages, and persistence.
- `wallet`: CLI/TUI-style wallet core for balances and transactions.

## Quick Check

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

More detailed runbooks live in `docs/`.
