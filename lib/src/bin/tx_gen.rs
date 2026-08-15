use btclib::crypto::PrivateKey;
use btclib::types::{Transaction, TransactionOutput};
use btclib::util::Saveable;
use std::env;

fn main() {
    let path = env::args().nth(1).expect("usage: tx_gen <tx_file>");
    let private_key = PrivateKey::new_key();
    let tx = Transaction::coinbase(TransactionOutput::new(
        btclib::initial_reward_sats(),
        private_key.public_key(),
    ));
    tx.save_to_file(path).expect("save transaction");
}
