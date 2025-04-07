use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::blockchain::{Block, Blockchain};
use crate::network::message::{PeerError, PeerMessage};

pub async fn start_server(
    port: u16,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::debug!("Server listening on port {}", port);

    loop {
        let (socket, addr) = listener.accept().await?;
        let blockchain = Arc::clone(&blockchain);
        tokio::spawn(accept_socket(socket, addr, blockchain));
    }
}

#[tracing::instrument(skip(socket, blockchain))]
async fn accept_socket(
    mut socket: TcpStream,
    addr: SocketAddr,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), PeerError> {
    tracing::debug!("Accepted new connection from {:?}", addr);

    let mut buffer = vec![0u8; 8192];
    let read_bytes = socket.read(&mut buffer).await?;

    let message = serde_json::from_slice::<PeerMessage>(&buffer[..read_bytes])?;
    tracing::debug!("Received new request from peer: {:?}", message);

    match message {
        PeerMessage::GetChainRequest => send_chain(blockchain, &mut socket).await,
        PeerMessage::GetChainResponse(chain) => replace_chain(chain, blockchain),
        PeerMessage::AddBlockRequest(data) => add_block(data, blockchain),
    }
}

async fn send_chain(
    blockchain: Arc<Mutex<Blockchain>>,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let chain = blockchain.lock().unwrap().chain.clone();
    tracing::debug!("Sending chain to peer");
    let response = PeerMessage::GetChainResponse(chain);
    let serialized_response = serde_json::to_vec(&response)?;
    socket.write_all(&serialized_response).await?;
    Ok(())
}

fn replace_chain(chain: Vec<Block>, blockchain: Arc<Mutex<Blockchain>>) -> Result<(), PeerError> {
    let is_chain_replaced = blockchain.lock().unwrap().try_replace_chain(chain);
    // todo: return error if is_chain_replaced is equal to false
    tracing::debug!(is_chain_replaced, "Replacing chain received from peer");
    Ok(())
}

fn add_block(data: String, blockchain: Arc<Mutex<Blockchain>>) -> Result<(), PeerError> {
    blockchain.lock().unwrap().add_block(data);
    // todo: send response with block index
    tracing::debug!("Adding new block to peer");
    Ok(())
}
