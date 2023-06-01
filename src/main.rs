mod block_chain {
    pub mod block;
    pub mod node;
    pub mod shared;
}

use block_chain::node::detect_interlock;
use block_chain::node::{self, Name, Node};
// use core::num::flt2dec::strategy;
use crate::shared::Shared;
use block_chain::shared;
use std::env;

use block_chain::node::p2p_simulate;
use lib_block::{hash, Block, Transaction};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // detect_interlock();
    // p2p_simulate();
    let args: Vec<String> = env::args().collect();
    let name = Name::creat_str(&args[1]);
    let me = Node::create(name);
    let me_clone = me.clone();

    let should_stop = Arc::new(Mutex::new(false));

    let peer = Arc::new(Mutex::new(vec![
        SocketAddr::from(([127, 0, 0, 1], 6021)),
        SocketAddr::from(([127, 0, 0, 2], 6021)),
    ]));

    let (rx, tx) = mpsc::channel();
    let share = Shared::new(peer, should_stop);
    let share_copy = share.clone();

    let thread = thread::spawn(move || {
        me.listen(share_copy,rx);
    });

    let starting_block = Block::new(vec![]);

    me_clone.mine(share, starting_block,tx);
}

fn fakemine() {
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
