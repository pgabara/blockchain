use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub sender: Sender,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(signing_key: SigningKey, receiver: String, amount: u64) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let sender = Sender::from_public_key(&signing_key.verifying_key().to_bytes());
        let tx_data = format!("{}{}{}{}", sender.address, receiver, amount, timestamp);
        let signature = signing_key.sign(tx_data.as_bytes());
        Transaction {
            sender,
            receiver,
            amount,
            timestamp,
            signature: signature.to_bytes().to_vec(),
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        let derived_address = Sender::from_public_key(&self.sender.public_key).address;
        if self.sender.address != derived_address {
            return Err("Sender address does not match public key-derived address".to_string());
        }

        let verifying_key = match VerifyingKey::try_from(&self.sender.public_key[..]) {
            Ok(key) => key,
            Err(_) => return Err("Invalid public key bytes".to_string()),
        };

        let signature = match Signature::try_from(&self.signature[..]) {
            Ok(sig) => sig,
            Err(_) => return Err("Invalid signature bytes".to_string()),
        };

        let tx_data = format!(
            "{}{}{}{}",
            self.sender.address, self.receiver, self.amount, self.timestamp
        );

        verifying_key
            .verify(tx_data.as_bytes(), &signature)
            .map_err(|_| "Invalid transaction signature".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sender {
    pub address: String,
    pub public_key: Vec<u8>,
}

impl Sender {
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let hash = Sha256::digest(public_key);
        let address = hex::encode(&hash[..20]);
        let public_key = Vec::from(public_key);
        Self {
            address,
            public_key,
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::transaction::Transaction;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    pub fn create_transaction(receiver: &str, amount: u64) -> Transaction {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Transaction::new(signing_key, receiver.to_string(), amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn verify_signed_transaction() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        let transaction = Transaction::new(signing_key, "Alice".to_string(), 10);
        assert!(transaction.verify().is_ok());
    }
}
