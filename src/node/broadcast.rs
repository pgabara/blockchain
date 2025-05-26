use crate::blockchain::Block;
use crate::network::client;
use crate::node::Node;

pub trait NodeStateBroadcast {
    fn broadcast_peer_list(&self) -> impl Future<Output = Result<(), BroadcastError>> + Send;
    fn broadcast_new_block(
        &self,
        block: &Block,
    ) -> impl Future<Output = Result<(), BroadcastError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("Failed to broadcast a message: {0}")]
    NetworkError(#[from] client::ClientError),
}

impl NodeStateBroadcast for Node {
    async fn broadcast_peer_list(&self) -> Result<(), BroadcastError> {
        let peers: Vec<_> = self.cluster_peers().iter().map(|p| p.addr).collect();
        tracing::debug!(?peers, "Broadcasting the peer list");
        for &peer in self.peers.keys() {
            client::send_peer_list(peer.addr, &peers).await?;
        }
        Ok(())
    }

    async fn broadcast_new_block(&self, block: &Block) -> Result<(), BroadcastError> {
        tracing::debug!(block.index, "Broadcasting new block");
        for &peer in self.peers.keys() {
            let status = client::add_block(block.clone(), peer.addr).await?;
            tracing::debug!(status, "Broadcasting new block to {}", peer.addr);
        }
        Ok(())
    }
}
