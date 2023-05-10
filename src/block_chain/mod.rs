use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use rand::Rng;

const HASH_MAX: u64 = 100000000000;

pub struct Block {
    id: u64,
    last_block: u64,
    transactions: Vec<Transaction>,
    answer: u64,
}

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
    pub fn new(last_block: u64, answer: u64, transactions: Vec<Transaction>) -> Block {
        let id = answer; //for the momment
        return Block {
            id: id,
            last_block: last_block,
            transactions: transactions,
            answer: answer,
        };
    }

    pub fn verification(&self) -> bool {
        hash_int(self.last_block + self.answer) < HASH_MAX
    }
    pub fn new_block(&self, new_transa: Vec<Transaction>, answer: u64) -> Block {
        Block {
            id: answer,
            last_block: self.id,
            transactions: new_transa,
            answer: answer,
        }
    }
}

pub fn mine(last_block: &Block) -> u64 {
    let mut hasher: DefaultHasher = DefaultHasher::new();
    let last_answer = last_block.answer;
    let mut number = 0;
    let mut rng = rand::thread_rng();
    loop {
        number = rng.gen::<u64>();
        let to_hash = number + last_answer;
        to_hash.hash(&mut hasher);
        let answer = hasher.finish();
        if answer < HASH_MAX {
            return answer;
        }
    }
    return 0;
}
