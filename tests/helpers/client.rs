use blockchain::api::request::Request;
use blockchain::api::response::{AddBlockResponse, GetChainResponse, MineBlockResponse};
use blockchain::blockchain::Block;
use blockchain::network::client::{Client, TcpClient};
use std::net::SocketAddr;

pub async fn get_chain(peer_addr: SocketAddr) -> Vec<Block> {
    let client = TcpClient;
    let response: GetChainResponse = client
        .send(peer_addr, Request::GetChain)
        .await
        .expect("Failed to get a chain");
    response.chain
}

pub async fn add_block(block: Block, peer_addr: SocketAddr) -> bool {
    let client = TcpClient;
    let request = Request::AddBlock(block);
    let response: AddBlockResponse = client
        .send(peer_addr, request)
        .await
        .expect("Failed to add a block");
    response.is_block_added
}

pub async fn mine_block(data: String, peer_addr: SocketAddr) -> usize {
    let client = TcpClient;
    let request = Request::MineBlock(data);
    let response: MineBlockResponse = client
        .send(peer_addr, request)
        .await
        .expect("Failed to send block");
    response.block_index
}
