use std::net::SocketAddr;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Peer {
    pub addr: SocketAddr,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}
