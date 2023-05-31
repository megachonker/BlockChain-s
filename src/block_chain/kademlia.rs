use std::mem;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::thread;
use std::time;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
//remplacer par un énume les noms
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
enum Packet {
    GetPeers,
    RepPeers(Vec<SocketAddr>),
}

struct Node {
    node_addr: SocketAddr,
    peers_addr: Vec<SocketAddr>,
}

impl Node {
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>) {
        let mut node = Node {
            node_addr: address,
            peers_addr: bootstraps,
        };

        let socket_sender = UdpSocket::bind(node.node_addr).unwrap();

        thread::spawn(move || {
            let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 5]; //on veux
            loop {
                let (data, remote) = socket_sender.recv_from(&mut buffer).expect("err recv_from");
                match deserialize(&buffer[..data]).expect("errreur deserial") {
                    Packet::GetPeers => {
                        let serialized_packet =
                            serialize(&node.peers_addr).expect("Serialization error");
                        socket_sender
                            .send_to(&serialized_packet, remote)
                            .expect("err sendto");
                    }
                    Packet::RepPeers(mut response_packet) => {
                        node.peers_addr.append(&mut response_packet); // Modify the Vec
                    }
                }
            }
        });
    }
}

pub fn kademlia_simulate() {
    //INIT
    let mut rng = rand::thread_rng();
    for id in 1..=254 {
        let mut bootstrap_socket: Vec<SocketAddr> = Vec::with_capacity(5);

        //génère 5addresse random que la node peut rejoindre
        for _ in 0..5 {
            let socket = SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 1, rng.gen::<u8>()),
                6021,
            ));
            bootstrap_socket.push(socket);
        }
        Node::create(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 6021)),
            bootstrap_socket,
        );
    }
    //wait that all thread reach the recv_from
    //BTW i d'ont have solution to wait that recv_from fuction that reach and exectute,
    //with Arc and sync barreiere you can garantie to be beffort or after
    //if you are befort no garanty execution are correct
    //if you are after because it was blockant there you c'ant reach that region
    // and if you make that socket no blocking you can miss message
    thread::sleep(time::Duration::from_millis(100));

    //Fake starting

    // Packet::GetPeers


}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_mine_hasher_lessrng() {
//         let mut fist_block = Block::new(vec![]);
//         fist_block.nonce = mine_hasher_lessrng(&fist_block);
//         fist_block.block_id = hash(&fist_block);
//         assert!(fist_block.check());
//     }
// }
