use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

const HASH_MAX: u64 = 1000000000000;
#[derive(Debug)]
pub struct Block {
    number: u64,        //the block number increment each new block
    id: u64,            //the id of the block -> the hash of the rest of element. So it is depences of all parameters
    last_block: u64,    //the id of last block (block are chain with that)
    transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    answer: u64,        //the answer of the defi
    finder: u64,        //Who find the answer
}
#[derive(Debug)]
pub struct Transaction {    
    initiating: u64,    //who send coin
    receiver: u64,      //who recive
    amount: u32,        //the acount
}



fn hash<T : Hash>(value: T) -> u64{         //return the hash of the item (need to have Hash trait)
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() 
}

impl Block {
    pub fn new(
        last_block: u64,
        answer: u64,
        transactions: Vec<Transaction>,
        number: u64,
        finder: u64,
    ) -> Block {                            //create a new block (just use for create the first one)             
        let mut block= Block {
            number: number,
            id: 0,
            last_block: last_block,
            transactions: transactions,
            answer: answer,
            finder: finder,
        };
        block.id = hash(& block);           //set the correct id
        block

    }

    pub fn verification(&self) -> bool {        //simple function to check if a block is valid
        hash(self.last_block.wrapping_add(self.answer)) < HASH_MAX && hash(&self) == self.id
    }

    pub fn new_block(&self, new_transa: Vec<Transaction>, answer: u64, finder: u64) -> Block {  //create a new block with the precendent (self)
        let mut new_block = Block {
            number: self.number + 1,
            id: 0,
            last_block: self.id,
            transactions: new_transa,
            answer: answer,
            finder: finder,
        };
        let mut hasher = DefaultHasher::new();
        new_block.hash(&mut hasher);
        new_block.id = hasher.finish();             //set the id of the block 
        new_block

    }
}

impl Hash for Block {       //implement the Hash's trait for Block 
    fn hash<H: Hasher>(&self, state: &mut H) {      //If Block have more element had it here
        // self.transactions(state);        //need to impl hash for Transactions
        self.answer.hash(state);
        self.last_block.hash(state);
        self.number.hash(state);
        self.finder.hash(state);
    }
}

pub fn mine(last_block: &Block) -> u64 {            //mine the search the answer of the defi
    let last_id = last_block.id;
    let mut number;
    let mut rng = rand::thread_rng();   //to pick random value
    loop {
        number = rng.gen::<u64>();
        let to_hash = number.wrapping_add(last_id);
        let answer: u64 = hash(to_hash);
        if answer < HASH_MAX {              //If hash<u64>(anwser + last_block.id) < HASH_MAX
            return number;      
        }
    }
}
