use std::mem;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Barrier,Mutex};
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
    peers_addr: Arc<Mutex<Vec<SocketAddr>>>,
}

impl Node {
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>, start_barrier: Arc<Barrier>) {

        let node = Node {
            node_addr: address,
            peers_addr: Arc::new(Mutex::new(bootstraps)) ,
        };

        thread::spawn(move || {
            let socket = Arc::new(UdpSocket::bind(node.node_addr).unwrap());
            start_barrier.wait();

            let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 255]; //on veux 255 addres max 

            let socket_refresh_peer = socket.clone();
            let copy_peer_addr = node.peers_addr.clone();


            //demande
            thread::spawn(move || {
                
                //on demande j'usqua un certain niveaux
                while copy_peer_addr.lock().unwrap().len() < 250 {
                    
                    // let progression = (255/response_packet.len())*100;
                    // println!("RepPeers from {}: {}%", remote, progression);

                    println!("SENDPeers from {}: {}",address, copy_peer_addr.lock().unwrap().len());

                    let serialized_packet = serialize(&Packet::GetPeers).expect("Serialization error");
                    for peer in copy_peer_addr.lock().unwrap().iter()  {     
                        socket_refresh_peer.send_to(&serialized_packet, &peer).expect("send to imposible");
                    }
                    thread::sleep(time::Duration::from_millis(100)); //<= converge plus vite
                }
            });


            //écoute des message recus
            loop {
                let (offset, remote) = socket.recv_from(&mut buffer).expect("err recv_from");
                let message = deserialize(&buffer[..offset]).expect("errreur deserial");
                match message {
                    //renvoit une copy de tout les peer connue
                    Packet::GetPeers => {
                        let peers_addr_copy = node.peers_addr.lock().unwrap().to_vec();
                        let serialized_packet =
                            serialize(&Packet::RepPeers(peers_addr_copy)).expect("Serialization error"); //CLONE

                        socket
                            .send_to(&serialized_packet, remote)
                            .expect("err sendto");
                        // println!("GetPeers from {}:", remote);
                    }
                    //ajoute les élément non conue a la list
                    Packet::RepPeers(response_packet) => {
                        let mut peers_addr = node.peers_addr.lock().unwrap();
                        for remote_peer in response_packet {
                            if !peers_addr.iter().any(|&local_peer| local_peer == remote_peer) {
                                peers_addr.push(remote_peer);
                            }
                        }
                        println!("RepPeers from {}: {}", remote, peers_addr.len());
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

    let start_barrier = Arc::new(Barrier::new(nb_ip));
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
            start_barrier.clone(),
        );
    }
    //Fake starting
    thread::sleep(time::Duration::from_millis(5000));
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
