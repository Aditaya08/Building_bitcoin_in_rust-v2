use crate::crypto::{PrivateKey, PublicKey, Signature};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: PublicKey,
}

impl TransactionOutput {
    pub fn new(value: u64, pubkey: PublicKey) -> Self {
        Self {
            value,
            unique_id: Uuid::new_v4(),
            pubkey,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl TransactionInput {
    pub fn signed(prev_output: &TransactionOutput, private_key: &PrivateKey) -> Self {
        let prev_hash = prev_output.hash();
        Self {
            prev_transaction_output_hash: prev_hash,
            signature: private_key.sign_hash(prev_hash),
        }
    }
}

impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        Self { inputs, outputs }
    }

    pub fn coinbase(output: TransactionOutput) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![output],
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn validate_basic(&self) -> Result<()> {
        if self.outputs.is_empty() {
            return Err(BtcError::EmptyTransaction);
        }
        Ok(())
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }
}
