use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

use super::transaction::{Transaction, RxUtxo};


const HASH_MAX: u64 = 1000000000000;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Block {
    /////////////////rendre private quand on aura imported mine extern du serveur
    pub block_id: u64,                  //the hash of whole block
    pub block_height: u64,              //the number of the current block
    pub parent_hash: u64,               //the id of last block (block are chain with that)
    pub transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    pub miner_hash: u64,                //Who find the answer
    pub nonce: u64,                     //the answer of the defi
    pub quote: String,
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
║Id block: {}s
║block_height : {}
║last_block : {}
║transactions {}       
║miner_id : {}                           
║nonce : {}
║quote : {}
╚═══════════════════════════════════════╝ ",
            self.block_id,
            self.block_height,
            self.parent_hash,
            transa_str,
            self.miner_hash,
            self.nonce,
            self.quote
        )
        //if it is pub clefs very long maybe put a hash
    }
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
            transactions: vec![],
            nonce: 0,
            miner_hash: 0,
            quote: String::from(""),
        };
        block.nonce = 0;
        block.block_id = hash(&block); //the
        block
    }

    //PARDON ? ces pas clean ??
    pub fn new_wrong(value: u64) -> Block {
        let mut block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: vec![],
            nonce: value, //for the block zero the nonce indique the status of the block (use to response to GetBlock(i))
            miner_hash: 0,
            quote: String::from(""),
        };
        block.block_id = hash(&block); //the
        block
    }

    pub fn get_height_nonce(&self) -> (u64, u64) {
        (self.block_height, self.nonce)
    }

    //la structure transaction peut faire un check sur le block donc pourait être un trait requi d'une transaction  --> une transaction est verifier surtout par les mineurs, pas vraiment duarnt la creation mais plutot dans l'interegration dans un bloc
    //une transaction peut utiliser le trait check pour check si le node est correct (last version blockaine)       --> comment ca un node correct, pas un block plutot ?
    //la transaction peut check check si le compte est bon si on fait une structure compte on peut metre le trait check  --> Une struct compte peut être une bonne idée mais elle serait pour quoi ? Parce que si on tien a jour tout les compte ca peut faire beaucoup (en gros en soit a chaque transa un regarde si c'est valid ou alors on tiens les comptes a jours)
    pub fn check(&self) -> bool {
        let mut hasher = DefaultHasher::new(); //why don't use hash fun ? hash(self) ?? like in last commit  -> je pense faut refaire un peu les hash (nottament il faut que le hash prennent en compte plus de chose comme l'id du hasheur pour la securité)

        //playload of block to hash
        // self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        // self.transactions.hash(&mut hasher);     //tres variable donc osef
        // self.miner_hash.hash(&mut hasher);
        // self.quote.hash(&mut hasher);
        self.nonce.hash(&mut hasher);

        let answer = hasher.finish();
        answer < HASH_MAX && hash(self) == self.block_id && self.quote.len() < 100
    }

    /* pub fn generate_block(
        &self,
        finder: u64,
        transactions: Vec<Transaction>,
        mut quote: &str,
        should_stop: &AtomicBool,
    ) -> Option<Block> {
        //wesh ces l'enfer ça
        //si tu check comme ça ces que le buffer peut être gros
        //faut check si ces pas des carac chelou --> c'est vite fait quoi
        if quote.len() > 100 {
            quote = "";
        }

        let mut new_block = Block {
            block_height: self.block_height + 1,
            block_id: 0,
            parent_hash: self.block_id,
            transactions, //put befort because the proof of work are link to transaction
            nonce: 0,
            miner_hash: finder, //j'aime pas
            quote: String::from(quote),
        };
        new_block.nonce = mine(&new_block, should_stop)?; //putain...
        new_block.block_id = hash(&new_block); //set the correct id
        Some(new_block)
    } */

    /// return a list of all utxo for a address
    pub fn get_utxos(&self,addr:u64) -> Vec<RxUtxo>{
        self.transactions.iter()
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
        self.miner_hash.hash(state);
        self.quote.hash(state);
        self.nonce.hash(state);
    }
}

//exelent!
impl PartialEq for Block {
    fn eq(&self, o: &Block) -> bool {
        self.block_id == o.block_id
    }
}

//comment ça ?
pub fn mine(finder: u64, cur_block: &Arc<Mutex<Block>>, sender: Sender<Block>) {
    let block = cur_block.lock().unwrap().clone();

    let mut new_block = Block {
        block_height: block.block_height + 1,
        block_id: 0,
        parent_hash: block.block_id,
        transactions: vec![], //put befort because the proof of work are link to transaction
        nonce: 0,
        miner_hash: finder, //j'aime pas
        quote: String::from("bi"),
    };

    let mut rng = rand::thread_rng(); //to pick random value
    let mut hasher = DefaultHasher::new();

    //playload of block to hash
    // block.block_height.hash(&mut hasher);
    block.parent_hash.hash(&mut hasher);
    block.transactions.hash(&mut hasher); //on doit fixer la transaction a avoir
                                          // block.miner_hash.hash(&mut hasher);
                                          // block.quote.hash(& mut hasher);

    let mut nonce_to_test = rng.gen::<u64>();

    warn!("Commencemet");
    loop {
        let mut to_hash = hasher.clone(); //save l'état du hasher
        nonce_to_test.hash(&mut to_hash);

        let answer = to_hash.finish();

        if answer < HASH_MAX {
            new_block.nonce = answer;
            new_block.block_id = hash(&block);
            info!("found this block : {}",new_block);
            sender.send(new_block.clone()).unwrap();
        }
        nonce_to_test = nonce_to_test.wrapping_add(1);
        if nonce_to_test % 10000000 == 0 {
            warn!("Refersh");

            let n_block = cur_block.lock().unwrap().clone();
            /* if n_block == block {
                continue;
            } */

            //rehash
            new_block = Block {
                block_height: n_block.block_height + 1,
                block_id: 0,
                parent_hash: n_block.block_id,
                transactions: vec![], //put befort because the proof of work are link to transaction
                nonce: 0,
                miner_hash: finder, //j'aime pas
                quote: String::from("bn"),
            };

            let mut hasher = DefaultHasher::new();
            info!("NEW BLOCK {}",new_block);
            n_block.parent_hash.hash(&mut hasher);
            n_block.transactions.hash(&mut hasher);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let block = Block::new();
        println!("There is the block : {}", block);
    }
}
