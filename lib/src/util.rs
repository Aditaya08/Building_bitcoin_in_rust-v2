use crate::sha256::Hash;
use crate::types::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerkleRoot(pub Hash);

impl MerkleRoot {
    pub fn calculate(transactions: &[Transaction]) -> Self {
        if transactions.is_empty() {
            return Self(Hash::zero());
        }
        let mut layer: Vec<Hash> = transactions.iter().map(Hash::hash).collect();
        while layer.len() > 1 {
            let mut next = Vec::with_capacity((layer.len() + 1) / 2);
            for pair in layer.chunks(2) {
                let left = pair[0];
                let right = *pair.get(1).unwrap_or(&left);
                next.push(Hash::hash(&(left, right)));
            }
            layer = next;
        }
        Self(layer[0])
    }
}

pub trait Saveable: Sized {
    fn load<I: Read>(reader: I) -> IoResult<Self>;
    fn save<O: Write>(&self, writer: O) -> IoResult<()>;

    fn load_from_file(path: impl AsRef<std::path::Path>) -> IoResult<Self> {
        Self::load(File::open(path)?)
    }

    fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> IoResult<()> {
        self.save(File::create(path)?)
    }
}

impl Saveable for Block {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to load block"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to save block"))
    }
}

impl Saveable for Transaction {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to load transaction"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to save transaction"))
    }
}
