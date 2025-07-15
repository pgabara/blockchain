use crate::api::response::SyncPeerListResponse;
use crate::network::server::ServerError;
use crate::node::Node;
use crate::node::broadcast::Broadcaster;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request<B>(
    peers: Vec<SocketAddr>,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<B>,
) -> Result<SyncPeerListResponse, ServerError>
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
        tokio::spawn(async move { broadcaster.broadcast_peer_list(&peers, cluster_peers).await });
    }
    Ok(SyncPeerListResponse)
}

#[cfg(test)]
mod tests {
    // todo: write tests here
}
