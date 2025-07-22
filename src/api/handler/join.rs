use crate::api::response::JoinResponse;
use crate::node::Node;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(peer_addr: SocketAddr, node: Arc<Mutex<Node>>) -> JoinResponse {
    tracing::debug!("Received Join request from {}", peer_addr);
    let mut node = node.lock().await;
    node.add_peers(&[peer_addr]);
    let peers: Vec<SocketAddr> = node.cluster_peers().iter().map(|p| p.addr).collect();
    tracing::debug!("Sending back connected cluster peers {:?}", peers);
    JoinResponse { peers }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn return_list_of_all_cluster_peers() {
        let joining_peer: SocketAddr = ([0, 0, 0, 0], 2552).into();
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let cluster_peers = handle_request(joining_peer, Arc::clone(&node)).await;
        let expected_cluster_peers = vec![joining_peer, ([0, 0, 0, 0], 2551).into()];
        assert_eq!(cluster_peers.peers, expected_cluster_peers)
    }

    #[tokio::test]
    async fn add_joining_peer_to_node_peers() {
        let joining_peer: SocketAddr = ([0, 0, 0, 0], 2552).into();
        let node = Arc::new(Mutex::new(Node::new(2551)));
        let _ = handle_request(joining_peer, Arc::clone(&node)).await;
        let node_peers: Vec<_> = node
            .lock()
            .await
            .peers
            .keys()
            .map(|p| &p.addr)
            .copied()
            .collect();
        assert!(node_peers.contains(&joining_peer));
    }
}
