use std::{
    sync::mpsc::{self, Receiver, Sender},
    sync::{ Arc, Mutex}, thread,
};

use tracing::info;

use crate::block_chain::{
    block::{mine, Block, self},
    blockchain::Blockchain,
    // shared::Shared,
    node::network::Network, transaction::Transaction,
};
use crate::friendly_name::*;


pub enum BlockFrom{
    Mined(Block),
    Network(Block),
}


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
        let (block_tx, block_rx) = mpsc::channel();

        //need to link new transaction block to create block
        let (mined_block_tx, mined_block_rx) = mpsc::channel();

        //need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let (net_transaction_tx, net_transaction_rx) = mpsc::channel(); //RwLock

        //get the whole blochaine

        // thread::Builder::new().name("Network".to_string()).spawn(move ||{
        let blockaine = self
            .network
            .start(mined_block_rx, &block_tx, net_transaction_tx);
        // }).unwrap();

        println!("blockaine recus{:?}", blockaine);


        // net_block_tx.send(Block::default()).unwrap();
        Self::server_runtime(self.id, block_tx, block_rx);
    }


    fn server_runtime(
        //doit contenire le runetime
        finder: u64,
        block_tx: Sender<BlockFrom>,
        block_rx: Receiver<BlockFrom>, // net_transaction_rx: Receiver<Vec<Transaction>>, //Rwlock
    ) {
        info!("Runtime server start");

        let (mut blockchain, first_block) = Blockchain::new();

        let actual_top_block = Arc::new(Mutex::new(first_block));
        let actual_top_block_cpy = actual_top_block.clone();

        thread::Builder::new()
            .name("Miner".to_string())
            .spawn(move || {
                info!("start Miner");
                mine(finder, &actual_top_block_cpy, block_tx);
            })
            .unwrap();

        loop {
            let new_block = match block_rx.recv().unwrap(){
                BlockFrom::Mined(block) => {
                    //network send
                    block
            }
                BlockFrom::Network(block) => block,
            };
            let (new_top_block, block_need) = blockchain.append(&new_block);
            
            if let Some(top_block) = new_top_block {
                let mut lock_actual_top_block = actual_top_block.lock().unwrap();
                *lock_actual_top_block = top_block;
            }

            if let Some(needed_block) = block_need{
                // network.ask(needed_block);
            }

            // }
        }
    }
}
