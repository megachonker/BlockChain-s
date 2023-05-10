mod block_chain;

use block_chain::*;

fn main() {
    //find value above block
    let mut last_block = Block::new(0, 0, vec![],0,1);

    loop{
        println!("Search for the current block : {:?}",last_block);
        let answer = mine(&last_block);
        println!("Answer : {}",answer);
        last_block = last_block.new_block(vec![], answer,1);
        println!("The block is {} ",last_block.verification());
    }
}
