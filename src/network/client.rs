use crate::api::request::Request;
use crate::api::response::*;
use crate::blockchain::Block;
use serde::de::DeserializeOwned;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

pub async fn join_cluster<A: ToSocketAddrs>(
    peer: A,
    node_addr: SocketAddr,
) -> Result<Vec<SocketAddr>, ClientError> {
    let request = Request::Join(node_addr);
    let response: JoinResponse = send_message(peer, request).await?;
    tracing::debug!(?response.peers, ?node_addr, "Received join response");
    Ok(response.peers)
}

pub async fn send_peer_list<A: ToSocketAddrs>(
    peer: A,
    peers: &[SocketAddr],
) -> Result<PeerListResponse, ClientError> {
    let request = Request::PeerList(peers.to_vec());
    tracing::debug!(?peers, "Sending the peers list");
    send_message(peer, request).await
}

pub async fn get_chain<A: ToSocketAddrs>(peer: A) -> Result<Vec<Block>, ClientError> {
    let response: GetChainResponse = send_message(peer, Request::GetChain).await?;
    tracing::debug!(
        "Received the chain from peer. Chain length: {}",
        response.chain.len()
    );
    Ok(response.chain)
}

pub async fn add_block<A: ToSocketAddrs>(block: Block, peer: A) -> Result<bool, ClientError> {
    let request = Request::AddBlock(block);
    let response: AddBlockResponse = send_message(peer, request).await?;
    tracing::debug!(response.is_block_added, "Adding new block");
    Ok(response.is_block_added)
}

pub async fn mine_block<A: ToSocketAddrs>(data: String, peer: A) -> Result<usize, ClientError> {
    let request = Request::MineBlock(data);
    let response: MineBlockResponse = send_message(peer, request).await?;
    tracing::debug!(response.block_index, "Mining block");
    Ok(response.block_index)
}

pub trait Client {
    fn send<R: DeserializeOwned>(
        &self,
        addr: SocketAddr,
        request: Request,
    ) -> impl Future<Output = Result<R, ClientError>> + Send;
}

pub struct TcpClient;

impl Client for TcpClient {
    async fn send<R>(&self, addr: SocketAddr, request: Request) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let mut stream = TcpStream::connect(addr).await?;
        let json_message = serde_json::to_vec(&request)?;
        stream.write_all(&json_message).await?;

        let mut buffer = vec![0u8; 8192];
        let read_bytes = stream.read(&mut buffer).await?;
        let response = serde_json::from_slice::<R>(&buffer[..read_bytes])?;
        Ok(response)
    }
}

async fn send_message<A, B>(peer: A, message: Request) -> Result<B, ClientError>
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

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::api::request::Request;
    use std::collections::HashMap;

    pub struct TestClient {
        pub requests: tokio::sync::Mutex<HashMap<SocketAddr, Request>>,
        response: Vec<u8>,
    }

    impl TestClient {
        pub fn new(response: Vec<u8>) -> Self {
            TestClient {
                requests: tokio::sync::Mutex::new(HashMap::new()),
                response,
            }
        }
    }

    impl Client for TestClient {
        async fn send<R: DeserializeOwned>(
            &self,
            addr: SocketAddr,
            request: Request,
        ) -> Result<R, ClientError> {
            let mut messages = self.requests.lock().await;
            messages.insert(addr, request);
            let response = serde_json::from_slice::<R>(&self.response[..])?;
            Ok(response)
        }
    }
}
