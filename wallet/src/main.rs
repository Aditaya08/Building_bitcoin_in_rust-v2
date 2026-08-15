use anyhow::{anyhow, Result};
use btclib::types::Transaction;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use wallet::core::{Config, Core};

#[derive(Parser)]
#[command(author, version, about = "Toy bitcoin wallet", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, value_name = "FILE", default_value = "wallet_config.toml")]
    config: PathBuf,
    #[arg(short, long, value_name = "ADDRESS")]
    node: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    GenerateConfig {
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
    },
    Tui,
}

async fn update_utxos(core: Arc<Core>) {
    let mut interval = time::interval(Duration::from_secs(20));
    loop {
        interval.tick().await;
        let _ = core.fetch_utxos().await;
    }
}

async fn handle_transactions(mut rx: mpsc::UnboundedReceiver<Transaction>, core: Arc<Core>) {
    while let Some(transaction) = rx.recv().await {
        if let Err(error) = core.send_transaction(transaction).await {
            eprintln!("failed to send transaction: {error}");
        }
    }
}

async fn run_cli(core: Arc<Core>) -> Result<()> {
    core.fetch_utxos().await.ok();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "balance" => println!("Current balance: {} satoshis", core.get_balance().await),
            "send" => {
                if parts.len() != 3 {
                    println!("Usage: send <recipient> <amount-sats>");
                    continue;
                }
                let recipient = parts[1];
                let amount = parts[2].parse::<u64>()?;
                let recipient_key = core
                    .contacts()
                    .iter()
                    .find(|entry| entry.name == recipient)
                    .ok_or_else(|| anyhow!("recipient not found"))?
                    .key
                    .clone();
                core.fetch_utxos().await?;
                let tx = core.create_transaction(&recipient_key, amount).await?;
                core.tx_sender.send(tx)?;
                println!("Transaction queued");
            }
            "contacts" => {
                for contact in core.contacts() {
                    println!("{}", contact.name);
                }
            }
            "exit" => break,
            _ => println!("Unknown command"),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(Commands::GenerateConfig { output }) = &cli.command {
        Config::dummy().save(output)?;
        println!("Generated {}", output.display());
        return Ok(());
    }

    let mut config = Config::load(&cli.config)?;
    if let Some(node) = cli.node {
        config.default_node = node;
    }
    let (tx_sender, tx_receiver) = mpsc::unbounded_channel();
    let core = Arc::new(Core::from_config(config, tx_sender)?);
    tokio::spawn(update_utxos(Arc::clone(&core)));
    tokio::spawn(handle_transactions(tx_receiver, Arc::clone(&core)));

    if matches!(cli.command, Some(Commands::Tui)) {
        println!("BTC wallet");
        println!("Use commands: balance, contacts, send <recipient> <amount-sats>, exit");
    }
    run_cli(core).await
}
