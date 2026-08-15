use anyhow::{anyhow, Result};
use btclib::crypto::{PrivateKey, PublicKey};
use btclib::network::Message;
use btclib::types::{Transaction, TransactionInput, TransactionOutput};
use btclib::util::Saveable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Key {
    pub public: PathBuf,
    pub private: PathBuf,
}

#[derive(Clone)]
pub struct LoadedKey {
    pub public: PublicKey,
    pub private: PrivateKey,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Recipient {
    pub name: String,
    pub key: PathBuf,
}

#[derive(Clone, Debug)]
pub struct LoadedRecipient {
    pub name: String,
    pub key: PublicKey,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FeeType {
    Fixed,
    Percent,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeeConfig {
    pub fee_type: FeeType,
    pub value: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub my_keys: Vec<Key>,
    pub contacts: Vec<Recipient>,
    pub default_node: String,
    pub fee_config: FeeConfig,
}

pub struct Core {
    pub config: Config,
    keys: Vec<LoadedKey>,
    contacts: Vec<LoadedRecipient>,
    utxos: RwLock<HashMap<btclib::sha256::Hash, TransactionOutput>>,
    pub tx_sender: mpsc::UnboundedSender<Transaction>,
}

impl Recipient {
    pub fn load(&self) -> Result<LoadedRecipient> {
        Ok(LoadedRecipient {
            name: self.name.clone(),
            key: PublicKey::load_from_file(&self.key)?,
        })
    }
}

impl Config {
    pub fn dummy() -> Self {
        Self {
            my_keys: vec![Key {
                public: PathBuf::from("alice.pub"),
                private: PathBuf::from("alice.priv"),
            }],
            contacts: vec![Recipient {
                name: "Bob".to_string(),
                key: PathBuf::from("bob.pub"),
            }],
            default_node: "127.0.0.1:9000".to_string(),
            fee_config: FeeConfig {
                fee_type: FeeType::Percent,
                value: 0.1,
            },
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

impl Core {
    pub fn from_config(
        config: Config,
        tx_sender: mpsc::UnboundedSender<Transaction>,
    ) -> Result<Self> {
        let keys = config
            .my_keys
            .iter()
            .map(|key| {
                Ok(LoadedKey {
                    public: PublicKey::load_from_file(&key.public)?,
                    private: PrivateKey::load_from_file(&key.private)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let contacts = config
            .contacts
            .iter()
            .map(Recipient::load)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            config,
            keys,
            contacts,
            utxos: RwLock::new(HashMap::new()),
            tx_sender,
        })
    }

    pub fn contacts(&self) -> &[LoadedRecipient] {
        &self.contacts
    }

    pub async fn fetch_utxos(&self) -> Result<()> {
        let mut next = HashMap::new();
        for key in &self.keys {
            let mut stream = TcpStream::connect(&self.config.default_node).await?;
            Message::FetchUTXOs(key.public.clone())
                .send_async(&mut stream)
                .await?;
            match Message::receive_async(&mut stream).await? {
                Message::UTXOs(utxos) => {
                    for (utxo, marked) in utxos {
                        if !marked {
                            next.insert(utxo.hash(), utxo);
                        }
                    }
                }
                other => return Err(anyhow!("unexpected node response: {other:?}")),
            }
        }
        *self.utxos.write().await = next;
        Ok(())
    }

    pub async fn set_utxos_for_test(&self, outputs: Vec<TransactionOutput>) {
        *self.utxos.write().await = outputs
            .into_iter()
            .map(|output| (output.hash(), output))
            .collect();
    }

    pub async fn get_balance(&self) -> u64 {
        self.utxos
            .read()
            .await
            .values()
            .map(|output| output.value)
            .sum()
    }

    pub fn calculate_fee(&self, amount: u64) -> u64 {
        match self.config.fee_config.fee_type {
            FeeType::Fixed => self.config.fee_config.value.max(0.0) as u64,
            FeeType::Percent => ((amount as f64) * self.config.fee_config.value / 100.0) as u64,
        }
    }

    pub async fn create_transaction(
        &self,
        recipient_key: &PublicKey,
        amount: u64,
    ) -> Result<Transaction> {
        let fee = self.calculate_fee(amount);
        let required = amount + fee;
        let mut selected = Vec::new();
        let mut selected_total = 0u64;
        for output in self.utxos.read().await.values() {
            selected_total += output.value;
            selected.push(output.clone());
            if selected_total >= required {
                break;
            }
        }
        if selected_total < required {
            return Err(anyhow!("insufficient balance"));
        }
        let mut inputs = Vec::new();
        for output in &selected {
            let key = self
                .keys
                .iter()
                .find(|key| key.public == output.pubkey)
                .ok_or_else(|| anyhow!("no private key for selected output"))?;
            inputs.push(TransactionInput::signed(output, &key.private));
        }
        let mut outputs = vec![TransactionOutput::new(amount, recipient_key.clone())];
        let change = selected_total - required;
        if change > 0 {
            outputs.push(TransactionOutput::new(change, self.keys[0].public.clone()));
        }
        Ok(Transaction::new(inputs, outputs))
    }

    pub async fn send_transaction(&self, transaction: Transaction) -> Result<()> {
        let mut stream = TcpStream::connect(&self.config.default_node).await?;
        Message::SubmitTransaction(transaction)
            .send_async(&mut stream)
            .await?;
        Ok(())
    }
}
