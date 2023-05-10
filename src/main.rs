use BlockChain::Block;



const HASH_MAX:u64 = 3453209151749857438/100000;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
pub fn new_block(last_answer:u64)->u64{
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
            return  number;
        }
    }
    return  0;        
}

fn main() {
    //find value above block
    print!("answer {}",new_block(50)); 
}
