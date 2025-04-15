use crate::args::Args;
use crate::blockchain::Blockchain;
use crate::network::{client, server};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Node {
    pub node_addr: SocketAddr,
    pub peers: Vec<Peer>,
}

impl Node {
    pub fn from_args(args: &Args) -> Self {
        let node_addr = SocketAddr::from(([0, 0, 0, 0], args.port));
        if let Some(seed_addr) = args.seed_node {
            Self {
                node_addr,
                peers: vec![Peer::new(seed_addr)],
            }
        } else {
            Self {
                node_addr,
                peers: vec![],
            }
        }
    }

    pub fn add_peer(&mut self, addr: SocketAddr) {
        let peer = Peer::new(addr);
        if !self.peers.contains(&peer) {
            self.peers.push(peer);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Peer {
    pub addr: SocketAddr,
}

impl Peer {
    fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

pub async fn start(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain = Arc::new(Mutex::new(Blockchain::new(args.difficulty)));
    let node = Arc::new(Mutex::new(Node::from_args(&args)));

    if let Some(seed_peer) = args.seed_node {
        tracing::debug!(?seed_peer, "Joining existing blockchain");
        client::join_cluster(seed_peer, node.lock().await.node_addr).await?;
        tracing::debug!(?seed_peer, "Getting chain from seed node");
        let chain = client::get_chain(seed_peer).await?;
        let is_chain_replaced = blockchain.lock().await.try_replace_chain(chain);
        tracing::debug!(is_chain_replaced, "Replacing existing chain");
    }

    tracing::debug!("Starting new blockchain");
    server::start_server(args.port, blockchain.clone(), node.clone()).await
}
