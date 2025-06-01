use crate::api::request::Request;
use crate::api::response::Pong;
use crate::network::client::Client;
use crate::node::Node;
use futures::future::join_all;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum HealthStatus {
    Healthy,
    Unhealthy(u8),
}

pub trait HealthChecker {
    fn ping(&self, addr: SocketAddr) -> impl Future<Output = bool> + Send;
}

pub struct NodeHealthChecker<C: Client> {
    client: Arc<C>,
}

impl<C: Client> NodeHealthChecker<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> HealthChecker for NodeHealthChecker<C>
where
    C: Client + Send + Sync + 'static,
{
    async fn ping(&self, addr: SocketAddr) -> bool {
        self.client.send::<Pong>(addr, Request::Ping).await.is_ok()
    }
}

pub async fn run_health_check<A>(
    health_checker: Arc<A>,
    node: Arc<Mutex<Node>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    A: HealthChecker + Send + Sync,
{
    let peers: Vec<_> = {
        let node = node.lock().await;
        node.peers.keys().copied().collect()
    };

    let health_check_tasks: Vec<_> = peers
        .into_iter()
        .map(|peer| {
            let health_checker = Arc::clone(&health_checker);
            async move {
                let is_healthy = health_checker.ping(peer.addr).await;
                if is_healthy { None } else { Some(peer) }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::client::test_utils::TestClient;

    #[tokio::test]
    async fn test_health_checker_for_active_node() {
        let response = serde_json::to_vec(&Pong).unwrap();
        let client = Arc::new(TestClient::new(response));
        let health_checker = NodeHealthChecker::new(client);
        let peer_addr = ([0, 0, 0, 0], 52001).into();
        let is_healthy = health_checker.ping(peer_addr).await;
        assert!(is_healthy);
    }

    #[tokio::test]
    async fn test_health_checker_for_corrupted_node() {
        let client = Arc::new(TestClient::new(vec![]));
        let health_checker = NodeHealthChecker::new(client);
        let peer_addr = ([0, 0, 0, 0], 52001).into();
        let is_healthy = health_checker.ping(peer_addr).await;
        assert!(!is_healthy);
    }
}
