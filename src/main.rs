mod block_chain {
    pub mod interconnect;
}

use block_chain::interconnect::detect_interlock;

use block_chain::interconnect::p2p_simulate;
use lib_block::{hash, Block, Transaction};
use rand::{seq::SliceRandom, thread_rng, Rng};

fn main() {
    // detect_interlock();
    p2p_simulate();
}

fn fakemine(){

    let mut rng = thread_rng();

    //guy's
    let cottelle = hash("uss");
    let neeto = hash("neeto");
    let chonker = hash("chonker"); //pb if two people have the same hash

    let guys = [cottelle, neeto, chonker];

    let transaction = Transaction::new(
        *guys.choose(&mut rng).unwrap(),
        *guys.choose(&mut rng).unwrap(),
        rng.gen::<u32>(),
    );

    let origin_block = Block::new(vec![transaction]);
    if !origin_block.check() {
        println!("The block is false");
    }
    let mut block = origin_block;

    loop {
        println!("Current  {:?} ", block);
        let transaction = Transaction::new(
            *guys.choose(&mut rng).unwrap(),
            *guys.choose(&mut rng).unwrap(),
            rng.gen::<u32>() % 100,
        );
        block = block.new_block(vec![transaction], *guys.choose(&mut rng).unwrap());
        if !block.check() {
            println!("The block is false");
        }
    }
}