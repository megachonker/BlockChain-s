use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
    time::Duration,
};

use tracing::{info, warn};

use crate::block_chain::{
    block::{self, hash, mine, Block, Transaction},
    blockchain::Blockchain,
    // shared::Shared,
    node::network::{Network, Packet},
};
use crate::friendly_name::*;

use super::network;

pub struct Server {
    name: String,
    network: Network, // blockchaine
    //miner
    id: u64,
}

impl Server {
    pub fn new(network: Network) -> Self {
        let name =
            get_friendly_name(network.get_socket()).expect("generation name from ip imposble");
        let id = get_fake_id(&name);
        Self {
            name,
            network,
            id: id,
        }
    }
    pub fn start(self) {
        println!(
            "Server started {} facke id {} -> {:?}",
            &self.name,
            get_fake_id(&self.name),
            self.network
        );
        info!("server mode");
        let id = get_fake_id(&self.name);

        // network after starting need to return blockchaine!
        let (net_block_tx, net_block_rx) = mpsc::channel();

        //need to link new transaction block to create block
        let (mined_block_tx, mined_block_rx) = mpsc::channel();

        //need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let (net_transaction_tx, net_transaction_rx) = mpsc::channel(); //RwLock

        //get the whole blochaine

        // thread::Builder::new().name("Network".to_string()).spawn(move ||{
        let blockaine = self
            .network
            .start(mined_block_rx, &net_block_tx, net_transaction_tx);
        // }).unwrap();

        println!("blockaine recus{:?}", blockaine);
        // net_block_tx.send(Block::default()).unwrap();
        Self::server_runtime(self.id, net_block_tx,net_block_rx);
    }

    // fn verif_transa(&self, share: Shared, transa: Transaction) {
    //     //verification /////A FAIRE\\\\\\\\\\\\
    //     let mut val = share.transaction.lock().unwrap();
    //     (*val).push(transa);
    // }

    //need to be fixed ??

    //sould take at imput
    //

    fn server_runtime(
        //doit contenire le runetime
        finder: u64,
        block_tx: Sender<Block>,
        block_rx: Receiver<Block>, // net_transaction_rx: Receiver<Vec<Transaction>>, //Rwlock
    )  {
        info!("Runtime server start");
        
        let (mut blockchain,first_block) = Blockchain::new();
        
        let actual_block = Arc::new(Mutex::new(first_block));
        let actual_block_cpy = actual_block.clone();

        thread::Builder::new()
            .name("Miner".to_string())
            .spawn(move || {info!("start Miner"); mine(finder,&actual_block_cpy, block_tx); })
            .unwrap();

        loop {
            let new_block = block_rx.recv().unwrap();
            let cur_block = blockchain.append(&new_block);
            println!("Current block : {}",cur_block);
            let mut  lock_actual_block = actual_block.lock().unwrap();

            // if *lock_actual_block != cur_block{
            *lock_actual_block = cur_block;
            // }
            drop(lock_actual_block);
        }
    }
}
