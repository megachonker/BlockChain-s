use super::{block::Block, transaction::Transaction};

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain { blocks: vec![] }
    }

    pub fn append(&self, block: &Block) -> Block {
        block.clone()
    }

    // need to reimplement
    pub fn iter_transa(&self, pubkey: u64) -> Vec<&Transaction> {
        let iter_transa = self
            .blocks
            .iter()
            .flat_map(|block| block.transactions.iter());

        // qsdfqdsf
        let a = iter_transa.filter(|transa| transa.target_pubkey == pubkey);
        a.collect()
    }
}
