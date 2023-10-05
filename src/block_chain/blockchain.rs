use super::block::{Block, self};



pub struct Blockchain{

}


impl Blockchain{
    pub fn new() -> Blockchain{
        Blockchain{

        }
    }

    pub fn append(&self, block: &Block) -> Block{
        block.clone()
    }
}