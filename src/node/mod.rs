pub mod broadcast;
mod peer;
pub mod start;

use crate::node::peer::Peer;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct Node {
    pub node_addr: SocketAddr,
    pub peers: HashSet<Peer>,
}

impl Node {
    pub fn new(port: u16) -> Self {
        let node_addr = SocketAddr::from(([0, 0, 0, 0], port));
        let peers = HashSet::new();
        Self { node_addr, peers }
    }

    pub fn cluster_peers(&self) -> Vec<Peer> {
        let self_peer = Peer::new(self.node_addr);
        self.peers.iter().copied().chain(std::iter::once(self_peer)).collect()
    }

    pub fn add_peers(&mut self, new_peers: &[SocketAddr]) -> bool {
        let mut is_peer_added = false;
        for &peer in new_peers {
            let new_peer = Peer::new(peer);
            if peer != self.node_addr && !self.peers.contains(&new_peer) {
                self.peers.insert(new_peer);
                is_peer_added = true;
            }
        }
        is_peer_added
    }
}
