# Network Protocol

Messages are serialized as CBOR and framed with an 8-byte big-endian length prefix.

The shared enum lives in `btclib::network::Message`.

## Wallet Messages

- `FetchUTXOs(PublicKey)`: wallet asks for unspent outputs owned by a key.
- `UTXOs(Vec<(TransactionOutput, bool)>)`: node response. The boolean indicates whether an output is marked/reserved.
- `SubmitTransaction(Transaction)`: wallet submits a signed transaction.

## Miner Messages

- `FetchTemplate(PublicKey)`: miner asks for a block template paying the coinbase to a key.
- `Template(Block)`: node response with the candidate block.
- `ValidateTemplate(Block)`: miner checks whether the current template still points at the chain tip.
- `TemplateValidity(bool)`: node response.
- `SubmitTemplate(Block)`: miner submits a mined block.

## Node Messages

- `DiscoverNodes` and `NodeList(Vec<String>)`: node discovery.
- `AskDifference(u32)` and `Difference(i32)`: compare chain heights.
- `FetchBlock(usize)` and `NewBlock(Block)`: fetch or broadcast blocks.
- `NewTransaction(Transaction)`: broadcast a transaction.

The protocol is intentionally simple and has no version negotiation.
