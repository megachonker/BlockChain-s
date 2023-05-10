use BlockChain::Block;



const HASH_MAX:u64 = 3453209151749857438/100000;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
pub fn new_block(last_block:Block,new_transa:Vec<Transaction>)->Block{
    let last_answer = last_block.answer;
    println!("INF TO\t\t{}",HASH_MAX);
    let mut hasher: DefaultHasher = DefaultHasher::new();
    let mut number =0;
    while true {
        number+=1;
        let to_hash = number+last_answer;
        to_hash.hash(&mut hasher);
        let answer = hasher.finish();
        println!("Tested: {}=>\t{}",number,answer);
        if answer < HASH_MAX{
            return Block {
                id : answer,
                last_block : last_block.id,
                transactions : new_transa,
                answer : number,
            };
        }
    }
    return  0;        
}
pub struct Transaction{
    initiating : u64,
    receiver : u64,
    amount : u32,
}


pub struct Block {
    id : u64,
    last_block : u64,
    transactions : Vec<Transaction>,
    answer : u64,
}


fn main() {
    //find value above block
    print!("answer {}",new_block(50,)); 
}
