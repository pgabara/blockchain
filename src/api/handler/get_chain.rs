use crate::api::response::GetChainResponse;
use crate::blockchain::Blockchain;
use crate::network::server::ServerError;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<GetChainResponse, ServerError> {
    tracing::debug!("Received GetChain request");
    let chain = blockchain.lock().await.chain.clone();
    Ok(GetChainResponse { chain })
}

#[cfg(test)]
mod tests {
    // todo: add tests here...
}
