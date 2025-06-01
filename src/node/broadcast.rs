use crate::api::request::Request;
use crate::api::response::{AddBlockResponse, PeerListResponse};
use crate::blockchain::Block;
use crate::network::client::{Client, ClientError};
use crate::node::Node;
use std::net::SocketAddr;
use std::sync::Arc;

pub trait Broadcaster {
    fn broadcast_peer_list(
        &self,
        node: &Node,
    ) -> impl Future<Output = Result<(), BroadcasterError>> + Send;
    fn broadcast_new_block(
        &self,
        block: &Block,
        node: &Node,
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
    async fn broadcast_peer_list(&self, node: &Node) -> Result<(), BroadcasterError> {
        let peers: Vec<_> = node.cluster_peers().iter().map(|p| p.addr).collect();
        tracing::debug!(?peers, "Broadcasting the peer list");
        for &peer in node.peers.keys() {
            let client = Arc::clone(&self.client);
            send_peer_list(peer.addr, &peers, client).await?;
        }
        Ok(())
    }

    async fn broadcast_new_block(
        &self,
        block: &Block,
        node: &Node,
    ) -> Result<(), BroadcasterError> {
        tracing::debug!(block.index, "Broadcasting new block");
        for &peer in node.peers.keys() {
            let client = Arc::clone(&self.client);
            let status = add_block(block.clone(), peer.addr, client).await?;
            tracing::debug!(status, "Broadcasting new block to {}", peer.addr);
        }
        Ok(())
    }
}

async fn send_peer_list<C: Client>(
    peer_addr: SocketAddr,
    peers: &[SocketAddr],
    client: Arc<C>,
) -> Result<PeerListResponse, ClientError> {
    let request = Request::PeerList(peers.to_vec());
    tracing::debug!(?peers, "Sending the peers list");
    client.send(peer_addr, request).await
}

async fn add_block<C: Client>(
    block: Block,
    peer_addr: SocketAddr,
    client: Arc<C>,
) -> Result<bool, ClientError> {
    let request = Request::AddBlock(block);
    let response: AddBlockResponse = client.send(peer_addr, request).await?;
    tracing::debug!(response.is_block_added, "Adding new block");
    Ok(response.is_block_added)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::network::client::test_utils::TestClient;
    use crate::node::health::HealthStatus;
    use crate::node::peer::Peer;

    #[tokio::test]
    async fn test_return_error_on_invalid_response() {
        let client = TestClient::new(vec![]);
        let broadcaster = NodeBroadcaster::new(Arc::new(client));

        let mut node = Node::new(52000);
        node.peers.insert(
            Peer::new("0.0.0.0:52001".parse().unwrap()),
            HealthStatus::Healthy,
        );

        let response = broadcaster.broadcast_peer_list(&node).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_peer_list_to_peers() {
        let response = serde_json::to_vec(&PeerListResponse).unwrap();
        let client = TestClient::new(response);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let mut node = Node::new(52000);
        let peer_1 = ([0, 0, 0, 0], 52001).into();
        let peer_2 = ([0, 0, 0, 0], 52002).into();
        node.add_peers(&[peer_1, peer_2]);

        let response = broadcaster.broadcast_peer_list(&node).await;
        assert!(response.is_ok());

        let request = Request::PeerList(node.cluster_peers().iter().map(|p| p.addr).collect());
        let requests = client.requests.lock().await;

        assert_eq!(requests.get(&peer_1), Some(&request));
        assert_eq!(requests.get(&peer_2), Some(&request));
    }

    #[tokio::test]
    async fn test_broadcast_new_block_to_peers() {
        let response = serde_json::to_vec(&AddBlockResponse {
            is_block_added: true,
        })
        .unwrap();
        let client = TestClient::new(response);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let mut node = Node::new(52000);
        let peer_1 = ([0, 0, 0, 0], 52001).into();
        let peer_2 = ([0, 0, 0, 0], 52002).into();
        node.add_peers(&[peer_1, peer_2]);

        let block = Block::new(1, "test block".to_string(), "".to_string(), 1);

        let response = broadcaster.broadcast_new_block(&block, &node).await;
        assert!(response.is_ok());

        let request = Request::AddBlock(block);
        let requests = client.requests.lock().await;

        assert_eq!(requests.get(&peer_1), Some(&request));
        assert_eq!(requests.get(&peer_2), Some(&request));
    }

    #[tokio::test]
    async fn test_broadcast_peer_list_to_no_peers() {
        let client = TestClient::new(vec![]);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));
        let node = Node::new(52000);

        let response = broadcaster.broadcast_peer_list(&node).await;
        assert!(response.is_ok());

        let requests = client.requests.lock().await;
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn test_broadcast_new_block_to_no_peers() {
        let client = TestClient::new(vec![]);
        let client = Arc::new(client);
        let broadcaster = NodeBroadcaster::new(Arc::clone(&client));

        let node = Node::new(52000);
        let block = Block::new(1, "test block".to_string(), "".to_string(), 1);

        let response = broadcaster.broadcast_new_block(&block, &node).await;
        assert!(response.is_ok());

        let requests = client.requests.lock().await;
        assert!(requests.is_empty());
    }
}
