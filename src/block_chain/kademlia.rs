use core::time;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration, Instant};
//remplacer par un énume les noms
use rand::Rng;

//bootstrap
// 127.0.1.1/24
struct Node {
    node_addr: SocketAddr,
    peers_addr: Vec<SocketAddr>,
    output_lock: Arc<Mutex<()>>,
}

impl Node {
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>,output_lock: Arc<Mutex<()>>)  {
        let node = Node {
            node_addr: address,
            peers_addr: bootstraps,
            output_lock,
        };
        thread::spawn(move||{
            // time::Duration()
            node.print();
        });
    }

    // fn moar_peers(&self){
    //     for peer in self.peers_addr{
    //         //send get peers
    //         //receive
    //     }
    // }

    // fn search_item(&self){

    // }

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
    // let mut nodes: Vec<Node> = Vec::with_capacity(255);
    for id in 1..=254 {
        let mut bootstrap_socket: Vec<SocketAddr> = Vec::with_capacity(10);
        
        //génère 10addresse random que la node peut rejoindre
        for _ in 0..10 {
            let socket = SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 1, rng.gen::<u8>()),
                6021,
            ));
            bootstrap_socket.push(socket);
        }
        Node::create(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 6021)), bootstrap_socket,Arc::clone(&output_lock));
        //ajoute nouvel node
        // nodes.push(Node {
        //     node_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 6021)),
        //     peers_addr: bootstrap_socket,
        // });
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
