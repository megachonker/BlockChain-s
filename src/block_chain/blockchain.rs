use std::{collections::HashMap, process::exit};

use tracing::warn;

use super::block::{self, Block};

pub struct Blockchain {
    hash_map_block: HashMap<u64, Block>,
    last_block_hash: u64,
    potential_last_block: Option<(u64, u64)>, // (block, block need to finish the chain)
}

impl Blockchain {
    pub fn new() -> (Blockchain, Block) {
        let mut hash_map = HashMap::new();
        let first_block = Block::new();
        let hash_first_block = first_block.block_id;
        hash_map.insert(hash_first_block, first_block.clone());
        (
            Blockchain {
                hash_map_block: hash_map,
                last_block_hash: hash_first_block,
                potential_last_block: None,
            },
            first_block,
        )
    }

    pub fn append(&mut self, block: &Block) -> (Block, Option<u64>) {
        if self.hash_map_block.contains_key(&block.block_id) {
            return (
                self.last_block(),
                if let Some(plb) = self.potential_last_block {
                    Some(plb.1)
                } else {
                    None
                },
            );
        }

        if !block.check() {
            warn!("block is not valid ");
            return (
                self.last_block(),
                if let Some(plb) = self.potential_last_block {
                    Some(plb.1)
                } else {
                    None
                },
            );
        }

        self.hash_map_block.insert(block.block_id, block.clone());

        let cur_block = self.hash_map_block.get(&self.last_block_hash).unwrap();
        if block.block_height > cur_block.block_height {
            if block.parent_hash == cur_block.block_id
                && block.block_height == cur_block.block_height + 1
            {
                //basic case
                self.last_block_hash = block.block_id;
            } else { //block to high
                if self.potential_last_block == None || block.block_height > self.hash_map_block.get(&self.potential_last_block.unwrap().0).unwrap().block_height {

                } 
            }
        }

        (
            self.last_block(),
            if let Some(plb) = self.potential_last_block {
                Some(plb.1)
            } else {
                None
            },
        )
    }

    pub fn last_block(&self) -> Block {
        self.hash_map_block
            .get(&self.last_block_hash)
            .unwrap()
            .clone()
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
    fn append_blockchain() {
        let (mut block_chain, last) = Blockchain::new();

        let (cur_block, _) = block_chain.append(&Block {
            //not a valid block
            block_id: 7,
            block_height: 7,
            parent_hash: 7,
            transactions: vec![],
            miner_hash: 7,
            nonce: 7,
            quote: String::from(""),
        });
        assert_eq!(cur_block, last);
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

        assert_eq!(block, blockchain.append(&block).0);
    }
}
