mod block_chain;

use block_chain::*;

// const HASH_MAX: u64 = 3453209151749857438 / 100000;

// pub fn new_block(last_block: Block, new_transa: Vec<Transaction>) -> Block {
//     let last_answer = last_block.answer;
//     println!("INF TO\t\t{}", HASH_MAX);
//     let mut hasher: DefaultHasher = DefaultHasher::new();
//     let mut number = 0;
//     while true {
//         number += 1;
//         let to_hash = number + last_answer;
//         to_hash.hash(&mut hasher);
//         let answer = hasher.finish();
//         println!("Tested: {}=>\t{}", number, answer);
//         if answer < HASH_MAX {
//             return Block {
//                 id: answer,
//                 last_block: last_block.id,
//                 transactions: new_transa,
//                 answer: number,
//             };
//         }
//     }
//     return 0;
// }
// use BlockChain::Block;

fn main() {
    //find value above block
    let mut last_block = Block::new(0, 0, vec![]);

    loop{
        let answer = mine(&last_block);
        println!("Answer : {}",answer);
        last_block = last_block.new_block(vec![], answer);
    }
}
