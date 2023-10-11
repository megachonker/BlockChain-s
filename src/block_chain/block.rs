use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tracing::{info, warn, debug};

use super::node::server::{Event,NewBlock};
use super::transaction::{RxUtxo, Transaction};

const HASH_MAX: u64 = 1000000000000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    /////////////////rendre private quand on aura imported mine extern du serveur
    pub block_id: u64,                  //the hash of whole block
    pub block_height: u64,              //the number of the current block
    pub parent_hash: u64,               //the id of last block (block are chain with that)
    pub transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    pub finder: u64,                    //Who find the answer
    pub quote: String,
    pub answer: u64, //the answer of the defi
}

impl Default for Block {
    fn default() -> Self {
        Block::new()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let transa_str = if self.transactions.len() > 3 {
            format!(
                "<{}> : [ {:?} {:?} {:?} ... ]",
                self.transactions.len(),
                self.transactions[0],
                self.transactions[1],
                self.transactions[3]
            )
        } else {
            format!("<{}> : {:?}", self.transactions.len(), self.transactions)
        };

        write!(
            f,
            "\n
╔═══════════════════════════════════════╗
║Id block: {}
║block_height : {}
║parent_block : {}
║transactions {}       
║miner_id : {}                           
║nonce : {}
║quote : {}
╚═══════════════════════════════════════╝ ",
            self.block_id,
            self.block_height,
            self.parent_hash,
            transa_str,
            self.finder,
            self.answer,
            self.quote
        )
        //if it is pub clefs very long maybe put a hash
    }
}

impl Block {
    /// create the first block full empty
    pub fn new() -> Block {
        let block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: vec![],
            answer: 0,
            finder: 0,
            quote: String::from(""),
        };
        block
    }

    pub fn get_height_nonce(&self) -> (u64, u64) {
        (self.block_height, self.answer)
    }

    //la structure transaction peut faire un check sur le block donc pourait être un trait requi d'une transaction  --> une transaction est verifier surtout par les mineurs, pas vraiment duarnt la creation mais plutot dans l'interegration dans un bloc
    //une transaction peut utiliser le trait check pour check si le node est correct (last version blockaine)       --> comment ca un node correct, pas un block plutot ?
    //la transaction peut check check si le compte est bon si on fait une structure compte on peut metre le trait check  --> Une struct compte peut être une bonne idée mais elle serait pour quoi ? Parce que si on tien a jour tout les compte ca peut faire beaucoup (en gros en soit a chaque transa un regarde si c'est valid ou alors on tiens les comptes a jours)
    pub fn check(&self) -> bool {
        let mut hasher = DefaultHasher::new(); //why don't use hash fun ? hash(self) ?? like in last commit  -> je pense faut refaire un peu les hash (nottament il faut que le hash prennent en compte plus de chose comme l'id du hasheur pour la securité)

        //playload of block to hash
        self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        self.transactions.hash(&mut hasher);
        self.finder.hash(&mut hasher);
        self.quote.hash(&mut hasher);
        self.answer.hash(&mut hasher);

        let answer = hasher.finish();
        answer < HASH_MAX && answer == self.block_id && self.quote.len() < 100
    }

    fn find_next_block(&self, finder: u64, transactions: Vec<Transaction>) -> Option<Block> {
        let mut new_block: Block = Block {
            block_height: self.block_height + 1,
            parent_hash: self.block_id,
            finder,
            transactions,
            ..Default::default() //styler
        };

        let mut rng = rand::thread_rng(); //to pick random value
        let mut hasher = DefaultHasher::new();

        new_block.block_height.hash(&mut hasher);
        new_block.parent_hash.hash(&mut hasher);
        new_block.transactions.hash(&mut hasher); //on doit fixer la transaction a avoir
        new_block.finder.hash(&mut hasher);
        new_block.quote.hash(&mut hasher);

        let mut nonce_to_test = rng.gen::<u64>();

        loop {
            let mut to_hash = hasher.clone(); //save l'état du hasher
            nonce_to_test.hash(&mut to_hash);

            let answer = to_hash.finish();

            if answer < HASH_MAX {
                new_block.answer = nonce_to_test;
                new_block.block_id = answer; //a modif pour hash plus grand
                info!("found this block : {}", new_block);
                return Some(new_block);
            }

            if nonce_to_test % 50000000 == 0 {
                debug!("Refersh");
                return None;
            }

            nonce_to_test = nonce_to_test.wrapping_add(1);
        }
    }

    /// return a list of all utxo for a address
    pub fn get_utxos(&self, addr: u64) -> Vec<RxUtxo> {
        self.transactions
            .iter()
            .filter(|transa| transa.target_pubkey == addr)
            .flat_map(|transa| transa.get_utxos(self.block_id))
            .collect()
    }
}

impl Hash for Block {
    //implement the Hash's trait for Block
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_height.hash(state);
        self.parent_hash.hash(state);
        self.transactions.hash(state);
        self.finder.hash(state);
        self.quote.hash(state);
        self.answer.hash(state);
    }
}

//exelent!
impl PartialEq for Block {
    fn eq(&self, o: &Block) -> bool {
        self.block_id == o.block_id
    }
}

/// # Mining Runner
/// never ending function that feeded in transaction and block;
pub fn mine(finder: u64, cur_block: &Arc<Mutex<Block>>, sender: Sender<Event>) {
    info!("Begining mining operation");
    loop {
        let block_locked = cur_block.lock().unwrap();
        let block = block_locked.clone(); //presque toujour blocker
        drop(block_locked);
        let transaction = vec![];

        // do the same things
        // block
        //     .find_next_block(finder, transaction)
        //     .map(|block| sender.send(block))
        //     .unwrap();

        if let Some(mined_block) = block.find_next_block(finder, transaction) {
            sender.send(Event::NewBlock(NewBlock::Mined(mined_block))).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread};

    use super::*;

    #[test]
    fn test_display() {
        let block = Block::new();
        println!("There is the block : {}", block);
    }

    #[test]
    fn default() {
        assert!(Block::new() == Block::default())
    }

    #[test]
    fn test_block_mined_valid() {
        let (tx, rx) = mpsc::channel::<Event>();

        let cur_block = Arc::new(Mutex::new(Block::new()));

        thread::spawn(move || {
            mine(1, &cur_block, tx);
        });

        for _ in 0..2 {
            let b = rx.recv().unwrap();

            match b {
                Event::NewBlock(b) => {match b {
                    NewBlock::Mined(b) => assert!(b.check()),
                    NewBlock::Network(_) => assert!(false),
                }}
                Event::HashReq(_) => assert!(false),
                Event::Transaction(_) => assert!(false),
            }

            
        }
    }

    #[test]
    fn test_find_next_block() {
        let block = Block::default();
        loop {
            if let Some(block_to_test) =
                block.find_next_block(Default::default(), Default::default())
            {
                assert!(block_to_test.check());
                break;
            }
        }
    }
}
