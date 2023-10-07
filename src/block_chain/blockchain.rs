use super::{block::Block, transaction::RxUtxo};

#[derive(Default)]
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

    pub fn filter_utxo(&self, addr: u64) -> Vec<RxUtxo> {
        self.blocks.iter().map(|block| block.get_utxos(addr)).flatten().collect()
    }
}
