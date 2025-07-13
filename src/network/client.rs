use crate::api::request::Request;
use serde::de::DeserializeOwned;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub trait Client {
    fn send<R: DeserializeOwned>(
        &self,
        addr: SocketAddr,
        request: Request,
    ) -> impl Future<Output = Result<R, ClientError>> + Send;
}

pub struct TcpClient {
    connect_timeout: Duration,
    write_timeout: Duration,
    read_timeout: Duration,
}

impl Default for TcpClient {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(2),
            write_timeout: Duration::from_secs(1),
            read_timeout: Duration::from_secs(3),
        }
    }
}

impl Client for TcpClient {
    async fn send<R>(&self, addr: SocketAddr, request: Request) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let mut stream = match timeout(self.connect_timeout, TcpStream::connect(addr)).await {
            Ok(stream) => stream?,
            Err(_) => return Err(ClientError::ConnectTimeout),
        };

        let json_message = serde_json::to_vec(&request)?;

        match timeout(self.write_timeout, stream.write_all(&json_message)).await {
            Ok(write_status) => write_status?,
            Err(_) => return Err(ClientError::WriteTimeout),
        }

        let mut buffer = vec![0u8; 8192];
        let read_bytes = match timeout(self.read_timeout, stream.read(&mut buffer)).await {
            Ok(read_bytes) => read_bytes?,
            Err(_) => return Err(ClientError::ReadTimeout),
        };
        let response = serde_json::from_slice::<R>(&buffer[..read_bytes])?;
        Ok(response)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::error::Error),
    #[error("Read timeout")]
    ReadTimeout,
    #[error("Connect timeout")]
    ConnectTimeout,
    #[error("Write timeout")]
    WriteTimeout,
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
