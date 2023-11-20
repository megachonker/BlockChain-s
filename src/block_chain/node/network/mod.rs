use std::{
    collections::HashSet,
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use bincode::{deserialize, serialize};
use dryoc::sign::PublicKey;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::block_chain::{
    block::Block,
    node::server::ClientEvent,
    transaction::{Transaction, Utxo},
};

use super::server::{Event, NewBlock};

#[derive(Debug)]
pub struct Network {
    pub bootstrap: SocketAddr,
    binding: UdpSocket,
    peers: Arc<Mutex<HashSet<SocketAddr>>>,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "initial bootstrap: {}", self.bootstrap)?;
        writeln!(f, "socket binded: {:?}", self.binding)?;
        writeln!(f, "Peers list:")?;
        let socketlist = self.peers.lock().unwrap().clone();
        for peer in socketlist {
            write!(f, "{}", peer)?;
        }
        write!(f, "")
    }
}

impl Clone for Network {
    fn clone(&self) -> Self {
        Self {
            bootstrap: self.bootstrap,
            binding: self.binding.try_clone().unwrap(),
            peers: self.peers.clone(),
        }
    }
}



#[derive(Serialize, Deserialize, Debug)]
pub enum TypePeer {
    List(HashSet<SocketAddr>), //we now how manny peers we want
    Request(usize),            //number of peers to ask
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TypeBlock {
    Lastblock,
    Hash(i128),
    Block(Block),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TypeTransa {
    Push(Transaction),
    Req(u64),
    Ans(Vec<Utxo>),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ClientPackect {
    ReqUtxo(PublicKey),      //Request for the UTXO of u64
    RespUtxo((usize, UtxoLocation,Utxo)), //the response of RqUtxo : (number of utxo remains, the utxo -> (0,utxo..) is the last)
    ReqSave,                 //force save (debug)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    Keepalive,
    Transaction(TypeTransa),
    Block(TypeBlock),
    Peer(TypePeer),
    Client(ClientPackect),
    None,
}

// whole network function inside it
// send packet with action do scan block ect get peers
impl Network {
    ///// USED BY ROUTER

    /// append transaction when enought transa send it to miner to create a new block
    fn transaction(transa: TypeTransa, net_transa_tx: &Sender<Event>) {
        info!("Recv a newtransaction: {:?}", net_transa_tx);
        match transa {
            TypeTransa::Ans(_utxos) => { /* array of all utxo append */ }
            TypeTransa::Push(transaction) => {
                // let (key, msg) = transaction.clone().into_parts();

                // let transa: Transaction = bincode::deserialize(&msg).unwrap();
                // warn!("error  on deserialize rtransaction");

                // // information en double
                // // on peut choper la clef and la signature ET dans la structure.
                // if key.to_vec() != transa.sender_pubkey{
                //     panic!("NIQUE")
                // }

                // if let Err(e) = transaction.verify(&transa.sender_pubkey) {
                //     warn!("signature false! {:?}", e);
                // } else {
                net_transa_tx.send(Event::Transaction(transaction)).unwrap()
                // }
            }
            TypeTransa::Req(_userid) => { /* get all transa and filter by user id*/ }
        }

        //check préliminer avant d'utiliser un prim

        //send la transa au miner
    }

    /// If received empty reply with all peers
    /// If received append new peers to the list
    /// need to manage duplication
    fn peers(&mut self, peers: TypePeer, source: SocketAddr) {
        self.peers.lock().unwrap().insert(source);
        match peers {
            TypePeer::List(new_peers) => {
                /*test peers befort pls */
                warn!("Network LIST peers recus");
                self.peers.lock().unwrap().extend(new_peers);
                debug!("apres {:?}", self.peers.lock().unwrap());
            }
            TypePeer::Request(_sizer) => {
                warn!("Network Request Peers");
                self.send_packet(
                    &Packet::Peer(TypePeer::List(self.peers.lock().unwrap().clone())),
                    &source,
                )
                .unwrap();
            }
        }
        //on add aussi le remote dans la liste
    }

    /// Ping Pong but update timestamp
    fn keepalive(&self, sender: SocketAddr) -> Result<()> {
        // do here the timestamp things
        // todo!();
        self.send_packet(&Packet::Keepalive, &sender).map(|_u| ())
    }

    fn block(
        &mut self,
        typeblock: TypeBlock,
        sender: SocketAddr,
        network_server_tx: &Sender<Event>,
    ) {
        match typeblock {
            TypeBlock::Lastblock => {
                network_server_tx
                    .send(Event::HashReq((-1, sender)))
                    .unwrap();
            }
            TypeBlock::Block(block) => {
                // debug!("Block get:{}", block);

                network_server_tx
                    .send(Event::NewBlock(NewBlock::Network(block)))
                    .unwrap();
            }
            TypeBlock::Hash(number) => {
                network_server_tx
                    .send(Event::HashReq((number, sender)))
                    .unwrap();
            }
        }
    }

    pub fn start(self, event_tx: Sender<Event>) -> Result<()> {
        info!("network start");

        let mut self_cpy = self.clone();
        thread::Builder::new()
            .name("Net-Router".to_string())
            .spawn(move || {
                info!("Net-Router");
                loop {
                    let (message, sender) =
                        Self::recv_packet(&self_cpy.binding.try_clone().unwrap());
                    match message {
                        Packet::Transaction(transa) => Network::transaction(transa, &event_tx),
                        Packet::Peer(peers) => self_cpy.peers(peers, sender),
                        Packet::Keepalive => self_cpy.keepalive(sender).unwrap(),
                        Packet::Block(typeblock) => self_cpy.block(typeblock, sender, &event_tx),
                        Packet::Client(client_packet) => {
                            self_cpy.client(client_packet, sender, &event_tx).unwrap()
                        }
                        Packet::None => {}
                    }
                }
            })?;

        if self.bootstrap != SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6021)
        ////////////// need to be removed
        {
            self.send_packet(&Packet::Peer(TypePeer::Request(100)), &self.bootstrap)?; //to register and get peers
            self.send_packet(&Packet::Block(TypeBlock::Lastblock), &self.bootstrap)?;
        }
        Ok(())
    }

    /// Constructor of network
    pub fn new(bootstrap: IpAddr, binding: IpAddr) -> Self {
        let binding = UdpSocket::bind(SocketAddr::new(binding, 6021)).unwrap();
        let bootstrap = SocketAddr::new(bootstrap, 6021);
        Self {
            bootstrap,
            binding,
            peers: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// get socket from a network
    pub fn get_socket(&self) -> SocketAddr {
        self.binding
            .local_addr()
            .expect("Can not catch the SocketAddr")
    }

    /// verry cool send a packet
    pub fn send_packet(&self, packet: &Packet, dest: &SocketAddr) -> Result<usize> {
        let packet_serialized = serialize(&packet).context("Send: Cannot serialize Packet")?;
        self.binding
            .send_to(&packet_serialized, dest)
            .context("send: cannot send packet")
    }

    /// awsome send a packet broadcast
    pub fn broadcast(&self, packet: Packet) -> Result<()> {
        let peers = self.peers.lock().unwrap();

        for dest in peers.iter().filter(|&&x| x != self.get_socket()) {
            self.send_packet(&packet, dest)
                .context(format!("Failed to send to {}", dest))?;
        }

        Ok(())
    }

    /// awsome
    pub fn recv_packet(selff: &UdpSocket) -> (Packet, SocketAddr) {
        //faudrait éliminer les vecteur dans les structure pour avoir une taille prédictible

        const MAX_PACKET_SIZE: usize = 65507;
        let mut buf = [0u8; MAX_PACKET_SIZE]; //pourquoi 256 ??? <============= BESOIN DETRE choisie

        let (_, sender) = selff.recv_from(&mut buf).expect("Error recv block");
        let des = deserialize(&mut buf);
        if des.is_err() {
            error!("Can to deserialize packet");
            return (Packet::None, sender);
        }
        (des.unwrap(), sender)
    }

    /// wait for a wallet
    pub fn recv_packet_utxo_wallet(&self) -> Result<Vec<Utxo>> {
        let mut buf = [0u8; 256]; //pourquoi 256 ??? <============= BESOIN DETRE choisie
        let mut allutxo = vec![];
        loop {
            self.binding
                .set_read_timeout(Some(Duration::from_secs(1)))?;
            self.binding.recv_from(&mut buf).with_context(|| {
                format!(
                    "time out receiving wallet all server down ?\n\nDUMP Network:\n{}",
                    self
                )
            })?;
            let answer: Packet = deserialize(&mut buf).expect("Can not deserilize block");
            if let Packet::Client(ClientPackect::RespUtxo((size,position, utxo))) = answer {
                debug!("rev some wallet utxo: {} {}", size, utxo);
                allutxo.push((position,utxo));
                if size == 0 {
                    return Ok(allutxo);
                }
            }
        }
    }

    fn client(
        &self,
        client_packet: ClientPackect,
        sender: SocketAddr,
        event_tx: &Sender<Event>,
    ) -> Result<()> {
        match client_packet {
            ClientPackect::ReqUtxo(id_client) => {
                info!(
                    "Client ask utxo own by ({:?}) request UTXO ",
                    id_client.to_vec().get(..5).unwrap()
                );
                event_tx.send(Event::ClientEvent(ClientEvent::ReqUtxo(id_client), sender))?;
            }
            ClientPackect::RespUtxo(_) => {
                info!("Receive a response client packet but it is a server")
            }
            ClientPackect::ReqSave => {
                event_tx.send(Event::ClientEvent(ClientEvent::ReqSave, sender))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::block_chain::node::{
        network::{Network, Packet, TypeBlock},
        server::{Event, NewBlock},
    };
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::mpsc,
    };
    #[test]
    /// test client asking block
    /// test client recieved the block
    /// Test server added client
    fn create_blockchain() -> Result<()> {
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 1, 0, 2)), 6021);

        // New pair
        let client_ip = IpAddr::V4(Ipv4Addr::new(127, 1, 0, 2));
        let server_ip = IpAddr::V4(Ipv4Addr::new(127, 1, 0, 1));

        let client_bootstrap = IpAddr::V4(Ipv4Addr::new(127, 1, 0, 1));
        let server_bootstrap = IpAddr::V4(Ipv4Addr::new(127, 1, 1, 1));

        let client = Network::new(client_bootstrap, client_ip);
        let server = Network::new(server_bootstrap, server_ip);

        let (server_tx, server_rx) = mpsc::channel();
        let (client_tx, client_rx) = mpsc::channel();

        let tserver = server.clone();
        std::thread::spawn(move || {
            tserver.start(server_tx.clone()).unwrap();
            client.start(client_tx).unwrap();
        });

        let s1 = server_rx.recv().unwrap();

        assert_eq!(s1, Event::HashReq((-1, client_addr)));
        println!("{:?}", s1);

        server.send_packet(
            &Packet::Block(TypeBlock::Block(Default::default())),
            &client_addr,
        )?;

        let c1 = client_rx.recv().unwrap();
        assert_eq!(c1, Event::NewBlock(NewBlock::Network(Default::default())));
        println!("{:?}", c1);

        assert_eq!(
            *server.peers.lock().unwrap().get(&client_addr).unwrap(),
            client_addr
        );
        Ok(())
    }
}
