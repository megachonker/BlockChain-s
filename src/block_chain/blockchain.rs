use std::{collections::HashMap, process::exit};

use tracing::warn;

use super::block::{self, Block};

pub struct Blockchain {
    hash_map_block: HashMap<u64, Block>,
    last_block_hash: u64,
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
            },
            first_block,
        )
    }

    pub fn append(&mut self, block: &Block) -> Block {
        if !block.check() {
            warn!("block is not valid ");
        } else {
            self.hash_map_block
                .insert(block.block_id, block.clone());

            let cur_block = self.hash_map_block.get(&self.last_block_hash).unwrap();

            if block.parent_hash == cur_block.block_id {
                //basic case
                self.last_block_hash = block.block_id;
            } else {
                println!("{} \n--> {}",block, self.last_block_hash);
            }
        }

        self.last_block()
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

        let cur_block = block_chain.append(&Block {
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
}
