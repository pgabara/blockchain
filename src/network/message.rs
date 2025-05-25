use crate::blockchain::Block;
use crate::node::broadcast::BroadcastError;
use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum PeerRequest {
    Join(SocketAddr),
    PeerList(Vec<SocketAddr>),
    GetChain,
    AddBlock(Block),
    MineBlock(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct JoinResponse {
    pub peers: Vec<SocketAddr>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PeerListResponse;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetChainResponse {
    pub chain: Vec<Block>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AddBlockResponse {
    pub is_block_added: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MineBlockResponse {
    pub block_index: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum PeerError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
    #[error("Invalid request: {0}")]
    InvalidMessage(#[from] serde_json::error::Error),
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] BroadcastError),
}
