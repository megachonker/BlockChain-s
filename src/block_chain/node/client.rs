// use crate::block_chain::node::{network::Network,NewTransaction};

use crate::{
    block_chain::{transaction::Transaction, blockchain::Blockchain},
    friendly_name::{get_fake_id, get_friendly_name},
};

use super::network::Network;

pub struct Client {
    name: String,
    networking: Network,
    //un client va faire une action
    //// le client pourait etre un worker qui effectue les action dicter par un front end
    /*enum action{ // <= peut etre un flux comme un mscp
        balance //calcule argent compte
        transaction(destination)
    }*/
}

impl Client {
    pub fn new(networking: Network, destination: u64, secret: String, ammount: f64) -> Self {
        let name =
            get_friendly_name(networking.get_socket()).expect("generation name from ip imposble");

        Self { name, networking }
    }
    pub fn start(self) {
        let ip = self.networking.get_socket();
        let id = get_fake_id(&self.name);

        // let blockaine = Blockchain::default();
        // let transaction = Transaction::new_online(&blockaine, 10, 10, 10);

        let transactionb = Transaction::new_offline(&vec![], 10, 555);

        println!("vs\n{}",transactionb);
    }
}
