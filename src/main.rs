use clap::Parser;
use std::sync::{Arc, Mutex};
use tracing_subscriber::prelude::*;

use blockchain::args::Args;
use blockchain::blockchain::Blockchain;
use blockchain::network::{client, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_file(true)
        .with_line_number(true);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let blockchain = Arc::new(Mutex::new(Blockchain::new(args.difficulty)));

    match args.seed_node {
        Some(seed_node) => {
            tracing::debug!(?seed_node, "Joining existing blockchain");
            let chain = client::get_chain(seed_node).await?;
            let is_chain_replaced = blockchain.lock().unwrap().try_replace_chain(chain);
            tracing::debug!(is_chain_replaced, "Replacing existing chain");
        }
        None => tracing::debug!("Starting new blockchain"),
    }

    server::start_server(args.port, blockchain.clone()).await
}
