use btclib::crypto::PrivateKey;
use btclib::sha256::Hash;
use btclib::types::{Block, BlockHeader, Transaction, TransactionOutput};
use btclib::util::{MerkleRoot, Saveable};
use chrono::Utc;
use std::env;

fn main() {
    let path = env::args().nth(1).expect("usage: block_gen <block_file>");
    let private_key = PrivateKey::new_key();
    let transactions = vec![Transaction::coinbase(TransactionOutput::new(
        btclib::initial_reward_sats(),
        private_key.public_key(),
    ))];
    let mut block = Block::new(
        BlockHeader::new(
            Utc::now(),
            0,
            Hash::zero(),
            MerkleRoot::calculate(&transactions),
            btclib::min_target(),
        ),
        transactions,
    );
    while !block.mine(1_000_000) {}
    block.save_to_file(path).expect("save block");
}
