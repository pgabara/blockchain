pub mod broadcast;
mod health;
mod peer;
pub mod start;

use crate::node::health::HealthStatus;
use crate::node::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct Node {
    pub node_addr: SocketAddr,
    pub peers: HashMap<Peer, HealthStatus>,
}

impl Node {
    pub fn new(port: u16) -> Self {
        let node_addr = SocketAddr::from(([0, 0, 0, 0], port));
        let peers = HashMap::new();
        Self { node_addr, peers }
    }

    pub fn cluster_peers(&self) -> Vec<Peer> {
        let self_peer = Peer::new(self.node_addr);
        self.peers
            .keys()
            .copied()
            .chain(std::iter::once(self_peer))
            .collect()
    }

    pub fn add_peers(&mut self, new_peers: &[SocketAddr]) -> bool {
        let mut is_peer_added = false;
        for &peer in new_peers {
            let new_peer = Peer::new(peer);
            if peer != self.node_addr && !self.peers.contains_key(&new_peer) {
                self.peers.insert(new_peer, HealthStatus::Healthy);
                is_peer_added = true;
            }
        }
        is_peer_added
    }

    pub fn update_peers_health_status(&mut self, unreachable_peers: &[Peer]) {
        self.peers.retain(|peer, health_status| {
            if unreachable_peers.contains(peer) {
                return match health_status {
                    HealthStatus::Healthy => {
                        *health_status = HealthStatus::Unhealthy(1);
                        true
                    }
                    HealthStatus::Unhealthy(n) if *n < 3 => {
                        *health_status = HealthStatus::Unhealthy(*n + 1);
                        true
                    }
                    _ => false,
                };
            }
            true
        });
    }
}
