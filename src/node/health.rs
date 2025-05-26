use crate::network::client;
use crate::node::Node;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum HealthStatus {
    Healthy,
    Unhealthy(u8),
}

pub async fn run_health_check(node: Arc<Mutex<Node>>) -> Result<(), Box<dyn std::error::Error>> {
    let peers: Vec<_> = {
        let node = node.lock().await;
        node.peers.keys().copied().collect()
    };

    let health_check_tasks: Vec<_> = peers
        .into_iter()
        .map(|peer| async move {
            let is_healthy = client::ping_peer(peer.addr).await;
            if is_healthy { None } else { Some(peer) }
        })
        .collect();

    let unreachable_peers: Vec<_> = join_all(health_check_tasks)
        .await
        .into_iter()
        .flatten()
        .collect();

    let mut node = node.lock().await;
    node.update_peers_health_status(&unreachable_peers);

    Ok(())
}
