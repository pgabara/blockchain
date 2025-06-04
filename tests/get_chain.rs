pub mod helpers;

use helpers::server::start_node;

#[tokio::test]
async fn get_chain() {
    let server = start_node().await;
    let chain = helpers::client::get_chain(server.addr).await;
    assert_eq!(chain.len(), 1);
}
