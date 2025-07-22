use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
    #[error("Invalid request: {0}")]
    InvalidMessage(#[from] serde_json::error::Error),
    #[error("{0}")]
    InternalError(String),
}

pub struct ServerResponse {
    body: Vec<u8>,
}

impl ServerResponse {
    pub fn new<S: Serialize>(body: S) -> Result<Self, ServerError> {
        let body = serde_json::to_vec(&body)?;
        Ok(ServerResponse { body })
    }
}

pub trait Handler<R> {
    fn handle_request(
        &self,
        request: R,
    ) -> impl Future<Output = Result<ServerResponse, ServerError>> + Send;
}

pub async fn start_server<R, H>(port: u16, handler: H) -> Result<(), ServerError>
where
    R: for<'a> Deserialize<'a> + Debug + Send + 'static,
    H: Handler<R> + Send + Sync + 'static,
{
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    let handler = Arc::new(handler);
    tracing::debug!("Server is listening on port {}", port);

    loop {
        let (socket, addr) = listener.accept().await?;
        let handler = Arc::clone(&handler);
        tokio::spawn(accept_socket(socket, addr, handler));
    }
}

async fn accept_socket<R, H>(
    mut socket: TcpStream,
    addr: SocketAddr,
    handler: Arc<H>,
) -> Result<(), ServerError>
where
    R: for<'a> Deserialize<'a> + Debug,
    H: Handler<R>,
{
    tracing::debug!("Accepted new connection from {}", addr);

    let mut buffer = vec![0u8; 8192];
    let read_bytes = socket.read(&mut buffer).await?;

    let message = serde_json::from_slice::<R>(&buffer[..read_bytes])?;
    tracing::debug!("Received new request from peer: {:?}", message);
    let response = handler.handle_request(message).await?;
    socket.write_all(&response.body).await?;
    Ok(())
}
