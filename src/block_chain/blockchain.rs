use super::{block::Block, transaction::RxUtxo};

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
        self.blocks.iter().map(|block| {
            let block_utxo_vec:Vec<RxUtxo> = block
                .transactions
                .iter()
                .filter(|transa| transa.target_pubkey == addr)
                .flat_map(|transa| transa.get_utxos(block.block_id))
                .collect();
            block_utxo_vec
        }).flatten().collect()
    }
}
