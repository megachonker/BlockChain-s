use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    sync::{
        mpsc::{Receiver, Sender},
        Barrier, Arc, Mutex,
    },
    thread, fmt::Display,
};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::block_chain::block::{self, Block, Transaction};

#[derive(Debug)]
pub struct Network {
    pub bootstrap: SocketAddr,
    binding: UdpSocket,
    ///////
    peers: Vec<SocketAddr>,
    stack_transa: Vec<Transaction>, //is restarted
    /////
    fence_blockaine: Barrier, //wait start by receiving a blockaine
    blockchain: Vec<Block>,   //wroung wrong
}

#[derive(Serialize, Deserialize, Debug)]

enum TypeBlock {
    Hash(u64),
    Block(Block),
    Getall(Vec<Block>),
}
#[derive(Serialize, Deserialize, Debug)]

pub enum Packet {
    Keepalive,
    Transaction(Transaction),
    Block(TypeBlock),
    Peer(Vec<SocketAddr>),
}

// whole network function inside it
// send packet with action do scan block ect get peers
impl Network {
    ///// USED BY ROUTER

    /// append transaction when enought transa send it to miner to create a new block
    fn append_transa(&mut self, transa: Transaction, net_transa_tx: &Sender<Vec<Transaction>>) {
        self.stack_transa.push(transa.clone());
        println!(
            "App Transa size{}: Received {:?}",
            self.stack_transa.len(),
            transa
        );

        //quand on a 5 transactions => A changer 
        if self.stack_transa.len() == 5 {
            net_transa_tx.send(self.stack_transa.clone()).unwrap();
            self.stack_transa.clear()
        }
    }

    /// If received empty reply with all peers
    /// If received append new peers to the list
    /// need to manage duplication
    fn peers(&mut self, peers: Vec<SocketAddr>, source: SocketAddr) {
        //reception demande
        if peers.is_empty() {
            println!("Net: receive demande de peers");
            self.send_packet(&Packet::Peer(self.peers.clone()), &source);
        }
        //reception reponse
        else {
            println!("Net: receive peers");
            self.peers.append(&mut peers.clone());
        }

        //on add aussi le remote dans la liste
        self.peers.append(&mut vec![source.clone()]);
    }

    /// Ping Pong but update timestamp
    fn keepalive(&self, sender: SocketAddr) {
        // do here the timestamp things
        // todo!();
        self.send_packet(&Packet::Keepalive, &sender);
    }

    fn block(&mut self, typeblock: TypeBlock, sender: SocketAddr, sss: &Sender<Block>) {
        match typeblock {
            TypeBlock::Block(block) => {
                sss.send(block.clone()).unwrap();
                self.blockchain.push(block); //<=== block not ordered need real blockaine
            }
            TypeBlock::Hash(number) => {
                self.send_packet(
                    &Packet::Block(TypeBlock::Block(
                        self.blockchain
                            .iter()
                            .filter(|block| block.block_id == number)
                            .next()
                            .unwrap()
                            .clone(),
                    )),
                    &sender,
                );
            }
            TypeBlock::Getall(lists) => {
                if lists.is_empty() {
                    self.send_packet(
                        &Packet::Block(TypeBlock::Getall(self.blockchain.clone())),
                        &sender,
                    );
                } else {
                    self.blockchain = lists;
                    //barierre unlock
                    self.fence_blockaine.wait();
                }
            }
        }
    }

    ////// END USED BY ROUTER

    /// route tout les événemnet
    fn router(&mut self, net_block_tx: Sender<Block>, net_transa_tx: Sender<Vec<Transaction>>) {
        // thread::spawn( move ||
        loop {
            let (message, sender) = Self::recv_packet(&self.binding);
            match message {
                Packet::Transaction(transa) => self.append_transa(transa, &net_transa_tx),
                Packet::Peer(peers) => self.peers(peers, sender),
                Packet::Keepalive => self.keepalive(sender),
                Packet::Block(typeblock) => self.block(typeblock, sender, &net_block_tx),
            }
        }
        // );
    }

