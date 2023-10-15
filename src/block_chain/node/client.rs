// use crate::block_chain::node::{network::Network,NewTransaction};

use crate::{
    block_chain::{transaction::Transaction, blockchain::Blockchain, node::network::{Packet, ClientPackect}},
    friendly_name::{get_fake_id, get_friendly_name},
};

use super::network::Network;

pub struct TransaInfo{
    ammount : u64,
    destination : u64,
}

impl TransaInfo{
    pub fn new(ammount : u64, destination : u64) -> Self{
        TransaInfo { ammount: ammount, destination: destination }
    }
}

pub struct Client {
    name: String,
    networking: Network,
    transa_info : TransaInfo,
    
}

impl Client {
    pub fn new(networking: Network, destination: u64, secret: String, ammount: u64) -> Self {
        let name =
            get_friendly_name(networking.get_socket()).expect("generation name from ip imposble");

        Self { name, networking, transa_info : TransaInfo::new(ammount, destination) }
    }
    pub fn start(self) {
        let ip = self.networking.get_socket();
        let id = get_fake_id(&self.name);

        // let blockaine = Blockchain::default();
        // let transaction = Transaction::new_online(&blockaine, 10, 10, 10);

        self.networking.send_packet(&Packet::Client(ClientPackect::ReqUtxo(id)), &self.networking.bootstrap);


        let mut myutxo; 
        loop {
            match self.networking.recv_packet_true_function().0 {
                Packet::Client(ClientPackect::RespUtxo(utxo)) => {myutxo = utxo;break;}
                _ => continue,
            }
        };  

        let transactionb = Transaction::new_offline(&myutxo, 10, 555);

        println!("Transaction created : {:?}",transactionb);
    }
}
