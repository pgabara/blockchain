use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::blockchain::Blockchain;
use crate::network::message::{AddBlockResponse, GetChainResponse, PeerError, PeerRequest};

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

    let message = serde_json::from_slice::<PeerRequest>(&buffer[..read_bytes])?;
    tracing::debug!("Received new request from peer: {:?}", message);

    match message {
        PeerRequest::GetChainRequest => send_chain(blockchain, &mut socket).await,
        PeerRequest::AddBlockRequest(data) => mine_block(data, blockchain, &mut socket).await,
    }
}

async fn send_chain(
    blockchain: Arc<Mutex<Blockchain>>,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let chain = blockchain.lock().unwrap().chain.clone();
    tracing::debug!("Sending chain to peer");
    let response = GetChainResponse { chain };
    let serialized_response = serde_json::to_vec(&response)?;
    socket.write_all(&serialized_response).await?;
    Ok(())
}

async fn mine_block(
    data: String,
    blockchain: Arc<Mutex<Blockchain>>,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let index = blockchain.lock().unwrap().mine_block(data);
    tracing::debug!("Mining new block");
    let response = AddBlockResponse { index };
    let serialized_response = serde_json::to_vec(&response)?;
    socket.write_all(&serialized_response).await?;
    Ok(())
}
