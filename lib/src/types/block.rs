use super::{Transaction, TransactionOutput};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: Hash,
    pub merkle_root: MerkleRoot,
    pub target: Hash,
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: Hash,
    ) -> Self {
        Self {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn mine(&mut self, max_rounds: u64) -> bool {
        for _ in 0..max_rounds {
            if self.hash().matches_target(self.target) {
                return true;
            }
            self.nonce = self.nonce.wrapping_add(1);
        }
        false
    }
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    pub fn mine(&mut self, max_rounds: u64) -> bool {
        self.header.mine(max_rounds)
    }

    pub fn validate_merkle_root(&self) -> Result<()> {
        let expected = MerkleRoot::calculate(&self.transactions);
        if self.header.merkle_root == expected {
            Ok(())
        } else {
            Err(BtcError::InvalidMerkleRoot)
        }
    }

    pub fn validate_proof_of_work(&self) -> Result<()> {
        if self.hash().matches_target(self.header.target) {
            Ok(())
        } else {
            Err(BtcError::InvalidProofOfWork)
        }
    }

    pub fn calculate_miner_fees(
        &self,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<u64> {
        let mut fees = 0u64;
        for tx in self.transactions.iter().skip(1) {
            let input_total = tx
                .inputs
                .iter()
                .map(|input| {
                    utxos
                        .get(&input.prev_transaction_output_hash)
                        .map(|(_, output)| output.value)
                        .ok_or(BtcError::UnknownInput)
                })
                .try_fold(0u64, |sum, value| value.map(|value| sum + value))?;
            let output_total = tx.total_output_value();
            if output_total > input_total {
                return Err(BtcError::Overspend);
            }
            fees += input_total - output_total;
        }
        Ok(fees)
    }
}
