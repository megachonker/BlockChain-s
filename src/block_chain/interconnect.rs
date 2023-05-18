
use std::net::{UdpSocket, SocketAddr};
use std::thread;

//remplacer par un énume les noms

#[derive(Clone)]
#[repr(u8)]
enum Name {
    Isa,
    Net,
    Max,
    Lex,
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

}

struct Node{
    name:Name,
    socket:UdpSocket,
}


impl Node {
    pub fn create(id:Name) -> Node{
        let socket = UdpSocket::bind(id.get_ip()).expect(&(id.get_name()+": couldn't bind to address:"));//1
        Node{
            name: id,
            socket,
        }
    }

    fn run_listen(&self){
        let socket = self.socket.try_clone().expect("fail to clone socket");
        let name = self.name;
        thread::spawn(move || {
            let mut buf = [0; 3];
            loop {
                let (amt, src) = socket.recv_from(&mut buf).expect(&format!("{} Failed to receive data", name.get_str())); //2
                println!("Node {} received: {}",name.get_name(),String::from_utf8_lossy(&buf[..amt]));
                socket.send_to(name.get_name().as_bytes(), src).expect(&("Failed to send data to:".to_owned()+&name.get_name()));//3
            }
        });
    }

    fn run_send(&mut self,id:Name){
        println!("Node {} to {} send: {}",self.name.get_name(),id.get_name(),self.name.get_name());
        self.socket.send_to(self.name.get_name().as_bytes(), id.get_ip()).expect(&("Failed to send data to:".to_owned()+&self.name.get_name()));//3
    }
}

pub fn p2p_simulate(){

    let mut I = Node::create(Name::Isa);
    let mut L = Node::create(Name::Lex);
    let mut M = Node::create(Name::Max);

    I.run_listen();
    L.run_listen();
    M.run_listen();

    I.run_send(Name::Max);
    // L.run_send(id)
    // M.run_send(id)

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p2p_test() {
        p2p_simulate();
        assert!(true);        
    }
}
