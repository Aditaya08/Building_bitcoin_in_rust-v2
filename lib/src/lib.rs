pub mod crypto;
pub mod error;
pub mod network;
pub mod sha256;
pub mod types;
pub mod util;

pub use error::{BtcError, Result};

pub const SATOSHIS_PER_BTC: u64 = 100_000_000;
pub const INITIAL_REWARD: u64 = 50;
pub const HALVING_INTERVAL: u64 = 210;
pub const IDEAL_BLOCK_TIME: i64 = 10;
pub const DIFFICULTY_UPDATE_INTERVAL: u64 = 50;
pub const MAX_MEMPOOL_TRANSACTION_AGE: i64 = 600;
pub const BLOCK_TRANSACTION_CAP: usize = 20;

pub fn initial_reward_sats() -> u64 {
    INITIAL_REWARD * SATOSHIS_PER_BTC
}

pub fn min_target() -> sha256::Hash {
    sha256::Hash::from_bytes([
        0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff,
    ])
}
