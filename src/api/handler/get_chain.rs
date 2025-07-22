use crate::api::response::GetChainResponse;
use crate::blockchain::Blockchain;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_request(blockchain: Arc<Mutex<Blockchain>>) -> GetChainResponse {
    tracing::debug!("Received GetChain request");
    let chain = blockchain.lock().await.chain.clone();
    GetChainResponse { chain }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::test_utils::create_transaction;

    #[tokio::test]
    async fn get_chain() {
        let block1_transactions =
            vec![create_transaction("r1", 100), create_transaction("r2", 200)];

        let block2_transactions =
            vec![create_transaction("r3", 300), create_transaction("r4", 400)];

        let mut blockchain = Blockchain::new(0);
        blockchain.mine_block(block1_transactions.clone());
        blockchain.mine_block(block2_transactions.clone());

        let blockchain = Arc::new(Mutex::new(blockchain));
        let chain = handle_request(Arc::clone(&blockchain)).await;
        let transactions: Vec<_> = chain.chain.iter().map(|b| b.transactions.clone()).collect();
        assert_eq!(
            transactions,
            vec![vec![], block1_transactions, block2_transactions]
        );
    }
}
