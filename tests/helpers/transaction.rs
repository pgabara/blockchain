use blockchain::transaction::Transaction;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

pub fn create_transaction(receiver: &str, amount: u64) -> Transaction {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    Transaction::new(signing_key, receiver.to_string(), amount)
}
