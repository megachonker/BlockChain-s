// use crate::block_chain::node::{network::Network,NewTransaction};
use crate::{friendly_name::*, block_chain::node::Node};

use super::{network::Network, NewTransaction};

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
        let name = get_friendly_name(networking.get_socket()).expect("generation name from ip imposble");
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
        let ip = self.networking.get_socket();
        let id = get_fake_id(&self.name);

        let me: Node = Node::create(id,ip); // <=== éclater au sol
        me.send_transactions(self.networking.bootstrap,self.transaction.destination,self.transaction.ammount as u32);
        println!("Client started name is {} fack id{}", self.name,get_fake_id(&self.name))
    }
    
}