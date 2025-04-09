use std::sync::{Arc, Mutex};

use crate::args::Args;
use crate::blockchain::Blockchain;
use crate::network::{client, server};

pub async fn start(args: Args) -> Result<(), Box<dyn std::error::Error>> {
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
