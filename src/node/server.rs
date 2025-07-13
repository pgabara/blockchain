use crate::api::request::Request;
use crate::api::response::*;
use crate::blockchain::{Block, Blockchain};
use crate::node::Node;
use crate::node::broadcast::{Broadcaster, BroadcasterError};
use crate::transaction::Transaction;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
    #[error("Invalid request: {0}")]
    InvalidMessage(#[from] serde_json::error::Error),
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] BroadcasterError),
}

pub async fn start_server<B>(
    port: u16,
    blockchain: Arc<Mutex<Blockchain>>,
    transactions_queue: Arc<Mutex<Vec<Transaction>>>,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<B>,
) -> Result<(), ServerError>
where
    B: Broadcaster + Send + Sync + 'static,
{
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::debug!("Server is listening on port {}", port);

    loop {
        let (socket, addr) = listener.accept().await?;
        let blockchain = Arc::clone(&blockchain);
        let node = Arc::clone(&node);
        let broadcaster = Arc::clone(&broadcaster);
        let transactions_queue = Arc::clone(&transactions_queue);
        tokio::spawn(accept_socket(
            socket,
            addr,
            blockchain,
            transactions_queue,
            node,
            broadcaster,
        ));
    }
}

async fn accept_socket(
    mut socket: TcpStream,
    addr: SocketAddr,
    blockchain: Arc<Mutex<Blockchain>>,
    transactions_queue: Arc<Mutex<Vec<Transaction>>>,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<impl Broadcaster>,
) -> Result<(), ServerError> {
    tracing::debug!("Accepted new connection from {}", addr);

    let mut buffer = vec![0u8; 8192];
    let read_bytes = socket.read(&mut buffer).await?;

    let message = serde_json::from_slice::<Request>(&buffer[..read_bytes])?;
    tracing::debug!("Received new request from peer: {:?}", message);

    match message {
        Request::Ping => ping(&mut socket).await,
        Request::Join(addr) => join(node, addr, &mut socket).await,
        Request::PeerList(peers) => peer_list(peers, node, broadcaster, &mut socket).await,
        Request::GetChain => get_chain(blockchain, &mut socket).await,
        Request::SyncBlock(block) => {
            sync_block(block, blockchain, transactions_queue, &mut socket).await
        }
        Request::AddTransaction(transaction) => {
            add_transaction(
                transaction,
                transactions_queue,
                node,
                broadcaster,
                &mut socket,
            )
            .await
        }
        Request::SyncTransaction(transaction) => {
            sync_transaction(transaction, transactions_queue, &mut socket).await
        }
    }
}

async fn ping(socket: &mut TcpStream) -> Result<(), ServerError> {
    tracing::trace!("Received a ping message");
    send_response(Pong, socket).await
}

async fn join(
    node: Arc<Mutex<Node>>,
    addr: SocketAddr,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    let mut node = node.try_lock().unwrap();
    node.add_peers(&[addr]);
    let peers: Vec<SocketAddr> = node.cluster_peers().iter().map(|p| p.addr).collect();
    let response = JoinResponse { peers };
    send_response(response, socket).await
}

async fn peer_list(
    peers: Vec<SocketAddr>,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<impl Broadcaster>,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    let (peers, cluster_peers, is_updated) = {
        let mut node = node.try_lock().unwrap();
        let is_updated = node.add_peers(&peers);
        let cluster_peers = node.cluster_peers();
        let peers: Vec<_> = node.peers.keys().copied().collect();
        (peers, cluster_peers, is_updated)
    };
    if is_updated {
        tracing::debug!("Peer list updated. Broadcasting the updated list to all peers.");
        broadcaster
            .broadcast_peer_list(&peers, cluster_peers)
            .await?;
    }
    send_response(PeerListResponse, socket).await
}

async fn get_chain(
    blockchain: Arc<Mutex<Blockchain>>,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    let chain = blockchain.try_lock().unwrap().chain.clone();
    tracing::debug!("Sending the chain to peer");
    let response = GetChainResponse { chain };
    send_response(response, socket).await
}

async fn sync_block(
    block: Block,
    blockchain: Arc<Mutex<Blockchain>>,
    transactions_queue: Arc<Mutex<Vec<Transaction>>>,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    {
        let mut transactions = transactions_queue.try_lock().unwrap();
        for tx in block.transactions.iter() {
            transactions.retain(|mem_tx| mem_tx != tx);
        }
    }
    let is_block_added = blockchain.try_lock().unwrap().add_block(block);
    tracing::debug!(is_block_added, "Adding a new block (sync)");
    let response = SyncBlockResponse { is_block_added };
    send_response(response, socket).await
}

async fn add_transaction(
    transaction: Transaction,
    transactions_queue: Arc<Mutex<Vec<Transaction>>>,
    node: Arc<Mutex<Node>>,
    broadcaster: Arc<impl Broadcaster>,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    transactions_queue
        .try_lock()
        .unwrap()
        .push(transaction.clone());
    tracing::debug!("Adding a new transaction");
    let peers: Vec<_> = node.try_lock().unwrap().peers.keys().copied().collect();
    broadcaster
        .broadcast_transaction(transaction, &peers)
        .await?;
    let response = AddTransactionResponse {
        is_transaction_added: true,
    };
    send_response(response, socket).await
}

async fn sync_transaction(
    transaction: Transaction,
    transactions_queue: Arc<Mutex<Vec<Transaction>>>,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    tracing::debug!("Adding a new transaction (sync)");
    transactions_queue
        .try_lock()
        .unwrap()
        .push(transaction.clone());
    let response = SyncTransactionResponse {
        is_transaction_added: true,
    };
    send_response(response, socket).await
}

async fn send_response<A: Serialize>(
    response: A,
    socket: &mut TcpStream,
) -> Result<(), ServerError> {
    let serialized_response = serde_json::to_vec(&response)?;
    socket.write_all(&serialized_response).await?;
    Ok(())
}
