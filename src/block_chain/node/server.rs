use bincode::{deserialize, serialize};

use std::{
    fs::File,
    io::{Read, Write},
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};

use tracing::{debug, info, warn};

use crate::friendly_name::*;
use crate::{
    block_chain::{
        block::{mine, Block},
        blockchain::Blockchain,
        node::network::{Network, Packet, TypeBlock, TypeTransa},
        transaction::Transaction,
    },
    Cli,
};

use super::network::ClientPackect;

#[derive(Debug, PartialEq, Eq)]

pub enum NewBlock {
    Mined(Block),
    Network(Block),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
    ReqUtxo(u64),
    ReqSave, //force server to save the blockchain in file (debug)
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
    number_miner: u16, //number of thread of miner to be spawn
    path_save_json: String,
    path_save: String,
}

#[derive(Debug)]
pub struct MinerStuff {
    pub cur_block: Block,
    pub transa: Vec<Transaction>,
    pub difficulty: u64,
    pub miner_id : u64,
}

impl Server {
    pub fn new(network: Network, cli: Cli) -> Self {
        let name =
            get_friendly_name(network.get_socket()).expect("generation name from ip imposble");
        let id = get_fake_id(&name);
        Self {
            name,
            network,
            id,
            blockchain: Blockchain::new(),
            number_miner: cli.number_miner,
            path_save_json: cli.save_json,
            path_save: cli.save,
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
            transa: Transaction::transform_for_miner(vec![],finder),
            difficulty: self.blockchain.difficulty,
            miner_id : finder,
        }));
        for _ in 0..self.number_miner {
            let miner_stuff_cpy = miner_stuff.clone();
            let event_cpy = event_channels.0.clone();
            thread::Builder::new()
                .name("Miner {}".to_string())
                .spawn(move || {
                    info!("start Miner");
                    mine( &miner_stuff_cpy, event_cpy);
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
                        lock_miner_stuff.cur_block = top_block.clone();
                        lock_miner_stuff.transa = Transaction::transform_for_miner(vec![], lock_miner_stuff.miner_id); //for the moment reset transa not taken     //maybe check transa not accpted and already available
                        lock_miner_stuff.difficulty = new_difficulty; //for the moment reset transa not taken     //maybe check transa not accpted and already available

                        drop(lock_miner_stuff);
                        println!("New Top Block : {}", top_block);

                        self.network
                            .broadcast(Packet::Block(TypeBlock::Block(top_block.clone())));
                    }

                    if let Some(needed_block) = block_need {
                        self.network
                            .broadcast(Packet::Block(TypeBlock::Hash(needed_block as i128)));
                        info!("{} is needed to complete another branch", needed_block);
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
                    ClientEvent::ReqUtxo(id_client) => 
                    self.network.send_packet(
                        &Packet::Client(ClientPackect::RespUtxo(
                            self.blockchain.filter_utxo(id_client), //need to be parralizesd
                        )),
                        &addr_client,
                    ),
                    ClientEvent::ReqSave => {
                        if self.path_save_json != "" {
                            let file_json = File::create(&self.path_save_json).unwrap();
                            let chain = self
                                .blockchain
                                .get_chain()
                                .iter()
                                .map(|&b| b.clone())
                                .collect();
                            save_chain_readable(&chain, file_json);
                        }
                        if self.path_save != "" {
                            let file_json = File::create(&self.path_save).unwrap();
                            let chain = self
                                .blockchain
                                .get_chain()
                                .iter()
                                .map(|&b| b.clone())
                                .collect();
                            save_chain(&chain, file_json);
                        }
                    }
                },
            }
        }
    }
}

fn save_chain_readable(chain: &Vec<Block>, mut file: File) {
    file.write_all(b"[").unwrap();
    for b in chain {
        let ser_b = serde_json::to_string(&b).unwrap();
        file.write_all(ser_b.as_bytes()).unwrap();
        file.write_all(b",").unwrap();
    }
    file.write_all(b"{}]").unwrap();

}

fn save_chain(chain: &Vec<Block>, mut file: File) {
    file.write_all(&(chain.len() as u64).to_be_bytes()).unwrap();
    for b in chain {
        let ser_b = serialize(&b).unwrap();
        let len: u64 = ser_b.len() as u64;
        file.write_all(&len.to_be_bytes()).unwrap();
        file.write_all(&ser_b).unwrap();
    }
}

fn load_chain(mut file: File) -> Vec<Block> {
    let mut vec: Vec<Block> = vec![];

    let mut buf_u64: [u8; 8] = (0 as u64).to_be_bytes();

    file.read_exact(&mut buf_u64).unwrap();

    let mut number = u64::from_be_bytes(buf_u64);

    while number != 0 {
        file.read_exact(&mut buf_u64).unwrap();
        let size = u64::from_be_bytes(buf_u64);
        let mut buf = vec![0; size as usize];
        file.read_exact(&mut buf).unwrap();
        let b = deserialize::<Block>(&buf).unwrap();
        vec.push(b);
        number -= 1;
    }

    vec
}
