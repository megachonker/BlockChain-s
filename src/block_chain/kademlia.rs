use std::mem;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Barrier};
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
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>, starting_barr: Arc<Barrier>) {
        let mut node = Node {
            node_addr: address,
            peers_addr: bootstraps,
        };

        thread::spawn(move || {
            let socket = UdpSocket::bind(node.node_addr).unwrap();
            starting_barr.wait();

            let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 100]; //on veux 100 addres

            //send initial
            let serialized_packet = serialize(&Packet::GetPeers).expect("Serialization error");
            for peer in &node.peers_addr {
                socket.send_to(&serialized_packet, peer).expect("first batch send_to");
            }

            loop {
                let (offset, remote) = socket.recv_from(&mut buffer).expect("err recv_from");
                let message = deserialize(&buffer[..offset]).expect("errreur deserial");
                match message {
                    Packet::GetPeers => {
                        let serialized_packet = serialize(&Packet::RepPeers(node.peers_addr.clone())).expect("Serialization error"); //CLONE

                        socket
                            .send_to(&serialized_packet, remote)
                            .expect("err sendto");
                        println!("GetPeers from {}:", remote);
                    }
                    Packet::RepPeers(mut response_packet) => {
                        println!("RepPeers from {}: {:?}", remote,response_packet);
                        node.peers_addr.append(&mut response_packet);
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

    let start_barrierre = Arc::new(Barrier::new(nb_ip));
    let mut rng = rand::thread_rng();

    for id in 1..=nb_ip {
        let mut bootstrap_socket: Vec<SocketAddr> = Vec::with_capacity(nb_boostrap);

        //génère nb_boostrap addresse random que la node peut rejoindre
        for _ in 0..nb_boostrap {
            let socket = SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 1, rng.gen::<u8>()),
                9026,
            ));
            bootstrap_socket.push(socket);
        }
        Node::create(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 9026)),
            bootstrap_socket,
            start_barrierre.clone(),
        );
    }
    //Fake starting
    thread::sleep(time::Duration::from_millis(2000));

    // let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 100]; //on veux 100 addres

    // let src = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 10, 1 as u8), 9026));
    // let dst = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, 2 as u8), 9026));

    // let test_socket = UdpSocket::bind(src).unwrap();
    // let serialized_packet = serialize(&Packet::GetPeers).expect("Serialization error");

    // test_socket
    //     .send_to(&serialized_packet, dst)
    //     .expect("err sendto");

    // let (data, remote) = test_socket.recv_from(&mut buffer).expect("err receve to");
    // print!("from {}: {:?}", remote, &buffer[..data]);

    // match deserialize(&buffer[..data]).expect("errreur deserial") {
    //     Packet::GetPeers => {
    //         println!("GetPeers from: {}", remote);
    //     }
    //     Packet::RepPeers(response_packet) => {
    //         println!("RepPeers from: {}", remote);
    //         for rep in response_packet {
    //             println!("{}", rep);
    //         }
    //     }
    // }
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
