use crate::api::response::SyncBlockResponse;
use crate::blockchain::{Block, Blockchain};
use crate::network::server::ServerError;
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(
    block: Block,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
) -> Result<SyncBlockResponse, ServerError> {
    tracing::debug!(?block, "Received SyncBlock request");
    {
        let mut transactions = mempool.lock().await;
        for tx in block.transactions.iter() {
            transactions.retain(|mem_tx| mem_tx != tx);
        }
    }
    let is_block_added = blockchain.lock().await.add_block(block);
    tracing::debug!(is_block_added, "Block added to the blockchain");
    Ok(SyncBlockResponse { is_block_added })
}

#[cfg(test)]
mod tests {
    // todo: write tests here...
}
