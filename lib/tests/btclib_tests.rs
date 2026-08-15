use btclib::crypto::PrivateKey;
use btclib::network::Message;
use btclib::sha256::Hash;
use btclib::types::{Block, BlockHeader, Blockchain, Transaction, TransactionInput, TransactionOutput};
use btclib::util::{MerkleRoot, Saveable};
use chrono::Utc;

fn mined_coinbase_block(chain: &Blockchain, key: &PrivateKey, value: u64) -> Block {
    let transactions = vec![Transaction::coinbase(TransactionOutput::new(
        value,
        key.public_key(),
    ))];
    let mut block = Block::new(
        BlockHeader::new(
            Utc::now(),
            0,
            chain.blocks().last().map(Block::hash).unwrap_or_else(Hash::zero),
            MerkleRoot::calculate(&transactions),
            btclib::min_target(),
        ),
        transactions,
    );
    while !block.mine(1_000_000) {}
    block
}

#[test]
fn hash_is_deterministic() {
    assert_eq!(Hash::hash(&"bitcoin"), Hash::hash(&"bitcoin"));
    assert_ne!(Hash::hash(&"bitcoin"), Hash::hash(&"rust"));
}

#[test]
fn merkle_root_changes_when_transactions_change() {
    let key = PrivateKey::new_key();
    let tx1 = Transaction::coinbase(TransactionOutput::new(1, key.public_key()));
    let tx2 = Transaction::coinbase(TransactionOutput::new(2, key.public_key()));
    assert_ne!(MerkleRoot::calculate(&[tx1.clone()]), MerkleRoot::calculate(&[tx1, tx2]));
}

#[test]
fn keys_sign_and_verify_hashes() {
    let key = PrivateKey::new_key();
    let hash = Hash::hash(&"message");
    let signature = key.sign_hash(hash);
    assert!(key.public_key().verify_hash(hash, &signature));
}

#[test]
fn blockchain_accepts_mined_coinbase_block_and_tracks_utxo() {
    let key = PrivateKey::new_key();
    let mut chain = Blockchain::new();
    let block = mined_coinbase_block(&chain, &key, btclib::initial_reward_sats());
    chain.add_block(block).unwrap();
    assert_eq!(chain.block_height(), 1);
    assert_eq!(chain.utxos().len(), 1);
}

#[test]
fn blockchain_rejects_double_spend_in_block() {
    let miner = PrivateKey::new_key();
    let sender = PrivateKey::new_key();
    let recipient = PrivateKey::new_key();
    let mut chain = Blockchain::new();
    chain
        .add_block(mined_coinbase_block(&chain, &sender, btclib::initial_reward_sats()))
        .unwrap();
    let prev_output = chain.utxos().values().next().unwrap().1.clone();
    let input = TransactionInput::signed(&prev_output, &sender);
    let spend = Transaction::new(
        vec![input],
        vec![TransactionOutput::new(1_000, recipient.public_key())],
    );
    let mut transactions = vec![Transaction::coinbase(TransactionOutput::new(
        btclib::initial_reward_sats(),
        miner.public_key(),
    ))];
    transactions.push(spend.clone());
    transactions.push(spend);
    let mut block = Block::new(
        BlockHeader::new(
            Utc::now(),
            0,
            chain.blocks().last().map(Block::hash).unwrap(),
            MerkleRoot::calculate(&transactions),
            btclib::min_target(),
        ),
        transactions,
    );
    while !block.mine(1_000_000) {}
    assert!(chain.add_block(block).is_err());
}

#[test]
fn transaction_round_trips_through_saveable() {
    let key = PrivateKey::new_key();
    let tx = Transaction::coinbase(TransactionOutput::new(42, key.public_key()));
    let mut bytes = Vec::new();
    tx.save(&mut bytes).unwrap();
    let loaded = Transaction::load(bytes.as_slice()).unwrap();
    assert_eq!(tx.hash(), loaded.hash());
}

#[test]
fn network_message_round_trips_sync() {
    let key = PrivateKey::new_key();
    let message = Message::FetchTemplate(key.public_key());
    let mut bytes = Vec::new();
    message.send(&mut bytes).unwrap();
    let decoded = Message::receive(&mut bytes.as_slice()).unwrap();
    assert!(matches!(decoded, Message::FetchTemplate(_)));
}

#[tokio::test]
async fn network_message_round_trips_async() {
    let key = PrivateKey::new_key();
    let message = Message::FetchUTXOs(key.public_key());
    let (mut client, mut server) = tokio::io::duplex(4096);
    let writer = tokio::spawn(async move { message.send_async(&mut client).await.unwrap() });
    let decoded = Message::receive_async(&mut server).await.unwrap();
    writer.await.unwrap();
    assert!(matches!(decoded, Message::FetchUTXOs(_)));
}
