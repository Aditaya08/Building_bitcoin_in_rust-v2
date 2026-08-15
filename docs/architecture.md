# Architecture

The project is a four-crate Cargo workspace.

## `btclib`

`btclib` owns shared behavior:

- `sha256::Hash`: deterministic SHA-256 over CBOR-serialized values.
- `crypto`: secp256k1 private keys, public keys, signatures, signing, and verification.
- `types`: transactions, blocks, headers, and blockchain state.
- `util`: Merkle roots and the `Saveable` serialization trait.
- `network`: length-prefixed CBOR messages shared by node, miner, and wallet.

The chain uses a UTXO model. Transaction inputs reference previous output hashes and prove ownership by signing that output hash. Blocks validate previous hash, Merkle root, proof-of-work, coinbase reward, fees, and double-spend behavior.

## `node`

The node keeps blockchain state behind an async `RwLock`, handles protocol messages over TCP, periodically saves the blockchain file, and periodically cleans old mempool entries.

The testable core is `NodeState::handle_message`, so most protocol behavior can be tested without launching a long-running process.

## `miner`

The miner connects to a node, fetches block templates, mines on a blocking thread, validates templates while mining, and submits mined blocks.

The pure helper `mine_template` is tested independently.

## `wallet`

The wallet separates core behavior from the command loop. `Core` loads TOML config and key files, fetches UTXOs, calculates fees, creates signed transactions, and submits transactions to a node.

The `tui` subcommand currently starts the same interactive command surface with a TUI-style header. The core is intentionally UI-independent so a fuller Cursive interface can be added without changing transaction logic.
