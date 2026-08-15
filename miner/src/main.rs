use anyhow::{anyhow, Result};
use btclib::crypto::PublicKey;
use btclib::network::Message;
use btclib::types::Block;
use btclib::util::Saveable;
use clap::Parser;
use miner::mine_template;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex as StdMutex,
};
use std::thread;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};

#[derive(Parser)]
#[command(author, version, about = "Toy bitcoin CPU miner", long_about = None)]
struct Cli {
    #[arg(short, long)]
    address: String,
    #[arg(short = 'k', long)]
    public_key_file: String,
}

struct Miner {
    public_key: PublicKey,
    stream: Mutex<TcpStream>,
    current_template: Arc<StdMutex<Option<Block>>>,
    mining: Arc<AtomicBool>,
    mined_block_sender: mpsc::UnboundedSender<Block>,
    mined_block_receiver: Mutex<mpsc::UnboundedReceiver<Block>>,
}

impl Miner {
    async fn new(address: String, public_key: PublicKey) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let (mined_block_sender, mined_block_receiver) = mpsc::unbounded_channel();
        Ok(Self {
            public_key,
            stream: Mutex::new(stream),
            current_template: Arc::new(StdMutex::new(None)),
            mining: Arc::new(AtomicBool::new(false)),
            mined_block_sender,
            mined_block_receiver: Mutex::new(mined_block_receiver),
        })
    }

    async fn run(&self) -> Result<()> {
        self.spawn_mining_thread();
        let mut template_interval = interval(Duration::from_secs(5));
        loop {
            tokio::select! {
                _ = template_interval.tick() => self.fetch_and_validate_template().await?,
                mined_block = async {
                    self.mined_block_receiver.lock().await.recv().await
                } => {
                    if let Some(block) = mined_block {
                        self.submit_block(block).await?;
                    }
                }
            }
        }
    }

    fn spawn_mining_thread(&self) {
        let template = Arc::clone(&self.current_template);
        let mining = Arc::clone(&self.mining);
        let sender = self.mined_block_sender.clone();
        thread::spawn(move || loop {
            if mining.load(Ordering::Relaxed) {
                let next = template.lock().expect("template lock").clone();
                if let Some(block) = next.and_then(|block| mine_template(block, 100_000)) {
                    let _ = sender.send(block);
                    mining.store(false, Ordering::Relaxed);
                }
            }
            thread::yield_now();
        });
    }

    async fn fetch_and_validate_template(&self) -> Result<()> {
        if self.mining.load(Ordering::Relaxed) {
            self.validate_template().await
        } else {
            self.fetch_template().await
        }
    }

    async fn fetch_template(&self) -> Result<()> {
        let mut stream = self.stream.lock().await;
        Message::FetchTemplate(self.public_key.clone())
            .send_async(&mut *stream)
            .await?;
        match Message::receive_async(&mut *stream).await? {
            Message::Template(template) => {
                *self.current_template.lock().expect("template lock") = Some(template);
                self.mining.store(true, Ordering::Relaxed);
                Ok(())
            }
            other => Err(anyhow!(
                "unexpected message while fetching template: {other:?}"
            )),
        }
    }

    async fn validate_template(&self) -> Result<()> {
        let Some(template) = self.current_template.lock().expect("template lock").clone() else {
            return Ok(());
        };
        let mut stream = self.stream.lock().await;
        Message::ValidateTemplate(template)
            .send_async(&mut *stream)
            .await?;
        match Message::receive_async(&mut *stream).await? {
            Message::TemplateValidity(true) => Ok(()),
            Message::TemplateValidity(false) => {
                self.mining.store(false, Ordering::Relaxed);
                Ok(())
            }
            other => Err(anyhow!(
                "unexpected message while validating template: {other:?}"
            )),
        }
    }

    async fn submit_block(&self, block: Block) -> Result<()> {
        let mut stream = self.stream.lock().await;
        Message::SubmitTemplate(block)
            .send_async(&mut *stream)
            .await?;
        self.mining.store(false, Ordering::Relaxed);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let public_key = PublicKey::load_from_file(&cli.public_key_file)
        .map_err(|e| anyhow!("failed to read public key: {e}"))?;
    Miner::new(cli.address, public_key).await?.run().await
}
