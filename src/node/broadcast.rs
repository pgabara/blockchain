use crate::api::request::Request;
use crate::api::response::{SyncBlockResponse, SyncPeerListResponse, SyncTransactionResponse};
use crate::blockchain::Block;
use crate::network::client::{Client, ClientError};
use crate::node::peer::Peer;
use crate::transaction::Transaction;
use std::net::SocketAddr;
use std::sync::Arc;

pub trait Broadcaster {
    fn broadcast_peer_list(
        &self,
        cluster_peers: Vec<Peer>,
        peers: &[Peer],
    ) -> impl Future<Output = Result<(), BroadcasterError>> + Send;

    fn broadcast_new_block(
        &self,
        block: &Block,
        peers: &[Peer],
    ) -> impl Future<Output = Result<(), BroadcasterError>> + Send;

    fn broadcast_transaction(
        &self,
        transaction: Transaction,
        peers: &[Peer],
    ) -> impl Future<Output = Result<(), BroadcasterError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcasterError {
    #[error("Failed to broadcast a message: {0}")]
    NetworkError(#[from] ClientError),
}

pub struct NodeBroadcaster<C: Client> {
    client: Arc<C>,
}

impl<C: Client> NodeBroadcaster<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> Broadcaster for NodeBroadcaster<C>
where
    C: Client + Send + Sync,
{
    async fn broadcast_peer_list(
        &self,
        cluster_peers: Vec<Peer>,
        peers: &[Peer],
    ) -> Result<(), BroadcasterError> {
        let cluster_peers: Vec<_> = cluster_peers.iter().map(|p| p.addr).collect();
        tracing::debug!(?peers, "Broadcasting the peer list");
        for &peer in peers {
            let client = Arc::clone(&self.client);
            send_peer_list(peer.addr, &cluster_peers, client).await?;
        }
        Ok(())
    }

    async fn broadcast_new_block(
        &self,
        block: &Block,
        peers: &[Peer],
    ) -> Result<(), BroadcasterError> {
        tracing::debug!(block.index, "Broadcasting new block");
        for &peer in peers {
            let client = Arc::clone(&self.client);
            let status = add_block(block.clone(), peer.addr, client).await?;
            tracing::debug!(status, "Broadcasting new block to {}", peer.addr);
        }
        Ok(())
    }

    async fn broadcast_transaction(
        &self,
        transaction: Transaction,
        peers: &[Peer],
    ) -> Result<(), BroadcasterError> {
        for &peer in peers {
            let client = Arc::clone(&self.client);
            sync_transaction(transaction.clone(), peer.addr, client).await?;
        }
        Ok(())
    }
}

async fn send_peer_list<C: Client>(
    peer_addr: SocketAddr,
    peers: &[SocketAddr],
    client: Arc<C>,
) -> Result<SyncPeerListResponse, ClientError> {
    let request = Request::SyncPeerList(peers.to_vec());
    tracing::debug!(?peers, "Sending the peers list");
    client.send(peer_addr, request).await
}

async fn add_block<C: Client>(
    block: Block,
    peer_addr: SocketAddr,
    client: Arc<C>,
) -> Result<bool, ClientError> {
    let request = Request::SyncBlock(block);
    let response: SyncBlockResponse = client.send(peer_addr, request).await?;
    tracing::debug!(response.is_block_added, "Adding new block");
    Ok(response.is_block_added)
}

async fn sync_transaction<C: Client>(
    transaction: Transaction,
    peer_addr: SocketAddr,
    client: Arc<C>,
) -> Result<(), ClientError> {
    let request = Request::SyncTransaction(transaction);
    let _: SyncTransactionResponse = client.send(peer_addr, request).await?;
    Ok(())
}

#[cfg(test)]
pub mod test_utils {
    use crate::blockchain::Block;
    use crate::node::broadcast::{Broadcaster, BroadcasterError};
    use crate::node::peer::Peer;
    use crate::transaction::Transaction;
    use tokio::sync::Mutex;

    pub struct MockBroadcaster {
        pub peer_list: Mutex<Option<Vec<Peer>>>,
        pub new_block: Mutex<Option<Block>>,
        pub transaction: Mutex<Option<Transaction>>,
    }

    impl MockBroadcaster {
        pub fn new() -> Self {
            Self {
                peer_list: Mutex::new(None),
                new_block: Mutex::new(None),
                transaction: Mutex::new(None),
            }
        }
    }

    impl Broadcaster for MockBroadcaster {
        async fn broadcast_peer_list(
            &self,
            cluster_peers: Vec<Peer>,
            _peers: &[Peer],
        ) -> Result<(), BroadcasterError> {
            self.peer_list.lock().await.replace(cluster_peers);
            Ok(())
        }

        async fn broadcast_new_block(
            &self,
            block: &Block,
            _peers: &[Peer],
        ) -> Result<(), BroadcasterError> {
            self.new_block.lock().await.replace(block.clone());
            Ok(())
        }

        async fn broadcast_transaction(
            &self,
            transaction: Transaction,
            _peers: &[Peer],
        ) -> Result<(), BroadcasterError> {
            self.transaction.lock().await.replace(transaction);
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::network::client::test_utils::TestClient;
    use crate::node::peer::Peer;
    use crate::transaction::test_utils::create_transaction;

    #[tokio::test]
    async fn test_return_error_on_invalid_response() {
        let client = TestClient::new(vec![]);
        let broadcaster = NodeBroadcaster::new(Arc::new(client));

        let peer_1 = ([0, 0, 0, 0], 52001).into();
        let peer_2 = ([0, 0, 0, 0], 52002).into();
        let peers = vec![Peer::new(peer_1)];
        let cluster_peers = vec![Peer::new(peer_1), Peer::new(peer_2)];

        let response = broadcaster.broadcast_peer_list(cluster_peers, &peers).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_peer_list_to_peers() {
        let response = serde_json::to_vec(&SyncPeerListResponse).unwrap();
        let client = TestClient::new(response);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let peer_1 = ([0, 0, 0, 0], 52001).into();
        let peer_2 = ([0, 0, 0, 0], 52002).into();
        let peer_3 = ([0, 0, 0, 0], 52003).into();
        let peers = vec![Peer::new(peer_2), Peer::new(peer_3)];
        let cluster_peers = vec![Peer::new(peer_1), Peer::new(peer_2), Peer::new(peer_3)];

        let response = broadcaster
            .broadcast_peer_list(cluster_peers.clone(), &peers)
            .await;
        assert!(response.is_ok());

        let request = Request::SyncPeerList(cluster_peers.iter().map(|p| p.addr).collect());
        let requests = client.requests.lock().await;

        assert_eq!(requests.get(&peer_2), Some(&request));
        assert_eq!(requests.get(&peer_3), Some(&request));
    }

    #[tokio::test]
    async fn test_broadcast_new_block_to_peers() {
        let response = serde_json::to_vec(&SyncBlockResponse {
            is_block_added: true,
        })
        .unwrap();
        let client = TestClient::new(response);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let peer_1 = ([0, 0, 0, 0], 52001).into();
        let peer_2 = ([0, 0, 0, 0], 52002).into();
        let peers = vec![Peer::new(peer_1), Peer::new(peer_2)];

        let transactions = vec![create_transaction("1", 800), create_transaction("2", 500)];
        let block = Block::new(1, transactions, "".to_string(), 1);

        let response = broadcaster.broadcast_new_block(&block, &peers).await;
        assert!(response.is_ok());

        let request = Request::SyncBlock(block);
        let requests = client.requests.lock().await;

        assert_eq!(requests.get(&peer_1), Some(&request));
        assert_eq!(requests.get(&peer_2), Some(&request));
    }

    #[tokio::test]
    async fn test_broadcast_peer_list_to_no_peers() {
        let client = TestClient::new(vec![]);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let peers = vec![];
        let cluster_peers = vec![Peer::new(([0, 0, 0, 0], 52001).into())];

        let response = broadcaster.broadcast_peer_list(cluster_peers, &peers).await;
        assert!(response.is_ok());

        let requests = client.requests.lock().await;
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn test_broadcast_new_block_to_no_peers() {
        let client = TestClient::new(vec![]);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let transactions = vec![create_transaction("1", 800), create_transaction("2", 500)];
        let block = Block::new(1, transactions, "".to_string(), 1);

        let response = broadcaster.broadcast_new_block(&block, &[]).await;
        assert!(response.is_ok());

        let requests = client.requests.lock().await;
        assert!(requests.is_empty());
    }
}
