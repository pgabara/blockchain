use crate::args::Args;
use crate::blockchain::Blockchain;
use crate::network::{client, server};
use crate::node::Node;
use crate::node::broadcast::NodeStateBroadcast;
use crate::node::health::run_health_check;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_node(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain = Arc::new(Mutex::new(Blockchain::new(args.difficulty)));
    let node = Arc::new(Mutex::new(Node::new(args.port)));

    if let Some(seed_peer) = args.seed_node {
        join_cluster(seed_peer, Arc::clone(&node), Arc::clone(&blockchain)).await?;
    }

    sync_blockchain_peers(Arc::clone(&node), args.cluster_sync_period);

    tracing::debug!("Starting a new blockchain");
    server::start_server(args.port, Arc::clone(&blockchain), Arc::clone(&node)).await
}

async fn join_cluster(
    seed_peer: SocketAddr,
    node: Arc<Mutex<Node>>,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(?seed_peer, "Joining blockchain cluster");
    let mut node = node.lock().await;
    let peers = client::join_cluster(seed_peer, node.node_addr).await?;
    node.add_peers(&peers);
    tracing::debug!(?seed_peer, ?peers, "List of cluster peers updated");
    let chain = client::get_chain(seed_peer).await?;
    let is_chain_replaced = blockchain.lock().await.try_replace_chain(chain);
    tracing::debug!(is_chain_replaced, "Replacing the existing chain");
    Ok(())
}

fn sync_blockchain_peers(node: Arc<Mutex<Node>>, cluster_sync_period: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(cluster_sync_period)).await;
        tracing::debug!("Sending peer sync event");
        let _ = run_health_check(Arc::clone(&node)).await;
        let node = node.lock().await;
        let _ = node.broadcast_peer_list().await;
    });
}
