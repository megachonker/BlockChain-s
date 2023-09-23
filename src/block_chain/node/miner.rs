use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::block_chain::{shared::Shared,block::{Block, Transaction},node::network::{Network,Packet}};
use crate::friendly_name::*;

pub struct Miner {
    name: String,
    network: Network, // blockchaine
    //miner
    id: u64,
}

impl Miner {
    pub fn new(network: Network) -> Self {
        let name =
            get_friendly_name(network.get_socket()).expect("generation name from ip imposble");
        let id = get_fake_id(&name);
        Self {
            name,
            network,
            id: id,
        }
    }
    pub fn start(self) {
        println!(
            "Server started {} facke id {} -> {:?}",
            &self.name,
            get_fake_id(&self.name),
            self.network
        );
        let id = get_fake_id(&self.name);

        let mut peers: Vec<SocketAddr>;
        let mut block_chaine: Vec<Block> = vec![];

        //if user enter 0.0.0.0 Create a new blockaine
        if self.network.bootstrap == SocketAddr::from(([0, 0, 0, 0], 6021)) {
            peers = vec![];
            peers.push(self.network.get_socket());
            block_chaine.push(Block::new());
        }
        //if not retreive the blockaine
        else {
            peers = self.network.bootstrap();

            println!("Found {} peer", peers.len());

            block_chaine = self.network.get_chain(&peers).unwrap();

            println!("Catch a chain of {} lenght", block_chaine.len());
        }

        let should_stop = Arc::new(Mutex::new(false));

        //complexitée dans Blockhaine
        let starting_block = block_chaine.last().unwrap().clone();
        let peer = Arc::new(Mutex::new(peers));

        let (rx, tx) = mpsc::channel();
        let share = Shared::new(peer, should_stop, block_chaine);
        let share_copy = share.clone();

        let node_clone = self.clone();

        thread::spawn(move || {
            node_clone.listen(share_copy, rx);
        });

        //serait Miner::start
        self.mine(share, starting_block, tx);
    }

    pub fn listen(&self, share: Shared, rx: mpsc::Sender<Block>) {
        let mut peerdict: HashMap<SocketAddr, Duration> = HashMap::new();

        let mut last_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can not take time");

        let peer = share.peer.lock().expect("Can not get the peer");

        for &p in &*peer {
            peerdict.insert(p, last_time);
        }

        drop(peer);

        //dans mon llama j'appelle ça un router
        //le router map les les fonction a appler en fonction du enum recu
        //ça serait beaucoup plus court d'apeler que une ligne par type reucs       --> oui surrement plus lisible s'il y a pas trop d'arg a passer
        loop {
            let (message, sender) = self.network.recv_packet();
            let time_packet = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Impossible to get time");
            match message {
                Packet::Keepalive => {
                    //call la fc aproprier
                    println!("recv KeepAlive");
                    self.network.send_packet(Packet::AnswerKA, sender);
                }
                Packet::Block(block) => {
                    println!("recv Block");

                    let mut chain: MutexGuard<'_, Vec<Block>> = share.chain.lock().unwrap();
                    if !chain.contains(&block) {
                        if block.check() {
                            let cur_height = chain.last().unwrap().get_height_nonce().0;
                            let new_height = block.get_height_nonce().0;
                            if cur_height + 1 == new_height {
                                {
                                    chain.push(block.clone());
                                }
                                rx.send(block.clone()).unwrap();
                                {
                                    let mut val = share.should_stop.lock().unwrap();
                                    *val = true;
                                }
                                let mut val = share.transaction.lock().unwrap();
                                (*val) = vec![]; //on remet a zero les transactions peut être a modiifier

                                drop(chain);

                                self.network.send_packet_multi(
                                    Packet::Block(block),
                                    peerdict.keys().cloned().collect(),
                                );
                            } else if cur_height + 1 < new_height {
                                //we are retarded"
                                println!("We are to late");
                                let mut buf = [0u8; 256];
                                let mut dist_block = vec![];
                                dist_block.push(block.clone());
                                let mut last_block_common: i64 = -1;
                                for i in (0..new_height).rev() {
                                    self.network.send_packet(Packet::GetBlock(i as i64), sender);

                                    loop {
                                        let (packet, sender2) = self.network.recv_packet();

                                        if sender2 != sender {
                                            continue;
                                        }

                                        if let Packet::Block(b) = packet {
                                            if b.get_height_nonce().0 != i {
                                                println!(
                                                    "Pb diff heigh demand {} recv {}",
                                                    i,
                                                    b.get_height_nonce().0
                                                )
                                            }
                                            if b.get_height_nonce().0 > cur_height {
                                                dist_block.push(b.clone());
                                            } else {
                                                println!(
                                                    "&& The block is {:?} \n {:?}",
                                                    b, chain[i as usize]
                                                );
                                                if b == chain[i as usize] {
                                                    println!("{} is same", i);
                                                    last_block_common = i as i64;
                                                } else {
                                                    dist_block.push(b.clone());
                                                }
                                            }
                                            break;
                                        }
                                    }
                                    if last_block_common != -1 {
                                        break;
                                    }
                                }
                                if last_block_common != -1 {
                                    println!("Ici");
                                    dist_block.reverse();
                                    for (i, b) in dist_block.iter().enumerate() {
                                        if i + last_block_common as usize <= cur_height as usize {
                                            chain[i + last_block_common as usize] = b.clone();
                                        } else {
                                            chain.push(b.clone());
                                        }
                                    }
                                    rx.send(block).unwrap();

                                    let mut val = share.should_stop.lock().unwrap();
                                    *val = true;
                                    drop(val);
                                }
                            }
                        }
                    }
                }
                Packet::Transaction(trans) => {
                    //use transactiont itself ?     --> ??
                    println!("recv Transaction");

                    let clone_share = share.clone();
                    println!("Recive a new transactions");
                    // thread::spawn(move || {          //mieux mais marche pas
                    self.verif_transa(clone_share, trans);
                    // });
                }

                Packet::GetPeer => {
                    println!("recv GetPeer");

                    self.network
                        .send_packet(Packet::RepPeers(peerdict.keys().cloned().collect()), sender);
                }

                Packet::GetBlock(i) => {
                    //call blockchain directly      --> un autre node demande des block
                    println!("Recv getBlock {}", i);
                    let chain: std::sync::MutexGuard<'_, Vec<Block>> =
                        share.chain.lock().expect("Can not lock chain");
                    if i == -1 {
                        //ask the last one
                        self.network
                            .send_packet(Packet::Block(chain.last().unwrap().clone()), sender);
                        drop(chain);
                    } else if chain.len() < i as usize {
                        drop(chain);
                        self.network
                            .send_packet(Packet::Block(Block::new_wrong(1)), sender);
                    //No enought block
                    } else {
                        if chain[i as usize].get_height_nonce().0 != i as u64 {
                            println!("Pb diff heigh");
                        }
                        self.network
                            .send_packet(Packet::Block(chain[i as usize].clone()), sender);
                        drop(chain);
                    }
                }

                Packet::Connexion => {
                    println!("recv Connexion");

                    let mut peer: Vec<SocketAddr> = peerdict.keys().cloned().collect();
                    if !peer.contains(&sender) {
                        peerdict.insert(sender, time_packet);

                        self.network
                            .send_packet(Packet::RepPeers(peer.clone()), sender);

                        self.network.send_packet_multi(
                            Packet::NewNode(sender),
                            peerdict.keys().filter(|&&x| x != sender).cloned().collect(),
                        );
                    }
                    peer.push(sender);
                    update_peer_share(&mut share.peer.lock().unwrap(), peer);
                }

                Packet::NewNode(new) => {
                    println!("recv newnode ");
                    let mut peer: Vec<SocketAddr> = peerdict.keys().cloned().collect();
                    if !peer.contains(&new) {
                        peerdict.insert(sender, time_packet);
                        peer.push(new);

                        self.network.send_packet_multi(
                            Packet::NewNode(new),
                            peer.iter().filter(|&&x| x != sender).cloned().collect(),
                        );
                    }
                    update_peer_share(&mut share.peer.lock().unwrap(), peer);
                }

                _ => {}
            }
            peerdict.entry(sender).and_modify(|entry| {
                *entry = time_packet;
            });

            if time_packet - last_time > Duration::from_secs(120) {
                println!("Check node already here ? ");
                self.check_keep_alive(&mut peerdict, time_packet);
                update_peer_share(
                    &mut share.peer.lock().unwrap(),
                    peerdict.keys().cloned().collect(),
                );
                last_time = time_packet;
            }
        }
    }

