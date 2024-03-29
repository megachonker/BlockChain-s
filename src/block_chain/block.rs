use anyhow::{bail, ensure, Context, Result};
use chrono::{TimeZone, Utc};
use rand::Rng;

use serde::{Deserialize, Serialize};

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{info, trace, warn};

use super::blockchain::{Balance, Blockchain};
use super::node::server::{Event, MinerStuff, NewBlock};
use super::transaction::{Amount, HashValue, Transaction, TxIn, Utxo, UtxoValidator};

//variable d'envirnement

// const HASH_MAX: u64 = 100000000000000;           //for test
// const HASH_MAX: u64 = 1000000000;                //slow
// const HASH_MAX: u64 = 1000000000000; //fast
const CLOCK_DRIFT: u64 = 10; //second
pub const MINER_REWARD: Amount = 1; //the coin create for the miner

///
///
/// - get_block_hash_proof_work <  difficulty
/// - get_block_hash_proof_work is Composed of Hash(
///     - ~block_id~ used to cerfify timestamp
///     - block_height
///     - parent_hash
///     - transactions
///     - difficulty
///     - quote
///     - answer VARIABLE
///     - ~timestamp~ calculation with it are not eficient
#[derive(Debug, Serialize, Deserialize, Clone, Eq)]
pub struct Block {
    /////////////////rendre private quand on aura imported mine extern du serveur
    pub block_id: HashValue, //the hash That certify nothing ? but used to refere a block ?
    pub block_height: u64,   //the number of the current block
    pub parent_hash: HashValue, //the id of last block (block are chain with that)
    pub transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    pub difficulty: u64,     //the current difficulty fot the block (hash_proof <difficulty).
    pub quote: String,
    pub answer: u64, //the answer of the defi
    pub timestamp: Duration,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: vec![],
            answer: 0,
            quote: String::from(""),
            difficulty: 0,
            timestamp: Duration::from_secs(1683059400), //Begin of the project
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
║difficulty : {}   
║nonce : {}
║quote : {}
║timestamp : {}
╚═══════════════════════════════════════╝ ",
            self.block_id,
            self.block_height,
            self.parent_hash,
            transa_str,
            u64::MAX - self.difficulty,
            self.answer,
            self.quote,
            Utc.timestamp_millis_opt(self.timestamp.as_millis() as i64)
                .unwrap()
                .format("%Y %B %d %H:%M:%S"),
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

/* //without block id because need to be removed
impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_height.hash(state);
        self.parent_hash.hash(state);
        self.transactions.hash(state);
        self.difficulty.hash(state);
        self.quote.hash(state);
        self.answer.hash(state);
        self.timestamp.hash(state);
    }
}
 */
impl From<Profile> for u64 {
    fn from(prof: Profile) -> Self {
        match prof {
            Profile::INFINIT => u64::MAX,
            Profile::Normal => 50000000,
            Profile::Reactive => 5000000,
            Profile::Slow => 500000000,
        }
    }
}

impl Block {
    /*     pub fn get_hash(&self) ->HashValue{
           let mut hasher = DefaultHasher::new();
           self.hash(&mut hasher);
           //returning answer
           hasher.finish()
       }
    */
    /// auto hash block by init hasher
    pub fn get_block_hash_proof_work(&self) -> HashValue {
        //ini hasher
        let mut hasher = DefaultHasher::new();

        //playload of block to hash
        self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        self.transactions.hash(&mut hasher);
        self.difficulty.hash(&mut hasher);
        self.quote.hash(&mut hasher);
        self.answer.hash(&mut hasher);

        //calculate answer

        //returning answer
        hasher.finish()
    }

    /// create the first block full empty
    pub fn new() -> Block {
        Default::default()
    }


    ///check if the block is valid as agreed with Balence (the incoming utxo is inside balence)
    pub fn valid(&self, balance: &Balance) -> Result<()> {

        self.check()?;
        self.check_transactions(balance)?;
        

        Ok(())
    }


    //check if the transactions is right as agreed with the Balence (the current Balence)
    pub fn check_transactions(&self, balance: &Balance) -> Result<(), anyhow::Error> {
        let mut already_see = false;
        let mut miner_reward: Amount = 0;
        let mut transa_remain: Amount = 0;
        for t in &self.transactions {
            // if we have a mined transaction
            if t.rx.is_empty() && t.tx.len() == 1 {
                ensure!(!already_see, "double miner transa detected");
                already_see = true;

                //we skip validation because it unusal transa
                miner_reward = t.tx.first().context("t.tx.first()")?.get_amount();
            }
            // standar transaction
            else {
                t.valid(balance).context("transaction invalid")?;
                transa_remain += t
                    .remains(balance)?;
            }
        }
        ensure!(transa_remain + MINER_REWARD >= miner_reward, "reward");
        Ok(())
    }


    ///check if the block is consistant (Do not check the transactions)
    pub fn check(&self) -> Result<()> {
        // Check if the given block id is correct
        let answer = self.get_block_hash_proof_work();


        if self.block_id != get_id_block(self, answer) {
            bail!("the given block id is correct");
        }

        
        ensure!(answer < self.difficulty, "difficulty error");
        // ensure!(transa_remain + MINER_REWARD >= miner_reward, "reward");
        ensure!(get_id_block(self, answer) == self.block_id, "block id");
        ensure!(self.quote.len() < 100, "quote too long");
        ensure!(
            self.timestamp.as_secs()
                <= (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + CLOCK_DRIFT),
            "timestamp"
        );

        Ok(())
    }


    

