use core::time;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc,Mutex};
use std::thread;
use std::mem;
use std::io::Cursor;

use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
//remplacer par un énume les noms
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
enum Packet {
    GET_PEERS,
    REP_PERS(Vec<SocketAddr>),
}


struct Node {
    node_addr: SocketAddr,
    peers_addr: Vec<SocketAddr>,
    output_lock: Arc<Mutex<()>>,
}


impl Node {
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>,output_lock: Arc<Mutex<()>>)  {
        let mut node = Node {
            node_addr: address,
            peers_addr: bootstraps,
            output_lock,
        };
        node.print();
        
        //listener
        let socket_sender = UdpSocket::bind(node.node_addr).unwrap();

        let socklist: UdpSocket = socket_sender.try_clone().expect("try clone socket err");
        let bootstrap_list = node.peers_addr.clone();

        thread::spawn(move||{
            let mut buffer = vec![0u8; mem::size_of::<SocketAddr>()*5];//on veux 

            loop {
                match socklist.recv(&mut buffer) {
                    
                    Ok(received) => {
                        
                        
                        let mut v: Vec<SocketAddr>  = deserialize(&buffer[..received]).expect("errreur deserial");
                        for peer in &v {
                            println!("ADDED:{}",peer);
                        }
                        node.peers_addr.append(&mut v);
                    
                    } ,
                    Err(e) => println!("recv function failed: {e:?}"),}
            }

        });

        //sender
        // let socket = UdpSocket::bind(node.node_addr).unwrap();
        let serialized_packet = serialize(&bootstrap_list).expect("Serialization error");
        for dest in &bootstrap_list{
            socket_sender.send_to(&serialized_packet, dest).expect("envoit erreur");
        }

        // node.print();


    }


    fn print(&self) {

        let _lock = self.output_lock.lock().unwrap();
        println!("{}", self.node_addr);
        println!("---------------");
        for peer in &self.peers_addr {
            println!("{}", peer);
        }
        println!("");//saut ligne
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
        Node::create(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 6021)), bootstrap_socket,Arc::clone(&output_lock));
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
