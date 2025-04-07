use crate::blockchain::Block;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum PeerMessage {
    GetChainRequest,
    GetChainResponse(Vec<Block>),
    AddBlockRequest(String),
}

#[derive(thiserror::Error, Debug)]
pub enum PeerError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
    #[error("Invalid request: {0}")]
    InvalidMessage(#[from] serde_json::error::Error),
    #[error("Unsupported message type")]
    UnsupportedMessageType,
}
