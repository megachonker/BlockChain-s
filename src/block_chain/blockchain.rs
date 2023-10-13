use core::fmt;
use std::collections::{HashMap, HashSet};

use tracing::{info, warn};

use super::{block::Block, transaction::Utxo};

#[derive(Default)]
struct PotentialsTopBlock {
    hmap: HashMap<u64, (u64, u64)>, //k : potentail top block,  v: (needed,height_of_k)
}

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
            (new_needed_block, last_needed_block.block_id),
        ); //create
    }

    fn found_potential_from_need(&self, need: u64) -> Option<u64> {
        for (k, v) in &self.hmap {
            if v.0 == need {
                return Some(*k);
            }
        }
        return None;
    }

    fn erease_old(&mut self, height_top_block: u64) {
        for (k, v) in self.hmap.clone() {
            if v.1 <= height_top_block {
                self.hmap.remove(&k);
            }
        }
    }

    fn needed(&self, block: u64) -> bool {
        for (_, v) in &self.hmap {
            if v.0 == block {
                return true;
            }
        }
        return false;
    }
}

/// Keep track of transaction and utxo
#[derive(Default)]
struct Balance {
    utxo: HashSet<Utxo>,
}

impl Balance {
    /// Revert change until src with sub
    /// Replay change until dst with add
    pub fn calculation<'a, 'b>(&mut self, src: Vec<&'a Block>, dst: Vec<&'b Block>) -> &'b Block {
        src.iter().for_each(|p| self.sub(p));
        dst.iter()
            .find(|p| !self.add(p))
            .unwrap_or(dst.last().unwrap())
    }

    /// # Undo add
    /// when we want to drill downside
    /// we need to cancel transaction
    fn sub(&mut self, block: &Block) {
        //get utxo to append
        let to_remove = block.find_new_utxo();

        //get utxo to remove
        let to_append = block.find_used_utxo();

        if !to_append.iter().all(|t| self.utxo.insert(t.clone())) {
            warn!("sub: adding new transa double entry")
        }

        if !to_remove.iter().all(|t| self.utxo.remove(t)) {
            warn!("sub: removing transa double delet => Using unknow transaction!")
        }
    }

    /// # Drill up
    /// normal whay to update the Balance with one block
    /// when we need to append a new block we run that
    fn add(&mut self, block: &Block) -> bool {
        //get utxo to append
        let to_append = block.find_new_utxo();

        //need to check if it aleready used !


        //get utxo to remove
        let to_remove = block.find_used_utxo();

        //need to check validity in hashmap


        if !to_append.iter().all(|t| self.utxo.insert(t.clone())) {
            panic!("add: adding new transa double entry")
        }

        if !to_remove.iter().all(|t| self.utxo.remove(t)) {
            info!("add: removing transa double delet => Using unknow transaction!");
            return false;
        }
        return true;
    }
}

pub struct Blockchain {
    hash_map_block: HashMap<u64, Block>,
    top_block_hash: u64,
    potentials_top_block: PotentialsTopBlock, // block need to finish the chain)
    balance: Balance,
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Block actuel: {}", self.top_block_hash).unwrap();
        let block = self
            .search_chain(self.hash_map_block.get(&self.top_block_hash).unwrap())
            .unwrap().into_iter().map(|b| self.get_block(b).unwrap());

