use core::fmt;
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    f32::consts::E,
    fs::File,
    io::{Chain, Read, Write}, hash::{Hash, Hasher},
};

use tracing::{debug, info, warn, error};

use crate::block_chain::block;

use super::{block::Block, node::server::MinerStuff, transaction::Utxo};
const N_BLOCK_DIFFICULTY_CHANGE: u64 = 100;
const TIME_N_BLOCK: u64 = 100 * 60; //time for 100 blocks in seconds
pub const FIRST_DIFFICULTY: u64 = 10000000000000;
// pub const FIRST_DIFFICULTY: u64 = 100000000;

/// Key of hashmap is the top block of the branch that need to be explorer
/// Value stored is a tuple of:
/// Hash of the the **next** block needed to validate the branch
/// The hight of the actual block ? can be found ?
///

#[derive(Debug, Default)]
struct PotentialsTopBlock {
    hmap: HashMap<u64, (u64, u64)>, //k : potentail top block,  v: (needed,height_of_k)
}

impl PotentialsTopBlock {
    fn new() -> PotentialsTopBlock {
        PotentialsTopBlock {
            hmap: HashMap::new(),
        }
    }

    fn replace_or_create(&mut self, last_needed_block: &Block, new_needed_block: u64) {
        for (pot, v) in self.hmap.clone() {
            if v.0 == last_needed_block.block_id {
                self.hmap.insert(pot, (new_needed_block, v.1)); //replace
                return;
            }
        }
        self.hmap.insert(
            last_needed_block.block_id,
            (new_needed_block, last_needed_block.block_height),
        );
    }

    fn found_potential_from_need(&self, need: u64) -> Option<u64> {
        self.hmap
            .iter()
            .find_map(|(&k, v)| (v.0 == need).then_some(k))
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
pub struct Balance {
    utxo_hmap: HashMap<Utxo, Status>,
}

impl Balance {
    /// Revert change until src with sub
    /// Replay change until dst with add
    pub fn calculation<'b>(&mut self, src: &Vec<&Block>, dst: &Vec<&'b Block>) -> &'b Block {
        src.iter().all(|p| self.sub(p));
        for (index,b) in dst.iter().enumerate(){
            if !self.add(b){
                debug!("{} as incorrect rx utxo",b);
                return dst.get(index-1).expect("first block has no valid transactions in dst ????");
            }
        }
        dst.last().expect("dst empty")
        
    }

    pub fn valid(&self, utxo: &Utxo) -> bool {
        let u = self.utxo_hmap.get(&utxo);
        if let Some(statue) = u {
            statue == &Status::Avaible
        } else {
            false
        }
    }

    /// # Undo add
    /// when we want to drill downside
    /// we need to cancel transaction
    fn sub(&mut self, block: &Block) -> bool {
        let to_remove = block.find_created_utxo();

        let to_append = block.find_used_utxo();

        for utxo in to_append {
            self.utxo_hmap.insert(utxo, Status::Avaible);
        }

        for utxo in to_remove {
            if self.utxo_hmap.remove(&utxo).is_some() {
            } else {
                warn!("sub: la transa qui a été crée n'existe pas dans la hashmap");
                return false;
            }
        }

        true
    }

