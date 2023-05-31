use std::mem;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;

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
    peers_addr: Arc<Vec<SocketAddr>>,
    output_lock: Arc<Mutex<()>>,
}

impl Node {
    fn create(address: SocketAddr, bootstraps: Arc<Vec<SocketAddr>>, output_lock: Arc<Mutex<()>>) {
        let mut node = Node {
            node_addr: address,
            peers_addr: bootstraps,
            output_lock,
        };
        // node.print();

        let socket_sender = UdpSocket::bind(node.node_addr).unwrap();

        thread::spawn(move || {
            let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 5]; //on veux
            loop {
                let (data,remote) = socket_sender.recv_from(&mut buffer).expect("err recv_from");
                match deserialize(&buffer[..data]).expect("errreur deserial") {
                    Packet::GetPeers => {
                        let serialized_packet = serialize::<[SocketAddr]>(&node.peers_addr).expect("Serialization error");
                        socket_sender.send_to(&serialized_packet, remote).expect("err sendto");
                    }
                    Packet::RepPeers(mut response_packet) => {
                        let mut peers = Arc::try_unwrap(node.peers_addr).expect("Failed to unwrap Arc"); // Unwrap the Arc to get ownership of the Vec
                        peers.append(&mut response_packet); // Modify the Vec
                        node.peers_addr = Arc::new(peers); // Convert the Vec back to an Arc and assign it to the node
                    }
                }
            }
        });

        // node.print();
    }

    fn print(&self) {
        let _lock = self.output_lock.lock().unwrap();
        println!("{}", self.node_addr);
        println!("---------------");
        for peer in self.peers_addr.iter() {
            println!("{}", peer);
        }
        println!(""); //saut ligne
    }
}

pub fn kademlia_simulate() {
    //INIT
    let output_lock = Arc::new(Mutex::new(())); // Create a single output lock
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
            bootstrap_socket.into(),
            Arc::clone(&output_lock),
        );
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
