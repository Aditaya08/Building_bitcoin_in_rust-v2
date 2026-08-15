use btclib::types::Block;
use btclib::util::Saveable;
use std::env;

fn main() {
    let path = env::args().nth(1).expect("usage: block_print <block_file>");
    let block = Block::load_from_file(path).expect("load block");
    println!("{block:#?}");
}
