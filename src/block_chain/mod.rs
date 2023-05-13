use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

const HASH_MAX: u64 = 100000000000000;
#[derive(Debug)]
pub struct Block {
    block_id: u64, //the hash of whole block
    block_height: u64, //the number of the current block
    parent_hash: u64, //the id of last block (block are chain with that)
    transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    miner_hash: u64, //Who find the answer
    nonce: u64, //the answer of the defi
}
#[derive(Debug)]
#[derive(Hash)]
pub struct Transaction {
    src: u64, //who send coin
    dst: u64,   //who recive
    qqty: u32,     //the acount
}

pub fn hash<T: Hash>(value: T) -> u64 {
    //return the hash of the item (need to have Hash trait)
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl Block {
    /// crée le bloque génésis
    pub fn new(
        transactions: Vec<Transaction>,
    ) -> Block {
        //create a new block (just use for create the first one)
        let mut block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: transactions,
            nonce: 0,
            miner_hash: 0,
        };
        block.nonce = mine(&block);
        block.block_id = hash(&block);
        block
    }

    pub fn check(&self) -> bool {

        let mut hasher = DefaultHasher::new();

        //playload of block to hash
        self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        self.transactions.hash(&mut hasher);
        self.miner_hash.hash(&mut hasher);
        self.nonce.hash(&mut hasher);

        let answer = hasher.finish();
        answer < HASH_MAX && hash(&self) == self.block_id
    }

    pub fn generate_block(&self, new_transa: Vec<Transaction>, answer: u64, finder: u64) -> Block {
        let mut new_block = Block {
            block_height: self.block_height + 1,
            block_id: 0,
            parent_hash: self.block_id,
            transactions: new_transa,
            nonce: answer,
            miner_hash: finder,
        };
        new_block.nonce = mine(&new_block);
        new_block.block_id = hash(&new_block); //set the correct id
        new_block
    }
    pub fn new_block(&self, new_transa: Vec<Transaction>, finder: u64) -> Block {
        self.generate_block(new_transa,mine(self),finder)
    }
}

impl Hash for Block {
    //implement the Hash's trait for Block
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_height.hash(state);
        self.parent_hash.hash(state);
        self.transactions.hash(state);
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
        block.transactions.hash(&mut hasher);
        block.miner_hash.hash(&mut hasher);
        nonce_to_test.hash(&mut hasher);   
        let answer: u64 = hasher.finish();

        if answer < HASH_MAX {
            return nonce_to_test;
        }
    }
}

pub fn mine_fc_hash(block: &Block) -> u64 {
    let mut rng = rand::thread_rng(); //to pick random value
    loop {
        let nonce_to_test = rng.gen::<u64>();
        let mut hasher = DefaultHasher::new();

        //playload of block to hash
        block.block_height.hash(&mut hasher);
        block.parent_hash.hash(&mut hasher);
        block.transactions.hash(&mut hasher);
        block.miner_hash.hash(&mut hasher);
        nonce_to_test.hash(&mut hasher);   
        let answer: u64 = hasher.finish();

        if answer < HASH_MAX {
            return nonce_to_test;
        }
    }
}

impl Transaction {
    pub fn new(src: u64, dst: u64, qqt: u32) -> Transaction {
        let transaction = Transaction {
            src,
            dst,
            qqty: qqt
        };
        transaction
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_creation_and_check() {
        let maximator = hash("uss");
        let neeto = hash("neeto");
        let chonker = hash("chonker");

        let transaction_a = Transaction::new(maximator, chonker, 100);
        let transaction_b = Transaction::new(chonker, neeto, 10);

        let origin_block = Block::new(vec![transaction_a]);
        assert!(origin_block.check());

        let block_1 = origin_block.new_block(vec![transaction_b], chonker);
        assert!(block_1.check());
    }

    #[test]
    fn test_miner_hash_standar(){
        let mut fist_block = Block::new(vec![]);
        fist_block.nonce =  mine(&fist_block);
        fist_block.block_id = hash(&fist_block);
        assert!(fist_block.check());
    }

    #[test]
    fn test_mine_hasher_clone(){
        let mut fist_block = Block::new(vec![]);
        fist_block.nonce =  mine_hasher_clone(&fist_block);
        fist_block.block_id = hash(&fist_block);
        assert!(fist_block.check());
    }
}