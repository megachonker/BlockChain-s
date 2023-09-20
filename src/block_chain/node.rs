use crate::friendly_name::{get_friendly_name, get_fake_address};

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{Arc, Barrier, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use crate::shared::Shared;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{arg, ArgMatches, Parser};

use super::block::{Block, Transaction};

/////////important/////////////
// on peut faire l'abre des dépendance au niveaux du systeme de fichier aussi
/*
idée de changement de structure

enum Node{          --> ca peut être une très bonne idée de separer client server a voire si ca ce fait bien
    // struct emule
    //     node::server
    //     node::client
    struct Server
        miner
        Network
            kamelia
        blockaine
            block
    struct client
        User
            Kripto --> Y'en a besoin pour tout le monde je pense
        transaction
}
*/

////////////////////: USFUL DE FOU
//serait dans utiliser pas kamelia
#[derive(Serialize, Deserialize, Debug)]
enum Packet {
    Keepalive,
    AnswerKA,
    Transaction(Transaction),
    Block(Block),
    GetPeer,
    GetBlock(i64),
    RepPeers(Vec<SocketAddr>),
    Connexion,
    NewNode(SocketAddr),
}

// on est sur que quand on manipule une node on a que un des 3 mode
pub enum NewNode {
    Srv(Server),
    Cli(Client),
}

// un trait serait mieux ?
// permet de start un node sans connaitre son type au préalable
impl NewNode {
    pub fn start(self) {
        match self {
            Self::Cli(cli) => cli.start(),
            Self::Srv(srv) => srv.start(),
        }
    }
}

//permet de stoquer ce qui est lier au network
pub struct Network {
    bootstrap: SocketAddr,
    binding: SocketAddr,
    //maybe kamelia
}

// whole network function inside it
// send packet with action do scan block ect get peers
impl Network {
    pub fn new(bootstrap: IpAddr, binding: IpAddr) -> Self {
        let binding = SocketAddr::new(binding, 9026);
        let bootstrap = SocketAddr::new(bootstrap, 9026);
        Self { bootstrap, binding }
    }
}

pub struct Server {
    name: String,
    networking: Network, // blockchaine
                         //miner
}
impl Server {
    pub fn new(networking: Network) -> Self {
        let name = get_friendly_name(networking.binding).expect("generation name from ip imposble");
        Self { name, networking }
    }
    fn start(self) {
        println!("Server started {} facke id {}", &self.name, get_fake_address(&self.name));
        let ip = self.networking.binding;
        let id = get_fake_address(&self.name);

        let me: Node = Node::create(id,ip.to_string());
        me.setup_mine(self.networking.bootstrap);    
    }
}

struct NewTransaction {
    destination: u64,
    secret: String,
    ammount: f64,
}

/*
to do transa need to have block i think and network
*/

pub struct Client {
    name: String,
    networking: Network,
    //un client va faire une action
    //// le client pourait etre un worker qui effectue les action dicter par un front end
    /*enum action{ // <= peut etre un flux comme un mscp
        balance //calcule argent compte
        transaction(destination)
    }*/
    transaction: NewTransaction,
}

impl Client {
    pub fn new(networking: Network, destination: u64, secret: String, ammount: f64) -> Self {
        let name = get_friendly_name(networking.binding).expect("generation name from ip imposble");
        let transaction = NewTransaction {
            destination,
            secret,
            ammount,
        }; //can make check here
        Self {
            name,
            networking,
            transaction,
        }
    }
    pub fn start(self) {
        let ip = self.networking.binding;
        let id = get_fake_address(&self.name);

        let me: Node = Node::create(id,ip.to_string());
        me.send_transactions(self.networking.bootstrap,self.transaction.destination,self.transaction.ammount as u32);
        println!("Client started name is {} fack id{}", self.name,get_fake_address(&self.name))
    }
}

pub struct Node {
    // uname: String,
    id: u64,
    socket: UdpSocket,
    barrier: Arc<Barrier>,
    // voir changement préscrit   --> ?
}

impl Node {

    //new
    pub fn create(id: u64, ip: String) -> Node {
        let socket = UdpSocket::bind(ip).expect("{id} couldn't bind to address:"); //1
        let barrier = Arc::new(Barrier::new(2));
        Node {
            id,
            socket,
            barrier,
        }
    }

    //comment ça ? j'ai jamais fait des impl de clone --> on peut faire le trait Clone plus clean en effet
    pub fn clone(&self) -> Node {
        let barrier = Arc::new(Barrier::new(2));

        Node {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            barrier: barrier,
        }
    }

    //dans ému
    pub fn run_listen(&self) {
        let socket = self.socket.try_clone().expect("fail to clone socket");
        let id = self.id;
        let barrier = self.barrier.clone();

        let mut buf = [0; 3];
        thread::spawn(move || {
            //CASSER La qsdmlfjhnqsdfiogu avec timeout   --> pas compris

            socket
                .set_read_timeout(Some(Duration::new(0, 1000000)))
                .expect("set_read_timeout call failed");
            println!("{} Whait Timeout: ", id);
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    barrier.wait(); // Unblock the send operation
                    println!(
                        "Node {} from {} received: {}",
                        id,
                        get_friendly_name(src).unwrap(),
                        String::from_utf8_lossy(&buf[..amt])
                    );
                    socket
                        .send_to("Here".as_bytes(), format!("17.0.0.{}", id))
                        .expect("Failed to send data");
                }
                Err(_) => {
                    // Handle timeout here
                    barrier.wait(); // Unblock the send operation even if no packet received
                    println!("{} unlock Timeout", id);
                }
            }
            socket
                .set_read_timeout(None)
                .expect("set_read_timeout call failed");

            println!("{}: started", id);
            loop {
                let (amt, src) = socket
                    .recv_from(&mut buf)
                    .expect(&format!("{} Failed to receive data", id)); //2
                barrier.wait();
                println!(
                    "Node {} from {} received: {}",
                    id,
                    get_friendly_name(src).unwrap(),
                    String::from_utf8_lossy(&buf[..amt])
                );
                socket
                    .send_to("Here".as_bytes(), format!("17.0.0.{}", id))
                    .expect(&("Failed to send data to:"));
                //3
            }
        });
    }

    //ému
    fn run_send(&mut self, id: u64) {
        self.barrier.wait();
        println!("Node {} to {} send: {}", self.id, id, self.id);
        self.socket
            .send_to("Here".as_bytes(), format!("17.0.0.{}", id))
            .expect(&("Failed to send data to:"));
        //3
    }

    // ?
    fn quit(&mut self) {}

    //ça fait quoi ?
    fn hear(&self) -> (Packet, SocketAddr) {
        let mut buffer = vec![0u8; 1024]; //MAXSIZE a def ??

        let (offset, sender) = self.socket.recv_from(&mut buffer).expect("err recv_from");
        (
            deserialize(&buffer[..offset]).expect("errreur deserial"),
            sender,
        )
    }
    //network
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
            let (message, sender) = self.hear();
            let time_packet = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Impossible to get time");
            match message {
                Packet::Keepalive => {
                    //call la fc aproprier
                    println!("recv KeepAlive");
                    let sera_answer =
                        serialize(&Packet::AnswerKA).expect("Can not serialize AswerKA");
                    self.socket
                        .send_to(&sera_answer, sender)
                        .expect("Can not send AnswerKA");
                }
                Packet::Block(block) => {
                    //self.blockaine.append(block) ??

                    println!("recv Block");

                    let mut chain: MutexGuard<'_, Vec<Block>> = share.chain.lock().unwrap(); //pas fou
                    if !chain.contains(&block) {
                        if block.check() {
                            let cur_height = chain.last().unwrap().get_height_nonce().0;
                            let new_height = block.get_height_nonce().0;
                            if cur_height + 1 == new_height {
                                {
                                    chain.push(block.clone());
                                }
                                rx.send(block.clone()).unwrap(); //on send un doublon du block a qui ?
                                {
                                    let mut val = share.should_stop.lock().unwrap();
                                    *val = true;
                                }
                                let mut val = share.transaction.lock().unwrap();
                                (*val) = vec![]; //on remet a zero les transactions peut être a modiifier

                                drop(chain);

                                //on send a tt le monde le block fait ?  --> oui et si on a deja recut le block et ignore, j'aivais reflechie et c'est assez commpliqué mais si ca marche avec kaùelia c'est cool
                                //donc network.publish(block)? qui appelle kamelia.peers
                                let seria_block = serialize(&Packet::Block(block))
                                    .expect("Can not serialize block");

                                // let peer = peerdict.keys().cloned()

                                for p in peerdict.keys().cloned() {
                                    self.socket
                                        .send_to(&seria_block, p)
                                        .expect("Error send block");
                                }
                            } else if cur_height + 1 < new_height {
                                //we are retarded"
                                println!("We are to late");
                                let mut buf = [0u8; 256];
                                let mut dist_block = vec![];
                                dist_block.push(block.clone());
                                let mut last_block_common: i64 = -1;
                                for i in (0..new_height).rev() {
                                    let seri_getblock: Vec<u8> =
                                        serialize(&Packet::GetBlock(i as i64)).unwrap();
                                    self.socket
                                        .send_to(&seri_getblock, sender)
                                        .expect("Can not send getblock");
                                    loop {
                                        if self.socket.recv_from(&mut buf).expect("Can not recv").1
                                            != sender
                                        {
                                            continue;
                                        }

                                        if let Packet::Block(b) = deserialize(&buf).unwrap() {
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
                    thread::spawn(move || {
                        //Maybe put this in the setup_mine and pass channel for new transa
                        verif_transa(clone_share, trans);
                    });

                    // //share the new transa ???           --> maybe a missing , pas damatique.
                }

                Packet::GetPeer => {
                    //call network..... kamelia.peers       --> pour l'instant juste tout les nodes mais a paufinner
                    println!("recv GetPeer");

                    let serialize_peer =
                        serialize(&Packet::RepPeers(peerdict.keys().cloned().collect()))
                            .expect("Error serialize peer");

                    self.socket
                        .send_to(&serialize_peer, sender)
                        .expect("Error sending peers");

                    println!("Send the peer at {}", sender);
                }

                Packet::GetBlock(i) => {
                    //call blockchain directly      --> un autre node demande des block
                    println!("Recv getBlock {}", i);
                    let chain: std::sync::MutexGuard<'_, Vec<Block>> =
                        share.chain.lock().expect("Can not lock chain");
                    let serialize_block;
                    if i == -1 {
                        //ask the last one
                        serialize_block = serialize(&Packet::Block(chain.last().unwrap().clone()));
                        drop(chain);
                    } else if chain.len() < i as usize {
                        drop(chain);
                        serialize_block = serialize(&Packet::Block(Block::new_wrong(1)));
                    //No enought block
                    } else {
                        if chain[i as usize].get_height_nonce().0 != i as u64 {
                            println!("Pb diff heigh");
                        }
                        serialize_block = serialize(&Packet::Block(chain[i as usize].clone())); //peut etre mettre des & dans Block
                        drop(chain);
                    }
                    let serialize_block = serialize_block.expect("Can not serialize the block");
                    self.socket
                        .send_to(&serialize_block, sender)
                        .expect("Error send serialize block");
                }

                Packet::Connexion => {
                    //call ntwork
                    println!("recv Connexion");

                    let peer: Vec<SocketAddr> = peerdict.keys().cloned().collect();
                    if !peer.contains(&sender) {
                        peerdict.insert(sender, time_packet);
                        let serialize_peer = serialize(&Packet::RepPeers(peer.clone()))
                            .expect("Error serialize peer");
                        self.socket
                            .send_to(&serialize_peer, sender)
                            .expect("Error sending peers");

                        let serialize_new =
                            serialize(&Packet::NewNode(sender)).expect("Can not serialize NewNode");
                        for p in peerdict.keys().cloned() {
                            //send at all
                            if p != sender {
                                self.socket
                                    .send_to(&serialize_new, p)
                                    .expect("Can not send NewNode");
                            }
                        }
                    }
                    update_peer_share(&mut share.peer.lock().unwrap(), peer);
                }

                Packet::NewNode(new) => {
                    //kamelia ?.
                    println!("recv newnode ");
                    let peer: Vec<SocketAddr> = peerdict.keys().cloned().collect();
                    if !peer.contains(&new) {
                        peerdict.insert(sender, time_packet);
                        let serialize_new =
                            serialize(&Packet::NewNode(sender)).expect("Can not serialize NewNode");
                        for p in &*peer {
                            //send at all
                            if *p != sender {
                                self.socket
                                    .send_to(&serialize_new, p)
                                    .expect("Can not send NewNode");
                            }
                        }
                    }
                    update_peer_share(&mut share.peer.lock().unwrap(), peer);
                }

                _ => {}
            }
            peerdict.entry(sender).and_modify(|entry| {
                *entry = time_packet;
            });

            if time_packet - last_time > Duration::from_secs(60) {
                //lul
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

    //inside network get blcok
    fn get_block(&self, index: i64, gate: SocketAddr) -> Block {
        self.socket
            .send_to(
                &serialize(&Packet::GetBlock(index)).expect("Can not serialize GetBlock(-1)"),
                gate,
            )
            .expect("Can no send GetBlock to the gate");

        let mut buf = [0u8; 256];
        loop {
            let (_, sender) = self.socket.recv_from(&mut buf).expect("Error recv block");
            if sender != gate {
                continue;
            }
            if let Packet::Block(b) = deserialize(&mut buf).expect("Can not deserilize block") {
                return b;
            }
        }
    }

    //network:: get block       --> ??
    fn get_chain(&self, gate: SocketAddr) -> Option<Vec<Block>> {
        let last_block = self.get_block(-1, gate);
        let mut chain = vec![];
        let (height, nonce) = last_block.get_height_nonce();
        println!("Height {}", height);
        if height == 0 && nonce != 0 {
            return None;
        }
        if height > 0 {
            for i in 0..height {
                let block = self.get_block(i as i64, gate);
                let (h, n) = block.get_height_nonce();
                if (h != i) || (h == 0 && n != 0) {
                    println!("{} {} {}", i, h, n);
                    return None;
                }
                chain.push(block);
            }
        }
        chain.push(last_block);
        println!("get the chain : {:?}", chain);

        Some(chain)
    }

    //struc miner avec Blockaine && Kamelia dedant qui sera initialiser
    //donc serait une structure miner
    //en Miner::new
    fn setup_mine(&self, gate: SocketAddr) {
        let me_clone: Node = self.clone();

        let mut peers: Vec<SocketAddr>;
        let mut chain: Vec<Block> = vec![]; //devrais etre une struc blockaine ?

        //si j'ai une gateway je send connection sur elle ?
        if !(gate == SocketAddr::from(([0, 0, 0, 0], 6021))) {
            //6021 devrais estre un static const (en gros un DEFINE pour plus de lisibilitée)
            self.socket
                .send_to(&serialize(&Packet::Connexion).unwrap(), gate)
                .expect("Error send Connecion Packet");
            //ces du network
            //Kamelia::new()
            //azer.last_peers() => on poura metre ça en backgroud apres
            peers = self.recive_peers();

            //on retreive la "blockhaine"
            //Blockchaine::new(gate) que l'on met dans cette structure miner
            chain = self.get_chain(gate).expect("Can not grap the chain");

            //return Miner avec son network de peers et ça blockaine
            println!("Catch a chain of {} lenght", chain.len());

            println!("Found {} peer", peers.len());
        } else {
            //devrais être une fonction Miner::new(Default:default)
            peers = vec![];
            peers.push(self.socket.local_addr().expect("Can not catch the ip "));
            chain.push(Block::new());
        }

        let should_stop = Arc::new(Mutex::new(false));

        // let peer: Arc<Mutex<Vec<SocketAddr>>> = Arc::new(Mutex::new(vec![
        //     SocketAddr::from(([127, 0, 0, 1], 6021)),
        //     SocketAddr::from(([127, 0, 0, 2], 6021)),
        // ]));

        //complexitée dans Blockhaine
        let starting_block = chain.last().unwrap().clone();
        let peer = Arc::new(Mutex::new(peers));

        let (rx, tx) = mpsc::channel();
        let share = Shared::new(peer, should_stop, chain);
        let share_copy = share.clone();

        thread::spawn(move || {
            me_clone.listen(share_copy, rx);
        });

        //serait Miner::start
        self.mine(share, starting_block, tx);
    }

    //Miner::start(&self) //plus simple ?
    pub fn mine(&self, share: Shared, mut block: Block, tx: mpsc::Receiver<Block>) {
        loop {
            println!("The block is {:?} ", block);

            match block.generate_block_stop(self.id, &share.should_stop, "It is a quote") {
                Some(mut new_block) => {
                    println!("I found the new_block !!!");
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
                        let block_sera: Vec<u8> = serialize(&Packet::Block(new_block.clone()))
                            .expect("Error serialize new_block");
                        for addr in peer.iter().filter(move |p| {
                            **p != self.socket.local_addr().expect("Can not take local addr")
                        }) {
                            self.send_block(&block_sera, *addr);
                        }
                    }
                    block = new_block;
                }
                None => {
                    println!("An other found the block");
                    block = tx
                        .recv()
                        .expect("Error block can't be read from the channel");
                    {
                        let mut val = share.should_stop.lock().unwrap();
                        *val = false;
                    }
                }
            }
        }
    }
    //serait miner::send(&self) qui ferait un Kamelia::publish(Blockaine::lastblock())
    fn send_block(&self, block: &Vec<u8>, addr: SocketAddr) {
        self.socket
            .send_to(&block, addr)
            .expect("Error to send the block");
    }

    pub fn get_ip(&self) -> SocketAddr {
        self.socket
            .local_addr()
            .expect("Error the catch the ip from the socket")
    }

    //important d'avoir une structure pour les transa avec plein de check into algo qui store la structure  --> pour moi pas besoin de check si on envoit c'est les miner qui check
    pub fn send_transactions(&self, gate: SocketAddr, to: u64, count: u32) {
        // let him = Node::create(to);
        let transa = Transaction::new(0, to, count);
        let transa =
            serialize(&Packet::Transaction(transa)).expect("Error serialize transactions ");
        self.socket
            .send_to(&transa, gate)
            .expect("Error send transaction ");
    }

    //kamelia things
    fn ask_recive_peer(&self, gate: SocketAddr) -> Vec<SocketAddr> {
        let serialize_getpeer = serialize(&Packet::GetPeer).expect("Error serialize GetPeers");

        self.socket
            .send_to(&serialize_getpeer, gate)
            .expect("Error sending getPeers");

        self.recive_peers()
    }

    //devrais être dans un struct network
    fn recive_peers(&self) -> Vec<SocketAddr> {
        let mut buffer = [0u8; 256]; //on veux 255 addres max //<= a cahnger

        let (_, _remote) = self.socket.recv_from(&mut buffer).expect("Error recvfrom ");

        loop {
            if let Packet::RepPeers(peer) = deserialize(&buffer).expect("Error deserialize ") {
                return peer;
            }

            let (_, _remote) = self.socket.recv_from(&mut buffer).expect("Error recvfrom ");
        }
    }

    //dans network
    fn check_keep_alive(&self, peer: &mut HashMap<SocketAddr, Duration>, time: Duration) {
        let clone = peer.clone();
        for (p, t) in clone {
            if time - t > Duration::from_secs(120) {
                peer.remove(&p);
                println!("Remove the peer {}", p);
            } else if time - t > Duration::from_secs(60) {
                println!("Send a keep alive to {}", p);
                let seria_keep = serialize(&Packet::Keepalive).unwrap();
                self.socket
                    .send_to(&seria_keep, p)
                    .expect("Can not send keep alive");
            }
        }
    }
}

//dans transaction
fn verif_transa(share: Shared, transa: Transaction) {
    //verification

    let mut val = share.transaction.lock().unwrap();
    (*val).push(transa);
}

// serait dans emul      --> c'est quoi emul ?
pub fn p2p_simulate() {
    let mut nodes = vec![
        Node::create(1, String::from("27.0.0.1")),
        Node::create(2, String::from("27.0.0.2")),
        Node::create(3, String::from("27.0.0.3")),
    ];

    for node in &mut nodes {
        node.run_listen();
    }

    for node in nodes.iter_mut().enumerate() {
        node.1.run_send(1);
        node.1.run_send(2);
        node.1.run_send(3);
    }
}
//comment ça ?      --> ca ca sert a rien
pub fn detect_interlock() {
    for _ in [..10] {
        // Specify the timeout duration in milliseconds
        let timeout_duration_ms = 1500;

        // Spawn a new thread to perform the time-consuming operation
        let handle = thread::spawn(move || {
            // Perform the time-consuming operation here
            p2p_simulate();
        });

        // Wait for the timeout duration
        thread::sleep(Duration::from_millis(timeout_duration_ms));

        // Check if the spawned thread has finished executing
        if handle.join().is_err() {
            // Timeout exceeded, the test should fail
            assert!(false, "Timeout exceeded!");
        }
    }
}

fn update_peer_share(shared: &mut MutexGuard<Vec<SocketAddr>>, peer: Vec<SocketAddr>) {
    **shared = peer;
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use super::*;

    #[test]
    fn p2p_test() {
        p2p_simulate();
        assert!(true);
    }

    #[test]
    //d'ont work idk
    fn p2p_deadlock() {
        detect_interlock();
    }

    //#[test]
    // fn sendrecive_block() {
    //     let block = Block::new(vec![]);
    //     let me = Node::create(1,String::from("Isa"));

    //     me.send_block(&block, me.get_ip());
    //     let new_block = me.recive_block().unwrap();

    //     assert_eq!(block::hash(block), block::hash(new_block));
    // }
}
