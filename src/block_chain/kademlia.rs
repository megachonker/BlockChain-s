use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
//remplacer par un énume les noms

//bootstrap
// 127.0.1.1/24
struct Node {
    node_addr: SocketAddr,
    peers_addr: Vec<SocketAddr>,
}

impl Node {
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>) -> Node {
        Node {
            node_addr: address,
            peers_addr: bootstraps,
        }
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
        println!("{}", self.node_addr);
        println!("---------------");
        for peer in &self.peers_addr {
            println!("{}", peer);
        }
        println!("");//saut ligne
    }
}

use rand::Rng;

pub fn kademlia_simulate() {
    let mut rng = rand::thread_rng();
    let mut nodes: Vec<Node> = Vec::with_capacity(255);
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
        //ajoute nouvel node
        nodes.push(Node {
            node_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, id as u8), 6021)),
            peers_addr: bootstrap_socket,
        });
    }


    for node in &nodes{
        node.print();
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_hasher_lessrng() {
        let mut fist_block = Block::new(vec![]);
        fist_block.nonce = mine_hasher_lessrng(&fist_block);
        fist_block.block_id = hash(&fist_block);
        assert!(fist_block.check());
    }
}