    /// continusly ask for new peers
    /// peer on router eliminate usless
    /// need to create elaborated mecanisme of pasive block
    fn peers_manager(&self) {
        if self.peers.len() < 100 {
            self.broadcast(Packet::Peer(vec![]))
        }
    }

    /// return the a blockaine
    /// catch from another peer or created
    pub fn start(
        self,
        mined_block_rx: Receiver<Block>,
        net_block_tx: Sender<Block>,
        net_transa_tx: Sender<Vec<Transaction>>,
    ) -> Vec<Block> {
        let mut block_chaine: Vec<Block> = vec![];

        let shared_net = Arc::new(Mutex::new(self));
        
        // when miner have a block send it to all people
        let forthread = shared_net.clone();
        thread::spawn(move || {
            loop {
                let mined_block = mined_block_rx.recv().unwrap();
                
                let locked = forthread.lock().unwrap();
                // send to all
                locked.broadcast(Packet::Block(TypeBlock::Block(mined_block)));
            }

        });

        // routing all message
        let forthread = shared_net.clone();
        thread::spawn(move || {
            loop {
                let cum = forthread.lock().unwrap().binding.try_clone().unwrap();
                let (message, sender) = Self::recv_packet(&cum);
                let mut locked = forthread.lock().unwrap();
                println!("RCV from: {:?} {:?}",sender,message);
                match message {
                    Packet::Transaction(transa) => locked.append_transa(transa, &net_transa_tx),
                    Packet::Peer(peers) => locked.peers(peers, sender),
                    Packet::Keepalive => locked.keepalive(sender),
                    Packet::Block(typeblock) => locked.block(typeblock, sender, &net_block_tx),
                }
            }
        });

        let network = shared_net.clone();
        let network  = network.lock().unwrap();

        //if no bootstrap init blockaine
        if network.bootstrap == SocketAddr::from(([0, 0, 0, 0], 6021)) {
            block_chaine.push(Block::new());
        }
        //if not retreive the blockaine
        else {
            //send une demande de peers
            // a voir si on doit faire plusieur cicle ect
            network.send_packet(&Packet::Peer(vec![]), &network.bootstrap);

            //request blockaine
            network.send_packet(&Packet::Block(TypeBlock::Getall(vec![])), &network.bootstrap);
            //on attend que l'on a recus toute la blockaine
            network.fence_blockaine.wait();
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
            blockchain: vec![], //// WRONG NOT NEED THAT
            fence_blockaine: Barrier::new(2),
            stack_transa: vec![],
            bootstrap,
            binding,
            peers: vec![],
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
            .iter()
            .filter(|&&x| x != self.get_socket())
            .for_each(|dest| self.send_packet(&packet, dest));
    }

    /// awsome
    pub fn recv_packet(selff:&UdpSocket) -> (Packet, SocketAddr) {
        //faudrait éliminer les vecteur dans les structure pour avoir une taille prédictible
        let mut buf = [0u8; 256]; //pourquoi 256 ???
        let (_, sender) = selff.recv_from(&mut buf).expect("Error recv block");
        let des = deserialize(&mut buf).expect("Can not deserilize block");
        (des, sender)
    }

    // need that to filter block recived
    // hashmap would be better
    //
    // pub fn get_chain(&self) -> Option<Vec<Block>> {
    //     //for the moment just take the gate maybe after take a radam peer for each loop

    //     let last_block = self.get_block(-1, self.bootstrap); //take the last block
    //     let mut chain = vec![];
    //     let (height, nonce) = last_block.get_height_nonce();
    //     if height == 0 && nonce != 0 {
    //         return None;
    //     }
    //     if height > 0 {
    //         for i in 0..height {
    //             let block = self.get_block(i as i64, self.bootstrap);
    //             let (h, n) = block.get_height_nonce();
    //             if (h != i) || (h == 0 && n != 0) {
    //                 return None;
    //             }
    //             chain.push(block);
    //         }
    //     }
    //     chain.push(last_block);
    //     println!("get the chain : {:?}", chain);

    //     Some(chain)
    // }
}
