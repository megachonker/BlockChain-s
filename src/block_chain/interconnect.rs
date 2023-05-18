
use std::net::{UdpSocket, SocketAddr,IpAddr};
use std::thread;
use std::sync::{Arc,Barrier};
use std::time::{Duration,Instant};
//remplacer par un énume les noms


#[derive(Clone)]
#[repr(u8)]
enum Name {
    Isa=1,
    Net=2,
    Max=3,
    Lex=4,
}

impl Copy for Name {}

impl  Name {
    fn get_name(&self)->String{
        self.get_str().to_string()
    }
    fn get_str(&self)->&str{
        match self {
            Name::Isa => "Isa",
            Name::Net => "Net",
            Name::Max => "Max",
            Name::Lex => "Lex",
        }
    }
    fn get_number(&self) -> u8{
        *self as u8
    }

    fn get_ip(&self) -> SocketAddr{
        SocketAddr::from(([127, 0, 0, self.get_number()], 6021))
    }

    fn from_ip(addr:&SocketAddr)->Name{
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

struct Node{
    name:Name,
    socket:UdpSocket,
    barrier:Arc<Barrier>,
}


impl Node {
    pub fn create(name:Name) -> Node{
        let socket = UdpSocket::bind(name.get_ip()).expect(&(name.get_name()+": couldn't bind to address:"));//1
        let  barrier = Arc::new(Barrier::new(2));
        Node{
            name,
            socket,
            barrier
        }
    }

    pub fn run_listen(&self){
        let socket = self.socket.try_clone().expect("fail to clone socket");
        let name = self.name;
        let barrier = self.barrier.clone();
    
        let mut buf = [0; 3];
        thread::spawn(move || {


        //CASSER La qsdmlfjhnqsdfiogu avec timeout

        socket.set_read_timeout(Some(Duration::new(0, 1000000))).expect("set_read_timeout call failed");
        println!("{} Whait Timeout: ",name.get_name());
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                barrier.wait(); // Unblock the send operation
                println!(
                    "Node {} from {} received: {}",
                    name.get_name(),
                    Name::from_ip(&src).get_name(),
                    String::from_utf8_lossy(&buf[..amt])
                );
                socket
                    .send_to(name.get_name().as_bytes(), src)
                    .expect("Failed to send data");
            }
            Err(_) => {
                // Handle timeout here
                barrier.wait(); // Unblock the send operation even if no packet received
                println!("{} unlock Timeout",name.get_name());
            }
        }
        socket.set_read_timeout(None).expect("set_read_timeout call failed");

        println!("{}: started",name.get_name());
        loop {
            let (amt, src) = socket.recv_from(&mut buf).expect(&format!("{} Failed to receive data", name.get_str())); //2
            barrier.wait();
            println!("Node {} from {} received: {}",name.get_name(),Name::from_ip(&src).get_name(),String::from_utf8_lossy(&buf[..amt]));
            socket.send_to(name.get_name().as_bytes(), src).expect(&("Failed to send data to:".to_owned()+&name.get_name()));//3
        }
    });
    }

    fn run_send(&mut self,id:Name){
        self.barrier.wait();
        println!("Node {} to {} send: {}",self.name.get_name(),id.get_name(),self.name.get_name());
        self.socket.send_to(self.name.get_name().as_bytes(), id.get_ip()).expect(&("Failed to send data to:".to_owned()+&self.name.get_name()));//3
    }

    fn quit(&mut self){
        ;
    }
}

pub fn p2p_simulate(){
    let mut nodes = vec![
        Node::create(Name::Isa),
        Node::create(Name::Lex),
        Node::create(Name::Max),
    ];

    for node in &mut nodes {
        node.run_listen();
    }

    for (node) in nodes.iter_mut().enumerate() {
        node.1.run_send(Name::Isa);
        node.1.run_send(Name::Lex);
        node.1.run_send(Name::Max);
    }
}

pub fn detect_interlock(){
    for _ in [..10]{
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
    use super::*;

    #[test]
    fn p2p_test() {
        p2p_simulate();
        assert!(true);        
    }

    #[test]
//d'ont work idk
    fn p2p_deadlock(){
        detect_interlock();
    }
}
