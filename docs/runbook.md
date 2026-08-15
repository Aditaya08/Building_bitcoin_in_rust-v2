# Runbook

## Prerequisites

- Rust stable toolchain
- Network access for the first `cargo` dependency download

## Verify The Workspace

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Local Demo

Generate keys:

```bash
cargo run -p btclib --bin key_gen -- alice
cargo run -p btclib --bin key_gen -- bob
```

Start the node:

```bash
cargo run -p node -- --port 9000 --blockchain-file ./blockchain.cbor
```

Start the miner in a second terminal:

```bash
cargo run -p miner -- --address 127.0.0.1:9000 --public-key-file alice.pub
```

Create wallet config:

```bash
cargo run -p wallet -- generate-config --output wallet_config.toml
```

Edit the config so Alice owns funds and Bob is a contact:

```toml
default_node = "127.0.0.1:9000"

[fee_config]
fee_type = "Percent"
value = 0.1

[[my_keys]]
public = "alice.pub"
private = "alice.priv"

[[contacts]]
name = "Bob"
key = "bob.pub"
```

Run the wallet:

```bash
cargo run -p wallet -- --config wallet_config.toml --node 127.0.0.1:9000
```

Useful wallet commands:

```text
balance
contacts
send Bob 1000
exit
```

## Troubleshooting

- If `balance` is zero, let the miner run longer and run `balance` again.
- If wallet sends fail, confirm the node is still running on `127.0.0.1:9000`.
- If dependency downloads fail, rerun Cargo with network access.
- Delete `blockchain.cbor` to restart from an empty local chain.
