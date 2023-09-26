use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex, MutexGuard, atomic::AtomicBool},
    thread,
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
    pub async fn start(self) {
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
        let (net_transaction_tx, net_transaction_rx) = mpsc::channel(); //RwLock

        //get the whole blochaine
        let block_chaine = self
            .network
            .start(mined_block_rx, net_block_tx, net_transaction_tx);

        let first_block = block_chaine.last().unwrap().clone();

        Self::mining(
            self.id,
            first_block,
            mined_block_tx,
            net_block_rx,
            net_transaction_rx,
            sould_stop,
        )
        .await;
    }

    // fn verif_transa(&self, share: Shared, transa: Transaction) {
    //     //verification /////A FAIRE\\\\\\\\\\\\
    //     let mut val = share.transaction.lock().unwrap();
    //     (*val).push(transa);
    // }

    //need to be fixed ??

    //sould take at imput
    //

    async fn mining(
        //doit contenire le runetime
        finder: u64,
        mut block: Block,
        mined_block_tx: Sender<Block>, //return finder
        net_block_rx: Receiver<Block>,
        net_transaction_rx: Receiver<Vec<Transaction>>, //Rwlock
        sould_stop: &Arc<Mutex<bool>>,
    ) {
        let actual_block:Block ;

        let transaction = Arc::new(vec![]); // net_transaction_rx.recv().unwrap();
        let is_stoped = AtomicBool::new(false);
        thread::spawn(move || loop {
            //update
            if net_block_rx.recv().unwrap().block_height > actual_block.block_height{
                is_stoped.
            }
            //stop
        });

        //gen block
        loop {
            //clone
            let block_thread = block.clone();
            let transactionis_stoped  = transaction.clone();
            let is_stoped = is_stoped.clone();

            //start
            let handle = thread::spawn(move || {
                let newblock = block_thread
                    .generate_block(
                        finder,
                        // net_transaction_rx.recv().unwrap(),
                        transaction.to_vec(),
                        " quote",
                        is_stoped,
                    )
                    .unwrap();

                return newblock;
            });
            let mined_block = match handle.join() {
                Err(_) => None,
                Ok(block) => Some(block),
            };
        }
    }
}
// fn update_peer_share(shared: &mut MutexGuard<Vec<SocketAddr>>, peer: Vec<SocketAddr>) {
//     //marche pas sans function ??
//     **shared = peer;
// }
