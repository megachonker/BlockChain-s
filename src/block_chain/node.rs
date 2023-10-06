use std::net::{ SocketAddr, UdpSocket};
use std::sync::{Arc, Barrier};

// use crate::
use bincode::serialize;

use client::Client;
use server::Server;

use self::network::Packet;

pub mod client;
pub mod server;
pub mod network;


// on est sur que quand on manipule une node on a que un des 3 mode
pub enum NewNode {
    Srv(Server),
    Cli(Client),
}

// un trait serait mieux ?
// permet de start un node sans connaitre son type au préalable
impl NewNode {
    pub fn start(self) {
        match self {
            Self::Cli(cli) => cli.start(),
            Self::Srv(srv) => srv.start(),
        }
    }
}

/*
to do transa need to have block i think and network
*/
pub struct Node {
    // uname: String,
    id: u64,
    socket: UdpSocket,
    barrier: Arc<Barrier>,
    // voir changement préscrit   --> ?
}

impl Node {
    //new
    pub fn create(id: u64, ip: SocketAddr) -> Node {
        let socket = UdpSocket::bind(ip).expect(&format!("{} couldn't bind to address:", id)); //1
        let barrier = Arc::new(Barrier::new(2));
        Node {
            id,
            socket,
            barrier,
        }
    }

    //important d'avoir une structure pour les transa avec plein de check into algo qui store la structure  --> pour moi pas besoin de check si on envoit c'est les miner qui check
    // pub fn send_transactions(&self, gate: SocketAddr, to: u64, count: u32) {
    //     // let him = Node::create(to);
    //     let transa = Transaction::new(0, to, count);
    //     let transa =
    //         serialize(&Packet::Transaction(transa)).expect("Error serialize transactions ");
    //     self.socket
    //         .send_to(&transa, gate)
    //         .expect("Error send transaction ");
    // }


}