mod add_transaction;
mod get_chain;
mod join;
mod ping;
mod sync_block;
mod sync_peer_list;
mod sync_transaction;

use crate::api::request::Request;
use crate::blockchain::Blockchain;
use crate::network::server::{Handler, ServerError, ServerResponse};
use crate::node::Node;
use crate::node::broadcast::Broadcaster;
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NodeApiHandler<B: Broadcaster + Send + Sync> {
    node: Arc<Mutex<Node>>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Vec<Transaction>>>,
    broadcaster: Arc<B>,
}

impl<B: Broadcaster + Send + Sync> NodeApiHandler<B> {
    pub fn new(
        node: Arc<Mutex<Node>>,
        blockchain: Arc<Mutex<Blockchain>>,
        mempool: Arc<Mutex<Vec<Transaction>>>,
        broadcaster: Arc<B>,
    ) -> Self {
        Self {
            node,
            blockchain,
            mempool,
            broadcaster,
        }
    }
}

impl<B> Handler<Request> for NodeApiHandler<B>
where
    B: Broadcaster + Send + Sync + 'static,
{
    async fn handle_request(&self, request: Request) -> Result<ServerResponse, ServerError> {
        match request {
            Request::Join(peer_addr) => {
                let node = Arc::clone(&self.node);
                let response = join::handle_request(peer_addr, node).await?;
                ServerResponse::new(response)
            }
            Request::GetChain => {
                let blockchain = Arc::clone(&self.blockchain);
                let response = get_chain::handle_request(blockchain).await?;
                ServerResponse::new(response)
            }
            Request::Ping => {
                let response = ping::handle_request().await?;
                ServerResponse::new(response)
            }
            Request::AddTransaction(transaction) => {
                let node = Arc::clone(&self.node);
                let broadcaster = Arc::clone(&self.broadcaster);
                let mempool = Arc::clone(&self.mempool);
                let response =
                    add_transaction::handle_request(transaction, node, broadcaster, mempool)
                        .await?;
                ServerResponse::new(response)
            }
            Request::SyncPeerList(peers) => {
                let node = Arc::clone(&self.node);
                let broadcaster = Arc::clone(&self.broadcaster);
                let response = sync_peer_list::handle_request(peers, node, broadcaster).await?;
                ServerResponse::new(response)
            }
            Request::SyncBlock(block) => {
                let blockchain = Arc::clone(&self.blockchain);
                let mempool = Arc::clone(&self.mempool);
                let response = sync_block::handle_request(block, blockchain, mempool).await?;
                ServerResponse::new(response)
            }
            Request::SyncTransaction(transaction) => {
                let mempool = Arc::clone(&self.mempool);
                let response = sync_transaction::handle_request(transaction, mempool).await?;
                ServerResponse::new(response)
            }
        }
    }
}
