use super::{Block, BlockHeader, Transaction, TransactionOutput};
use crate::crypto::PublicKey;
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::{MerkleRoot, Saveable};
use crate::{
    initial_reward_sats, min_target, BLOCK_TRANSACTION_CAP, DIFFICULTY_UPDATE_INTERVAL,
    HALVING_INTERVAL, MAX_MEMPOOL_TRANSACTION_AGE,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MempoolEntry {
    pub transaction: Transaction,
    pub received_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
    utxos: HashMap<Hash, (bool, TransactionOutput)>,
    mempool: HashMap<Hash, MempoolEntry>,
    target: Hash,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            utxos: HashMap::new(),
            mempool: HashMap::new(),
            target: min_target(),
        }
    }

    pub fn blocks(&self) -> impl DoubleEndedIterator<Item = &Block> {
        self.blocks.iter()
    }

    pub fn block_height(&self) -> u64 {
        self.blocks.len() as u64
    }

    pub fn target(&self) -> Hash {
        self.target
    }

    pub fn utxos(&self) -> &HashMap<Hash, (bool, TransactionOutput)> {
        &self.utxos
    }

    pub fn mempool(&self) -> &HashMap<Hash, MempoolEntry> {
        &self.mempool
    }

    pub fn calculate_block_reward(&self) -> u64 {
        let halvings = self.block_height() / HALVING_INTERVAL;
        initial_reward_sats()
            .checked_shr(halvings as u32)
            .unwrap_or(0)
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        block.validate_merkle_root()?;
        block.validate_proof_of_work()?;
        let expected_prev = self
            .blocks
            .last()
            .map(Block::hash)
            .unwrap_or_else(Hash::zero);
        if block.header.prev_block_hash != expected_prev {
            return Err(BtcError::InvalidPreviousHash);
        }
        self.validate_block_transactions(&block)?;
        self.apply_block(&block)?;
        self.blocks.push(block);
        self.try_adjust_target();
        Ok(())
    }

    pub fn add_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        transaction.validate_basic()?;
        let hash = transaction.hash();
        if self.mempool.contains_key(&hash) {
            return Ok(());
        }
        self.validate_regular_transaction(&transaction, &mut HashSet::new())?;
        self.mempool.insert(
            hash,
            MempoolEntry {
                transaction,
                received_at: Utc::now(),
            },
        );
        Ok(())
    }

    pub fn cleanup_mempool(&mut self) {
        let now = Utc::now();
        self.mempool.retain(|_, entry| {
            (now - entry.received_at).num_seconds() <= MAX_MEMPOOL_TRANSACTION_AGE
        });
    }

    pub fn rebuild_utxos(&mut self) {
        self.utxos.clear();
        let blocks = self.blocks.clone();
        for block in blocks {
            let _ = self.apply_block(&block);
        }
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.len() < DIFFICULTY_UPDATE_INTERVAL as usize
            || self.blocks.len() % DIFFICULTY_UPDATE_INTERVAL as usize != 0
        {
            return;
        }
        self.target = min_target();
    }

    pub fn create_template(&self, pubkey: PublicKey) -> Result<Block> {
        let mut transactions: Vec<Transaction> = self
            .mempool
            .values()
            .take(BLOCK_TRANSACTION_CAP)
            .map(|entry| entry.transaction.clone())
            .collect();
        transactions.insert(
            0,
            Transaction::coinbase(TransactionOutput {
                value: 0,
                unique_id: Uuid::new_v4(),
                pubkey,
            }),
        );

        let mut block = Block::new(
            BlockHeader::new(
                Utc::now(),
                0,
                self.blocks
                    .last()
                    .map(Block::hash)
                    .unwrap_or_else(Hash::zero),
                MerkleRoot::calculate(&transactions),
                self.target,
            ),
            transactions,
        );
        let fees = block.calculate_miner_fees(&self.utxos)?;
        block.transactions[0].outputs[0].value = self.calculate_block_reward() + fees;
        block.header.merkle_root = MerkleRoot::calculate(&block.transactions);
        Ok(block)
    }

    fn validate_block_transactions(&self, block: &Block) -> Result<()> {
        if block.transactions.is_empty() || !block.transactions[0].inputs.is_empty() {
            return Err(BtcError::InvalidCoinbase);
        }
        let reward = self.calculate_block_reward() + block.calculate_miner_fees(&self.utxos)?;
        if block.transactions[0].total_output_value() > reward {
            return Err(BtcError::InvalidCoinbase);
        }
        let mut spent = HashSet::new();
        for tx in block.transactions.iter().skip(1) {
            self.validate_regular_transaction(tx, &mut spent)?;
        }
        Ok(())
    }

    fn validate_regular_transaction(
        &self,
        transaction: &Transaction,
        spent_in_context: &mut HashSet<Hash>,
    ) -> Result<()> {
        transaction.validate_basic()?;
        if transaction.inputs.is_empty() {
            return Err(BtcError::InvalidCoinbase);
        }
        let mut input_total = 0u64;
        for input in &transaction.inputs {
            if !spent_in_context.insert(input.prev_transaction_output_hash) {
                return Err(BtcError::DuplicateSpend);
            }
            let (_, output) = self
                .utxos
                .get(&input.prev_transaction_output_hash)
                .ok_or(BtcError::UnknownInput)?;
            if !output
                .pubkey
                .verify_hash(input.prev_transaction_output_hash, &input.signature)
            {
                return Err(BtcError::InvalidSignature);
            }
            input_total += output.value;
        }
        if transaction.total_output_value() > input_total {
            return Err(BtcError::Overspend);
        }
        Ok(())
    }

    fn apply_block(&mut self, block: &Block) -> Result<()> {
        for tx in &block.transactions {
            self.mempool.remove(&tx.hash());
            for input in &tx.inputs {
                self.utxos.remove(&input.prev_transaction_output_hash);
            }
            for output in &tx.outputs {
                self.utxos.insert(output.hash(), (false, output.clone()));
            }
        }
        Ok(())
    }
}

impl Saveable for Blockchain {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to load blockchain"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to save blockchain"))
    }
}
