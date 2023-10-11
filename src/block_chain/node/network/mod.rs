use std::{
    default,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
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

use super::server::{Event, NewBlock};

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
    Hash(i128),
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
    fn transaction(transa: TypeTransa, net_transa_tx: &Sender<Event>) {
        match transa {
            TypeTransa::Ans(utxos) => { /* array of all utxo append */ }
            TypeTransa::Push(transaction) => {
                if !transaction.check() {
                    return;
                } else {
                    net_transa_tx.send(Event::Transaction(transaction)).unwrap();
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
        let mut ownpeers = self.peers.lock().unwrap();
        if !ownpeers.contains(&source){
            ownpeers.push(source);
        }
        drop(ownpeers);

        if peers.is_empty() {
            debug!("Net: receive demande de peers");
            self.send_packet(&Packet::Peer(self.peers.lock().unwrap().clone()), &source);
        }
        //reception reponse
        else {
            println!("Net: receive peers");
            self.peers.lock().unwrap().append(&mut peers.clone());
        }

        //on add aussi le remote dans la liste
    }

    /// Ping Pong but update timestamp
    fn keepalive(&self, sender: SocketAddr) {
        // do here the timestamp things
        // todo!();
        self.send_packet(&Packet::Keepalive, &sender);
    }

    fn block(
        &mut self,
        typeblock: TypeBlock,
        sender: SocketAddr,
        network_server_tx: &Sender<Event>,
    ) {
        match typeblock {
            TypeBlock::Block(block) => {
                // debug!("Block get:{}", block);
                network_server_tx
                    .send(Event::NewBlock(NewBlock::Network(block)))
                    .unwrap();
            }
            TypeBlock::Hash(number) => {    
                network_server_tx
                    .send(Event::HashReq((number,sender)))
                    .unwrap();
            }
        }
    }

    ////// END USED BY ROUTER

    /// continusly ask for new peers
    /// peer on router eliminate usless
    /// need to create elaborated mecanisme of pasive block
/*     fn peers_manager(&self) {
        if self.peers.lock().unwrap().len() < 100 {
            self.broadcast(Packet::Peer(vec![]))
        }
    } */

    /// return the a blockaine
    /// catch from another peer or created
    pub fn start(
        self,
        // mined_block_rx: Receiver<Block>,
        event_tx : Sender<Event>, 
        // server_network_rx: Receiver<RequestServer>,
    ) {
        info!("network start");

        
        let mut self_cpy = self.clone();
        thread::Builder::new()
            .name("Net-Router".to_string())
            .spawn(move || {
                debug!("Net-Router");
                loop {

                    let (message, sender) = Self::recv_packet(&self_cpy.binding.try_clone().unwrap());
                    // let mut locked = forthread.lock().unwrap(); /////////BLOCKED
                    match message {
                        Packet::Transaction(transa) => Network::transaction(transa, &event_tx),
                        Packet::Peer(peers) => self_cpy.peers(peers, sender),
                        Packet::Keepalive => self_cpy.keepalive(sender),
                        Packet::Block(typeblock) => {
                            self_cpy.block(typeblock, sender, &event_tx)
                        }
                    }
                }
            })
            .unwrap();

            if self.bootstrap != SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6021) //ask for last block 
            {
                self.send_packet(&Packet::Peer(vec![]), &self.bootstrap);          //to register and get peers
                self.send_packet(&Packet::Block(TypeBlock::Hash(-1)), &self.bootstrap);
            }
    }


    /// Constructor of network
    pub fn new(bootstrap: IpAddr, binding: IpAddr) -> Self {
        let binding = UdpSocket::bind(SocketAddr::new(binding, 6021)).unwrap();
        let bootstrap = SocketAddr::new(bootstrap, 6021);
        Self {
            bootstrap,
            binding,
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    /// # gen Client Server Network runer
    fn new_pair() -> (Network, Network) {
        let client_ip = Some(IpAddr::V4(Ipv4Addr::new(127, 1, 0, 2))).unwrap();
        let server_ip = Some(IpAddr::V4(Ipv4Addr::new(127, 1, 0, 1))).unwrap();

        let client_bootstrap = Some(IpAddr::V4(Ipv4Addr::new(127, 1, 0, 1))).unwrap();
        let server_bootstrap = Some(IpAddr::V4(Ipv4Addr::new(0, 1, 0, 0))).unwrap();

        let client = Network::new(client_bootstrap, client_ip);
        let server = Network::new(server_bootstrap, server_ip);
        (client, server)
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

#[cfg(test)]
mod tests {
    use crate::block_chain::node::network::Network;

    #[test]
    fn create_blockchain() {
        let (client, server) = Network::new_pair();

        std::thread::spawn(||{
            // server.start(net_transa_tx, network_server_tx);
        });

        assert!(true)
    }
}
