use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use futures::{pin_mut, select, FutureExt};

use crate::block_chain::{
    block::{Block, Transaction},
    node::network::{Network, Packet},
    // shared::Shared,
};
use crate::friendly_name::*;

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
        let id = get_fake_id(&self.name);

        // need to be a enum of reason of future stop eg
        // shared between server an network
        let sould_stop = &Arc::new(Mutex::new(false)); //il y a un type pour ça

        // network after starting need to return blockchaine!
        let (net_block_tx, net_block_rx) = mpsc::channel();

        //need to link new transaction block to create block
        let (mined_block_tx, mined_block_rx) = mpsc::channel();

        //need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let (net_transaction_tx, net_transaction_rx) = mpsc::channel();

        //get the whole blochaine
        let block_chaine = self
            .network
            .start(mined_block_rx, net_block_tx, net_transaction_tx);

        let first_block = block_chaine.last().unwrap().clone();

        /// start le process de mining
        self.mining(
            first_block,
            mined_block_tx,
            net_block_rx,
            net_transaction_rx,
            sould_stop,
        );
    }

    // fn verif_transa(&self, share: Shared, transa: Transaction) {
    //     //verification /////A FAIRE\\\\\\\\\\\\
    //     let mut val = share.transaction.lock().unwrap();
    //     (*val).push(transa);
    // }

    //need to be fixed ??
    async fn mining(
        &self,
        mut block: Block,
        mined_block_tx: Sender<Block>,
        net_block_rx: Receiver<Block>,
        net_transaction_rx: Receiver<Vec<Transaction>>,
        sould_stop: &Arc<Mutex<bool>>,
    ) {
        loop {
            //mining_task
            let mining_task = async {
                block.generate_block(
                    self.id,
                    net_transaction_rx.recv().unwrap(),
                    " quote",
                    &Arc::new(Mutex::new(false)),
                ).unwrap()
            }
            .fuse();

            //receving_task
            let receving_task = async { net_block_rx.recv().unwrap() }.fuse();

            //async stuff
            pin_mut!(mining_task, receving_task);

            //take the best of two
            block = select! {
                //we find a block and we send it
                block = mining_task => {
                    print!("Mined\t");
                    mined_block_tx.send(block.clone()).unwrap();
                    block
                },
                //we receive a "valide" block befort finding a block
                block = receving_task => {print!("Received\t"); block},
            };
            //printing debug
            println!(" => {:?} ", block);
        }
    }
}
// fn update_peer_share(shared: &mut MutexGuard<Vec<SocketAddr>>, peer: Vec<SocketAddr>) {
//     //marche pas sans function ??
//     **shared = peer;
// }
