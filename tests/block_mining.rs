mod helpers;

use blockchain::network::client;

#[tokio::test]
async fn get_chain() {
    let server = helpers::start_node().await;

    let chain = client::get_chain(server.addr)
        .await
        .expect("Failed to get chain from the server");

    assert_eq!(chain.len(), 1);
}

#[tokio::test]
async fn mine_blocks() {
    let server = helpers::start_node().await;

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
