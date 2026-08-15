use btclib::types::Transaction;
use btclib::util::Saveable;
use std::env;

fn main() {
    let path = env::args().nth(1).expect("usage: tx_print <tx_file>");
    let tx = Transaction::load_from_file(path).expect("load transaction");
    println!("{tx:#?}");
}
