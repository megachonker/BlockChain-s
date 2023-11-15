use bincode::{deserialize, serialize};
use dryoc::sign::PublicKey;

use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    net::SocketAddr,
    sync::mpsc::{self, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
};

use tracing::{debug, info, trace, warn};

use crate::block_chain::{
    block::{mine, Block},
    blockchain::Blockchain,
    node::network::{Network, Packet, TypeBlock},
    transaction::Transaction,
};
use crate::{block_chain::acount::Keypair, friendly_name::*};

const path_save_json: &str = "path_save_json.save";
const path_save: &str = "path_save_json.save";

use super::network::ClientPackect;

#[derive(Debug, PartialEq, Eq)]
pub enum NewBlock {
    Mined(Block),
    Network(Block),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
    ReqUtxo(PublicKey),
    ReqSave, //force server to save the blockchain in file (debug)
}

#[derive(Debug, PartialEq)]
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
    keypair: Keypair,
    blockchain: Blockchain,
    number_miner: u16, //number of thread of miner to be spawn //<= use once
                       // path_save_json: String,
                       // path_save: String,
}

#[derive(Debug)]
/// Structure that hold wat needed to mine
/// IMPORTANT this structure is updated and locked
pub struct MinerStuff {
    /// curent block to
    pub cur_block: Block,
    /// all transaction to hash (already checked)
    pub transa: Vec<Transaction>,
    /// actual difficulty
    pub difficulty: u64,
    // pub miner_id: PublicKey,
}

impl Server {
    pub fn new(network: Network, keypair: Keypair, thread: u16) -> Self {
        let name =
            get_friendly_name(network.get_socket()).expect("generation name from ip imposble");

        Self {
            name,
            network,
            keypair,
            blockchain: Blockchain::new(),
            number_miner: thread,
        }
    }
    pub fn start(mut self) -> Result<()> {
        info!(
            "Server started {} facke id {} -> {:?}",
            &self.name,
            get_fake_id(&self.name),
            self.network
        );

        // need to link new stack of transaction because the miner need continue to mine without aprouvale of the network
        let event_channel = mpsc::channel::<Event>();

        self.network.clone().start(event_channel.0.clone())?;

        self.server_runtime(event_channel)
    }

    /// Routing event and adding block and transaction
    fn server_runtime(&mut self, event_channels: (Sender<Event>, Receiver<Event>)) -> Result<()> {
        info!("Runtime server start");

        let miner_stuff = Arc::new(Mutex::new(MinerStuff {
            cur_block: self.blockchain.last_block(),
            transa: Transaction::transform_for_miner(
                vec![],
                self.keypair.clone(),
                1,
                &self.blockchain,
            ),
            difficulty: self.blockchain.difficulty,
            // miner_id:self.miner_pubkey.clone(),
        }));

        // manny whay to do better <== need to me move  in mine !!
        for _ in 0..self.number_miner {
            let miner_stuff_cpy = miner_stuff.clone();
            let event_cpy = event_channels.0.clone();
            thread::Builder::new()
                .name("Miner {}".to_string())
                .spawn(move || {
                    info!("start Miner");
                    mine(&miner_stuff_cpy, event_cpy);
                })
                .unwrap();
        }

        loop {
            //Routing Event
            match event_channels.1.recv().unwrap() {
                Event::HashReq((hash, dest)) => {
                    // remplace -1 par un enum top block
                    if hash == -1 {
                        self.network.send_packet(
                            &Packet::Block(TypeBlock::Block(self.blockchain.last_block())),
                            &dest,
                        )?;
                    }
                    //ça partira ducoup
                    else if hash.is_negative() {
                        warn!("Reciv negative hash != -1 : {}", hash);
                    }
                    //ça sera le enum hash
                    else if let Some(block) = self.blockchain.get_block(hash as u64) {
                        self.network
                            .send_packet(&Packet::Block(TypeBlock::Block(block.clone())), &dest)?;
                    } else {
                        warn!("hash not found in database :{}", hash);
                    }
                }
                Event::NewBlock(new_block) => {
                    let new_block = match new_block {
                        NewBlock::Mined(b) => {
                            debug!("Export {}:{}", b.block_height, b.block_id);
                            self.network
                                .broadcast(Packet::Block(TypeBlock::Block(b.clone())))?;
                            b
                        }
                        NewBlock::Network(b) => {
                            debug!("Import {}:{}", b.block_height, b.block_id);

                            b
                        }
                    };
                    let (new_top_block, block_need) = self.blockchain.try_append(&new_block);

                    // when blockain accept new block
                    if let Some(top_block) = new_top_block {
                        //inform transaction runner that a new block was accepted a
                        //ned to check if parent are same
                        //need to resync db

                        let new_difficulty = self.blockchain.new_difficutly();

                        // update the miner stuff
                        let mut lock_miner_stuff = miner_stuff.lock().unwrap();
                        lock_miner_stuff.cur_block = top_block.clone();

                        lock_miner_stuff.transa = Transaction::transform_for_miner(
                            vec![],
                            self.keypair.clone(),
                            top_block.block_height + 1,
                            &self.blockchain,
                        ); //for the moment reset transa not taken     //maybe check transa not accpted and already available
                        lock_miner_stuff.difficulty = new_difficulty; //for the moment reset transa not taken     //maybe check transa not accpted and already available

                        drop(lock_miner_stuff);
                        println!("New Top Block : {}", top_block);

                        self.network
                            .broadcast(Packet::Block(TypeBlock::Block(top_block.clone())))?;
                    }

                    if let Some(needed_block) = block_need {
                        self.network
                            .broadcast(Packet::Block(TypeBlock::Hash(needed_block as i128)))?;
                        debug!("Req\t[{}] to complete a branch on Broadcast ", needed_block);
                    }
                }
                Event::Transaction(transa) => {
                    let mut minner_stuff_lock = miner_stuff.lock().unwrap();

                    if self.blockchain.transa_is_valid(&transa, &minner_stuff_lock) {
                        minner_stuff_lock.transa.push(transa.clone());
                    }
                }
                Event::ClientEvent(event, addr_client) => match event {
                    ClientEvent::ReqUtxo(id_client) => {
                        let utxos = self.blockchain.filter_utxo(id_client);
                        let nb_utxo = utxos.len();

                        // si le client n'es pas trouve
                        if utxos.is_empty() {
                            self.network.send_packet(
                                &Packet::Client(ClientPackect::RespUtxo((
                                    0,
                                    Default::default(),
                                    Default::default(),
                                ))),
                                &addr_client,
                            )?;
                        }

                        for (index, utxo) in utxos.iter().enumerate() {
                            trace!(
                                "Reply to {} by sending transa {} {}",
                                addr_client,
                                index,
                                utxo
                            );
                            self.network.send_packet(
                                &Packet::Client(ClientPackect::RespUtxo((
                                    nb_utxo - 1 - index,
                                    self.blockchain
                                        .get_utxo_location(utxo)
                                        .context("self.blockchain.get_utxo_location")?,
                                    utxo.clone(),
                                ))),
                                &addr_client,
                            )?;
                        }
                    }

                    ClientEvent::ReqSave => {
                        if !path_save_json.is_empty() {
                            let file_json = File::create(path_save_json).unwrap();
                            let chain = self
                                .blockchain
                                .get_chain()
                                .iter()
                                .map(|&b| b.clone())
                                .collect();
                            save_chain_readable(&chain, file_json);
                        }
                        if !path_save.is_empty() {
                            let file_json = File::create(path_save).unwrap();
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

    let mut buf_u64: [u8; 8] = 0_u64.to_be_bytes();

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
