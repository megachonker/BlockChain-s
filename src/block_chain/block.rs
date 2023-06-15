use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Arc, Mutex};

const HASH_MAX: u64 = 1000000000000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    block_id: u64,                  //the hash of whole block
    block_height: u64,              //the number of the current block
    parent_hash: u64,               //the id of last block (block are chain with that)
    transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    miner_hash: u64,                //Who find the answer
    nonce: u64,                     //the answer of the defi
}
#[derive(Debug, Hash, Serialize, Deserialize, Clone)]
pub struct Transaction {
    src: u64,  //who send coin
    dst: u64,  //who recive
    qqty: u32, //the acount
}

pub fn hash<T: Hash>(value: T) -> u64 {
    //return the hash of the item (need to have Hash trait)
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl Block {
    /// create the first block full empty 
    pub fn new() -> Block {
        let mut block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions : vec![],
            nonce: 0,
            miner_hash: 0,
        };
        block.nonce = 0;
        block.block_id = hash(&block);         //the                 
        block
    }

    pub fn new_wrong(value : u64) -> Block{
        let mut block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions : vec![],
            nonce: value,    //for the block zero the nonce indique the status of the block (use to response to GetBlock(i))
            miner_hash: 0,
        };
        block.block_id = hash(&block);         //the                 
        block
    }

    pub fn get_height_nonce(&self) -> (u64,u64) {
        (self.block_height,self.nonce)
    }

    pub fn check(&self) -> bool {
        let mut hasher = DefaultHasher::new(); //why don't use hash fun ? hash(self) ?? like in last commit

        //playload of block to hash
        self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        // self.transactions.hash(&mut hasher);     //tres variable donc osef
        self.miner_hash.hash(&mut hasher);
        self.nonce.hash(&mut hasher);

        let answer = hasher.finish();
        answer < HASH_MAX && hash(self) == self.block_id
    }

    pub fn generate_block(&self, new_transa: Vec<Transaction>, finder: u64) -> Block {
        let mut new_block = Block {
            block_height: self.block_height + 1,
            block_id: 0,
            parent_hash: self.block_id,
            transactions: new_transa,
            nonce: 0,
            miner_hash: finder,
        };
        new_block.nonce = mine(&new_block);
        new_block.block_id = hash(&new_block); //set the correct id
        new_block
    }

    pub fn generate_block_stop(&self, finder: u64, sould_stop: &Arc<Mutex<bool>>) -> Option<Block> {
        let mut new_block = Block {
            block_height: self.block_height + 1,
            block_id: 0,
            parent_hash: self.block_id,
            transactions: vec![], //put after
            nonce: 0,
            miner_hash: finder,
        };
        new_block.nonce = mine_stop(&new_block, sould_stop)?;
        new_block.block_id = hash(&new_block); //set the correct id
        Some(new_block)
    }
    pub fn new_block(&self, new_transa: Vec<Transaction>, finder: u64) -> Block {
        self.generate_block(new_transa, finder)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.block_id.to_be_bytes());
        bytes.extend_from_slice(&self.block_height.to_be_bytes());
        bytes.extend_from_slice(&self.parent_hash.to_be_bytes());
        bytes.extend_from_slice(&(self.transactions.len() as u32).to_be_bytes());
        //put the transaction here
        bytes.extend_from_slice(&self.miner_hash.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Block> {
        if bytes.len() < 48 {
            // Ensure the byte slice has enough length to deserialize a Block
            return None;
        }

        let block_id_bytes = &bytes[0..8];
        let block_height_bytes = &bytes[8..16];
        let parent_hash_bytes = &bytes[16..24];
        // let transactions_len_bytes = &bytes[24..28];
        let transactions_bytes = &bytes[28..];
        let miner_hash_bytes = &transactions_bytes[0..8];
        let nonce_bytes = &transactions_bytes[8..16];

        let block_id = u64::from_be_bytes(block_id_bytes.try_into().ok()?);
        let block_height = u64::from_be_bytes(block_height_bytes.try_into().ok()?);
        let parent_hash = u64::from_be_bytes(parent_hash_bytes.try_into().ok()?);
        // let transactions_len = u32::from_be_bytes(transactions_len_bytes.try_into().ok()?);

        // Extract transactions from byte slice (assuming Transaction has its own serialization logic)
        // let transactions: Vec<Transaction> = (0..transactions_len)
        //     .into_iter()
        //     .flat_map(|i| {
        //         let start = 16 * i as usize;
        //         let end = start + 16;
        //         Transaction::from_bytes(&transactions_bytes[start..end])
        //     })
        //     .collect::<Option<Vec<Transaction>>>()?;
        let transactions = vec![];

        let miner_hash = u64::from_be_bytes(miner_hash_bytes.try_into().ok()?);
        let nonce = u64::from_be_bytes(nonce_bytes.try_into().ok()?);

        Some(Block {
            block_id,
            block_height,
            parent_hash,
            transactions,
            miner_hash,
            nonce,
        })
    }

    pub fn set_transactions(mut self, transactions: Vec<Transaction>) -> Self {
        self.transactions = transactions;
        self
    }
}

