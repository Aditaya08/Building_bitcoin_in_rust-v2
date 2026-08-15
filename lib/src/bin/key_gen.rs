use btclib::crypto::PrivateKey;
use btclib::util::Saveable;
use std::env;

fn main() {
    let name = env::args().nth(1).expect("usage: key_gen <name>");
    let private_key = PrivateKey::new_key();
    let public_key = private_key.public_key();
    private_key
        .save_to_file(format!("{name}.priv"))
        .expect("save private key");
    public_key
        .save_to_file(format!("{name}.pub"))
        .expect("save public key");
}
