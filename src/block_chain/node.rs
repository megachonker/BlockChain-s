
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Barrier};

// use crate::


use client::Client;
use server::Server;



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
}