# Building Bitcoin in Rust v2

Implementation of the `Building bitcoin in Rust` guide as a Rust Cargo workspace.

This is a toy bitcoin-like system for learning. It is not Bitcoin Core compatible and must not be used with real money.

## Workspace

- `lib`: `btclib`, shared hashes, crypto, transactions, blocks, blockchain validation, serialization, and network messages.
- `miner`: CPU miner that fetches templates from a node and submits mined blocks.
- `node`: async TCP node with chain state, mempool, template generation, UTXO lookup, and persistence.
- `wallet`: wallet core plus interactive CLI commands for config, balance, contacts, and sending transactions.

## Build And Test

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Generate Keys

```bash
cargo run -p btclib --bin key_gen -- alice
cargo run -p btclib --bin key_gen -- bob
```

This creates `alice.priv`, `alice.pub`, `bob.priv`, and `bob.pub`.

## Run A Node

```bash
cargo run -p node -- --port 9000 --blockchain-file ./blockchain.cbor
```

## Run A Miner

In another terminal:

```bash
cargo run -p miner -- --address 127.0.0.1:9000 --public-key-file alice.pub
```

## Run The Wallet

Create a config:

```bash
cargo run -p wallet -- generate-config --output wallet_config.toml
```

Edit `wallet_config.toml` to point at the generated key files, then run:

```bash
cargo run -p wallet -- --config wallet_config.toml --node 127.0.0.1:9000
```

Interactive commands:

```text
balance
contacts
send <recipient> <amount-sats>
exit
```

## Docs

- [Architecture](docs/architecture.md)
- [Runbook](docs/runbook.md)
- [Network Protocol](docs/network-protocol.md)
- [Commit Standard](docs/commit-standard.md)
