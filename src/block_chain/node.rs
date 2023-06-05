use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{Arc, Barrier, Mutex};
use std::time::{Duration, Instant};
use std::{default, thread};

use crate::shared::Shared;
use bincode::{deserialize, serialize};
use lib_block::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use clap::{arg, ArgAction, ArgMatches, Command, Parser};

use super::{block, shared};

//remplacer par un énume les noms

// mod block_chain {
//     pub use super::block_chain::Block;
// }

#[derive(Serialize, Deserialize, Debug)]
enum Packet {
    Keepalive,
    Transaction(Transaction),
    Block(Block),
    GetPeer,
    RepPeers(Vec<SocketAddr>),
}

#[derive(Clone)]
#[repr(u8)]
pub enum Name {
    Isa = 1,
    Net = 2,
    Max = 3,
    Lex = 4,
}

impl Copy for Name {}

impl Name {
    pub fn create(num: u8) -> Name {
        match num {
            1 => Name::Isa,
            2 => Name::Net,
            3 => Name::Max,
            4 => Name::Lex,
            _ => Name::Isa,
        }
    }
    pub fn creat_str(name: &str) -> Name {
        match name {
            "Isa" => Name::Isa,
            "Net" => Name::Net,
            "Max" => Name::Max,
            "Lex" => Name::Lex,
            _ => Name::Isa,
        }
    }

    fn get_name(&self) -> String {
        self.get_str().to_string()
    }
    fn get_str(&self) -> &str {
        match self {
            Name::Isa => "Isa",
            Name::Net => "Net",
            Name::Max => "Max",
            Name::Lex => "Lex",
        }
    }
    fn get_number(&self) -> u8 {
        *self as u8
    }

    fn get_ip(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, self.get_number()], 6021))
    }

    fn from_ip(addr: &SocketAddr) -> Name {
        match addr.ip() {
            IpAddr::V4(ipv4) => match ipv4.octets()[3] {
                1 => Name::Isa,
                2 => Name::Net,
                3 => Name::Max,
                4 => Name::Lex,
                _ => panic!("Invalid value"),
            },
            _ => panic!("Invalid IP address"),
        }
    }
}

pub struct Node {
    id: u64,
    socket: UdpSocket,
    barrier: Arc<Barrier>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn parse_args() -> ArgMatches {
    Command::new("NIC")
        .version("1.0")
        .author("Thompson")
        .about("A great Block Chain")
        .arg(
            arg!(-p --ip <IP> "Your IP:port for bind the socket")
                .required(false)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-r --receive <num> "The id of the receiver ")
                .required(false)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-s --sender  <num> "Your Id")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-m --mode  <MODE> "Wich mode (send, mine) ")
                .required(false)
                .action(ArgAction::Set)
                .default_value("mine"),
        )
        .arg(
            arg!(-g --gate <IP> "The IP:port of the entry point")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-c --count <count> "The value amount for the  transaction")
                .required(false)
                .action(ArgAction::Set)
                .default_value("0"),
        )
        .get_matches()
}

impl Node {
    pub fn start() -> Option<()> {
        let matches = parse_args();
        let me: Node = Node::create(
            matches
                .get_one::<String>("sender")?
                .parse::<u64>()
                .expect("Pas un entier"),
            String::from(matches.get_one::<String>("ip")?),
        );
        if matches.get_one::<String>("mode")? == "send" {
            me.send_transactions(
                matches.get_one::<String>("gate")?.parse().unwrap(),
                Name::creat_str(matches.get_one::<String>("receive")?),
                matches.get_one::<String>("count")?.parse::<u32>().unwrap(),
            )
        } else {
            me.setup_mine();
        }
        Some(())
    }

    pub fn create(id: u64, ip: String) -> Node {
        let socket = UdpSocket::bind(ip).expect("{id} couldn't bind to address:"); //1
        let barrier = Arc::new(Barrier::new(2));
        Node {
            id,
            socket,
            barrier,
        }
    }

    pub fn clone(&self) -> Node {
        let barrier = Arc::new(Barrier::new(2));

        Node {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            barrier: barrier,
        }
    }

