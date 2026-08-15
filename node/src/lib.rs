use anyhow::{anyhow, Result};
use btclib::network::Message;
use btclib::sha256::Hash;
use btclib::types::Blockchain;
use btclib::util::Saveable;
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{self, Duration};

#[derive(Clone)]
pub struct NodeState {
    blockchain: Arc<RwLock<Blockchain>>,
    nodes: Arc<DashMap<String, ()>>,
}

impl Default for NodeState {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            nodes: Arc::new(DashMap::new()),
        }
    }

    pub async fn load_or_new(path: &str) -> Result<Self> {
        let state = Self::new();
        if Path::new(path).exists() {
            let mut chain = Blockchain::load_from_file(path)?;
            chain.rebuild_utxos();
            *state.blockchain.write().await = chain;
        }
        Ok(state)
    }

    pub async fn blockchain(&self) -> Blockchain {
        self.blockchain.read().await.clone()
    }

    pub fn remember_node(&self, address: String) {
        self.nodes.insert(address, ());
    }

    pub fn known_nodes(&self) -> Vec<String> {
        self.nodes.iter().map(|entry| entry.key().clone()).collect()
    }

    pub async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
        use Message::*;
        match message {
            UTXOs(_) | Template(_) | Difference(_) | TemplateValidity(_) | NodeList(_) => {
                Err(anyhow!("node received response-only message"))
            }
            FetchBlock(height) => {
                let chain = self.blockchain.read().await;
                let response = chain.blocks().nth(height).cloned().map(NewBlock);
                Ok(response)
            }
            DiscoverNodes => Ok(Some(NodeList(self.known_nodes()))),
            AskDifference(height) => {
                let chain = self.blockchain.read().await;
                Ok(Some(Difference(
                    chain.block_height() as i32 - height as i32,
                )))
            }
            FetchUTXOs(key) => {
                let chain = self.blockchain.read().await;
                let utxos = chain
                    .utxos()
                    .values()
                    .filter(|(_, output)| output.pubkey == key)
                    .map(|(marked, output)| (output.clone(), *marked))
                    .collect();
                Ok(Some(UTXOs(utxos)))
            }
            NewBlock(block) => {
                let mut chain = self.blockchain.write().await;
                chain.add_block(block)?;
                Ok(None)
            }
            NewTransaction(tx) | SubmitTransaction(tx) => {
                let mut chain = self.blockchain.write().await;
                chain.add_to_mempool(tx)?;
                Ok(None)
            }
            ValidateTemplate(block_template) => {
                let chain = self.blockchain.read().await;
                let current_tip = chain
                    .blocks()
                    .last()
                    .map(|block| block.hash())
                    .unwrap_or(Hash::zero());
                Ok(Some(TemplateValidity(
                    block_template.header.prev_block_hash == current_tip,
                )))
            }
            SubmitTemplate(block) => {
                let mut chain = self.blockchain.write().await;
                chain.add_block(block)?;
                chain.rebuild_utxos();
                Ok(None)
            }
            FetchTemplate(pubkey) => {
                let chain = self.blockchain.read().await;
                Ok(Some(Template(chain.create_template(pubkey)?)))
            }
        }
    }

    pub async fn save_loop(self, path: String) {
        let mut interval = time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            let chain = self.blockchain.read().await;
            let _ = chain.save_to_file(&path);
        }
    }

    pub async fn cleanup_loop(self) {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            self.blockchain.write().await.cleanup_mempool();
        }
    }
}

pub async fn handle_connection(state: NodeState, mut socket: TcpStream) {
    loop {
        let message = match Message::receive_async(&mut socket).await {
            Ok(message) => message,
            Err(_) => return,
        };
        match state.handle_message(message).await {
            Ok(Some(response)) => {
                if response.send_async(&mut socket).await.is_err() {
                    return;
                }
            }
            Ok(None) => {}
            Err(_) => return,
        }
    }
}

pub async fn run_node(state: NodeState, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    loop {
        let (socket, peer) = listener.accept().await?;
        state.remember_node(peer.to_string());
        tokio::spawn(handle_connection(state.clone(), socket));
    }
}
