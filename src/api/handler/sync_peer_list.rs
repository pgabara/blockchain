use crate::api::response::SyncPeerListResponse;
use crate::node::Node;
use crate::node::broadcast::Broadcaster;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request<B>(
    peers: Vec<SocketAddr>,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<B>,
) -> SyncPeerListResponse
where
    B: Broadcaster + Send + Sync + 'static,
{
    tracing::debug!(?peers, "Received SyncPeerList request");
    let mut node = node.lock().await;
    let are_peers_updated = node.add_peers(&peers);
    if are_peers_updated {
        tracing::debug!(?peers, "List of connected cluster peers updated");
        let peers: Vec<_> = node.peers.keys().copied().collect();
        let cluster_peers = node.cluster_peers();
        tokio::spawn(async move { broadcaster.broadcast_peer_list(cluster_peers, &peers).await });
    }
    SyncPeerListResponse
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::broadcast::test_utils::MockBroadcaster;

    #[tokio::test]
    async fn do_noting_when_no_new_peers() {
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let broadcaster = Arc::new(MockBroadcaster::new());

        let peers = vec![([0, 0, 0, 0], 2551).into()];
        let _ = handle_request(peers, Arc::clone(&node), broadcaster).await;

        assert_eq!(node.lock().await.peers.len(), 0);
    }

    #[tokio::test]
    async fn add_new_peers() {
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let broadcaster = Arc::new(MockBroadcaster::new());

        let peers = vec![([0, 0, 0, 0], 2551).into(), ([0, 0, 0, 0], 2552).into()];
        let _ = handle_request(peers.clone(), Arc::clone(&node), broadcaster).await;

        assert_eq!(node.lock().await.peers.len(), 1);
    }
}
