use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    str::CharIndices,
};

use tracing::{debug, info, warn};

use super::{block::Block, node::server::MinerStuff, transaction::Utxo};

const TIME_100_BLOCK: u64 = 100 * 60; //time for 100 blocks in seconds
pub const FIRST_DIFFICULTY: u64 = 1000000000000000000;

#[derive(Default)]

/// Key of hashmap is the top block of the branch that need to be explorer
/// Value stored is a tuple of:
/// Hash of the the **next** block needed to validate the branch
/// The hight of the actual block ? can be found ?
///
struct PotentialsTopBlock {
    hmap: HashMap<u64, (u64, u64)>, //k : potentail top block,  v: (needed,height_of_k)
}
/// ne gère pas le cas out on
///
///
/// D   D' ok
///
/// C   C' ok
///
/// B on a deja
///
/// A
impl PotentialsTopBlock {
    fn new() -> PotentialsTopBlock {
        PotentialsTopBlock {
            hmap: HashMap::new(),
        }
    }

    fn get_needed_block(self) -> Vec<u64> {
        self.hmap.values().map(|v| v.0).collect()
    }

    fn add_new(&mut self, pot_top: &Block, needed_block: u64) {
        self.hmap
            .insert(pot_top.block_id, (needed_block, pot_top.block_height));
    }

    fn replace_or_create(&mut self, last_needed_block: &Block, new_needed_block: u64) {
        for (pot, v) in self.hmap.clone() {
            if v.0 == last_needed_block.block_id {
                self.hmap.insert(pot, (new_needed_block, v.1)); //replace
            }
        }
        self.hmap.insert(
            last_needed_block.block_id,
            (new_needed_block, last_needed_block.block_height),
        ); //create
    }

    fn found_potential_from_need(&self, need: u64) -> Option<u64> {
        self.hmap
            .iter()
            .find_map(|(&k, v)| (v.0 == need).then(|| k))
    }

    fn erease_old(&mut self, height_top_block: u64) {
        for (k, v) in self.hmap.clone() {
            if v.1 <= height_top_block {
                self.hmap.remove(&k);
            }
        }
    }

    fn is_block_needed(&self, block: u64) -> bool {
        self.hmap.values().any(|&(needed, _)| needed == block)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Status {
    Consumed,
    Avaible,
}
/// Keep track of transaction and utxo
/// Used to know the balance of somone ?
#[derive(Default, Clone)]
struct Balance {
    utxo: HashMap<Utxo, Status>,
}

impl Balance {
    /// Revert change until src with sub
    /// Replay change until dst with add
    pub fn calculation<'a, 'b>(&mut self, src: Vec<&'a Block>, dst: Vec<&'b Block>) -> &'b Block {
        src.iter().all(|p| self.sub(p));
        dst.iter()
            .find(|p| !self.add(p))
            .unwrap_or(dst.last().unwrap())
    }

    /// # Undo add
    /// when we want to drill downside
    /// we need to cancel transaction
    fn sub(&mut self, block: &Block) -> bool {
        let to_remove = block.find_new_utxo();

        let to_append = block.find_used_utxo();

        // debug!("to append");
        // to_append.iter().for_each(|f| debug!("{}", f));
        // debug!("to remove");
        // to_remove.iter().for_each(|f| debug!("{}", f));
        // debug!("----");

        for utxo in to_append {
            self.utxo.insert(utxo, Status::Avaible);
        }

        for utxo in to_remove {
            if let Some(_) = self.utxo.remove(&utxo) {
            } else {
                warn!("sub: la transa qui a été crée n'existe pas dans la hashmap");
                return false;
            }
        }

        // self.utxo.iter().for_each(|f| debug!("{}->{:?}", f.0, f.1));
        true
    }

    /// # Drill up
    /// normal whay to update the Balance with one block
    /// when we need to append a new block we run that
    fn add(&mut self, block: &Block) -> bool {
        //get utxo to append
        let to_append = block.find_new_utxo();

        //get utxo to remove
        let to_remove = block.find_used_utxo();

        // debug!("to append");
        // to_append.iter().for_each(|f| debug!("{}", f));
        // debug!("to remove");
        // to_remove.iter().for_each(|f| debug!("{}", f));
        // debug!("----");

        // Append transaction
        for utxo in to_append {
            if self.utxo.contains_key(&utxo) {
                warn!("add: double utxo entry");
                return false;
            } else {
                self.utxo.insert(utxo, Status::Avaible);
            }
        }

        // Consume transaction
        for utxo in to_remove {
            if let Some(utxo) = self.utxo.get_mut(&utxo) {
                *utxo = match utxo {
                    Status::Avaible => Status::Consumed,
                    Status::Consumed => {
                        warn!("utxo already consumed");
                        return false;
                    }
                }
            } else {
                warn!("add: consume using unknow utxo");
                return false;
            }
        }
        // self.utxo.iter().for_each(|f| debug!("{}->{:?}", f.0, f.1));debug!("Add");
        true
    }
}

pub struct Blockchain {
    hash_map_block: HashMap<u64, Block>,
    top_block_hash: u64,
    potentials_top_block: PotentialsTopBlock, // block need to finish the chain)
    balance: Balance,
    pub difficulty: u64,
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Block actuel: {}", self.top_block_hash).unwrap();
        self.get_chain()
            .into_iter()
            .for_each(|b| writeln!(f, "{}", b).unwrap());
        write!(f, "")
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain::new()
    }
}

