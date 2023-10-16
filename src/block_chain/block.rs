use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

use super::node::server::{Event, NewBlock};
use super::transaction::{Transaction, Utxo};

const HASH_MAX: u64 = 100000000000000;           //for test
// const HASH_MAX: u64 = 1000000000;

#[derive(Debug, Serialize, Deserialize, Clone, Eq)]
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
        Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: vec![],
            answer: 0,
            finder: 0,
            quote: String::from(""),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let transa_str = if self.transactions.len() > 6 {
            format!(
                "<{}>:\n║   {}\n║   {}\n║   {}\n║...\n║   {}\n║   {}\n║   {}",
                self.transactions.len(),
                self.transactions[0].display_for_bock(),
                self.transactions[1].display_for_bock(),
                self.transactions[2].display_for_bock(),
                self.transactions[self.transactions.len() - 3].display_for_bock(),
                self.transactions[self.transactions.len() - 2].display_for_bock(),
                self.transactions[self.transactions.len() - 1].display_for_bock()
            )
        } else {
            format!(
                "\n║   {}",
                self.transactions
                    .iter()
                    .map(|transa| transa.display_for_bock())
                    .collect::<Vec<_>>()
                    .join("\n║   ")
            )
        };

        write!(
            f,
            "
╔═══════════════════════════════════════╗
║Id block: {}
║block_height : {}
║parent_block : {}
║transactions {}:       
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

/// # Hasher comportement
/// Define how manny hash to try
/// beffort reseting
///
/// INFINI never ending
/// Reactive try to show how ipc are bad
/// Slow optimise perf but should create more branch
/// Normal default beavior
pub enum Profile {
    INFINIT,
    Reactive,
    Slow,
    Normal,
}

impl From<Profile> for u64 {
    fn from(prof: Profile) -> Self {
        match prof {
            Profile::INFINIT => return u64::MAX,
            Profile::Normal => return 50000000,
            Profile::Reactive => return 5000000,
            Profile::Slow => return 500000000,
        }
    }
}

impl Block {
    /// auto hash block by init hasher
    pub fn get_block_id(&self) -> u64 {
        //ini hasher
        let mut hasher = DefaultHasher::new();

        //playload of block to hash
        self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        self.transactions.hash(&mut hasher);
        self.finder.hash(&mut hasher);
        self.quote.hash(&mut hasher);
        self.answer.hash(&mut hasher);

        //calculate answer
        let answer = hasher.finish();

        //returning answer
        answer
    }

    /// create the first block full empty
    pub fn new() -> Block {
        Default::default()
    }

    pub fn get_height_nonce(&self) -> (u64, u64) {
        (self.block_height, self.answer)
    }

    pub fn check(&self) -> bool {
        let answer = self.get_block_id();

        //check block_id
        //check answer
        answer < HASH_MAX && answer == self.block_id && self.quote.len() < 100
    }

    /// Lunch every time need to change transaction content or block
    /// using profile infinit can be use to not create a loop calling find next for test
    pub fn find_next_block(
        &self,
        finder: u64,
        mut transactions: Vec<Transaction>,
        profile: Profile,
    ) -> Option<Block> {
        // ad the self revenue explicite
        transactions.push(Transaction::new(Default::default(), vec![100], finder));

        let mut new_block: Block = Block {
            block_height: self.block_height + 1,
            parent_hash: self.block_id,
            finder,
            transactions,
            ..Default::default() //styler
        };


        let number_iter: u64 = profile.into(); //tell me if it is better or superflue

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

            if nonce_to_test % number_iter == 0 {
                debug!("Refersh");
                return None;
            }

            nonce_to_test = nonce_to_test.wrapping_add(1);
        }
    }

    /// find unspend transaction
    /// need to convert u128 to utxo
    pub fn find_new_utxo(&self) -> Vec<Utxo> {
        self.transactions
            .iter()
            .flat_map(|t| t.find_new_utxo(self.block_id))
            .collect()
    }

    /// Find inside block all spended operation
    pub fn find_used_utxo(&self) -> Vec<Utxo> {
        self.transactions
            .iter()
            .flat_map(|t| t.find_used_utxo())
            .collect()
    }

    /// return a list of all utxo for a address
    pub fn search_utxos(&self, addr: u64) -> Vec<Utxo> {
        self.transactions
            .iter()
            .filter(|transa| transa.target_pubkey == addr)
            .flat_map(|transa| transa.find_new_utxo(self.block_id))
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
        // println!("qskdgkjsqdh{}",block);
        drop(block_locked);

        // do the same things
        // block
        //     .find_next_block(finder, transaction)
        //     .map(|block| sender.send(block))
        //     .unwrap();

        if let Some(mined_block) = block.find_next_block(finder, vec![], Profile::Slow) {
            sender
                .send(Event::NewBlock(NewBlock::Mined(mined_block)))
                .unwrap();

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
                Event::NewBlock(b) => match b {
                    NewBlock::Mined(b) => assert!(b.check()),
                    NewBlock::Network(_) => assert!(false),
                },
                Event::HashReq(_) => assert!(false),
                Event::Transaction(_) => assert!(false),
                Event::ClientEvent(_, _) => todo!(),
                
            }
        }
    }

    #[test]
    fn mine2block(){
        let b0 = Block::default();

        let b1 = b0.find_next_block(0, vec![], Profile::INFINIT).unwrap();
        let b2 = b1.find_next_block(0, vec![], Profile::INFINIT).unwrap();

        println!("{}\n{}",b1,b2);

        assert_eq!(b2.parent_hash,b1.block_id);
        assert_eq!(b2.block_height,b1.block_height+1);
    }

    #[test]
    fn find_next_block() {
        let block = Block::default();
        loop {
            if let Some(block_to_test) =
                block.find_next_block(Default::default(), Default::default(), Profile::Normal)
            {
                assert!(block_to_test.check());
                break;
            }
        }
    }
}
