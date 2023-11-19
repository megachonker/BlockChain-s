//usless actuelemnt ??  --> oui mais en gros le code a été implementer dans node

use std::{
    collections::HashSet,
    mem,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    sync::{Arc, Barrier, Mutex},
    thread, time,
};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
//remplacer par un énume les noms
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
enum Packet {
    GetPeers,
    RepPeers(Vec<SocketAddr>),
}
#[derive(Clone)]
pub struct Node {
    node_addr: SocketAddr,
    peers_addr: Arc<Mutex<Vec<SocketAddr>>>,
}

impl Node {
    fn create(address: SocketAddr, bootstraps: Vec<SocketAddr>) -> Node {
        Node {
            node_addr: address,
            peers_addr: Arc::new(Mutex::new(bootstraps)),
        }
    }

    pub fn start(self, start_barrier: Arc<Barrier>) {
        thread::spawn(move || {
            let socket = Arc::new(UdpSocket::bind(self.node_addr).unwrap());
            start_barrier.wait();

            let mut buffer = vec![0u8; mem::size_of::<SocketAddr>() * 255]; //on veux 255 addres max //<= a cahnger

            let socket_refresh_peer = socket.clone();
            let copy_peer_addr = self.peers_addr.clone();

            //Thread demande
            thread::spawn(move || {
                //on demande j'usqua un certain niveaux
                while copy_peer_addr.lock().unwrap().len() < 250 {
                    // <=========== a changer
                    // println!(
                    //     "SENDPeers from {}: {}",
                    //     self.node_addr,
                    //     copy_peer_addr.lock().unwrap().len()
                    // );

                    let serialized_packet =
                        serialize(&Packet::GetPeers).expect("Serialization error");
                    for peer in copy_peer_addr.lock().unwrap().iter() {
                        socket_refresh_peer
                            .send_to(&serialized_packet, &peer)
                            .expect("send to imposible");
                    }
                    thread::sleep(time::Duration::from_millis(100)); //<= converge plus vite AFIXER INADMISIBLE
                }
            });

            //écoute des message recus
            loop {
                let (offset, remote) = socket.recv_from(&mut buffer).expect("err recv_from");
                let message = deserialize(&buffer[..offset]).expect("errreur deserial");
                match message {
                    //renvoit une copy de tout les peer connue
                    Packet::GetPeers => {
                        let peers_addr_copy = self.peers_addr.lock().unwrap().to_vec();
                        let serialized_packet = serialize(&Packet::RepPeers(peers_addr_copy))
                            .expect("Serialization error"); //CLONE

                        socket
                            .send_to(&serialized_packet, remote)
                            .expect("err sendto");
                        // println!("GetPeers from {}:", remote);
                    }
                    //ajoute les élément non conue a la list
                    Packet::RepPeers(response_packet) => {
                        let mut peers_addr = self.peers_addr.lock().unwrap();
                        for remote_peer in response_packet {
                            if !peers_addr
                                .iter()
                                .any(|&local_peer| local_peer == remote_peer)
                            {
                                peers_addr.push(remote_peer);
                            }
                        }
                        // println!("RepPeers from {}: {}", remote, peers_addr.len());
                    }
                }
            }
        });
    }
}

pub struct Simulate {
    nb_ip: usize,
    nodes: Vec<Node>,
}

impl Simulate {
    pub fn init(nb_ip: usize, nb_bootstrap: usize) -> Simulate {
        //INIT
        let mut vector: Vec<Node> = vec![];

        let mut rng = rand::thread_rng();

        let reseauxrandom = rng.gen::<u8>();

        for id in 1..=nb_ip {
            let mut bootstrap_socket: Vec<SocketAddr> = Vec::with_capacity(nb_bootstrap);

            //génère nb_bootstrap addresse random que la node peut rejoindre
            for _ in 0..nb_bootstrap {
                let socket = SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, reseauxrandom, rng.gen::<u8>()),
                    9026,
                ));
                bootstrap_socket.push(socket);
            }

            vector.push(Node::create(
                SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, reseauxrandom, id as u8),
                    9026,
                )),
                bootstrap_socket,
            ));
        }
        return Simulate {
            nb_ip: nb_ip,
            nodes: vector,
        };
    }

    pub fn start(&self) {
        let start_barrier: Arc<Barrier> = Arc::new(Barrier::new(self.nb_ip));
        for node in &self.nodes {
            let cclne: Node = node.clone();
            cclne.start(start_barrier.clone());
        }
    }
    pub fn whait(&self) {
        thread::sleep(time::Duration::from_millis(5000));
    }
    pub fn duplicate(&self) -> bool {
        for node in &self.nodes {
            let mut unique_addresses = HashSet::new();
            let possesion = node.peers_addr.lock().unwrap().clone();
            for address in &possesion {
                if !unique_addresses.insert(address) {
                    println!("local addr {:?}", node.node_addr);
                    println!("Err dans: {:?} as {}", possesion, address);
                    return false;
                }
            }
        }
        true
    }

    pub fn converge(&self) -> bool {
        for node in &self.nodes {
            if node.peers_addr.lock().unwrap().clone().len() < self.nb_ip / 2 {
                println!(
                    "le node: {} taille réseaux:{:?}",
                    node.node_addr,
                    node.peers_addr.lock().unwrap().clone()
                );
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniciter() {
        let simu: Simulate = Simulate::init(255, 5);
        simu.start();
        simu.whait();
        assert!(simu.duplicate());
    }

    #[test]
    fn convegence() {
        let simu: Simulate = Simulate::init(255, 5);
        simu.start();
        simu.whait();
        assert!(simu.converge());
    }
}
