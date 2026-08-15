use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn digest<T: Serialize>(data: &T) -> Self {
        let mut serialized = Vec::new();
        ciborium::ser::into_writer(data, &mut serialized)
            .expect("serializing hash input should not fail");
        let digest = Sha256::digest(serialized);
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Self(out)
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn matches_target(&self, target: Hash) -> bool {
        self.0 <= target.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}
