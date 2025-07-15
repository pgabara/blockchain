use crate::api::response::AddTransactionResponse;
use crate::network::server::ServerError;
use crate::node::Node;
use crate::node::broadcast::Broadcaster;
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(
    transaction: Transaction,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<impl Broadcaster>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
) -> Result<AddTransactionResponse, ServerError> {
    tracing::debug!(?transaction, "Received AddTransaction request");
    mempool.lock().await.push(transaction.clone());
    let cluster_peers: Vec<_> = node.lock().await.peers.keys().copied().collect();
    let _ = broadcaster
        .broadcast_transaction(transaction, &cluster_peers)
        .await; // todo: handle error
    Ok(AddTransactionResponse {
        is_transaction_added: true,
    })
}

#[cfg(test)]
mod tests {
    // todo: write tests here...
}
