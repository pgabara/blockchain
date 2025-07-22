use crate::api::response::SyncBlockResponse;
use crate::blockchain::{Block, Blockchain};
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(
    block: Block,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
) -> SyncBlockResponse {
    tracing::debug!(?block, "Received SyncBlock request");
    {
        let mut transactions = mempool.lock().await;
        for tx in block.transactions.iter() {
            transactions.retain(|mem_tx| mem_tx != tx);
        }
    }
    let is_block_added = blockchain.lock().await.add_block(block);
    tracing::debug!(is_block_added, "Block added to the blockchain");
    SyncBlockResponse { is_block_added }
}
