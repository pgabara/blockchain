use crate::blockchain::Block;
use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct JoinResponse {
    pub peers: Vec<SocketAddr>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetChainResponse {
    pub chain: Vec<Block>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Pong;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AddTransactionResponse {
    pub is_transaction_added: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SyncTransactionResponse;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SyncPeerListResponse;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SyncBlockResponse {
    pub is_block_added: bool,
}
