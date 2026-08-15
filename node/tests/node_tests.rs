use btclib::crypto::PrivateKey;
use btclib::network::Message;
use btclib::sha256::Hash;
use btclib::types::{Block, BlockHeader, Transaction, TransactionOutput};
use btclib::util::MerkleRoot;
use chrono::Utc;
use node::NodeState;

fn mined_block(prev: Hash, key: &PrivateKey, value: u64) -> Block {
    let transactions = vec![Transaction::coinbase(TransactionOutput::new(
        value,
        key.public_key(),
    ))];
    let mut block = Block::new(
        BlockHeader::new(
            Utc::now(),
            0,
            prev,
            MerkleRoot::calculate(&transactions),
            btclib::min_target(),
        ),
        transactions,
    );
    while !block.mine(1_000_000) {}
    block
}

#[tokio::test]
async fn node_accepts_submitted_template_and_returns_utxos() {
    let state = NodeState::new();
    let key = PrivateKey::new_key();
    let block = mined_block(Hash::zero(), &key, btclib::initial_reward_sats());
    state
        .handle_message(Message::SubmitTemplate(block))
        .await
        .unwrap();
    let response = state
        .handle_message(Message::FetchUTXOs(key.public_key()))
        .await
        .unwrap()
        .unwrap();
    let Message::UTXOs(utxos) = response else {
        panic!("expected utxo response");
    };
    assert_eq!(utxos.len(), 1);
}

#[tokio::test]
async fn node_builds_templates_against_current_tip() {
    let state = NodeState::new();
    let key = PrivateKey::new_key();
    let response = state
        .handle_message(Message::FetchTemplate(key.public_key()))
        .await
        .unwrap()
        .unwrap();
    let Message::Template(template) = response else {
        panic!("expected template");
    };
    assert_eq!(template.header.prev_block_hash, Hash::zero());
}
