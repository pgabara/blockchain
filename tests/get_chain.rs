pub mod helpers;

use blockchain::network::client;

use helpers::server::start_node;

#[tokio::test]
async fn get_chain() {
    let server = start_node().await;

    let chain = client::get_chain(server.addr)
        .await
        .expect("Failed to get chain from the server");

    assert_eq!(chain.len(), 1);
}
