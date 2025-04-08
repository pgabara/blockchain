use crate::blockchain::Block;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum PeerRequest {
    GetChainRequest,
    AddBlockRequest(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetChainResponse {
    pub chain: Vec<Block>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AddBlockResponse {
    pub index: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum PeerError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
    #[error("Invalid request: {0}")]
    InvalidMessage(#[from] serde_json::error::Error),
}
