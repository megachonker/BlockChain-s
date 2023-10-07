use crate::block_chain::block::Block;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{Arc, Barrier, Mutex, MutexGuard};

// use crate::
use bincode::{deserialize, serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use client::Client;
use server::Server;

use self::network::Packet;

use super::block::Transaction;
// use super::shared::Shared;

pub mod client;
pub mod server;
pub mod network;

/////////important/////////////
// on peut faire l'abre des dépendance au niveaux du systeme de fichier aussi
/*
idée de changement de structure

enum Node{          --> ca peut être une très bonne idée de separer client server a voire si ca ce fait bien
    // struct emule
    //     node::server
    //     node::client
    struct Server
        miner
        Network
            kamelia
        blockaine
            block
    struct client
        User
            Kripto --> Y'en a besoin pour tout le monde je pense
        transaction
}
*/


/// A vector of peer
// struct Vec<SocketAddr>(Vec<SocketAddr>);



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

//permet de stoquer ce qui est lier au network
#[derive(Debug)]


pub struct NewTransaction {
    destination: u64,
    secret: String,
    ammount: f64,
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

    // //comment ça ? j'ai jamais fait des impl de clone --> on peut faire le trait Clone plus clean en effet
    // pub fn clone(&self) -> Node {
    //     let barrier = Arc::new(Barrier::new(2));

    //     Node {
    //         id: self.id,
    //         socket: self.socket.try_clone().unwrap(),
    //         barrier: barrier,
    //     }
    // }


    //important d'avoir une structure pour les transa avec plein de check into algo qui store la structure  --> pour moi pas besoin de check si on envoit c'est les miner qui check
    pub fn send_transactions(&self, gate: SocketAddr, to: u64, count: u32) {
        // let him = Node::create(to);
        let transa = Transaction::new(0, to, count);
        let transa =
            serialize(&Packet::Transaction(transa)).expect("Error serialize transactions ");
        self.socket
            .send_to(&transa, gate)
            .expect("Error send transaction ");
    }


}

// //dans transaction
// fn verif_transa(share: Shared, transa: Transaction) {
//     //verification
//     let mut val = share.transaction.lock().unwrap();
//     (*val).push(transa);
// }


// fn update_peer_share(shared: &mut MutexGuard<Vec<SocketAddr>>, peer: Vec<SocketAddr>) {
//     **shared = peer;
// }

