use crate::api::request::Request;
use crate::api::response::{GetChainResponse, JoinResponse};
use crate::args::Args;
use crate::blockchain::{Block, Blockchain};
use crate::network::client::{Client, ClientError, TcpClient};
use crate::node::broadcast::{Broadcaster, NodeBroadcaster};
use crate::node::health::{HealthChecker, NodeHealthChecker, run_health_check};
use crate::node::peer::Peer;
use crate::node::server::ServerError;
use crate::node::{Node, server};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(thiserror::Error, Debug)]
pub enum ClusterInitError {
    #[error("Server Error: {0}")]
    ServerError(#[from] ServerError),
    #[error("Client Error: {0}")]
    ClientError(#[from] ClientError),
}

pub trait ClusterInit {
    fn join(
        &self,
        seed_addr: SocketAddr,
        node_addr: SocketAddr,
    ) -> impl Future<Output = Result<Vec<Peer>, ClusterInitError>> + Send;
    fn get_chain(
        &self,
        seed_addr: SocketAddr,
    ) -> impl Future<Output = Result<Vec<Block>, ClusterInitError>> + Send;
}

pub struct ClusterInitializer<C: Client> {
    client: Arc<C>,
}

impl<C: Client> ClusterInitializer<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> ClusterInit for ClusterInitializer<C>
where
    C: Client + Send + Sync,
{
    async fn join(
        &self,
        seed_addr: SocketAddr,
        node_addr: SocketAddr,
    ) -> Result<Vec<Peer>, ClusterInitError> {
        let request = Request::Join(node_addr);
        let response: JoinResponse = self.client.send(seed_addr, request).await?;
        tracing::debug!(?response.peers, ?node_addr, "Received join response");
        let peers = response.peers.into_iter().map(Peer::new).collect();
        Ok(peers)
    }

    async fn get_chain(&self, seed_addr: SocketAddr) -> Result<Vec<Block>, ClusterInitError> {
        let response: GetChainResponse = self.client.send(seed_addr, Request::GetChain).await?;
        let chain_len = response.chain.len();
        tracing::debug!(chain_len, "Received the chain from peer");
        Ok(response.chain)
    }
}

pub async fn start_node(args: Args) -> Result<(), ClusterInitError> {
    let blockchain = Arc::new(Mutex::new(Blockchain::new(args.difficulty)));
    let node = Arc::new(Mutex::new(Node::new(args.port)));
    let client = Arc::new(TcpClient);
    let broadcaster = Arc::new(NodeBroadcaster::new(Arc::clone(&client)));
    let cluster_init = Arc::new(ClusterInitializer::new(Arc::clone(&client)));
    let health_checker = Arc::new(NodeHealthChecker::new(client));

    if let Some(seed_peer) = args.seed_node {
        join_cluster(
            seed_peer,
            Arc::clone(&node),
            Arc::clone(&blockchain),
            cluster_init,
        )
        .await?;
    }

    sync_blockchain_peers(
        health_checker,
        Arc::clone(&broadcaster),
        Arc::clone(&node),
        args.cluster_sync_period,
    );

    tracing::debug!("Starting a blockchain server");
    server::start_server(
        args.port,
        Arc::clone(&blockchain),
        Arc::clone(&node),
        broadcaster,
    )
    .await?;

    Ok(())
}

async fn join_cluster<A>(
    seed_peer: SocketAddr,
    node: Arc<Mutex<Node>>,
    blockchain: Arc<Mutex<Blockchain>>,
    cluster_init: Arc<A>,
) -> Result<(), ClusterInitError>
where
    A: ClusterInit + Send + Sync,
{
    tracing::debug!(?seed_peer, "Joining a blockchain cluster");
    let mut node = node.lock().await;
    let peers = cluster_init.join(seed_peer, node.node_addr).await?;
    let peers: Vec<_> = peers.iter().map(|p| p.addr).collect();
    node.add_peers(&peers);
    tracing::debug!(?seed_peer, ?peers, "List of cluster peers updated");
    let chain = cluster_init.get_chain(seed_peer).await?;
    let is_chain_replaced = blockchain.lock().await.try_replace_chain(chain);
    tracing::debug!(is_chain_replaced, "Replacing the existing chain");
    Ok(())
}

fn sync_blockchain_peers<A, B>(
    health_checker: Arc<A>,
    broadcaster: Arc<B>,
    node: Arc<Mutex<Node>>,
    cluster_sync_period: u64,
) where
    A: HealthChecker + Send + Sync + 'static,
    B: Broadcaster + Send + Sync + 'static,
{
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(cluster_sync_period)).await;
        tracing::debug!("Sending peer sync event");
        let _ = run_health_check(health_checker, Arc::clone(&node)).await;
        let node = node.lock().await;
        let _ = broadcaster.broadcast_peer_list(&node).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::client::test_utils::TestClient;

    #[tokio::test]
    async fn join_cluster_sends_node_address_to_seed_node() {
        let response = serde_json::to_vec(&JoinResponse { peers: vec![] }).unwrap();
        let client = Arc::new(TestClient::new(response));
        let cluster_init = ClusterInitializer::new(Arc::clone(&client));

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let node_addr = ([127, 0, 0, 1], 50002).into();
        let _ = cluster_init.join(seed_addr, node_addr).await;

        let request = client.requests.lock().await;
        assert_eq!(request.get(&seed_addr), Some(&Request::Join(node_addr)));
    }

    #[tokio::test]
    async fn join_cluster_returns_list_of_cluster_peers() {
        let peers = vec![
            ([127, 0, 0, 1], 50001).into(),
            ([127, 0, 0, 1], 50002).into(),
            ([127, 0, 0, 1], 50003).into(),
        ];
        let response = serde_json::to_vec(&JoinResponse {
            peers: peers.clone(),
        })
        .unwrap();
        let client = Arc::new(TestClient::new(response));
        let cluster_init = ClusterInitializer::new(Arc::clone(&client));

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let node_addr = ([127, 0, 0, 1], 50004).into();
        let response = cluster_init.join(seed_addr, node_addr).await.unwrap();

        let expected_peers: Vec<_> = peers.into_iter().map(Peer::new).collect();
        assert_eq!(response, expected_peers);
    }

    #[tokio::test]
    async fn join_cluster_returns_error_on_invalid_seed_node_response() {
        let client = Arc::new(TestClient::new(vec![]));
        let cluster_init = ClusterInitializer::new(Arc::clone(&client));

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let node_addr = ([127, 0, 0, 1], 50002).into();
        let response = cluster_init.join(seed_addr, node_addr).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn get_chain_returns_chain_stored_by_cluster_peer() {
        let blocks = vec![
            Block::new(1, "test block 1".to_string(), "1".to_string(), 1),
            Block::new(2, "test block 2".to_string(), "2".to_string(), 1),
            Block::new(3, "test block 3".to_string(), "3".to_string(), 1),
        ];
        let response = serde_json::to_vec(&GetChainResponse {
            chain: blocks.clone(),
        })
        .unwrap();
        let client = Arc::new(TestClient::new(response));
        let cluster_init = ClusterInitializer::new(Arc::clone(&client));

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let response = cluster_init.get_chain(seed_addr).await.unwrap();
        assert_eq!(response, blocks);
    }
}
