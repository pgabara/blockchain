use crate::blockchain::Block;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum Request {
    Join(std::net::SocketAddr),
    PeerList(Vec<std::net::SocketAddr>),
    GetChain,
    AddBlock(Block),
    MineBlock(String),
    Ping,
}