    /// # Drill up
    /// normal whay to update the Balance with one block
    /// when we need to append a new block we run that
    fn add(&mut self, block: &Block) -> bool {
        //get utxo to append
        let to_append = block.find_created_utxo();

        //get utxo to remove
        let to_remove = block.find_used_utxo();

        // Append transaction
        for utxo in to_append {
            if let std::collections::hash_map::Entry::Vacant(e) = self.utxo_hmap.entry(utxo) {
                e.insert(Status::Avaible);
            } else {
                error!("add: double utxo entry");
                return false;
            }
        }

        // Consume transaction
         for utxo in to_remove {
            if let Some(utxo) = self.utxo_hmap.get_mut(&utxo) {
                *utxo = match utxo {
                    Status::Avaible => Status::Consumed,
                    Status::Consumed => {
                        error!("utxo already consumed");
                        return false;
                    }
                }
            } else {
                error!("add: consume using unknow utxo : {}",utxo);
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
    pub balance: Balance,
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
    pub fn filter_utxo(&self, addr: u64) -> Vec<Utxo> {
        self.balance
            .utxo_hmap
            .iter()
            .filter(|(utxo, statue)| statue == &&Status::Avaible && utxo.onwer == addr)
            .map(|(utxo, _)| utxo)
            .cloned()
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

    pub fn get_block(&self, hash: u64) -> Option<&Block> {
        self.hash_map_block.get(&hash)
    }

    /// # Appand bloc to blockchain struct
    /// block_to_append will be included in the struct this block can be :
    /// - ignored if wrong or to old
    /// - the new top block (block_height +1)
    /// - a potential new top bloc but other block are needed (to complete the chain to block 0)
    /// - complete a chain a for a potential top block
    /// This function return a (Option<Block>, Option<u64>).
    ///
    /// The first Option is conataint the new top block if a new top block is found (not necessary the block_to_append).
    ///
    /// The second Option is containt the hash of a block which are needed to complete a chain.
    ///
    pub fn try_append(&mut self, block_to_append: &Block) -> (Option<Block>, Option<u64>) {
        if self.hash_map_block.contains_key(&block_to_append.block_id) {
            return (None, None); //already prensent
        }

        if !block_to_append.check() {
            //<== full check here
            info!("block is not valid");
            return (None, None);
        }

        //add the block to the HashMap
        self.hash_map_block
            .insert(block_to_append.block_id, block_to_append.clone());

        //get the current block from the db to compare to the new one
        let cur_block = self
            .hash_map_block
            .get(&self.top_block_hash)
            .unwrap()
            .clone();

        //does have same direct ancestor
        if self.check_block_linked(block_to_append, &cur_block)
        //check if block can be linked to the previous one and to the blockchain
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
            /* if cur_block.block_height < block_to_append.block_height
            || (self
                .potentials_top_block
                .is_block_needed(block_to_append.block_id)
                && block_to_append.difficulty
                    >= self
                        .get_height_block(block_to_append.block_height)
                        .unwrap()
                        .difficulty)

            //block to high */
            match self.search_chain(block_to_append) {
                //do we have chain from block 0
                Ok(_) => {
                    //the block can be chained into the initial block
                    let potential_top = match self
                        .potentials_top_block
                        .found_potential_from_need(block_to_append.block_id)
                    {
                        Some(new_top_block) => new_top_block,
                        None => block_to_append.block_id,
                    };

                    //take the two chain to link cur_block and potential_top
                    let (cur_chain, new_chain) =
                        self.get_path_2_block(self.top_block_hash, potential_top);

                    let mut new_chain: Vec<&Block> = new_chain.iter().rev().cloned().collect();

                    match self.check_correct_chain(&new_chain) {
                        //update the chain if there is the end of the new_chain is not valid
                        Ok(_) => {}
                        Err(last_ok) => {
                            if last_ok == None {
                                return (None, None);
                            }
                            new_chain = self
                                .get_path_2_block(self.top_block_hash, last_ok.unwrap())
                                .1
                                .iter()
                                .rev()
                                .cloned()
                                .collect();
                        }
                    }

                    //sale
                    let mut new_balence = self.balance.clone();
                    let last_top_transa_ok =
                        new_balence.calculation(&cur_chain, &new_chain).block_id;
                    //last_top_transa_ok : bloc where is transa is valid to the chain
                    dbg!(last_top_transa_ok!=potential_top);
                    if last_top_transa_ok != potential_top {
                        //update the chain if there is the end of the new_chain is not valid
                        info!(
                            "New branche not complete right, wrong after {}",
                            last_top_transa_ok
                        );
                        new_chain = self
                            .get_path_2_block(self.top_block_hash, last_top_transa_ok)
                            .1
                            .iter()
                            .rev()
                            .cloned()
                            .collect();
                    }

                    if !best_difficulty(&cur_chain, &new_chain) {
                        //test if the new chain is better or not
                        return (None, None);
                    }
                    warn!(
                        "New better branch found, blockchain update {} {:?}",
                        last_top_transa_ok, self.potentials_top_block
                    );

                    self.balance = new_balence;
                    self.top_block_hash = last_top_transa_ok;
                    let dif = if self.get_block(self.top_block_hash).unwrap().block_height
                        % N_BLOCK_DIFFICULTY_CHANGE
                        == 0
                    {
                        self.new_difficutly()
                    } else {
                        self.get_block(self.top_block_hash).unwrap().difficulty
                    };
                    self.difficulty = dif;
                }
                Err(needed) => {
                    //the block can not be chained into the initial block : needed is missing
                    self.potentials_top_block
                        .replace_or_create(block_to_append, needed);
                    return (None, Some(needed));
                }
            }

            //drop the search cache
        }

        //tricky -> if better branch has a top block < cur_top block, can be ignored (but if it found fast it is ok).
        self.potentials_top_block
            .erease_old(self.get_block(self.top_block_hash).unwrap().block_height);

        return (Some(self.last_block()), None);
    }

    fn check_block_linked(&self, block_to_append: &Block, parent: &Block) -> bool {

        

        
        

        self.check_parent(block_to_append, parent)
            && !self.potentials_top_block.is_block_needed(block_to_append.block_id)     //not needed by a higher block in a queue 
            && block_to_append.transactions.iter().all(|t| t.check(&self.balance))
            && block_to_append.difficulty == self.difficulty
    }

    //Check if this two block can be linked (child.parent_id = parent.block_id & time  )
    fn check_parent(&self, child: &Block, parent: &Block) -> bool {
        child.parent_hash == parent.block_id
            && child.block_height == parent.block_height + 1
             // these block is needed from a higher block 
            && child.timestamp > parent.timestamp
    }
    /// Return two chains which are the link between last_top and new_top.
    /// These two chain have a common block at the end
    ///
    ///      chain1       chain2
    ///
    ///                  new_top
    ///                     |
    ///     last_top        b
    ///         |           |
    ///    common_block--------b
    fn get_path_2_block(&self, last_top: u64, new_top: u64) -> (Vec<&Block>, Vec<&Block>) {
        let mut vec1: Vec<&Block> = vec![];
        let mut vec2: Vec<&Block> = vec![];

        let mut last = self.get_block(last_top).unwrap();
        let mut new = self.get_block(new_top).expect(&format!("{}", new_top));

        while last.block_height != new.block_height {
            if last.block_height < new.block_height {
                vec2.push(new);
                new = self.get_block(new.parent_hash).unwrap();
            } else {
                vec1.push(last);
                last = self.get_block(last.parent_hash).unwrap();
            }
        }

        while new.block_id != last.block_id {
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
        Ok(vec)
    }

    pub fn get_chain(&self) -> Vec<&Block> {
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
        vec
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
        if height % N_BLOCK_DIFFICULTY_CHANGE == 0 {
            let chain = self.get_chain();
            if chain.len() >= N_BLOCK_DIFFICULTY_CHANGE as usize {
                let time_between = top_block.timestamp - chain[99].timestamp;
                let mut rate_time = (TIME_N_BLOCK as f64) / (time_between.as_secs() as f64);
                debug!(
                    "Rate time {} blocks {}",
                    N_BLOCK_DIFFICULTY_CHANGE, rate_time
                );
                if rate_time < 0.90 || rate_time > 0.110 {
                    /* let new_dif = if rate_time >= 1.10 {
                        self.difficulty / 2
                    } else {
                        self.difficulty * 2
                    }; */
                    if rate_time == f64::INFINITY {
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

    fn check_correct_chain(&self, new_chain: &Vec<&Block>) -> Result<(), Option<u64>> {
        let mut last_ok = None;
        for (index, b) in new_chain.iter().enumerate() {
            if index == 0 {
                continue;
            }
            if !self.check_parent(&b, new_chain[index - 1]) {
                info!("two node can not be linked {} {}", b, new_chain[index - 1]);
                return Err(last_ok);
            }
            if new_chain[index - 1].block_height % N_BLOCK_DIFFICULTY_CHANGE == 0
                && new_chain[index - 1].block_height != 0
            {
                if get_difficulty(
                    self.get_n_block_from(N_BLOCK_DIFFICULTY_CHANGE, new_chain[index - 1])
                        .unwrap(),
                )
                .unwrap()
                    != b.difficulty
                {
                    info!("Not the correct addaptative difficulty {} ", b);
                    return Err(last_ok);
                }
            } else if new_chain[index - 1].difficulty != b.difficulty
                && new_chain[index - 1].block_height != 0
            {
                info!("Not the same difficulty {} {}", b, new_chain[index - 1]);
                return Err(last_ok);
            }
            last_ok = Some(new_chain[index].block_id);
        }

        Ok(())
    }

    fn get_n_block_from<'a>(&'a self, mut n: u64, mut b: &'a Block) -> Option<Vec<&Block>> {
        let mut vec = vec![];
        n -= 1;
        vec.push(b);
        while n != 0 {
            let hash = b.parent_hash;
            b = self.get_block(hash)?;
            vec.push(b);
            n -= 1;
        }

        Some(vec)
    }
}

fn get_difficulty(chunk: Vec<&Block>) -> Option<u64> {
    if chunk.len() != N_BLOCK_DIFFICULTY_CHANGE as usize {
        return None;
    }

    let time_between = chunk[0].timestamp - chunk[99].timestamp;
    let mut rate_time = (TIME_N_BLOCK as f64) / (time_between.as_secs() as f64);

    if rate_time == f64::INFINITY {
        rate_time = 1000.0;
    }

    Some((chunk[0].difficulty as f64 / rate_time) as u64)
}

fn best_difficulty(chain1: &Vec<&Block>, chain2: &Vec<&Block>) -> bool {
    let sum_dif1: u128 = chain1
        .iter()
        .map(|&b| (u64::MAX - b.difficulty) as u128)
        .sum();
    let sum_dif2: u128 = chain2
        .iter()
        .map(|&b| (u64::MAX - b.difficulty) as u128)
        .sum();
    sum_dif1 < sum_dif2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_chain::{self, block::Profile, transaction::Transaction};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_potential_top_block() {
        let mut pot = PotentialsTopBlock::new();
        let mut b1 = Block::default();
        let mut b2 = Block::default();
        let mut b3 = Block::default();
        let mut b4 = Block::default();
        let mut b5 = Block::default();
        b1.block_height = 1;
        b1.block_id = 1;
        b2.block_height = 2;
        b2.block_id = 2;
        b2.parent_hash = 1;
        b3.block_height = 3;
        b3.block_id = 3;
        b3.parent_hash = 2;
        b4.block_height = 4;
        b4.parent_hash = 3;
        b4.block_id = 4;
        b5.block_height = 5;
        b5.block_id = 5;
        b5.parent_hash = 4;

        pot.replace_or_create(&b5, b5.parent_hash);
        pot.replace_or_create(&b4, b4.parent_hash);
        pot.replace_or_create(&b3, b3.parent_hash);

        assert!(pot.is_block_needed(b2.block_id));
        assert_eq!(
            pot.found_potential_from_need(b2.block_id).unwrap(),
            b5.block_id
        );
    }

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
            .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        assert!(block.check());
        assert_eq!(block, blockchain.try_append(&block).0.unwrap());
    }

    #[test]
    fn add_block_unchainned() {
        let mut blockchain = Blockchain::new();
        let b1 = Block::default()
            .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        let b2 = b1
            .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();

        let (new, need) = blockchain.try_append(&b2);

        assert_eq!(new, None);
        assert_eq!(need.unwrap(), b1.block_id);

        let (new, need) = blockchain.try_append(&b1);
        let new = new.unwrap();
        assert_eq!(new, b2);
        assert_eq!(need, None);
    }

    #[test]
    fn try_append_2_branchs() {
        let mut parrallele_best_branch: Vec<Block> = vec![Block::new()];
        let mut cur_branch: Vec<Block> = vec![Block::new()];
        for _ in 0..3 {
            parrallele_best_branch.push(
                parrallele_best_branch
                    .last()
                    .unwrap()
                    .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
                    .unwrap(),
            );
        }

        for _ in 0..1 {
            cur_branch.push(
                cur_branch
                    .last()
                    .unwrap()
                    .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
                    .unwrap(),
            );
        }

        let mut block_chain = Blockchain::new();

        let (nw, need) = block_chain.try_append(&parrallele_best_branch[3]);
        assert_eq!(nw, None);
        assert_eq!(need, Some(parrallele_best_branch[2].block_id));
        assert_ne!(
            block_chain
                .potentials_top_block
                .hmap
                .get(&parrallele_best_branch[3].block_id),
            None
        );

        let (nw, need) = block_chain.try_append(&cur_branch[1]);
        assert_eq!(need, None);
        assert_eq!(nw.unwrap(), cur_branch[1]);
        assert_ne!(
            block_chain
                .potentials_top_block
                .hmap
                .get(&parrallele_best_branch[3].block_id),
            None
        );

        let (nw, need) = block_chain.try_append(&parrallele_best_branch[2]);
        assert_eq!(nw, None);
        assert_eq!(need, Some(parrallele_best_branch[1].block_id));
        assert_ne!(
            block_chain
                .potentials_top_block
                .hmap
                .get(&parrallele_best_branch[3].block_id),
            None
        );

        let (nw, need) = block_chain.try_append(&parrallele_best_branch[1]);
        assert_eq!(nw.unwrap(), parrallele_best_branch[3]);
        assert_eq!(need, None);
        assert_eq!(
            block_chain
                .potentials_top_block
                .hmap
                .get(&parrallele_best_branch[3].block_id),
            None
        );
    }

    #[test]
    fn remove_old_potential_top() {
        for _ in 1..4 {
            let mut blockchain = Blockchain::new();

            let b0 = Block::default();
            let b1: Block = b0
                .clone()
                .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
                .unwrap();
            let b1_bis: Block = b0
                .clone()
                .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
                .unwrap();
            let b2 = b1
                .clone()
                .find_next_block(
                    
                    vec![Default::default()],
                    Profile::INFINIT,
                    FIRST_DIFFICULTY,
                )
                .unwrap();
            let b2_bis = b1_bis
                .clone()
                .find_next_block(
                    
                    vec![Default::default()],
                    Profile::INFINIT,
                    FIRST_DIFFICULTY,
                )
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
            .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        blockchain.try_append(&block);
        assert_eq!(blockchain.get_chain(), vec![&block, &Block::new()]);
    }

    /*#[test]
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
            .utxo_hmap
            .iter()
            .for_each(|f| println!("{}==>{:?}", f.0, f.1));

        let ret = instance.calculation(&vec_r, &vec);
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
    } */
}
