use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use super::block::{*,Block};

////////////////////ça veux dire quoi ça sert a quoi ?              --> il y a des pas mal de chose qui on besoin d'être clone ici c'ets ce qui est partager entre les thread du node
///////// faire une structure intermédiaire un éta chared ça peut être bien mais jsp a quoi ça serrt iuci
#[derive(Debug)]
pub struct Shared {
    pub peer: Arc<Mutex<Vec<SocketAddr>>>,
    pub should_stop: Arc<Mutex<bool>>,
    pub transaction: Arc<Mutex<Vec<Transaction>>>,
    pub chain: Arc<Mutex<Vec<Block>>>,
}

impl Clone for Shared {
    fn clone(&self) -> Self {
        Shared {
            peer: self.peer.clone(),
            should_stop: self.should_stop.clone(),
            transaction: self.transaction.clone(),
            chain: self.chain.clone(),
        }
    }
}

impl Shared {
    pub fn new(
        peer: Arc<Mutex<Vec<SocketAddr>>>,
        should_stop: Arc<Mutex<bool>>,
        chain: Vec<Block>,
    ) -> Shared {
        Shared {
            peer: peer,
            should_stop: should_stop,
            transaction: Arc::new(Mutex::new(vec![])),
            chain: Arc::new(Mutex::new(chain)),
        }
    }
}
