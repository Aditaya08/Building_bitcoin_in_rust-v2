use anyhow::Result;
use argh::FromArgs;
use node::{run_node, NodeState};

#[derive(FromArgs)]
/// A toy bitcoin-like node.
struct Args {
    #[argh(option, default = "9000")]
    /// port number to listen on
    port: u16,
    #[argh(option, default = "String::from(\"./blockchain.cbor\")")]
    /// blockchain file location
    blockchain_file: String,
    #[argh(positional)]
    /// initial peer node addresses
    nodes: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let state = NodeState::load_or_new(&args.blockchain_file).await?;
    for node in args.nodes {
        state.remember_node(node);
    }
    tokio::spawn(state.clone().cleanup_loop());
    tokio::spawn(state.clone().save_loop(args.blockchain_file));
    run_node(state, args.port).await
}
