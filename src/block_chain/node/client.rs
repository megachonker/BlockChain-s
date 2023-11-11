use super::super::user;
use super::network::Network;
use crate::{
    block_chain::{
        node::network::{ClientPackect, Packet, TypeTransa},
        transaction::Transaction,
        user::User,
    },
    friendly_name::get_friendly_name,
};
use anyhow::{Context, Result, anyhow};
use dryoc::sign::PublicKey;
use tracing::info;

pub struct TransaInfo {
    pub ammount: u64,
    pub destination: PublicKey,
}

pub struct Client {
    user: User,
    name: String,
    networking: Network,
    transa_info: TransaInfo,
}

impl Client {
    pub fn new(networking: Network, destination: PublicKey, ammount: u64) -> Result<Self> {
        let name = get_friendly_name(networking.get_socket())
            .context("generation name from ip imposible PATH?")?;
        let user = user::User::new_user("cli.usr");

        Ok(Self {
            name,
            user,
            networking,
            transa_info: TransaInfo {
                ammount,
                destination,
            },
        })
    }

    /// create empty wallet annd write it
    pub fn new_wallet(path: &str) -> Result<()> {
        let user = user::User::new_user(path);
        user.save()
    }

    /// ask to all peer balance and take first balance received and update
    fn refresh_wallet(&mut self) -> Result<()>{
        // pull utxo avaible

        self.networking.send_packet(
            &Packet::Client(ClientPackect::ReqUtxo(self.user.get_key().public_key.clone())),
            &self.networking.bootstrap,
        )?;

        // register utxo
        let myutxo = self.networking.recv_packet_utxo_wallet();

        self.user.refresh_wallet(myutxo);
        Ok(())
    }

    pub fn start(mut self) -> Result<()> {
        
        // json blockainne
        self.networking.send_packet(
            &Packet::Client(ClientPackect::ReqSave),
            &self.networking.bootstrap,
        )?;

        let transactionb = Transaction::create_transa_from(
            &mut self.user,
            self.transa_info.ammount,
            self.transa_info.destination,
        ).ok_or_else(||anyhow!("You not have enought money"))?;

        info!("Transaction created : {:?}", transactionb);

        self.networking.send_packet(
            &Packet::Transaction(TypeTransa::Push(transactionb)),
            &self.networking.bootstrap,
        )?;
        self.user.save()
    }
}


#[cfg(test)]
mod  test{

    #[test]
    fn make_transaction(){
        
    }
}