impl Blockchain {
    // /// check if a utxo in the current blockaine
    // pub fn check_utxo(&self,utxo:&Utxo) -> bool{
    //     if let Some(block) = self.get_block(utxo.block_location){
    //         utxo.check(block)
    //     }
    //     false
    // }

    pub fn filter_utxo(&self, addr: u64) -> Vec<Utxo> {
        self.get_chain()
            .iter()
            .flat_map(|block| block.search_utxos(addr))
            .filter(|utxo| {
                self.balance
                    .utxo
                    .get(utxo)
                    .is_some_and(|state| matches!(state, Status::Avaible))
            })
            .collect()
    }

    pub fn new() -> Blockchain {
        let mut hash_map = HashMap::new();
        let first_block = Block::new();
        let hash_first_block = first_block.block_id;
        hash_map.insert(hash_first_block, first_block);

        Blockchain {
            hash_map_block: hash_map,
            top_block_hash: hash_first_block,
            potentials_top_block: PotentialsTopBlock::new(),
            balance: Default::default(),
            difficulty: FIRST_DIFFICULTY,
        }
    }

    pub fn get_block<'a>(&'a self, hash: u64) -> Option<&'a Block> {
        self.hash_map_block.get(&hash)
    }


    /// # Appand bloc to blockchain struct
    /// block_to_append will be included in the struct this block can be :
    ///     -ignored if wrong or to old
    ///     -the new top block (block_height +1)
    ///     -a potential new top bloc but other block are needed (to complete the chain to block 0)
    ///     -complete a chain a for a potential top block
    /// This function return a (Option<Block>, Option<u64>).
    /// The first Option is conataint the new top block if a new top block is found (not necessary the block_to_append).
    /// The second Option is containt the hash of a block which are needed to complete a chain.
    pub fn try_append(&mut self, block_to_append: &Block) -> (Option<Block>, Option<u64>) {
        if self.hash_map_block.contains_key(&block_to_append.block_id) {
            return (None, None); //already prensent
        }

        if !block_to_append.check() {
            //<== full check here
            warn!("block is not valid");
            return (None, None);
        }

        //add the block to the HashMap
        self.hash_map_block
            .insert(block_to_append.block_id, block_to_append.clone());

        //get the current block from the db to compare to the new one
        let cur_block = self.hash_map_block.get(&self.top_block_hash).unwrap();

        // the block is superior than my actual progress ?
        if block_to_append.block_height > cur_block.block_height {
            //does have same direct ancestor
            if block_to_append.parent_hash == cur_block.block_id
                && block_to_append.block_height == cur_block.block_height + 1
                && !self
                    .potentials_top_block
                    .is_block_needed(block_to_append.block_id) // these block is needed from a higher block 
                && !block_to_append.transactions.iter().all(|t| t.check(&self))
            //<= need to move to the
            {
                //basic case
                let mut backup = self.balance.clone();
                if backup.add(block_to_append) {
                    //comit modification
                    self.balance = backup;
                    //valid and ellect block to top pos
                    self.top_block_hash = block_to_append.block_id;
                } else {
                    info!("Transaction is false");
                    return (None, None);
                }
            } else {
                //block to high
                match self.search_chain(block_to_append) {
                    //do we have chain from block 0
                    Ok(_) => {
                        //the block can be chained into the initial block
                        let new_top_b = match self
                            .potentials_top_block
                            .found_potential_from_need(block_to_append.block_id)
                        {
                            Some(new_top_block) => {
                                println!("lkqjdopqkhfpm");
                                new_top_block
                            }
                            None => block_to_append.block_id,
                        };

                        //chack transa and udpate balence
                        let two_chain = self.get_path_2_block(self.top_block_hash, new_top_b);
                        //sale
                        let mut new_balence = self.balance.clone();
                        let last_top_transa_ok = new_balence
                            .calculation(two_chain.0, two_chain.1.iter().rev().cloned().collect())
                            .block_id;
                        //last_top_transa_ok : bloc where is transa is valid to the chain

                        if last_top_transa_ok == new_top_b {
                            //all it is ok
                            warn!("New branche better branches founds, blockchain update");
                            self.balance = new_balence;
                            self.top_block_hash = last_top_transa_ok;
                        } else if cur_block.block_height
                            < self.get_block(last_top_transa_ok).unwrap().block_height
                        {
                            info!(
                                "New branche not complete right, wrong after {}",
                                last_top_transa_ok
                            );
                            //also ok maybe
                            self.balance = new_balence;
                            self.top_block_hash = last_top_transa_ok;

                            //need maybe to earse wrong block which transa is not good with the chain (last_top_ok + 1 +2 ...) <= you need to flush potendial block ?
                        } else {
                            info!("Branch is not wrong ");
                            return (None, None);
                        }
                    }
                    Err(needed) => {
                        //the block can not be chained into the initial block : needed is missing
                        self.potentials_top_block
                            .replace_or_create(&block_to_append, needed);
                        return (None, Some(needed));
                    }
                }
            }
            //drop the search cache
            self.potentials_top_block.erease_old(self.top_block_hash);

            return (Some(self.last_block()), None);
        }

        (None, None)
    }

    fn get_path_2_block(&self, last_top: u64, new_top: u64) -> (Vec<&Block>, Vec<&Block>) {
        let mut vec1: Vec<&Block> = vec![];
        let mut vec2: Vec<&Block> = vec![];

        let mut last = self.get_block(last_top).unwrap();
        let mut new = self.get_block(new_top).unwrap();

        while last.block_height < new.block_height {
            println!("Ici");
            vec2.push(new);
            new = self.get_block(new.parent_hash).unwrap();
        }

        while new.block_id != last.block_id {
            println!("La");

            vec1.push(last);
            vec2.push(new);
            new = self.get_block(new.parent_hash).unwrap();
            last = self.get_block(last.parent_hash).unwrap();
        }

        vec1.push(last);
        vec2.push(new);

        (vec1, vec2)
    }

    pub fn last_block(&self) -> Block {
        self.hash_map_block
            .get(&self.top_block_hash)
            .unwrap()
            .clone()
    }

    fn search_chain<'a>(&'a self, mut block: &'a Block) -> Result<Vec<u64>, u64> {
        //the second u64 is a block which we don't have (need for the chain)
        let mut vec = vec![block.block_id];
        while block.block_id != 0 {
            vec.push(block.parent_hash);
            match self.hash_map_block.get(&block.parent_hash) {
                Some(parent) => block = parent,
                None => return Err(block.parent_hash),
            }
        }

        return Ok(vec);
    }

    pub fn get_chain<'a>(&'a self) -> Vec<&'a Block> {
        let mut vec = vec![];
        let mut hash = self.top_block_hash;

        loop {
            let b = self.hash_map_block.get(&hash).unwrap();
            vec.push(b);

            hash = b.parent_hash;

            if hash == 0 {
                vec.push(self.hash_map_block.get(&0).unwrap());
                break;
            }
        }
        return vec;
    }

    pub fn get_utxo(&self, id_client: u64) -> Vec<Utxo> {
        let mut res = vec![];
        for (u, a) in &self.balance.utxo {
            if a == &Status::Avaible {
                if self
                    .get_block(u.block_location)
                    .expect("Error utxo present in balence, his block is not present")
                    .utxo_owned(u)
                    == id_client
                {
                    res.push(u.clone());
                }
            }
        }

        res
    }

    pub fn transa_is_valid(
        &self,
        transa: &super::transaction::Transaction,
        miner_stuff: &MinerStuff,
    ) -> bool {
        //check all
        //NEED TO FIX : check in balence if present, check in miner_stuff.transa to see if utxo is already use or not
        !miner_stuff.transa.contains(transa)
    }

    pub fn new_difficutly(&mut self) -> u64 {
        let top_block = self.get_block(self.top_block_hash).unwrap();
        let height: u64 = top_block.block_height;
        if height % 100 == 0 {
            let chain = self.get_chain();
            if chain.len() >= 100 {
                let time_between_100 = top_block.timestamp - chain[99].timestamp;
                let mut rate_time = (TIME_100_BLOCK as f64) / (time_between_100.as_secs() as f64);
                debug!("Rate time 100 blocks {}", rate_time);
                if rate_time < 0.90 || rate_time > 0.110 {
                    /* let new_dif = if rate_time >= 1.10 {
                        self.difficulty / 2
                    } else {
                        self.difficulty * 2
                    }; */
                    if rate_time == f64::INFINITY{
                        rate_time = 1000.0; 
                    }
                    let new_dif = (self.difficulty as f64 / rate_time) as u64;
                    self.difficulty = new_dif;
                    warn!("New difficulty {} ", new_dif);
                }
            }
        }
        self.difficulty
    }
}

