#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: i64,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            sender,
            receiver,
            amount,
            timestamp,
        }
    }
}
