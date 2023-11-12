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
use anyhow::{anyhow, Context, Result};
use dryoc::sign::PublicKey;
use tracing::{info, debug,warn,trace};

pub struct TransaInfo {
    pub ammount: u64,
    pub destination: PublicKey,
}

pub struct Client {
    user: User,
    networking: Network,
    transa_info: TransaInfo,
}

impl Client {
    pub fn new(networking: Network, user: User, destination: PublicKey, ammount: u64) -> Self{
        Self {
            user,
            networking,
            transa_info: TransaInfo {
                ammount,
                destination,
            },
        }
    }

    /// create empty wallet annd write it
    pub fn new_wallet(path: &str) -> Result<()> {
        let user = user::User::new_user(path);
        debug!("new wallet:\n{}",user);
        user.save()
    }

    /// ask to all peer balance and take first balance received and update
    fn refresh_wallet(&mut self) -> Result<()> {
        // pull utxo avaible

        let pubkey:PublicKey= self.user.get_key().clone().into();
        debug!("Ask for wallet value for {:?}",pubkey);
        self.networking.send_packet(
            &Packet::Client(ClientPackect::ReqUtxo(
                pubkey,
            )),
            &self.networking.bootstrap,
        )?;

        // register utxo
        trace!("waiting receiving packet of wallet");
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

        
        
        self.refresh_wallet()?;
        info!("Wallet: {}",self.user);

        let transactionb = Transaction::create_transa_from(
            &mut self.user,
            self.transa_info.ammount,
            self.transa_info.destination,
        )
        .ok_or_else(|| anyhow!("You not have enought money"))?;

        info!("Transaction created : {}", transactionb);

        self.networking.send_packet(
            &Packet::Transaction(TypeTransa::Push(transactionb)),
            &self.networking.bootstrap,
        )?;
        self.user.save()
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;
    use crate::block_chain::{node::network::Network, user::User};
    use super::Client;

    #[test]
    fn make_transaction() {
        let bind = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let bootstrap = std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

        let net = Network::new(bootstrap, bind);
        let user = User::default();

        let cli = Client::new(net, user, Default::default(), 1);
        cli.start().unwrap();
    }
}