        for b in block {
            writeln!(f, "{}", b).unwrap();
        }
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
            .flat_map(|block| block.get_utxos(addr))
            .filter(|utxo| self.balance.utxo.contains(utxo))
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
        }
    }

    pub fn get_block<'a>(&'a self, hash: u64) -> Option<&'a Block> {
        self.hash_map_block.get(&hash)
    }

    fn get_needed_block(self) -> Vec<u64> {
        self.potentials_top_block.get_needed_block()
    }

    //retourd de fonction Imbuvable
    //option pour les 2 ? 
    //qui fait cquoi ?
    pub fn append(&mut self, block: &Block) -> (Option<Block>, Option<u64>) {
        if self.hash_map_block.contains_key(&block.block_id) {
            warn!("block already exist");
            return (None, None); //already prensent
        }

        if !block.check() {
            warn!("block is not valid");
            return (None, None);
        }

        //add the block to the DB
        self.hash_map_block.insert(block.block_id, block.clone());

        //get the current block from the db
        let cur_block = self.hash_map_block.get(&self.top_block_hash).unwrap();

        // the block is superior than my actual progress ?
        if block.block_height > cur_block.block_height {
            //does have same direct ancestor
            if block.parent_hash == cur_block.block_id
                && block.block_height == cur_block.block_height + 1
                && !self.potentials_top_block.needed(block.block_id)
            {
                //basic case
                self.top_block_hash = block.block_id;

                //inform the balance that the block is accepted
                self.balance.add(block);

            } else {
                //block to high
                match self.search_chain(block) {
                    Ok(_) => {
                        //the block can be chained into the initial block
                        let new_top_b = match self
                            .potentials_top_block
                            .found_potential_from_need(block.block_id)
                        {
                            Some(new_top_block) => new_top_block,
                            None => block.block_id,
                        };

                        //chack transa and udpate balence
                        let two_chain = self.get_path_2_block(self.top_block_hash, new_top_b);
                        //let (new_balence, last_top_ok) = balence.try_branche(two_chain);

                        let last_top_ok = new_top_b; //for the moment supr when balence.try_brance implmented

                        if (last_top_ok == new_top_b) {
                            //all it is ok
                            info!("New branche better branches founds, blockchain update");
                            //sel.balence = new_balence
                            self.top_block_hash = last_top_ok;
                        } else if self.last_block().block_height
                            < self.get_block(last_top_ok).unwrap().block_height
                        {
                            info!(
                                "New branche not complete right, wrong after {}",
                                last_top_ok
                            );
                            //also ok maybe
                            //sel.balence = new_balence
                            self.top_block_hash = last_top_ok;

                            //need maybe to earse wrong block which transa is not good with the chain (last_top_ok + 1 +2 ...)
                        } else {
                            info!("Branch is not wrong ");
                        }
                    }
                    Err(needed) => {
                        //the block can not be chained into the initial block : needed is missing
                        self.potentials_top_block.replace_or_create(&block, needed);
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

    fn get_path_2_block(&self, last_top: u64, new_top: u64) -> (Vec<Block>, Vec<Block>) {
        let mut vec1: Vec<Block> = vec![];
        let mut vec2: Vec<Block> = vec![];

        let mut last = self.get_block(last_top).unwrap();
        let mut new = self.get_block(new_top).unwrap();

        while last.block_height < new.block_height {
            println!("Ici");
            vec2.push(new.clone());
            new = self.get_block(new.parent_hash).unwrap();
        }

        while new.block_id != last.block_id {
            println!("La");

            vec1.push(last.clone());
            vec2.push(new.clone());
            new = self.get_block(new.parent_hash).unwrap();
            last = self.get_block(last.parent_hash).unwrap();
        }

        vec1.push(last.clone());
        vec2.push(new.clone());

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
}

#[cfg(test)]
mod tests {

    use crate::block_chain::transaction::Transaction;

    use super::*;

    #[test]
    fn create_blockchain() {
        let block_chain = Blockchain::new();

        assert_eq!(block_chain.last_block(), Block::new());
    }

    #[test]
    fn append_wrong_blockchain() {
        let mut block_chain = Blockchain::new();

        let (cur_block, _) = block_chain.append(&Block {
            //not a valid block
            block_id: 7,
            block_height: 1,
            parent_hash: 7,
            transactions: vec![],
            finder: 7,
            answer: 7,
            quote: String::from(""),
        });
        assert_eq!(cur_block, None);
    }

    #[test]
    fn append_blockchain_second_block() {
        let mut blockchain = Blockchain::new();

        let block = Block {
            //hard code
            block_height: 1,
            block_id: 38250827465,
            parent_hash: 0,
            transactions: vec![],
            answer: 3675872114024089965,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        assert_eq!(block, blockchain.append(&block).0.unwrap());
    }

    #[test]
    fn add_block_unchainned() {
        let mut blockchain = Blockchain::new();

        let b2 = Block {
            //hard code
            block_height: 2,
            block_id: 38293290087,
            parent_hash: 8958567695,
            transactions: vec![],
            answer: 3322205353230188497,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        let (new, need) = blockchain.append(&b2);

        assert_eq!(new, None);
        assert_eq!(need.unwrap(), 8958567695);

        let b1 = Block {
            //hard code
            block_height: 1,
            block_id: 8958567695,
            parent_hash: 0,
            transactions: vec![],
            answer: 7478944047245117081,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        let (new, need) = blockchain.append(&b1);

        assert_eq!(new.unwrap(), b2);
        assert_eq!(need, None);
    }

    #[test]
    fn remove_old_potential_top() {
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
            block_id: 32479786738,
            parent_hash: 29090761102,
            transactions: vec![],
            answer: 16060077928867923892,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        let (_, _) = blockchain.append(&b2_bis);
        if blockchain.potentials_top_block.hmap.get(&b2_bis.block_id) == None {
            //present here
            assert!(false);
        }

        let (_, _) = blockchain.append(&b1);
        let (_, _) = blockchain.append(&b2);

        if blockchain.potentials_top_block.hmap.get(&b2_bis.block_id) != None {
            //erease here
            assert!(false);
        }
    }

    #[test]
    fn get_chain() {
        let mut blockchain = Blockchain::new();

        let block = Block {
            //hard code
            block_height: 1,
            block_id: 38250827465,
            parent_hash: 0,
            transactions: vec![],
            answer: 3675872114024089965,
            finder: 17904917467964170301,
            quote: String::from("bi"),
        };

        blockchain.append(&block);

        assert_eq!(blockchain.get_chain(), vec![&block, &Block::new()]);
    }

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

        blockchain.append(&b1);
        blockchain.append(&b2_bis);
        blockchain.append(&b3);
        blockchain.append(&b2);

        let res = blockchain.get_path_2_block(b2_bis.block_id, b3.block_id);

        let must = (vec![b2_bis, b1.clone()], vec![b3, b2, b1]);

        assert_eq!(res, must);
    }

    #[test]
    fn transaction_simple() {
        // let mut blockchain = Blockchain::new();
        // let block = Block::new();
        // let transaction : Transaction::new

        // block.find_next_block(621, transactions)
        // blockchain.append(block)
    }
}