#[cfg(test)]
mod tests {

    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::Duration;

    use crate::block_chain::{
        block::Profile,
        transaction::{self, Transaction},
    };

    use super::*;

    #[test]
    fn create_blockchain() {
        let block_chain = Blockchain::new();

        assert_eq!(block_chain.last_block(), Block::new());
    }

    #[test]
    fn append_wrong_blockchain() {
        let mut block_chain = Blockchain::new();

        let (cur_block, _) = block_chain.try_append(&Block {
            //not a valid block
            block_id: 7,
            block_height: 1,
            parent_hash: 7,
            transactions: vec![],
            finder: 7,
            answer: 7,
            quote: String::from(""),
            difficulty: 10000000,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
        });
        assert_eq!(cur_block, None);
    }

    #[test]
    fn append_blockchain_second_block() {
        let mut blockchain = Blockchain::new();
        let block = Block::default()
            .find_next_block(0, vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        assert_eq!(block, blockchain.try_append(&block).0.unwrap());
    }

    #[test]
    /// lol ça marche pas quand need block est remplis
    fn add_block_unchainned() {
        let mut blockchain = Blockchain::new();
        let b1 = Block::default()
            .find_next_block(0, vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        let b2 = b1.find_next_block(0, vec![], Profile::INFINIT,FIRST_DIFFICULTY).unwrap();

        ///////////////////////////////////////////////////
        //// SI commneter ça marche
        ///// Le fait d'ajouter un block rend imposible de marcher apres
        let (new, need) = blockchain.try_append(&b2);

        assert_eq!(new, None);
        assert_eq!(need.unwrap(), b1.block_id);
        ///////////////////////////////////////////////////////////////

        let (new, need) = blockchain.try_append(&b1);
        // println!("[B1{}b2{}]{}", b1, b2, blockchain);
        let new = new.unwrap();
        assert_eq!(new, b2);
        assert_eq!(need, None);
    }

    #[test]
    fn remove_old_potential_top() {
        for _ in 1..2 {
            let mut blockchain = Blockchain::new();

            let b0 = Block::default();
            let b1: Block = b0
                .clone()
                .find_next_block(0, vec![], Profile::INFINIT,FIRST_DIFFICULTY)
                .unwrap();
            let b1_bis: Block = b0
                .clone()
                .find_next_block(0, vec![], Profile::INFINIT,FIRST_DIFFICULTY)
                .unwrap();
            let b2 = b1
                .clone()
                .find_next_block(10, vec![Default::default()], Profile::INFINIT,FIRST_DIFFICULTY)
                .unwrap();
            let b2_bis = b1_bis
                .clone()
                .find_next_block(10, vec![Default::default()], Profile::INFINIT,FIRST_DIFFICULTY)
                .unwrap();

            blockchain.try_append(&b2_bis);
            assert_ne!(
                blockchain.potentials_top_block.hmap.get(&b2_bis.block_id),
                None
            );

            blockchain.try_append(&b1);
            blockchain.try_append(&b2);

            assert_eq!(
                blockchain.potentials_top_block.hmap.get(&b2_bis.block_id),
                None
            );
        }
    }

    #[test]
    fn get_chain() {
        let mut blockchain = Blockchain::new();
        let block = Block::default()
            .find_next_block(0, vec![], Profile::INFINIT,FIRST_DIFFICULTY)
            .unwrap();
        blockchain.try_append(&block);
        assert_eq!(blockchain.get_chain(), vec![&block, &Block::new()]);
    }
    /*
    #[test]
    fn get_path_2_block() {
        let mut blockchain = Blockchain::new();

        let b1 = Block {
            block_height: 1,
            block_id: 84739656938,
            parent_hash: 0,
            transactions: vec![],
            answer: 8308871350387475192,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        let b2 = Block {
            block_height: 2,
            block_id: 32147335136,
            parent_hash: 84739656938,
            transactions: vec![],
            answer: 9377674440955505,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        let b2_bis = Block {
            block_height: 2,
            block_id: 190296940020,
            parent_hash: 84739656938,
            transactions: vec![],
            answer: 11832120156767897387,
            finder: 0,
            quote: String::from(""),
        };

        let b3 = Block {
            block_height: 3,
            block_id: 44263391524,
            parent_hash: 32147335136,
            transactions: vec![],
            answer: 13893443482872540816,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        assert!(b1.check());

        blockchain.try_append(&b1);
        blockchain.try_append(&b2_bis);
        blockchain.try_append(&b2);
        blockchain.try_append(&b3);

        let res = blockchain.get_path_2_block(b2_bis.block_id, b3.block_id);
        let a = vec![b2_bis, b1.clone()];
        let b = vec![b3, b2, b1];

        let must = (a.iter().collect(), b.iter().collect());

        assert_eq!(res, must);
    } */

    #[test]
    /// check rewind
    /// check add
    /// check sub
    /// check double transa
    /// check double usage utxo
    fn balance_calculation_simple() {
        let mut balance = Balance::default();

        // Create Block
        let mut block1 = Block::default();
        let mut block2 = Block::default();
        let mut block3 = Block::default();
        let mut block4 = Block::default();

        // Create Transactions
        let transaction1 = Transaction::new(Default::default(), vec![10], 0);
        let transaction2 = Transaction::new(Default::default(), vec![20], 0);

        block1.transactions = vec![transaction1, transaction2];

        let transaction3 = Transaction::new(
            block1.transactions[0].find_new_utxo(block1.block_id),
            vec![5],
            0,
        );
        let transaction4 = Transaction::new(
            block1.transactions[1].find_new_utxo(block1.block_id),
            vec![15],
            0,
        );

        block2.transactions = vec![transaction3, transaction4];

        let transaction5 = Transaction::new(block2.find_new_utxo(), vec![25], 0);

        // Create Blocks
        block3.transactions = vec![transaction5];

        let tmp = vec![block1.clone(), block2.clone(), block3];
        let vec: Vec<&Block> = tmp.iter().collect();

        vec.iter().for_each(|v| println!("{}", v));

        let vec_r: Vec<&Block> = vec.clone().iter().cloned().rev().collect();

        vec.iter().for_each(|f| {
            let _ = balance.add(f);
        });
        println!("INITIALISED");
        let mut instance = balance.clone();

        instance
            .utxo
            .iter()
            .for_each(|f| println!("{}==>{:?}", f.0, f.1));

        let ret = instance.calculation(vec_r, vec);
        println!("{}", ret);
        assert_eq!(*ret, block1);

        //try replay transaction
        let transaction6 = Transaction::new(block2.find_new_utxo(), vec![25], 0);
        block4.transactions = vec![transaction6];
        assert!(!balance.clone().add(&block4));

        //try reusing already spend utxo
        let transaction6 = Transaction::new(block2.find_new_utxo(), vec![5], 0);
        block4.transactions = vec![transaction6];
        assert!(!balance.clone().add(&block4))
    }

    // #[test]
    // fn transaction_simple() {
    //     // let mut blockchain = Blockchain::new();
    //     // let block = Block::new();
    //     // let transaction : Transaction::new

    //     // block.find_next_block(621, transactions)
    //     // blockchain.append(block)
    // }
}
