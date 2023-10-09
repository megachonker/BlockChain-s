use std::{
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};

use tracing::info;

use crate::block_chain::{
    block::{self, mine, Block},
    blockchain::Blockchain,
    // shared::Shared,
    node::network::Network,
    transaction::Transaction,
};
use crate::friendly_name::*;

pub enum RequestNetwork {
    SendHash((u64, SocketAddr)),
    NewBlock(Block),
}

pub enum RequestServer {
    AnswerHash((Block, SocketAddr)),
    AskHash(u64),
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
        let (mined_block_tx, mined_block_rx) = mpsc::channel();

        //need to link new transaction block to create block
        // let (new_block_tx, new_block_rx) = mpsc::channel();

        //need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let (net_transaction_tx, net_transaction_rx) = mpsc::channel(); //RwLock

        let (server_network_tx, server_network_rx) = mpsc::channel();
        let (network_server_tx, network_server_rx) = mpsc::channel();

        //get the whole blochaine
        let blockaine = self.network.start(
            mined_block_rx,
            net_transaction_tx,
            network_server_tx,
            server_network_rx,
        );

        println!("blockaine recus{:?}", blockaine);

        Self::server_runtime(
            self.id,
            mined_block_tx,
            net_transaction_rx,
            network_server_rx,
            server_network_tx,
        );
    }

    fn server_runtime(
        //doit contenire le runetime
        finder: u64,
        mined_block_tx: Sender<Block>,
        net_transaction: Receiver<Transaction>,
        network_server_rx: Receiver<RequestNetwork>,
        server_network_tx: Sender<RequestServer>,
    ) {
        info!("Runtime server start");

        let (mut blockchain, first_block) = Blockchain::new();

        let actual_top_block = Arc::new(Mutex::new(first_block));
        let actual_top_block_cpy = actual_top_block.clone();

        thread::Builder::new()
            .name("Miner".to_string())
            .spawn(move || {
                info!("start Miner");
                mine(finder, &actual_top_block_cpy, mined_block_tx);
            })
            .unwrap();

        loop {
            // let new_block: Block = match new_block_rx.recv().unwrap(){
            //     BlockFrom::Mined(block) => {
            //         //network send
            //         block
            // }
            //     BlockFrom::Network(block) => block,
            // };
            match network_server_rx.recv().unwrap() {
                RequestNetwork::SendHash((hash, dest)) => {
                    if let Some(block) = blockchain.get_block(hash) {
                        server_network_tx.send(RequestServer::AnswerHash((block.clone(), dest))).unwrap();
                    }
                }
                RequestNetwork::NewBlock(new_block) => {
                    println!("New block");
                    let (new_top_block, block_need) = blockchain.append(&new_block);

                    if let Some(top_block) = new_top_block {
                        let mut lock_actual_top_block = actual_top_block.lock().unwrap();
                        *lock_actual_top_block = top_block;
                    }

                    if let Some(needed_block) = block_need {
                        server_network_tx.send(RequestServer::AskHash(needed_block)).unwrap();
                    }
                }
            }
        }
    }
}
