use std::{
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};

use tracing::{debug, info, warn};

use crate::block_chain::{
    block::{mine, Block},
    blockchain::Blockchain,
    node::network::{Network, Packet, TypeBlock, TypeTransa},
    transaction::Transaction,
};
use crate::friendly_name::*;

use super::network::ClientPackect;

#[derive(Debug, PartialEq, Eq)]

pub enum NewBlock {
    Mined(Block),
    Network(Block),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
    ReqUtxo(u64),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    NewBlock(NewBlock),
    HashReq((i128, SocketAddr)),
    Transaction(Transaction),
    ClientEvent(ClientEvent, SocketAddr), //event of client : e.g ask all utxo of a client
}

//impl Event

pub struct Server {
    name: String,
    network: Network, // blockchaine
    //miner
    id: u64,
    blockchain: Blockchain,
}

#[derive(Debug)]
pub struct MinerStuff {
    pub cur_block: Block,
    pub transa: Vec<Transaction>,
    pub difficulty: u64,
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


        self.network.clone().start(event_channel.0.clone());

        self.server_runtime(self.id, event_channel);
    }

    /// Routing event and adding block and transaction
    fn server_runtime(&mut self, finder: u64, event_channels: (Sender<Event>, Receiver<Event>)) {
        info!("Runtime server start");

        let miner_stuff = Arc::new(Mutex::new(MinerStuff {
            cur_block: self.blockchain.last_block(),
            transa: vec![],
            difficulty: self.blockchain.difficulty,
        }));
        for _ in 1..2 {
            let miner_stuff_cpy = miner_stuff.clone();
            let event_cpy = event_channels.0.clone();
            thread::Builder::new()
                .name("Miner {}".to_string())
                .spawn(move || {
                    info!("start Miner");
                    mine(finder, &miner_stuff_cpy, event_cpy);
                })
                .unwrap();
        }

        loop {
            //Routing Event
            match event_channels.1.recv().unwrap() {
                Event::HashReq((hash, dest)) => {
                    info!("Recieved Hash resquest");
                    // remplace -1 par un enum top block
                    if hash == -1 {
                        self.network.send_packet(
                            &Packet::Block(TypeBlock::Block(self.blockchain.last_block())),
                            &dest,
                        )
                    }
                    //ça partira ducoup
                    else if hash.is_negative() {
                        warn!("Reciv negative hash != -1 : {}", hash);
                    }
                    //ça sera le enum hash
                    else if let Some(block) = self.blockchain.get_block(hash as u64) {
                        self.network
                            .send_packet(&Packet::Block(TypeBlock::Block(block.clone())), &dest)
                    } else {
                        warn!("hash not found in database :{}", hash);
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
                    debug!("recv new block h:{}", new_block.block_height);
                    let (new_top_block, block_need) = self.blockchain.try_append(&new_block);

                    // when blockain accept new block
                    if let Some(top_block) = new_top_block {
                        //inform transaction runner that a new block was accepted a
                        //ned to check if parent are same
                        //need to resync db

                        let new_difficulty = self.blockchain.new_difficutly();

                        let mut lock_miner_stuff = miner_stuff.lock().unwrap();
                        (*lock_miner_stuff).cur_block = top_block.clone();
                        (*lock_miner_stuff).transa = vec![]; //for the moment reset transa not taken     //maybe check transa not accpted and already available
                        (*lock_miner_stuff).difficulty = new_difficulty; //for the moment reset transa not taken     //maybe check transa not accpted and already available

                        drop(lock_miner_stuff);
                        println!("New Top Block : {}", top_block);

                        self.network
                            .broadcast(Packet::Block(TypeBlock::Block(top_block.clone())));
                    }

                    if let Some(needed_block) = block_need {
                        self.network
                            .broadcast(Packet::Block(TypeBlock::Hash(needed_block as i128)));
                        println!("Ask for {}", needed_block);
                    }
                }
                Event::Transaction(transa) => {
                    //check if is valid
                    let mut minner_stuff_lock = miner_stuff.lock().unwrap();

                    if self.blockchain.transa_is_valid(&transa, &minner_stuff_lock) {
                        minner_stuff_lock.transa.push(transa.clone());
                        self.network
                            .broadcast(Packet::Transaction(TypeTransa::Push(transa)));
                    }
                }
                Event::ClientEvent(event, addr_client) => match event {
                    ClientEvent::ReqUtxo(id_client) => self.network.send_packet(
                        &Packet::Client(ClientPackect::RespUtxo(
                            self.blockchain.filter_utxo(id_client), //need to be parralizesd
                        )),
                        &addr_client,
                    ),
                },
            }
        }
    }
}
