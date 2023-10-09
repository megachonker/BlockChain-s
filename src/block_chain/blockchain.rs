use std::collections::HashMap;

use tracing::warn;

use super::{block::Block, transaction::RxUtxo};

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

#[derive(Default)]
pub struct Blockchain {
    hash_map_block: HashMap<u64, Block>,
    top_block_hash: u64,
    potentials_top_block: PotentialsTopBlock, // block need to finish the chain)
}

impl Blockchain {


    pub fn filter_utxo(&self, addr: u64) -> Vec<RxUtxo> {
        self.get_chain().iter().map(|block| block.get_utxos(addr)).flatten().collect()
    }

    pub fn new() -> (Blockchain, Block) {
        let mut hash_map = HashMap::new();
        let first_block = Block::new();
        let hash_first_block = first_block.block_id;
        hash_map.insert(hash_first_block, first_block.clone());
        (
            Blockchain {
                hash_map_block: hash_map,
                top_block_hash: hash_first_block,
                potentials_top_block: PotentialsTopBlock::new(),
            },
            first_block,
        )
    }

    pub fn get_block<'a>(& 'a self, hash : u64) -> Option<&'a Block>{
        self.hash_map_block.get(&hash)
    }

    fn get_needed_block(self) -> Vec<u64> {
        self.potentials_top_block.get_needed_block()
    }

    pub fn append(&mut self, block: &Block) -> (Option<Block>, Option<u64>) {
        if self.hash_map_block.contains_key(&block.block_id) {
            return (None, None); //already prensent
        }

        if !block.check() {
            warn!("block is not valid ");
            return (None, None);
        }

        self.hash_map_block.insert(block.block_id, block.clone());

        let cur_block = self.hash_map_block.get(&self.top_block_hash).unwrap();
        if block.block_height > cur_block.block_height {
            if block.parent_hash == cur_block.block_id
                && block.block_height == cur_block.block_height + 1
                && !self.potentials_top_block.needed(block.block_id)
            {
                //basic case
                self.top_block_hash = block.block_id;
            } else {
                //block to high
                match self.search_chain(block) {
                    Ok(_) => {
                        //the block can be chained into the initial block
                        match self
                            .potentials_top_block
                            .found_potential_from_need(block.block_id)
                        {
                            Some(new_top_block) => {
                                self.top_block_hash = new_top_block;
                            }
                            None => {
                                self.top_block_hash = block.block_id;
                            }
                        }
                    }
                    Err(needed) => {
                        //the block can not be chained into the initial block : needed is missing
                        self.potentials_top_block.replace_or_create(&block, needed);
                        return (None, Some(needed));
                    }
                }
            }

            self.potentials_top_block.erease_old(self.top_block_hash);

            return (Some(self.last_block()), None);
        }

        (None, None)
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
        return  vec;
    }
}

#[cfg(test)]
mod tests {
    use crate::block_chain;

    use super::*;

    #[test]
    fn create_blockchain() {
        let (block_chain, last) = Blockchain::new();

        assert_eq!(last, Block::new());

        let last_block = block_chain.last_block();

        assert_eq!(last_block, last);
    }

    #[test]
    fn append_wrong_blockchain() {
        let (mut block_chain, _) = Blockchain::new();

        let (cur_block, _) = block_chain.append(&Block {
            //not a valid block
            block_id: 7,
            block_height: 1,
            parent_hash: 7,
            transactions: vec![],
            miner_hash: 7,
            nonce: 7,
            quote: String::from(""),
        });
        assert_eq!(cur_block, None);
    }

    #[test]
    fn append_blockchain_second_block() {
        let (mut blockchain, _) = Blockchain::new();

        let block = Block {
            //hard code
            block_height: 1,
            block_id: 38250827465,
            parent_hash: 0,
            transactions: vec![],
            nonce: 3675872114024089965,
            miner_hash: 17904917467964170301,
            quote: String::from("bi"),
        };

        assert_eq!(block, blockchain.append(&block).0.unwrap());
    }

    #[test]
    fn add_block_unchainned() {
        let (mut blockchain, _) = Blockchain::new();

        let b2 = Block {
            //hard code
            block_height: 2,
            block_id: 38293290087,
            parent_hash: 8958567695,
            transactions: vec![],
            nonce: 3322205353230188497,
            miner_hash: 17904917467964170301,
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
            nonce: 7478944047245117081,
            miner_hash: 17904917467964170301,
            quote: String::from("bi"),
        };

        let (new, need) = blockchain.append(&b1);

        assert_eq!(new.unwrap(), b2);
        assert_eq!(need, None);
    }

    #[test]
    fn remove_old_potential_top() {
        let (mut blockchain, _) = Blockchain::new();

        let b1 = Block {
            block_height: 1,
            block_id: 84739656938,
            parent_hash: 0,
            transactions: vec![],
            nonce: 8308871350387475192,
            miner_hash: 17904917467964170301,
            quote: String::from("bi"),
        };

        let b2 = Block {
            block_height: 2,
            block_id: 32147335136,
            parent_hash: 84739656938,
            transactions: vec![],
            nonce: 9377674440955505,
            miner_hash: 17904917467964170301,
            quote: String::from("bi"),
        };

        let b2_bis = Block {
            block_height: 2,
            block_id: 32479786738,
            parent_hash: 29090761102,
            transactions: vec![],
            nonce: 16060077928867923892,
            miner_hash: 17904917467964170301,
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
    fn get_chain(){
        let (mut blockchain, _) = Blockchain::new();

        let block = Block {
            //hard code
            block_height: 1,
            block_id: 38250827465,
            parent_hash: 0,
            transactions: vec![],
            nonce: 3675872114024089965,
            miner_hash: 17904917467964170301,
            quote: String::from("bi"),
        };

        blockchain.append(&block);

        assert_eq!(blockchain.get_chain(), vec![&block,& Block::new()]);
    }
}
