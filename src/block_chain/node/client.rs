use super::super::user;
use super::network::Network;
use crate::{
    block_chain::{
        node::network::{ClientPackect, Packet, TypeTransa},
        transaction::Transaction,
    },
    friendly_name::get_friendly_name,
};
use anyhow::{Context, Result};

pub struct TransaInfo {
    ammount: u64,
    destination: u64,
    from: u64,
}

impl TransaInfo {
    pub fn new(ammount: u64, destination: u64, from: u64) -> Self {
        TransaInfo {
            ammount,
            destination,
            from,
        }
    }
}

pub struct Client {
    name: String,
    networking: Network,
    transa_info: TransaInfo,
}

impl Client {
    pub fn new(
        networking: Network,
        destination: u64,
        _secret: String,
        ammount: u64,
        from: u64,
    ) -> Result<Self> {
        let name = get_friendly_name(networking.get_socket())
            .context("generation name from ip imposble")?;

        Ok(Self {
            name,
            networking,
            transa_info: TransaInfo::new(ammount, destination, from),
        })
    }
    pub fn start(self) -> Result<()> {
        let mut user = user::User::new_user("cli.usr");

        //force to save (debug)
        self.networking.send_packet(
            &Packet::Client(ClientPackect::ReqSave),
            &self.networking.bootstrap,
        );

        self.networking.send_packet(
            &Packet::Client(ClientPackect::ReqUtxo(self.transa_info.from)),
            &self.networking.bootstrap,
        );

        let myutxo = self.networking.recv_packet_utxo_wallet();
        user.refresh_wallet(myutxo.clone());

        let transactionb = Transaction::new_offline(&myutxo, 10, 555).context("You not have enought money")?;
        println!("Transaction created : {:?}", transactionb);

        self.networking.send_packet(
            &Packet::Transaction(TypeTransa::Push(transactionb)),
            &self.networking.bootstrap,
        );
        user.save()
    }
}