    pub fn run_listen(&self) {
        let socket = self.socket.try_clone().expect("fail to clone socket");
        let id = self.id;
        let barrier = self.barrier.clone();

        let mut buf = [0; 3];
        thread::spawn(move || {
            //CASSER La qsdmlfjhnqsdfiogu avec timeout

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
                        Name::from_ip(&src).get_name(),
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
                    Name::from_ip(&src).get_name(),
                    String::from_utf8_lossy(&buf[..amt])
                );
                socket
                    .send_to("Here".as_bytes(), format!("17.0.0.{}", id))
                    .expect(&("Failed to send data to:"));
                //3
            }
        });
    }

    fn run_send(&mut self, id: u64) {
        self.barrier.wait();
        println!("Node {} to {} send: {}", self.id, id, self.id);
        self.socket
            .send_to("Here".as_bytes(), format!("17.0.0.{}", id))
            .expect(&("Failed to send data to:"));
        //3
    }

    fn quit(&mut self) {}

    fn send_block(&self, block: &Vec<u8>, addr: SocketAddr) {
        self.socket
            .send_to(&block, addr)
            .expect("Error to send the block");
    }

    fn recive_block(&self) -> Option<Block> {
        let mut buf: [u8; 100] = [0; 100];
        self.socket.recv_from(&mut buf).unwrap();
        let new_block = Block::from_bytes(&mut buf)?;
        Some(new_block)
    }

    fn hear(&self) -> Packet {
        let mut buffer = vec![0u8; 1024]; //MAXSIZE a def ??

        let (offset, _) = self.socket.recv_from(&mut buffer).expect("err recv_from");
        deserialize(&buffer[..offset]).expect("errreur deserial")
    }

    pub fn listen(&self, share: Shared, rx: mpsc::Sender<Block>) {
        loop {
            match self.hear() {
                Packet::Keepalive => {}
                Packet::Block(block) => {
                    if block.check() {
                        {
                            let mut chain = share.chain.lock().unwrap();
                            chain.push(block.clone());

                        }
                        rx.send(block).unwrap();
                        {
                            let mut val = share.should_stop.lock().unwrap();
                            *val = true;
                        }
                        let mut val = share.transaction.lock().unwrap();
                        (*val) = vec![]; //on remet a zero les transactions peut être a modiifier
                    }
                }
                Packet::Transaction(trans) => {
                    println!("Recive a new transactions");
                    let mut val = share.transaction.lock().unwrap();
                    (*val).push(trans);
                    //share the new transa ???
                }
                _ => {
                }
            }
        }
    }

    fn setup_mine(&self) {
        let me_clone: Node = self.clone();

        let should_stop = Arc::new(Mutex::new(false));

        let peer: Arc<Mutex<Vec<SocketAddr>>> = Arc::new(Mutex::new(vec![
            SocketAddr::from(([127, 0, 0, 1], 6021)),
            SocketAddr::from(([127, 0, 0, 2], 6021)),
        ]));

        let (rx, tx) = mpsc::channel();
        let share = Shared::new(peer, should_stop, vec![]);
        let share_copy = share.clone();

        let thread = thread::spawn(move || {
            me_clone.listen(share_copy, rx);
        });

        let starting_block = Block::new(vec![]);

        self.mine(share, starting_block, tx);
    }

    pub fn mine(&self, share: Shared, mut block: Block, tx: mpsc::Receiver<Block>) {
        loop {
            println!("The block is {:?} ", block);

            match block.generate_block_stop(self.id, &share.should_stop) {
                Some(mut block) => {
                    println!("I found the block !!!");
                    {
                        //add the transactions see during the mining
                        let val = share
                            .transaction
                            .lock()
                            .expect("Error during lock of transaction");
                        block = block.set_transactions((*val).clone());
                    }
                    {
                        let peer = share.peer.lock().unwrap();
                        let block_sera: Vec<u8> =
                            serialize(&Packet::Block(block)).expect("Error serialize block");
                        for addr in &*peer {
                            self.send_block(&block_sera, *addr);
                        }
                    }
                }
                None => {
                    println!("An other found the block")
                }
            }

            block = tx
                .recv()
                .expect("Error block can't be read from the channel");
            {
                let mut val = share.should_stop.lock().unwrap();
                *val = false;
            }
        }
    }

    pub fn get_ip(&self) -> SocketAddr {
        self.socket
            .local_addr()
            .expect("Error the catch the ip from the socket")
    }

    pub fn send_transactions(&self, gate: SocketAddr, to: Name, count: u32) {
        // let him = Node::create(to);
        let transa = Transaction::new(0, 1, count);
        let transa =
            serialize(&Packet::Transaction(transa)).expect("Error serialize transactions ");
        self.socket
            .send_to(&transa, gate)
            .expect("Error send transaction ");
    }
}

pub fn p2p_simulate() {
    let mut nodes = vec![
        Node::create(1, String::from("27.0.0.1")),
        Node::create(2, String::from("27.0.0.2")),
        Node::create(3, String::from("27.0.0.3")),
    ];

    for node in &mut nodes {
        node.run_listen();
    }

    for (node) in nodes.iter_mut().enumerate() {
        node.1.run_send(1);
        node.1.run_send(2);
        node.1.run_send(3);
    }
}

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

    #[test]
    fn sendrecive_block() {
        let block = Block::new(vec![]);
        let me = Node::create(Name::Isa);

        me.send_block(&block, me.get_ip());
        let new_block = me.recive_block().unwrap();

        assert_eq!(block::hash(block), block::hash(new_block));
    }
}
