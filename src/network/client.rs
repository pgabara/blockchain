use crate::blockchain::Block;
use crate::network::message::{AddBlockResponse, GetChainResponse, PeerError, PeerRequest};
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

pub async fn get_chain<A: ToSocketAddrs>(peer: A) -> Result<Vec<Block>, PeerError> {
    let response: GetChainResponse = send_message(peer, PeerRequest::GetChainRequest).await?;
    tracing::debug!(
        "Received chain from peer. Chain length: {}",
        response.chain.len()
    );
    Ok(response.chain)
}

pub async fn mine_block<A: ToSocketAddrs>(data: String, peer: A) -> Result<usize, PeerError> {
    let response: AddBlockResponse = send_message(peer, PeerRequest::AddBlockRequest(data)).await?;
    tracing::debug!("New block mined. Block index: {}", response.index);
    Ok(response.index)
}

async fn send_message<A, B>(peer: A, message: PeerRequest) -> Result<B, PeerError>
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
