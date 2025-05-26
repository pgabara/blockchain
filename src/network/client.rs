use crate::blockchain::Block;
use crate::network::message::*;
use std::net::SocketAddr;

use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

pub async fn ping_peer<A: ToSocketAddrs>(peer: A) -> bool {
    match send_message::<A, Pong>(peer, PeerRequest::Ping).await {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Failed to ping peer: {}", e);
            false
        }
    }
}

pub async fn join_cluster<A: ToSocketAddrs>(
    peer: A,
    node_addr: SocketAddr,
) -> Result<Vec<SocketAddr>, ClientError> {
    let request = PeerRequest::Join(node_addr);
    let response: JoinResponse = send_message(peer, request).await?;
    tracing::debug!(?response.peers, ?node_addr, "Received join response");
    Ok(response.peers)
}

pub async fn send_peer_list<A: ToSocketAddrs>(
    peer: A,
    peers: &[SocketAddr],
) -> Result<PeerListResponse, ClientError> {
    let request = PeerRequest::PeerList(peers.to_vec());
    tracing::debug!(?peers, "Sending the peers list");
    send_message(peer, request).await
}

pub async fn get_chain<A: ToSocketAddrs>(peer: A) -> Result<Vec<Block>, ClientError> {
    let response: GetChainResponse = send_message(peer, PeerRequest::GetChain).await?;
    tracing::debug!(
        "Received the chain from peer. Chain length: {}",
        response.chain.len()
    );
    Ok(response.chain)
}

pub async fn add_block<A: ToSocketAddrs>(block: Block, peer: A) -> Result<bool, ClientError> {
    let request = PeerRequest::AddBlock(block);
    let response: AddBlockResponse = send_message(peer, request).await?;
    tracing::debug!(response.is_block_added, "Adding new block");
    Ok(response.is_block_added)
}

pub async fn mine_block<A: ToSocketAddrs>(data: String, peer: A) -> Result<usize, ClientError> {
    let request = PeerRequest::MineBlock(data);
    let response: MineBlockResponse = send_message(peer, request).await?;
    tracing::debug!(response.block_index, "Mining block");
    Ok(response.block_index)
}

async fn send_message<A, B>(peer: A, message: PeerRequest) -> Result<B, ClientError>
where
    A: ToSocketAddrs,
    B: DeserializeOwned,
{
    let mut stream = TcpStream::connect(peer).await?;
    let json_message = serde_json::to_vec(&message)?;
    stream.write_all(&json_message).await?;

    let mut buffer = vec![0u8; 8192];
    let read_bytes = stream.read(&mut buffer).await?;
    let response = serde_json::from_slice::<B>(&buffer[..read_bytes])?;
    Ok(response)
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::error::Error),
}
