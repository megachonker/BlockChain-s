mod block_chain {
    pub mod interconnect;
    pub mod block;
}

use block_chain::interconnect::{self, Node,Name};
use block_chain::interconnect::detect_interlock;
use std::env;

use block_chain::interconnect::p2p_simulate;
use lib_block::{hash, Block, Transaction};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::net::SocketAddr;

fn main() {
    // detect_interlock();
    // p2p_simulate();
    let args: Vec<String> = env::args().collect();
    let name = Name::creat_str(& args[1]);    
    let me = Node::create(name);
    let me_clone =me.clone();

    let (tx,rx) =mpsc::channel();

    let should_stop = Arc::new(Mutex::new(false));
    let should_stop_clone = Arc::clone(&should_stop);

    let participent = vec![
        SocketAddr::from(([127, 0, 0, 1], 6021)),
        SocketAddr::from(([127, 0, 0, 2], 6021)),
    ];



    let thread = thread::spawn(move || {
        me.listen_newblock(tx, should_stop_clone);
    });

    let starting_block = Block::new(vec![]);

    me_clone.mine(participent,rx, should_stop,starting_block);   


    

    
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