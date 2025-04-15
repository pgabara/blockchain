pub mod helpers;

use blockchain::network::client;

use helpers::server::start_node;

#[tokio::test]
async fn mine_blocks() {
    let server = start_node().await;

    let block_index_1 = client::mine_block(String::from("Test Block 1"), server.addr)
        .await
        .expect("Failed to mine block");
    assert_eq!(block_index_1, 1);

    let block_index_2 = client::mine_block(String::from("Test Block 1"), server.addr)
        .await
        .expect("Failed to mine block");
    assert_eq!(block_index_2, 2);

    let chain = client::get_chain(server.addr)
        .await
        .expect("Failed to get chain from the server");
    assert_eq!(chain.len(), 3);
}
