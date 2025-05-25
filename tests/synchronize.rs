pub mod helpers;

use blockchain::network::client;

use helpers::repeat_until;
use helpers::server::{start_node, start_node_with_seed};

#[tokio::test]
async fn synchronize_joining_node() {
    let node_1 = start_node().await;

    client::mine_block(String::from("Test Block 1"), node_1.addr)
        .await
        .expect("Failed to mine block");

    client::mine_block(String::from("Test Block 2"), node_1.addr)
        .await
        .expect("Failed to mine block");

    let node_2 = start_node_with_seed(node_1.addr).await;

    let get_chain = || async {
        client::get_chain(node_2.addr)
            .await
            .expect("Failed to get the chain")
    };

    let chain = repeat_until(get_chain, |n| n.len() == 3, 5).await;
    assert_eq!(chain.len(), 3);
}

#[tokio::test]
async fn broadcast_new_blocks() {
    let node_1 = start_node().await;
    let node_2 = start_node_with_seed(node_1.addr).await;

    client::mine_block(String::from("Test Block 1"), node_2.addr)
        .await
        .expect("Failed to mine block");

    let get_chain = || async {
        client::get_chain(node_1.addr)
            .await
            .expect("Failed to get the chain")
    };

    let chain = repeat_until(get_chain, |n| n.len() == 2, 5).await;
    assert_eq!(chain.len(), 2);

    client::mine_block(String::from("Test Block 2"), node_1.addr)
        .await
        .expect("Failed to mine block");

    let get_chain = || async {
        client::get_chain(node_2.addr)
            .await
            .expect("Failed to get a chain")
    };

    let chain = repeat_until(get_chain, |n| n.len() == 3, 5).await;
    assert_eq!(chain.len(), 3);
}
