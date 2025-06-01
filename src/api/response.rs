use crate::blockchain::Block;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct JoinResponse {
    pub peers: Vec<std::net::SocketAddr>,
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Pong;
