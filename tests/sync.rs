pub mod helpers;

use crate::helpers::{client, retry, server};
use blockchain::transaction::Transaction;

#[tokio::test]
async fn join_cluster_and_update_chain() {
    let node_1 = server::start_node().await;

    let transaction1 = Transaction::new("s1".to_string(), "r1".to_string(), 200);
    let is_transaction_added = client::add_transaction(transaction1.clone(), node_1.addr).await;
    assert!(is_transaction_added);

    let transaction2 = Transaction::new("s2".to_string(), "r2".to_string(), 400);
    let is_transaction_added = client::add_transaction(transaction2.clone(), node_1.addr).await;
    assert!(is_transaction_added);

    let get_transactions = || async {
        client::get_chain(node_1.addr)
            .await
            .into_iter()
            .flat_map(|b| b.transactions)
            .collect::<Vec<_>>()
    };

    let transactions = retry(get_transactions, |cs| cs.len() == 2, 10).await;
    assert_eq!(
        transactions,
        vec![transaction1.clone(), transaction2.clone()]
    );

    let node_2 = server::start_node_with_seed(node_1.addr).await;

    let get_transactions = || async {
        client::get_chain(node_2.addr)
            .await
            .into_iter()
            .flat_map(|b| b.transactions)
            .collect::<Vec<_>>()
    };

    let transactions = retry(get_transactions, |cs| cs.len() == 2, 10).await;
    assert_eq!(transactions, vec![transaction1, transaction2]);
}

#[tokio::test]
async fn broadcast_transactions() {
    let node_1 = server::start_node().await;
    let node_2 = server::start_node_with_seed(node_1.addr).await;

    let transaction1 = Transaction::new("s1".to_string(), "r1".to_string(), 200);
    let is_transaction_added = client::add_transaction(transaction1.clone(), node_2.addr).await;
    assert!(is_transaction_added);

    let transaction2 = Transaction::new("s2".to_string(), "r2".to_string(), 400);
    let is_transaction_added = client::add_transaction(transaction2.clone(), node_2.addr).await;
    assert!(is_transaction_added);

    let get_transactions = || async {
        client::get_chain(node_1.addr)
            .await
            .into_iter()
            .flat_map(|b| b.transactions)
            .collect::<Vec<_>>()
    };

    let transactions = retry(get_transactions, |cs| cs.len() == 2, 10).await;
    assert_eq!(transactions, vec![transaction1, transaction2]);
}