    fn verif_transa(&self, share: Shared, transa: Transaction) {
        //verification /////A FAIRE\\\\\\\\\\\\

        let mut val = share.transaction.lock().unwrap();
        (*val).push(transa);
    }

    fn check_keep_alive(&self, peer: &mut HashMap<SocketAddr, Duration>, time: Duration) {
        let clone = peer.clone();
        for (p, t) in clone {
            if time - t > Duration::from_secs(240) {
                peer.remove(&p);
                println!("Remove the peer {}", p);
            } else if time - t > Duration::from_secs(60) {
                println!("Send a keep alive to {}", p);

                self.network.send_packet(Packet::Keepalive, p);
            }
        }
    }

    fn mine(&self, share: Shared, mut block: Block, tx: mpsc::Receiver<Block>) {
        loop {
            match block.generate_block_stop(self.id, &share.should_stop, "It is a quote") {
                Some(mut new_block) => {
                    print!("FOUND ");
                    {
                        //add the transactions see during the mining
                        let val = share
                            .transaction
                            .lock()
                            .expect("Error during lock of transaction");
                        new_block = new_block.set_transactions((*val).clone());
                    }
                    let mut chain = share.chain.lock().expect("Can not lock chain");
                    chain.push(new_block.clone());
                    drop(chain); // { } peut être utiliser  --> oui mais moche
                    {
                        let peer = share.peer.lock().unwrap();

                        self.network.send_packet_multi(
                            Packet::Block(new_block.clone()),
                            peer.iter()
                                .filter(|&&x| x != self.network.get_socket())
                                .cloned()
                                .collect(),
                        );
                    }
                    block = new_block;
                }
                None => {
                    println!("External ");
                    block = tx
                        .recv()
                        .expect("Error block can't be read from the channel");
                    {
                        let mut val = share.should_stop.lock().unwrap();
                        *val = false;
                    }
                }
            }
            println!(" => {:?} ", block);
        }
    }
}

impl Clone for Miner {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            network: self.network.clone(),
            id: self.id.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

fn update_peer_share(shared: &mut MutexGuard<Vec<SocketAddr>>, peer: Vec<SocketAddr>) {
    //marche pas sans function ??
    **shared = peer;
}
