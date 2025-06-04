use crate::api::request::Request;
use serde::de::DeserializeOwned;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
