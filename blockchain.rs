use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Hash)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub proof_hash: String,
    pub prev_hash: String,
    pub nonce: u64,
}

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();
        chain.push(Self::create_genesis_block());
        Blockchain { chain }
    }

    fn create_genesis_block() -> Block {
        Block {
            index: 0,
            timestamp: 0,
            proof_hash: "Genesis_Proof_Hash".to_string(),
            prev_hash: "0".to_string(),
            nonce: 0,
        }
    }

    pub fn calculate_hash<T: Hash>(t: &T) -> String {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        format!("{:x}", s.finish())
    }

    pub fn add_block(&mut self, proof_hash: String) -> Block {
        let prev_block = self.chain.last().unwrap();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut new_block = Block {
            index: prev_block.index + 1,
            timestamp,
            proof_hash,
            prev_hash: Blockchain::calculate_hash(prev_block),
            nonce: 0,
        };

        new_block.nonce = self.mine_proof_of_work(&new_block);
        println!("[H-CHAIN] Block {} added. Hash: {}", new_block.index, Blockchain::calculate_hash(&new_block));
        self.chain.push(new_block.clone());
        new_block
    }

    fn mine_proof_of_work(&self, block: &Block) -> u64 {
        let mut nonce = 0;
        let target_prefix = "000";

        loop {
            let mut temp_block = block.clone();
            temp_block.nonce = nonce;
            let hash = Blockchain::calculate_hash(&temp_block);
            if hash.starts_with(target_prefix) {
                return nonce;
            }
            nonce += 1;
            if nonce > 1000 { break; }
        }
        nonce
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            if current.prev_hash != Blockchain::calculate_hash(previous) {
                return false;
            }
            if !Blockchain::calculate_hash(current).starts_with("000") {
                return false;
            }
        }
        true
    }
}
