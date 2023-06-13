mod block_chain {
    pub mod block;
    pub mod kademlia;
    pub mod node;
    pub mod shared;
}

use block_chain::node::{ Node};
use block_chain::shared;

use block_chain::node::p2p_simulate;

use block_chain::kademlia::Simulate;
use lib_block::{hash, Block, Transaction};
use rand::{seq::SliceRandom, thread_rng, Rng};

use clap::{arg, ArgAction, ArgMatches, Command};



fn parse_args() -> ArgMatches {
    Command::new("NIC")
        .version("1.0")
        .author("Thompson")
        .about("A great Block Chain")
        .arg(
            arg!(-p --ip <IP> "Your IP:port for bind the socket")
                .required(false)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-r --receive <num> "The id of the receiver ")
                .required(false)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-s --sender  <num> "Your Id")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-m --mode  <MODE> "Wich mode (send, mine) ")
                .required(false)
                .action(ArgAction::Set)
                .default_value("mine"),
        )
        .arg(
            arg!(-g --gate <IP> "The IP:port of the entry point")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-c --count <count> "The value amount for the  transaction")
                .required(false)
                .action(ArgAction::Set)
                .default_value("0"),
        )
        .get_matches()
}

fn main() {
    let matches = parse_args();
    Node::start(matches);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kamelia() {
        let simu: Simulate = Simulate::init(255, 5);
        simu.start();
        simu.whait();
        assert!(simu.duplicate());
    }
}
