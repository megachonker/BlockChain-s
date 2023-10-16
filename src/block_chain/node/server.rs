use std::{
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};

use tracing::{debug, info, warn};
use tracing_subscriber::field::debug;

use crate::block_chain::{
    block::{self, mine, Block},
    blockchain::Blockchain,
    // shared::Shared,
    node::network::{Network, Packet, TypeBlock},
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


#[derive(Debug,PartialEq, Eq)]

pub enum NewBlock {
    Mined(Block),
    Network(Block),
}

#[derive(Debug,PartialEq, Eq)]
pub enum Event {
    NewBlock(NewBlock),
    HashReq((i128, SocketAddr)),
    Transaction(Transaction),
    ClientEvent,    //event of client : e.g ask all utxo of a client 
}

pub struct Server {
    name: String,
    network: Network, // blockchaine
    //miner
    id: u64,
    blockchain: Blockchain,
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
            blockchain: Blockchain::new(),
        }
    }
    pub fn start(mut self) {
        info!(
            "Server started {} facke id {} -> {:?}",
            &self.name,
            get_fake_id(&self.name),
            self.network
        );

        // need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let event_channel = mpsc::channel::<Event>();

        //transaction
        thread::spawn(move || {
            /*
            Mutex of
            when receive a transaction
             */
        });

        self.network.clone().start(event_channel.0.clone());

        self.server_runtime(self.id, event_channel);
    }

    /// Routing event and adding block and transaction
    fn server_runtime(&mut self, finder: u64, event_channels: (Sender<Event>, Receiver<Event>)) {
        info!("Runtime server start");

        let actual_top_block = Arc::new(Mutex::new(self.blockchain.last_block()));
        let actual_top_block_cpy = actual_top_block.clone();

        thread::Builder::new()
            .name("Miner".to_string())
            .spawn(move || {
                info!("start Miner");
                mine(finder, &actual_top_block_cpy, event_channels.0);
            })
            .unwrap();

        loop {
            // debug!("main loop");
            //Routing Event
            match event_channels.1.recv().unwrap() {
                Event::HashReq((hash, dest)) => {
                    debug!("Recieved Hash resquest");
                    // remplace -1 par un enum top block
                    if hash == -1{
                        self.network
                            .send_packet(&Packet::Block(TypeBlock::Block(self.blockchain.last_block())), &dest)
                    }
                    //ça partira ducoup
                    else if hash.is_negative(){
                        warn!("Reciv negative hash != -1 : {}",hash);
                    }
                    //ça sera le enum hash
                    else if let Some(block) = self.blockchain.get_block(hash as u64) {
                        self.network
                            .send_packet(&Packet::Block(TypeBlock::Block(block.clone())), &dest)
                    }else {
                        warn!("hash not found in database :{}",hash);
                    }
                }
                Event::NewBlock(new_block) => {
                    let new_block = match new_block {
                        NewBlock::Mined(b) => {
                            self.network
                                .broadcast(Packet::Block(TypeBlock::Block(b.clone())));
                            debug!("Broadcast mined block");
                            b
                        }
                        NewBlock::Network(b) => b,
                    };
                    debug!("New block h:{}",new_block.block_height);
                    let (new_top_block, block_need) = self.blockchain.try_append(&new_block);

                    /// when blockain accept new block
                    if let Some(top_block) = new_top_block {
                        //inform transaction runner that a new block was accepted a
                        //ned to check if parent are same
                        //need to resync db
                        let mut lock_actual_top_block = actual_top_block.lock().unwrap();
                        *lock_actual_top_block = top_block.clone();

                        self.network
                                .broadcast(Packet::Block(TypeBlock::Block(top_block.clone())));

                        // debug!("Salut");
                    }

                    if let Some(needed_block) = block_need {
                        self.network
                            .broadcast(Packet::Block(TypeBlock::Hash(needed_block as i128)));
                    }
                }
                Event::Transaction(_) => todo!(),
                Event::ClientEvent => todo!(),
            }
        }
    }
}
