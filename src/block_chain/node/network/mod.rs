use std::net::{IpAddr, SocketAddr, UdpSocket};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::block_chain::block::{Block, Transaction};

use crate::block_chain::node::Packet;

#[derive(Debug)]
pub struct Network {
    pub bootstrap: SocketAddr,
    binding: UdpSocket,
}

// whole network function inside it
// send packet with action do scan block ect get peers
impl Network {
    pub fn new(bootstrap: IpAddr, binding: IpAddr) -> Self {
        let binding = UdpSocket::bind(SocketAddr::new(binding, 6021)).unwrap();
        let bootstrap = SocketAddr::new(bootstrap, 6021);
        Self { bootstrap, binding }
    }

    pub fn get_socket(&self) -> SocketAddr {
        self.binding
            .local_addr()
            .expect("Can not catch the SocketAddr")
    }

    pub fn send_packet(&self, packet: Packet, dest: SocketAddr) -> usize {
        let sera_answer = serialize(&Packet::AnswerKA).expect("Can not serialize AswerKA");
        self.binding
            .send_to(&sera_answer, dest)
            .expect(&format!("Can not send packet {:?}", packet))
    }

    pub fn send_packet_multi(&self, packet: Packet, dests: Vec<SocketAddr>) -> Vec<usize> {
        let mut res = vec![];
        let sera_answer = serialize(&Packet::AnswerKA).expect("Can not serialize AswerKA");
        for d in dests {
            res.push(
                self.binding
                    .send_to(&sera_answer, d)
                    .expect(&format!("Can not send packet {:?}", packet)),
            );
        }
        res
    }

    pub fn recv_packet(&self) -> (Packet, SocketAddr) {
        let mut buf = [0u8; 256];
        let (_, sender) = self.binding.recv_from(&mut buf).expect("Error recv block");
        let des = deserialize(&mut buf).expect("Can not deserilize block");
        (des, sender)
    }

    pub fn bootstrap(&self) -> Vec<SocketAddr> {
        self.send_packet(Packet::Connexion, self.bootstrap);

        self.recive_peers()
    }

    pub fn recive_peers(&self) -> Vec<SocketAddr> {
        //kemelia ???
        let mut buffer = [0u8; 256]; //on veux 255 addres max //<= a cahnger

        let (packet, _) = self.recv_packet();

        loop {
            if let Packet::RepPeers(peer) = packet {
                return peer;
            }

            let (packet, _) = self.recv_packet();
            println!("Wait for peers, recive another things -> ignore");
        }
    }

    pub fn get_chain(&self, peers: &Vec<SocketAddr>) -> Option<Vec<Block>> {
        //for the moment just take the gate maybe after take a radam peer for each loop

        let last_block = self.get_block(-1, self.bootstrap); //take the last block
        let mut chain = vec![];
        let (height, nonce) = last_block.get_height_nonce();
        if height == 0 && nonce != 0 {
            return None;
        }
        if height > 0 {
            for i in 0..height {
                let block = self.get_block(i as i64, self.bootstrap);
                let (h, n) = block.get_height_nonce();
                if (h != i) || (h == 0 && n != 0) {
                    return None;
                }
                chain.push(block);
            }
        }
        chain.push(last_block);
        println!("get the chain : {:?}", chain);

        Some(chain)
    }

    fn get_block(&self, index: i64, peer: SocketAddr) -> Block {
        self.send_packet(Packet::GetBlock(index), peer);
        loop {
            let (packet, sender) = self.recv_packet();
            if sender != peer {
                continue;
            }
            if let Packet::Block(b) = packet {
                return b;
            }
        }
    }
}

impl Clone for Network {
    fn clone(&self) -> Self {
        Self {
            bootstrap: self.bootstrap.clone(),
            binding: self.binding.try_clone().unwrap(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}
