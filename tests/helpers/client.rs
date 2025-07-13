use blockchain::api::request::Request;
use blockchain::api::response::{AddTransactionResponse, GetChainResponse};
use blockchain::blockchain::Block;
use blockchain::network::client::{Client, TcpClient};
use blockchain::transaction::Transaction;
use std::net::SocketAddr;

pub async fn get_chain(peer_addr: SocketAddr) -> Vec<Block> {
    let client = TcpClient::default();
    let response: GetChainResponse = client
        .send(peer_addr, Request::GetChain)
        .await
        .expect("Failed to get a chain");
    response.chain
}

pub async fn add_transaction(transaction: Transaction, peer_addr: SocketAddr) -> bool {
    let client = TcpClient::default();
    let response: AddTransactionResponse = client
        .send(peer_addr, Request::AddTransaction(transaction))
        .await
        .expect("Failed to add a transaction");
    response.is_transaction_added
}
