use crate::blockchain::Block;
use crate::network::message::{PeerError, PeerMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

pub async fn get_chain<A: ToSocketAddrs>(peer: A) -> Result<Vec<Block>, PeerError> {
    let response = send_message(peer, PeerMessage::GetChainRequest).await?;
    if let PeerMessage::GetChainResponse(response) = response {
        return Ok(response);
    }
    Err(PeerError::UnsupportedMessageType)
}

async fn send_message<A: ToSocketAddrs>(
    peer: A,
    message: PeerMessage,
) -> Result<PeerMessage, PeerError> {
    let mut stream = TcpStream::connect(peer).await?;
    let json_message = serde_json::to_vec(&message)?;
    stream.write_all(&json_message).await?;
    tracing::debug!(?message, "Message sent to peer");

    let mut buffer = vec![0u8; 8192];
    let read_bytes = stream.read(&mut buffer).await?;
    let response = serde_json::from_slice::<PeerMessage>(&buffer[..read_bytes])?;
    tracing::debug!(?response, "Response received from peer");
    Ok(response)
}
