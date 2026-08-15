use crate::crypto::PublicKey;
use crate::types::{Block, Transaction, TransactionOutput};
use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    FetchUTXOs(PublicKey),
    UTXOs(Vec<(TransactionOutput, bool)>),
    SubmitTransaction(Transaction),
    NewTransaction(Transaction),
    FetchTemplate(PublicKey),
    Template(Block),
    ValidateTemplate(Block),
    TemplateValidity(bool),
    SubmitTemplate(Block),
    DiscoverNodes,
    NodeList(Vec<String>),
    AskDifference(u32),
    Difference(i32),
    FetchBlock(usize),
    NewBlock(Block),
}

impl Message {
    pub fn encode(&self) -> Result<Vec<u8>, ciborium::ser::Error<IoError>> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(self, &mut bytes)?;
        Ok(bytes)
    }

    pub fn decode(data: &[u8]) -> Result<Self, ciborium::de::Error<IoError>> {
        ciborium::de::from_reader(data)
    }

    pub fn send(&self, stream: &mut impl Write) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        stream.write_all(&(bytes.len() as u64).to_be_bytes())?;
        stream.write_all(&bytes)?;
        Ok(())
    }

    pub fn receive(stream: &mut impl Read) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        let mut data = vec![0u8; u64::from_be_bytes(len_bytes) as usize];
        stream.read_exact(&mut data)?;
        Self::decode(&data)
    }

    pub async fn send_async(
        &self,
        stream: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        stream.write_all(&(bytes.len() as u64).to_be_bytes()).await?;
        stream.write_all(&bytes).await?;
        Ok(())
    }

    pub async fn receive_async(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes).await?;
        let mut data = vec![0u8; u64::from_be_bytes(len_bytes) as usize];
        stream.read_exact(&mut data).await?;
        Self::decode(&data)
    }
}
