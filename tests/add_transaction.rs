pub mod helpers;

use helpers::{client, retry, server, transaction};

#[tokio::test]
async fn add_valid_transaction() {
    let server = server::start_node().await;

    let transaction1 = transaction::create_transaction("r1", 200);
    let is_transaction_added = client::add_transaction(transaction1.clone(), server.addr).await;
    assert!(is_transaction_added);

    let get_transactions = || async {
        client::get_chain(server.addr)
            .await
            .into_iter()
            .flat_map(|b| b.transactions)
            .collect::<Vec<_>>()
    };

    let transactions = retry(get_transactions, |tx| tx.len() == 1, 5).await;
    assert_eq!(transactions.len(), 1);

    let expected_transactions = vec![transaction1];
    assert_eq!(transactions, expected_transactions);
}

#[tokio::test]
async fn ignore_invalid_transactions() {
    let server = server::start_node().await;

    let mut transaction1 = transaction::create_transaction("r1", 200);
    transaction1.signature = vec![0; 32];
    let is_transaction_added = client::add_transaction(transaction1.clone(), server.addr).await;
    assert!(!is_transaction_added);
}
