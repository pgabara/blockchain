use crate::api::response::AddTransactionResponse;
use crate::network::server::ServerError;
use crate::node::Node;
use crate::node::broadcast::{Broadcaster, BroadcasterError};
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
    let is_transaction_added = match transaction.verify() {
        Ok(_) => {
            tracing::debug!("Transaction verified successfully. Adding transaction to the chain");
            mempool.lock().await.push(transaction.clone());
            let cluster_peers: Vec<_> = node.lock().await.peers.keys().copied().collect();
            broadcaster
                .broadcast_transaction(transaction, &cluster_peers)
                .await?;
            true
        }
        Err(e) => {
            tracing::warn!("Failed to verify transaction. {}", e);
            false
        }
    };

    let response = AddTransactionResponse {
        is_transaction_added,
    };
    Ok(response)
}

impl From<BroadcasterError> for ServerError {
    fn from(value: BroadcasterError) -> Self {
        let error_message = format!("Failed to broadcast message: {:?}", value);
        ServerError::InternalError(error_message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::broadcast::test_utils::MockBroadcaster;
    use crate::transaction::test_utils::create_transaction;

    #[tokio::test]
    async fn add_valid_transaction() {
        let broadcaster = Arc::new(MockBroadcaster::new());
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let mempool = Arc::new(Mutex::new(Vec::new()));

        let transaction = create_transaction("r1", 200);
        let response = handle_request(transaction, node, broadcaster, Arc::clone(&mempool)).await;

        assert_eq!(response.unwrap().is_transaction_added, true);
        assert_eq!(mempool.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn ignore_transaction_with_invalid_signature() {
        let broadcaster = Arc::new(MockBroadcaster::new());
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let mempool = Arc::new(Mutex::new(Vec::new()));

        let mut transaction = create_transaction("r1", 200);
        transaction.signature = vec![];
        let response = handle_request(transaction, node, broadcaster, Arc::clone(&mempool)).await;

        assert_eq!(response.unwrap().is_transaction_added, false);
        assert_eq!(mempool.lock().await.len(), 0);
    }

    #[tokio::test]
    async fn ignore_transaction_with_invalid_sender_address() {
        let broadcaster = Arc::new(MockBroadcaster::new());
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let mempool = Arc::new(Mutex::new(Vec::new()));

        let mut transaction = create_transaction("r1", 200);
        transaction.sender.address = "".to_string();
        let response = handle_request(transaction, node, broadcaster, Arc::clone(&mempool)).await;

        assert_eq!(response.unwrap().is_transaction_added, false);
        assert_eq!(mempool.lock().await.len(), 0);
    }
}
