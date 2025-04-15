use crate::blockchain::{Block, Blockchain};
use crate::network::message::*;
use crate::node::Node;
use crate::sync;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub async fn start_server(
    port: u16,
    blockchain: Arc<Mutex<Blockchain>>,
    node: Arc<Mutex<Node>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::debug!("Server listening on port {}", port);

    loop {
        let (socket, addr) = listener.accept().await?;
        let blockchain = Arc::clone(&blockchain);
        let node = Arc::clone(&node);
        tokio::spawn(accept_socket(socket, addr, blockchain, node));
    }
}

#[tracing::instrument(skip(socket, blockchain, node))]
async fn accept_socket(
    mut socket: TcpStream,
    addr: SocketAddr,
    blockchain: Arc<Mutex<Blockchain>>,
    node: Arc<Mutex<Node>>,
) -> Result<(), PeerError> {
    tracing::debug!("Accepted new connection from {:?}", addr);

    let mut buffer = vec![0u8; 8192];
    let read_bytes = socket.read(&mut buffer).await?;

    let message = serde_json::from_slice::<PeerRequest>(&buffer[..read_bytes])?;
    tracing::debug!("Received new request from peer: {:?}", message);

    match message {
        PeerRequest::Join(addr) => join(node, addr, &mut socket).await,
        PeerRequest::GetChain => get_chain(blockchain, &mut socket).await,
        PeerRequest::AddBlock(block) => add_block(block, blockchain, &mut socket).await,
        PeerRequest::MineBlock(data) => mine_block(data, blockchain, node, &mut socket).await,
    }
}

async fn join(
    node: Arc<Mutex<Node>>,
    addr: SocketAddr,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let mut node = node.lock().await;
    let peers = &node.peers;
    let peers: Vec<_> = peers.iter().map(|p| p.addr).collect();
    node.add_peer(addr);
    let response = JoinResponse { peers };
    send_response(response, socket).await
}

async fn get_chain(
    blockchain: Arc<Mutex<Blockchain>>,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let chain = blockchain.lock().await.chain.clone();
    tracing::debug!("Sending chain to peer");
    let response = GetChainResponse { chain };
    send_response(response, socket).await
}

async fn add_block(
    block: Block,
    blockchain: Arc<Mutex<Blockchain>>,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let is_block_added = blockchain.lock().await.add_block(block);
    tracing::debug!(is_block_added, "Adding new block");
    let response = AddBlockResponse { is_block_added };
    send_response(response, socket).await
}

async fn mine_block(
    data: String,
    blockchain: Arc<Mutex<Blockchain>>,
    node: Arc<Mutex<Node>>,
    socket: &mut TcpStream,
) -> Result<(), PeerError> {
    let mut blockchain = blockchain.lock().await;
    let node = node.lock().await;
    let new_block = blockchain.mine_block(data);
    tracing::debug!(new_block.index, "Mining block");
    sync::broadcast_new_block(new_block, &node).await?;
    let response = MineBlockResponse {
        block_index: new_block.index,
    };
    send_response(response, socket).await
}

async fn send_response<A: Serialize>(response: A, socket: &mut TcpStream) -> Result<(), PeerError> {
    let serialized_response = serde_json::to_vec(&response)?;
    socket.write_all(&serialized_response).await?;
    Ok(())
}
