// use crate::block_chain::node::{network::Network,NewTransaction};



use crate::{
    block_chain::{transaction::Transaction, node::network::{Packet, ClientPackect, TypeTransa}},
    friendly_name::{get_friendly_name},
};

use super::network::Network;

pub struct TransaInfo{
    ammount : u64,
    destination : u64,
    from : u64
}

impl TransaInfo{
    pub fn new(ammount : u64, destination : u64,from : u64) -> Self{
        TransaInfo { ammount: ammount, destination: destination, from : from }
    }
}

pub struct Client {
    name: String,
    networking: Network,
    transa_info : TransaInfo,
    
}

impl Client {
    pub fn new(networking: Network, destination: u64, _secret: String, ammount: u64,from : u64) -> Self {
        let name =
            get_friendly_name(networking.get_socket()).expect("generation name from ip imposble");

        Self { name, networking, transa_info : TransaInfo::new(ammount, destination,from) }
    }
    pub fn start(self) {

        // let blockaine = Blockchain::default();
        // let transaction = Transaction::new_online(&blockaine, 10, 10, 10);

        self.networking.send_packet(&Packet::Client(ClientPackect::ReqUtxo(self.transa_info.from)), &self.networking.bootstrap);



        let myutxo; 
        loop {
            match self.networking.recv_packet_true_function().0 {
                Packet::Client(ClientPackect::RespUtxo(utxo)) => {myutxo = utxo;break;}
                _ => continue,
            }
        };  

        let transactionb = Transaction::new_offline(&myutxo, 10, 555);

        if transactionb == None{
            println!("You not have enought money");
            return;
        }
        let transactionb = transactionb.unwrap();
        println!("Transaction created : {:?}",transactionb);

        self.networking.send_packet(&Packet::Transaction(TypeTransa::Push(transactionb)), &self.networking.bootstrap);
    }
}
