use crate::blockchain::Block;
use crate::network::client;
use crate::node::Node;

pub async fn broadcast_new_block(block: &Block, node: &Node) -> Result<(), SyncError> {
    let peers = &node.peers;
    tracing::debug!(block.index, ?peers, "Broadcasting new block");
    for peer in peers {
        let status = client::add_block(block.clone(), peer.addr).await?;
        tracing::debug!(status, "Broadcasting new block to {}", peer.addr);
    }
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum SyncError {
    #[error("Client error: {0}")]
    ClientError(#[from] client::ClientError),
}
