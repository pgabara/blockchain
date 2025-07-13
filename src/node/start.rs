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
use crate::transaction::Transaction;
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

pub async fn start_node(args: Args) -> Result<(), ClusterInitError> {
    let blockchain = Arc::new(Mutex::new(Blockchain::new(args.difficulty)));
    let transactions_queue = Arc::new(Mutex::new(Vec::new()));
    let node = Arc::new(Mutex::new(Node::new(args.port)));
    let client = Arc::new(TcpClient::default());
    let broadcaster = Arc::new(NodeBroadcaster::new(Arc::clone(&client)));
    let health_checker = Arc::new(NodeHealthChecker::new(Arc::clone(&client)));

    if let Some(seed_peer) = args.seed_node {
        join_cluster(
            seed_peer,
            Arc::clone(&node),
            Arc::clone(&blockchain),
            Arc::clone(&client),
        )
        .await?;
    }

    sync_blockchain_peers_background_task(
        health_checker,
        Arc::clone(&broadcaster),
        Arc::clone(&node),
        args.cluster_sync_period,
    );

    mine_block_background_task(
        Arc::clone(&broadcaster),
        Arc::clone(&node),
        Arc::clone(&blockchain),
        Arc::clone(&transactions_queue),
        args.new_block_mine_period,
    );

    tracing::debug!("Starting a blockchain server");
    server::start_server(
        args.port,
        Arc::clone(&blockchain),
        Arc::clone(&transactions_queue),
        Arc::clone(&node),
        broadcaster,
    )
    .await?;

    Ok(())
}

async fn join_cluster<C>(
    seed_peer: SocketAddr,
    node: Arc<Mutex<Node>>,
    blockchain: Arc<Mutex<Blockchain>>,
    client: Arc<C>,
) -> Result<(), ClusterInitError>
where
    C: Client + Send + Sync,
{
    tracing::debug!(?seed_peer, "Joining a blockchain cluster");
    let peers = {
        let mut node = node.try_lock().unwrap();
        let peers = join(seed_peer, node.node_addr, client.as_ref()).await?;
        let peers: Vec<_> = peers.iter().map(|p| p.addr).collect();
        node.add_peers(&peers);
        peers
    };
    tracing::debug!(?seed_peer, ?peers, "List of cluster peers updated");
    let chain = get_chain(seed_peer, client.as_ref()).await?;
    let is_chain_replaced = blockchain.try_lock().unwrap().try_replace_chain(chain);
    tracing::debug!(is_chain_replaced, "Replacing the existing chain");
    Ok(())
}

fn sync_blockchain_peers_background_task<A, B>(
    health_checker: Arc<A>,
    broadcaster: Arc<B>,
    node: Arc<Mutex<Node>>,
    cluster_sync_period: u64,
) where
    A: HealthChecker + Send + Sync + 'static,
    B: Broadcaster + Send + Sync + 'static,
{
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(cluster_sync_period)).await;
            tracing::debug!("Sending peer sync event");
            let _ = run_health_check(Arc::clone(&health_checker), Arc::clone(&node)).await;
            let (peers, cluster_peers) = {
                let node = node.try_lock().unwrap();
                let peers: Vec<_> = node.peers.keys().copied().collect();
                let cluster_peers = node.cluster_peers();
                (peers, cluster_peers)
            };
            let _ = broadcaster.broadcast_peer_list(&peers, cluster_peers).await;
        }
    });
}

fn mine_block_background_task<B>(
    broadcaster: Arc<B>,
    node: Arc<Mutex<Node>>,
    blockchain: Arc<Mutex<Blockchain>>,
    transactions_queue: Arc<Mutex<Vec<Transaction>>>,
    new_block_mine_period: u64,
) where
    B: Broadcaster + Send + Sync + 'static,
{
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(new_block_mine_period)).await;
            let transactions = {
                let mut transactions_queue = transactions_queue.try_lock().unwrap();
                let transactions = transactions_queue.clone();
                transactions_queue.clear();
                transactions
            };
            if !transactions.is_empty() {
                tracing::debug!("Mining a new block");
                let block = blockchain
                    .try_lock()
                    .unwrap()
                    .mine_block(transactions)
                    .clone();
                let peers: Vec<_> = node.try_lock().unwrap().peers.keys().copied().collect();
                let _ = broadcaster.broadcast_new_block(&block, &peers).await;
            }
        }
    });
}

async fn join(
    seed_addr: SocketAddr,
    node_addr: SocketAddr,
    client: &impl Client,
) -> Result<Vec<Peer>, ClusterInitError> {
    let request = Request::Join(node_addr);
    let response: JoinResponse = client.send(seed_addr, request).await?;
    tracing::debug!(?response.peers, ?node_addr, "Received join response");
    let peers = response.peers.into_iter().map(Peer::new).collect();
    Ok(peers)
}

async fn get_chain(
    seed_addr: SocketAddr,
    client: &impl Client,
) -> Result<Vec<Block>, ClusterInitError> {
    let response: GetChainResponse = client.send(seed_addr, Request::GetChain).await?;
    let chain_len = response.chain.len();
    tracing::debug!(chain_len, "Received the chain from peer");
    Ok(response.chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::client::test_utils::TestClient;
    use crate::transaction::Transaction;

    #[tokio::test]
    async fn join_cluster_sends_node_address_to_seed_node() {
        let response = serde_json::to_vec(&JoinResponse { peers: vec![] }).unwrap();
        let client = TestClient::new(response);

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let node_addr = ([127, 0, 0, 1], 50002).into();
        let _ = join(seed_addr, node_addr, &client).await;

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
        let client = TestClient::new(response);

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let node_addr = ([127, 0, 0, 1], 50004).into();
        let response = join(seed_addr, node_addr, &client).await.unwrap();

        let expected_peers: Vec<_> = peers.into_iter().map(Peer::new).collect();
        assert_eq!(response, expected_peers);
    }

    #[tokio::test]
    async fn join_cluster_returns_error_on_invalid_seed_node_response() {
        let client = TestClient::new(vec![]);

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let node_addr = ([127, 0, 0, 1], 50002).into();
        let response = join(seed_addr, node_addr, &client).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn get_chain_returns_chain_stored_by_cluster_peer() {
        let transactions = vec![
            Transaction::new("0".to_string(), "1".to_string(), 800),
            Transaction::new("3".to_string(), "4".to_string(), 100),
        ];
        let blocks = vec![
            Block::new(1, transactions.clone(), "1".to_string(), 1),
            Block::new(2, transactions.clone(), "2".to_string(), 1),
            Block::new(3, transactions, "3".to_string(), 1),
        ];
        let response = serde_json::to_vec(&GetChainResponse {
            chain: blocks.clone(),
        })
        .unwrap();
        let client = TestClient::new(response);

        let seed_addr = ([127, 0, 0, 1], 50001).into();
        let response = get_chain(seed_addr, &client).await.unwrap();
        assert_eq!(response, blocks);
    }
}
