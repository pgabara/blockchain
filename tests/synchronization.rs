mod helpers;

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

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let get_chain_len = || async {
        client::get_chain(node_2.addr)
            .await
            .expect("Failed to get chain")
            .len()
    };

    let chain_len = repeat_until(get_chain_len, |n| *n == 3, 3).await;
    assert_eq!(chain_len, 3);
}
