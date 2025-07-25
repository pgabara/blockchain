use crate::api::response::SyncTransactionResponse;
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(
    transaction: Transaction,
    mempool: Arc<Mutex<Vec<Transaction>>>,
) -> SyncTransactionResponse {
    tracing::debug!(?transaction, "Received SyncTransaction request");
    mempool.lock().await.push(transaction);
    SyncTransactionResponse
}
