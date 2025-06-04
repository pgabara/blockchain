pub mod helpers;

use helpers::server::start_node;

#[tokio::test]
async fn mine_blocks() {
    let server = start_node().await;

    let block_index_1 =
        helpers::client::mine_block(String::from("Test Block 1"), server.addr).await;
    assert_eq!(block_index_1, 1);

    let block_index_2 =
        helpers::client::mine_block(String::from("Test Block 1"), server.addr).await;
    assert_eq!(block_index_2, 2);

    let chain = helpers::client::get_chain(server.addr).await;
    assert_eq!(chain.len(), 3);
}