    /// Lunch every time need to change transaction content or block
    /// using profile infinit can be use to not create a loop calling find next for test
    pub fn find_next_block(
        &self,
        transactions: Vec<Transaction>,
        profile: Profile,
        difficulty: u64,
    ) -> Option<Block> {
        // ad the self revenue explicite

        let mut new_block: Block = Block {
            block_height: self.block_height + 1,
            parent_hash: self.block_id,
            transactions,
            difficulty,
            ..Default::default() //styler
        };

        let number_iter: u64 = profile.into(); //tell me if it is better or superflue

        let mut rng = rand::thread_rng(); //to pick random value
        let mut hasher = DefaultHasher::new();

        new_block.block_height.hash(&mut hasher);
        new_block.parent_hash.hash(&mut hasher);
        new_block.transactions.hash(&mut hasher); //on doit fixer la transaction a avoir
        new_block.difficulty.hash(&mut hasher);
        new_block.quote.hash(&mut hasher);

        let mut nonce_to_test = rng.gen::<u64>();

        loop {
            let mut to_hash = hasher.clone(); //save l'état du hasher
            nonce_to_test.hash(&mut to_hash);

            let hash_proof_work = to_hash.finish();

            if hash_proof_work < difficulty {
                new_block.answer = nonce_to_test;
                new_block.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                new_block.block_id = get_id_block(&new_block, hash_proof_work); //a modif pour hash plus grand
                info!("found this block : {}", new_block);
                return Some(new_block);
            }

            if nonce_to_test % number_iter == 0 {
                return None;
            }

            nonce_to_test = nonce_to_test.wrapping_add(1);
        }
    }

    // can be turned to a trai ?

    /// find tx
    pub fn find_created_utxo(&self) -> Vec<Utxo> {
        self.transactions
            .iter()
            .flat_map(|t| t.tx.clone())
            .collect()
    }

    /// Find inside block all spended operation
    pub fn find_used_utxo(&self) -> Vec<TxIn> {
        self.transactions
            .iter()
            .flat_map(|t| t.rx.clone())
            .collect()
    }
}

fn get_id_block(new_block: &Block, hash_proof_work: HashValue) -> HashValue {
    let mut hasher = DefaultHasher::new();
    hash_proof_work.hash(&mut hasher); //data of block
    new_block.timestamp.hash(&mut hasher);
    hasher.finish()
}

//exelent!
impl PartialEq for Block {
    fn eq(&self, block: &Block) -> bool {
        self.block_id == block.block_id
    }
}

/// # Mining Runner
/// never ending function that feeded in transaction and block;
pub fn mine(miner_stuff: &Arc<Mutex<MinerStuff>>, sender: Sender<Event>) {
    info!("Miner task started");
    loop {
        // copy localy miner stuff
        let miner_stuff_lock = miner_stuff.lock().unwrap();
        let block = miner_stuff_lock.cur_block.clone(); //presque toujour blocker
        let transa = miner_stuff_lock.transa.clone();
        let difficulty = miner_stuff_lock.difficulty;
        drop(miner_stuff_lock);

        // lunch mining one time
        if let Some(mined_block) = block.find_next_block(transa, Profile::Normal, difficulty) {
            // if found send directly result
            sender
                .send(Event::NewBlock(NewBlock::Mined(mined_block)))
                .expect("mine Sender<Event>: cannot send a event");
        } else {
            trace!(
                "{:?}, found nothing for {}:{}",
                std::thread::current().id(),
                block.block_height,
                block.block_id
            );
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{sync::mpsc, thread};

    use super::*;
    use crate::block_chain::blockchain::FIRST_DIFFICULTY;

    #[test]
    fn default() {
        assert!(Block::new() == Block::default())
    }

    #[test]
    fn block_mined_valid() {
        let (tx, rx) = mpsc::channel::<Event>();
        let balance: Balance = Balance::default();

        let transa =
            Transaction::transform_for_miner(vec![], Default::default(), 1, &balance).unwrap();

        let miner_stuff = Arc::new(Mutex::new(MinerStuff {
            cur_block: Block::default(),
            transa,
            difficulty: crate::block_chain::blockchain::FIRST_DIFFICULTY,
        }));

        thread::spawn(move || {
            mine(&miner_stuff, tx);
        });

        for _ in 0..2 {
            let b = rx.recv().unwrap();

            match b {
                Event::NewBlock(b) => match b {
                    NewBlock::Mined(b) => b.valid(&balance).unwrap(),
                    NewBlock::Network(_) => assert!(false),
                },
                Event::HashReq(_) => assert!(false),
                Event::Transaction(_) => assert!(false),
                Event::ClientEvent(_, _) => todo!(),
            }
        }
    }

    #[test]
    fn mine2block() {
        let b0 = Block::default();

        let b1 = b0
            .find_next_block(vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        let b2 = b1
            .find_next_block(vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();

        println!("{}\n{}", b1, b2);

        assert_eq!(b2.parent_hash, b1.block_id);
        assert_eq!(b2.block_height, b1.block_height + 1);
    }

    #[test]
    fn find_next_block() {
        let block = Block::default();
        loop {
            if let Some(block_to_test) = block.find_next_block(
                Transaction::transform_for_miner(
                    vec![],
                    Default::default(),
                    1,
                    &Default::default(),
                )
                .unwrap(),
                Profile::Normal,
                FIRST_DIFFICULTY,
            ) {
                block_to_test.valid(&Default::default()).unwrap();
                break;
            }
        }
    }
}
