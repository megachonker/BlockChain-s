use std::mem;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::thread;
use std::sync::{Arc, Barrier};
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
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>, starting_barr: Arc<Barrier>) {
        let mut node = Node {
            node_addr: address,
            peers_addr: bootstraps,
        };

        let socket_sender = UdpSocket::bind(node.node_addr).unwrap();
        let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 100]; //on veux 100 addres

        
        thread::spawn(move || {
            starting_barr.wait();
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
    let nb_ip = 254;
    let nb_boostrap = 5;

    let start_barrierre =  Arc::new(Barrier::new(nb_ip));
    let mut rng = rand::thread_rng();

    for id in 1..=nb_ip {
        let mut bootstrap_socket: Vec<SocketAddr> = Vec::with_capacity(nb_boostrap);

        //génère nb_boostrap addresse random que la node peut rejoindre
        for _ in 0..nb_boostrap {
            let socket = SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 1, rng.gen::<u8>()),
                6021,
            ));
            bootstrap_socket.push(socket);
        }
        Node::create(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 6021)),
            bootstrap_socket,
            start_barrierre.clone(),

        );
    }
    //Fake starting
    let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 100]; //on veux 100 addres
    
    let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1 as u8), 6021));
    let dst = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, 1 as u8), 6021));
    
    let test_socket = UdpSocket::bind(src).unwrap();
    
    let serialized_packet =    serialize(&Packet::GetPeers).expect("Serialization error");
    
    test_socket.send_to(&serialized_packet, dst).expect("err sendto");
    let (data,remote) = test_socket.recv_from(&mut buffer).expect("err receve to");

    match deserialize(&buffer[..data]).expect("errreur deserial") {
        Packet::GetPeers => {
            println!("GetPeers from: {}", remote);
        }
        Packet::RepPeers(response_packet) => {
            println!("RepPeers from: {}", remote);
            for rep in response_packet  {
                println!("{}", rep);
            }

        }
    }

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
