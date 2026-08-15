use btclib::crypto::PrivateKey;
use btclib::sha256::Hash;
use btclib::types::{Block, BlockHeader, Transaction, TransactionOutput};
use btclib::util::MerkleRoot;
use chrono::Utc;
use miner::mine_template;

#[test]
fn mine_template_returns_a_block_that_satisfies_target() {
    let key = PrivateKey::new_key();
    let transactions = vec![Transaction::coinbase(TransactionOutput::new(
        50,
        key.public_key(),
    ))];
    let block = Block::new(
        BlockHeader::new(
            Utc::now(),
            0,
            Hash::zero(),
            MerkleRoot::calculate(&transactions),
            btclib::min_target(),
        ),
        transactions,
    );
    let mined = mine_template(block, 2_000_000).expect("block should mine with easy target");
    assert!(mined.hash().matches_target(mined.header.target));
}
