use sha2::{Digest, Sha256};

#[derive(PartialEq, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    index: usize,
    timestamp: i64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: usize,
}

impl Block {
    fn new(index: usize, data: String, previous_hash: String, difficulty: usize) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let mut block = Self {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.mine_block(difficulty);
        block
    }

    fn mine_block(&mut self, difficulty: usize) {
        let prefix = "0".repeat(difficulty);
        while !self.hash.starts_with(&prefix) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }

    fn calculate_hash(&self) -> String {
        let input = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.data, self.previous_hash, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let genesis_block = Block::new(0, "Genesis Block".to_string(), String::new(), difficulty);
        Self {
            chain: vec![genesis_block],
            difficulty,
        }
    }

    pub fn mine_block(&mut self, data: String) -> usize {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(self.chain.len(), data, previous_hash, self.difficulty);
        let index = new_block.index;
        self.chain.push(new_block);
        index
    }

    pub fn try_replace_chain(&mut self, new_chain: Vec<Block>) -> bool {
        if new_chain.len() > self.chain.len() && self.is_valid_chain(&new_chain) {
            self.chain = new_chain;
            return true;
        }
        false
    }

    fn is_valid_chain(&self, chain: &[Block]) -> bool {
        if chain.is_empty() {
            return false;
        }

        if chain[0] != self.chain[0] {
            return false;
        }

        for i in 1..self.chain.len() {
            let prev_block = &self.chain[i - 1];
            let curr_block = &self.chain[i];

            if curr_block.previous_hash != prev_block.hash {
                return false;
            }

            if curr_block.calculate_hash() != curr_block.hash {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blockchain_has_genesis_block() {
        let blockchain = Blockchain::new(1);
        let genesis_block = blockchain.chain.first().unwrap();
        let expected_genesis_block = Block::new(0, "Genesis Block".to_string(), String::new(), 1);
        assert_eq!(genesis_block, &expected_genesis_block);
    }

    #[test]
    fn test_add_new_block_to_blockchain() {
        let mut blockchain = Blockchain::new(1);
        blockchain.mine_block("Test Block".to_string());
        let genesis_block = blockchain.chain.first().unwrap();
        let new_block = blockchain.chain.last().unwrap();
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.data, "Test Block".to_string());
        assert_eq!(new_block.previous_hash, genesis_block.hash);
    }

    #[test]
    fn test_is_valid_chain_with_empty_chain() {
        let blockchain = Blockchain::new(1);
        let other_chain = vec![];
        assert_eq!(blockchain.is_valid_chain(other_chain.as_slice()), false);
    }

    #[test]
    fn test_is_valid_chain_with_different_genesis_block() {
        let blockchain = Blockchain::new(1);
        let other_chain = vec![Block::new(0, "Test Block".to_string(), String::new(), 1)];
        assert_eq!(blockchain.is_valid_chain(other_chain.as_slice()), false);
    }

    #[test]
    fn test_is_valid_chain_with_incorrect_previous_hash() {
        let blockchain = Blockchain::new(1);
        let other_chain = vec![
            Block::new(0, "Test Block 1".to_string(), String::new(), 1),
            Block::new(1, "Test Block 2".to_string(), String::new(), 1),
        ];
        assert_eq!(blockchain.is_valid_chain(other_chain.as_slice()), false);
    }

    #[test]
    fn test_is_valid_chain() {
        let mut blockchain = Blockchain::new(1);
        blockchain.mine_block("Test Block 1".to_string());
        blockchain.mine_block("Test Block 2".to_string());
        let other_chain = &blockchain.chain;
        assert!(blockchain.is_valid_chain(other_chain));
    }

    #[test]
    fn test_try_replace_chain_with_invalid_chain() {
        let mut blockchain = Blockchain::new(1);
        let other_chain = vec![
            Block::new(0, "Test Block 1".to_string(), String::new(), 1),
            Block::new(1, "Test Block 2".to_string(), String::new(), 1),
        ];

        let is_chain_replaced = blockchain.try_replace_chain(other_chain);
        assert_eq!(is_chain_replaced, false);
        assert_eq!(blockchain.chain.len(), 1);
    }

    #[test]
    fn test_try_replace_chain_with_shorter_chain() {
        let mut blockchain = Blockchain::new(1);
        blockchain.mine_block("Test Block 1".to_string());
        blockchain.mine_block("Test Block 2".to_string());
        let other_chain = vec![blockchain.chain[0].clone()];

        let is_chain_replaced = blockchain.try_replace_chain(other_chain);
        assert_eq!(is_chain_replaced, false);
        assert_eq!(blockchain.chain.len(), 3);
    }

    #[test]
    fn test_try_replace_chain() {
        let mut blockchain = Blockchain::new(1);
        blockchain.mine_block("Test Block 1".to_string());
        blockchain.mine_block("Test Block 2".to_string());
        let other_chain = blockchain.chain.clone();
        blockchain.chain.remove(2);

        let is_chain_replaced = blockchain.try_replace_chain(other_chain);
        assert_eq!(is_chain_replaced, true);
        assert_eq!(blockchain.chain.len(), 3);
    }
}
