use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Peer {
    pub addr: SocketAddr,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

impl ToSocketAddrs for Peer {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(vec![self.addr].into_iter())
    }
}
