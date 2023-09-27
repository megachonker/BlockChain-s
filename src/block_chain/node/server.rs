use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};


use crate::block_chain::{
    block::{hash, mine, Block, Transaction},
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

        // network after starting need to return blockchaine!
        let (net_block_tx, net_block_rx) = mpsc::channel();

        //need to link new transaction block to create block
        let (mined_block_tx, mined_block_rx) = mpsc::channel();

        //need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let (net_transaction_tx, net_transaction_rx) = mpsc::channel(); //RwLock

        //get the whole blochaine

        // thread::Builder::new().name("Network".to_string()).spawn(move ||{
        let blockaine = self.network.start(mined_block_rx, net_block_tx, net_transaction_tx);
        // }).unwrap();

        println!("blockaine recus{:?}",blockaine);

        Self::mining(self.id, mined_block_tx, net_block_rx, net_transaction_rx).await;
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
        mined_block_tx: Sender<Block>, //return finder
        net_block_rx: Receiver<Block>,
        net_transaction_rx: Receiver<Vec<Transaction>>, //Rwlock
    ) {
        let actual_block = Arc::new(Mutex::new(Block::default()));

        let transaction = Arc::new(vec![]); // net_transaction_rx.recv().unwrap();
        let is_stoped = Arc::new(AtomicBool::new(false));

        let is_stoped_cpy = is_stoped.clone();
        let actual_block_thread = actual_block.clone();
        thread::Builder::new().name("Miner-Controler".to_string()).spawn(move || loop {
            //update
            let tmp = net_block_rx.recv().unwrap();
            let mut actual_block_thread = actual_block_thread.lock().unwrap();
            if tmp.block_height > actual_block_thread.block_height {
                //stop
                *actual_block_thread = tmp;
                is_stoped_cpy.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }).unwrap();

        //gen block
        loop {
            let is_stoped = is_stoped.clone();
            let transactionis_stoped = transaction.clone();
            let actual_block_thread = actual_block.clone();
            let handle = thread::Builder::new().name("Miner".to_string()).spawn(move || {
                let actual_block_thread = actual_block_thread.lock().unwrap();

                let mut new_block = Block {
                    block_height: actual_block_thread.block_height + 1,
                    block_id: 0,
                    parent_hash: actual_block_thread.block_id,
                    transactions: transactionis_stoped.to_vec(), //put befort because the proof of work are link to transaction
                    nonce: 0,
                    miner_hash: finder, //j'aime pas
                    quote: String::from("quote"),
                };
                drop(actual_block_thread);

                ////// on peut multithread comme un gros sale!
                if let Some(nonce) = mine(&new_block, &is_stoped) {
                    new_block.nonce = nonce;
                    new_block.block_id = hash(&new_block);
                    return Some(new_block);
                }
                return None;
            }).unwrap();
            if let Some(mined_block) = handle.join().unwrap() {
                let mut locked_actual_block = actual_block.lock().unwrap();
                *locked_actual_block = mined_block;
                println!("{}",locked_actual_block);
                mined_block_tx.send(locked_actual_block.clone()).unwrap();
            }
        }
    }
}
// fn update_peer_share(shared: &mut MutexGuard<Vec<SocketAddr>>, peer: Vec<SocketAddr>) {
//     //marche pas sans function ??
//     **shared = peer;
// }
