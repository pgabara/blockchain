use crate::api::response::JoinResponse;
use crate::network::server::ServerError;
use crate::node::Node;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(
    peer_addr: SocketAddr,
    node: Arc<Mutex<Node>>,
) -> Result<JoinResponse, ServerError> {
    tracing::debug!("Received Join request from {}", peer_addr);
    let mut node = node.lock().await;
    node.add_peers(&[peer_addr]);
    let peers: Vec<SocketAddr> = node.cluster_peers().iter().map(|p| p.addr).collect();
    tracing::debug!("Sending back connected cluster peers {:?}", peers);
    Ok(JoinResponse { peers })
}

#[cfg(test)]
mod tests {
    // todo: write tests here....
}