impl Hash for Block {
    //implement the Hash's trait for Block
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_height.hash(state);
        self.parent_hash.hash(state);
        // self.transactions.hash(state);
        self.miner_hash.hash(state);
        self.nonce.hash(state);
    }
}
pub fn mine_hasher_clone(block: &Block) -> u64 {
    let mut rng = rand::thread_rng(); //to pick random value
    let mut hasher = DefaultHasher::new();

    //playload of block to hash
    block.block_height.hash(&mut hasher);
    block.parent_hash.hash(&mut hasher);
    block.transactions.hash(&mut hasher);
    block.miner_hash.hash(&mut hasher);

    loop {
        let mut to_hash = hasher.clone(); //save l'état du hasher
        let nonce_to_test = rng.gen::<u64>();

        nonce_to_test.hash(&mut to_hash);
        let answer = to_hash.finish();

        if answer < HASH_MAX {
            return nonce_to_test;
        }
    }
}

pub fn mine(block: &Block) -> u64 {
    let mut rng = rand::thread_rng(); //to pick random value
    loop {
        let nonce_to_test = rng.gen::<u64>();
        let mut hasher = DefaultHasher::new();

        //playload of block to hash
        block.block_height.hash(&mut hasher);
        block.parent_hash.hash(&mut hasher);
        // block.transactions.hash(&mut hasher);
        block.miner_hash.hash(&mut hasher);
        nonce_to_test.hash(&mut hasher);
        let answer: u64 = hasher.finish();

        if answer < HASH_MAX {
            return nonce_to_test;
        }
    }
}

pub fn mine_stop2(block: &Block, should_stop: &Arc<Mutex<bool>>) -> Option<u64> {
    let mut rng = rand::thread_rng(); //to pick random value
    loop {
        let nonce_to_test = rng.gen::<u64>();
        let mut hasher = DefaultHasher::new();

        //playload of block to hash
        block.block_height.hash(&mut hasher);
        block.parent_hash.hash(&mut hasher);
        // block.transactions.hash(&mut hasher);
        block.miner_hash.hash(&mut hasher);
        nonce_to_test.hash(&mut hasher);
        let answer: u64 = hasher.finish();

        if answer < HASH_MAX {
            return Some(nonce_to_test);
        }
        if nonce_to_test % 100000 == 0 {
            //test not all time (mutex has big complexity)
            {
                let mut val = should_stop.lock().unwrap();
                if *val {
                    *val = false;
                    return None;
                }
            }
        }
    }
}

pub fn mine_stop(block: &Block, should_stop: &Arc<Mutex<bool>>) -> Option<u64> {
    let mut rng = rand::thread_rng(); //to pick random value
    let mut hasher = DefaultHasher::new();

    //playload of block to hash
    block.block_height.hash(&mut hasher);
    block.parent_hash.hash(&mut hasher);
    // block.transactions.hash(&mut hasher);
    block.miner_hash.hash(&mut hasher);

    let mut nonce_to_test = rng.gen::<u64>();

    loop {
        let mut to_hash = hasher.clone(); //save l'état du hasher
        nonce_to_test.hash(&mut to_hash);

        let answer = to_hash.finish();

        if answer < HASH_MAX {
            return Some(nonce_to_test);
        }
        nonce_to_test = nonce_to_test.wrapping_add(1);
        if nonce_to_test % 100000 == 0 {
            //test not all time (mutex has big complexity)
            {
                let mut val = should_stop.lock().unwrap();
                if *val {
                    *val = false;
                    return None;
                }
            }
        }
    }
}

impl Transaction {
    pub fn new(src: u64, dst: u64, qqt: u32) -> Transaction {
        Transaction {
            src,
            dst,
            qqty: qqt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation_and_check() {
        let maximator = hash("uss");
        let chonker = hash("chonker");

        let transaction_a = Transaction::new(maximator, chonker, 100);

        let origin_block = Block::new(vec![]);
        assert!(origin_block.check());

        let block_1 = origin_block.new_block(vec![transaction_a], chonker);
        assert!(block_1.check());
    }

    #[test]
    fn test_miner_hash_standar() {
        let mut fist_block = Block::new(vec![]);
        fist_block.nonce = mine(&fist_block);
        fist_block.block_id = hash(&fist_block);
        assert!(fist_block.check());
    }

    #[test]
    fn test_mine_hasher_clone() {
        let mut fist_block = Block::new(vec![]);
        fist_block.nonce = mine_hasher_clone(&fist_block);
        fist_block.block_id = hash(&fist_block);
        assert!(fist_block.check());
    }

    #[test]
    fn test_mine_hasher_lessrng() {
        let mut fist_block = Block::new(vec![]);
        fist_block.nonce = mine_hasher_lessrng(&fist_block);
        fist_block.block_id = hash(&fist_block);
        assert!(fist_block.check());
    }
}
