pub mod helpers;

use helpers::{client, retry, server, transaction};

#[tokio::test]
async fn get_empty_chain() {
    let server = server::start_node().await;
    let chain = client::get_chain(server.addr).await;
    assert_eq!(chain.len(), 1);

    let transactions = chain
        .into_iter()
        .flat_map(|b| b.transactions)
        .collect::<Vec<_>>();
    assert_eq!(transactions.len(), 0);
}

#[tokio::test]
async fn get_non_empty_chain() {
    let server = server::start_node().await;

    let transaction1 = transaction::create_transaction("r1", 200);
    let is_transaction_added = client::add_transaction(transaction1.clone(), server.addr).await;
    assert!(is_transaction_added);

    let transaction2 = transaction::create_transaction("r2", 500);
    let is_transaction_added = client::add_transaction(transaction2.clone(), server.addr).await;
    assert!(is_transaction_added);

    let transaction3 = transaction::create_transaction("r3", 800);
    let is_transaction_added = client::add_transaction(transaction3.clone(), server.addr).await;
    assert!(is_transaction_added);

    let get_transactions = || async {
        client::get_chain(server.addr)
            .await
            .into_iter()
            .flat_map(|b| b.transactions)
            .collect::<Vec<_>>()
    };

    let transactions = retry(get_transactions, |tx| tx.len() == 3, 5).await;
    assert_eq!(transactions.len(), 3);

    let expected_transactions = vec![transaction1, transaction2, transaction3];
    assert_eq!(transactions, expected_transactions);
}
