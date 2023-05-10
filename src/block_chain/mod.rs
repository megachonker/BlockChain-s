use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

const HASH_MAX: u64 = 1000000000000;
#[derive(Debug)]
pub struct Block {
    number : u64,
    id: u64,
    last_block: u64,
    transactions: Vec<Transaction>,
    answer: u64,
    finder : u64,       //Who find the answer
}
#[derive(Debug)]
pub struct Transaction {
    initiating: u64,
    receiver: u64,
    amount: u32,
}

fn hash_int(value: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl Block {
    pub fn new(last_block: u64, answer: u64, transactions: Vec<Transaction>, number : u64, finder: u64) -> Block {
        let id = answer; //for the momment
        return Block {
            number : number,
            id : id,
            last_block: last_block,
            transactions: transactions,
            answer: answer,
            finder : finder,
        };
    }

    pub fn verification(&self) -> bool {
        hash_int(self.last_block.wrapping_add(self.answer)) < HASH_MAX
    }
    pub fn new_block(&self, new_transa: Vec<Transaction>, answer: u64, finder : u64) -> Block {
        Block {
            number : self.number+1,
            id: answer,
            last_block: self.id,
            transactions: new_transa,
            answer: answer,
            finder : finder,
        }
    }
}

pub fn mine(last_block: &Block) -> u64 {
    // let mut hasher: DefaultHasher = DefaultHasher::new();
    let last_id = last_block.id;
    let mut number;
    let mut rng = rand::thread_rng();
    loop {
        number = rng.gen::<u64>();
        let to_hash = number.wrapping_add(last_id);
        // to_hash.hash(&mut hasher);       //pose probleme
        // let answer: u64 = hasher.finish();
        let answer:u64 = hash_int(to_hash);
        if answer < HASH_MAX {
            return number;
        }
    }
}
