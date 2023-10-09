use std::{
    default,
    net::{IpAddr, SocketAddr, UdpSocket},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Barrier, Mutex,
    },
    thread,
};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing::{debug, info, warn};

use crate::block_chain::{
    block::Block,
    blockchain::Blockchain,
    transaction::{RxUtxo, Transaction},
};

use super::server::{RequestNetwork, RequestServer};

#[derive(Debug)]
pub struct Network {
    pub bootstrap: SocketAddr,
    binding: UdpSocket,
    peers: Arc<Mutex<Vec<SocketAddr>>>,
}

impl Clone for Network {
    fn clone(&self) -> Self {
        Self {
            bootstrap: self.bootstrap.clone(),
            binding: self.binding.try_clone().unwrap(),
            peers: self.peers.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]

pub enum TypeBlock {
    Hash(u64),
    Block(Block),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TypeTransa {
    Push(Transaction),
    Req(u64),
    Ans(Vec<RxUtxo>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    Keepalive,
    Transaction(TypeTransa),
    Block(TypeBlock),
    Peer(Vec<SocketAddr>),
}

// whole network function inside it
// send packet with action do scan block ect get peers
impl Network {
    ///// USED BY ROUTER

    /// append transaction when enought transa send it to miner to create a new block
    fn transaction(transa: TypeTransa, net_transa_tx: &Sender<Transaction>) {
        match transa {
            TypeTransa::Ans(utxos) => { /* array of all utxo append */ }
            TypeTransa::Push(transaction) => {
                if !transaction.check() {
                    return;
                } else {
                    net_transa_tx.send(transaction).unwrap();
                }
            }
            TypeTransa::Req(userid) => { /* get all transa and filter by user id*/ }
        }

        //check préliminer avant d'utiliser un prim

        //send la transa au miner
    }

    /// If received empty reply with all peers
    /// If received append new peers to the list
    /// need to manage duplication
    fn peers(&mut self, peers: Vec<SocketAddr>, source: SocketAddr) {
        //reception demande
        if peers.is_empty() {
            println!("Net: receive demande de peers");
            self.send_packet(&Packet::Peer(self.peers.lock().unwrap().clone()), &source);
        }
        //reception reponse
        else {
            println!("Net: receive peers");
            self.peers.lock().unwrap().append(&mut peers.clone());
        }

        //on add aussi le remote dans la liste
        self.peers.lock().unwrap().append(&mut vec![source.clone()]);
    }

    /// Ping Pong but update timestamp
    fn keepalive(&self, sender: SocketAddr) {
        // do here the timestamp things
        // todo!();
        self.send_packet(&Packet::Keepalive, &sender);
    }

    #[instrument]
    fn block(
        &mut self,
        typeblock: TypeBlock,
        sender: SocketAddr,
        network_server_tx: &Sender<RequestNetwork>,
    ) {
        match typeblock {
            TypeBlock::Block(block) => {
                // debug!("Block get:{}", block);
                network_server_tx
                    .send(RequestNetwork::NewBlock(block))
                    .unwrap();
            }
            TypeBlock::Hash(number) => {
                network_server_tx
                    .send(RequestNetwork::SendHash((number, sender)))
                    .unwrap();
                // debug!("Hash get Hash:{}", number);
                /*
                self.send_packet(
                    &Packet::Block(TypeBlock::Block(uwu)=>{}

                        self.blockchain
                            .iter()
                            .filter(|block| block.block_id == number)
                            .next()
                            .unwrap()
                            .clone(),
                            u
                        )),
                        &sender,
                    );
                    */
            }
        }
    }

    ////// END USED BY ROUTER

    /// continusly ask for new peers
    /// peer on router eliminate usless
    /// need to create elaborated mecanisme of pasive block
    fn peers_manager(&self) {
        if self.peers.lock().unwrap().len() < 100 {
            self.broadcast(Packet::Peer(vec![]))
        }
    }

    /// return the a blockaine
    /// catch from another peer or created
    pub fn start(
        self,
        mined_block_rx: Receiver<Block>,
        net_transa_tx: Sender<Transaction>,
        network_server_tx: Sender<RequestNetwork>,
        server_network_rx: Receiver<RequestServer>,
    ) -> Vec<Block> {
        info!("network start");
        let mut block_chaine: Vec<Block> = vec![Block::new()];

        // let shared_net = Arc::new(Mutex::new(self));

        // when miner have a block send it to all people
        // let for_thread = shared_net.clone();
        let self_cpy = self.clone();
        let network_server_tx_clone = network_server_tx.clone();

        thread::Builder::new()
            .name("Net-Block_Sender".to_string())
            .spawn(move || {
                debug!("Net-Block_Sender started");
                loop {
                    let mined_block = mined_block_rx.recv().unwrap();

                    // let locked = for_thread.lock().unwrap();

                    // send to all
                    self_cpy.broadcast(Packet::Block(TypeBlock::Block(mined_block.clone())));
                    network_server_tx_clone.send(RequestNetwork::NewBlock(mined_block)).unwrap();
                    // drop(locked);
                }
            })
            .unwrap();

        let self_cpy = self.clone();

        thread::Builder::new()
            .name("Asker of block at peers".to_string())
            .spawn(move || loop {
                info!("Launch thread asker of blocks");
                match server_network_rx.recv().unwrap() {
                    RequestServer::AnswerHash((block, dest)) => {
                        self_cpy.send_packet(&Packet::Block(TypeBlock::Block(block)), &dest)
                    }
                    RequestServer::AskHash(hash) => {
                        for p in self_cpy.peers.lock().unwrap().clone().into_iter() {
                            self_cpy.send_packet(&Packet::Block(TypeBlock::Hash(hash)), &p);
                        }
                    }
                }
            })
            .unwrap();

        let fence_blockaine = Arc::new(Barrier::new(2));

        // routing all message
        // let forthread = shared_net.clone(); //peut opti en ayan try clone
        let block_ack = fence_blockaine.clone();

        // let network_server_tx_clone = network_server_tx.clone();
        let mut self_cpy = self.clone();
        thread::Builder::new()
            .name("Net-Router".to_string())
            .spawn(move || {
                debug!("Net-Router");
                loop {
                    // let cim = forthread.lock().unwrap();
                    let sick = self_cpy.binding.try_clone().unwrap();
                    // drop(cim);
                    let (message, sender) = Self::recv_packet(&sick);
                    // let mut locked = forthread.lock().unwrap(); /////////BLOCKED
                    match message {
                        Packet::Transaction(transa) => Network::transaction(transa, &net_transa_tx),
                        Packet::Peer(peers) => self_cpy.peers(peers, sender),
                        Packet::Keepalive => self_cpy.keepalive(sender),
                        Packet::Block(typeblock) => {
                            self_cpy.block(typeblock, sender, &network_server_tx)
                        }
                    }
                }
            })
            .unwrap();

        // let network = shared_net.clone();
        // let network = network.lock().unwrap();

        //calquer
        if self.bootstrap != SocketAddr::from(([0, 0, 0, 0], 6021)) {
            //send une demande de peers
            // a voir si on doit faire plusieur cicle ect
            self.send_packet(&Packet::Peer(vec![]), &self.bootstrap);

            //request blockaine
            // let blockchain = Blockchain::default();
            //lunch function to do that

            // drop(network);
            //on attend que l'on a recus toute la blockaine
            fence_blockaine.wait();
            // let network = shared_net.clone();
            // let network = network.lock().unwrap();
            block_chaine = vec![]; //sale§/////////////////////////////
        }
        block_chaine
    }

    // fn check_keep_alive(&self, peer: &mut HashMap<SocketAddr, Duration>, time: Duration) {
    //     let clone = peer.clone();
    //     for (p, t) in clone {
    //         if time - t > Duration::from_secs(240) {
    //             peer.remove(&p);
    //             println!("Remove the peer {}", p);
    //         } else if time - t > Duration::from_secs(60) {
    //             println!("Send a keep alive to {}", p);

    //             self.network.send_packet(Packet::Keepalive, p);
    //         }
    //     }
    // }

    pub fn new(bootstrap: IpAddr, binding: IpAddr) -> Self {
        let binding = UdpSocket::bind(SocketAddr::new(binding, 6021)).unwrap();
        let bootstrap = SocketAddr::new(bootstrap, 6021);
        Self {
            bootstrap,
            binding,
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_socket(&self) -> SocketAddr {
        self.binding
            .local_addr()
            .expect("Can not catch the SocketAddr")
    }

    /// verry cool send a packet
    pub fn send_packet(&self, packet: &Packet, dest: &SocketAddr) {
        let packet_serialized = serialize(&packet).expect("Can not serialize AswerKA");
        self.binding
            .send_to(&packet_serialized, dest)
            .expect(&format!("Can not send packet {:?}", packet));
    }

    /// awsome send a packet broadcast
    pub fn broadcast(&self, packet: Packet) {
        self.peers
            .lock()
            .unwrap()
            .iter()
            .filter(|&&x| x != self.get_socket())
            .for_each(|dest| self.send_packet(&packet, dest)); ///////send socket differant
    }

    /// awsome
    pub fn recv_packet(selff: &UdpSocket) -> (Packet, SocketAddr) {
        //faudrait éliminer les vecteur dans les structure pour avoir une taille prédictible
        let mut buf = [0u8; 256]; //pourquoi 256 ??? <============= BESOIN DETRE choisie
        let (_, sender) = selff.recv_from(&mut buf).expect("Error recv block");
        let des = deserialize(&mut buf).expect("Can not deserilize block");
        (des, sender)
    }
}
