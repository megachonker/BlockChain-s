use super::super::acount;
use super::network::Network;
use crate::block_chain::{
    acount::Acount,
    node::network::{ClientPackect, Packet, TypeTransa},
    transaction::{Amount, Transaction},
};
use anyhow::{Result, Context};
use dryoc::sign::PublicKey;
use tracing::{debug, info, trace};

pub struct TransaInfo {
    pub ammount: Amount,
    pub destination: PublicKey,
}

pub struct Client {
    pub user: Acount,
    networking: Network,
    transa_info: TransaInfo,
}

impl Client {
    pub fn new(networking: Network, user: Acount, destination: PublicKey, ammount: Amount) -> Self {
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
        let user = acount::Acount::new_user(path);
        debug!("new wallet:\n{}", user);
        user.save()
    }

    /// ask to all peer balance and take first balance received and update
    pub fn refresh_wallet(&mut self) -> Result<Acount> {
        info!("asked to refresh the wallet");
        
        let mut new_wallet = vec![];
        
        // pull utxo avaible
        let pubkey: Vec<PublicKey> = self.user.get_key().iter().map(|k| k.0.public_key.clone()).collect();

        for pk in    pubkey{
            debug!("Ask for wallet value for {:?}", pk);
            self.networking.send_packet(
                &Packet::Client(ClientPackect::ReqUtxo(pk)),
                &self.networking.bootstrap,
            )?;
    
            // register utxo
            // on pourait start un demon en background
            trace!("waiting receiving packet of wallet");
            new_wallet.append(&mut self.networking.recv_packet_utxo_wallet()?);
        }

        self.user.refresh_wallet(new_wallet)?;
        self.user.clone().save()?;
        Ok(self.user.clone())
    }

    pub fn start(mut self) -> Result<()> {
        // json blockainne
        self.networking.send_packet(
            &Packet::Client(ClientPackect::ReqSave),
            &self.networking.bootstrap,
        )?;

        self.refresh_wallet()?;

        info!("Wallet:\n{}", self.user);
        
        let transactionb = Transaction::new_transaction(&mut self.user,self.transa_info.ammount, self.transa_info.destination).context("You not have enought money")?;

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

    // #[test]
    // fn make_transaction() {
    //     let bind = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    //     let bootstrap = std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    //     let net = Network::new(bootstrap, bind);
    //     let user = Acount::default();

    //     let cli = Client::new(net, user, Default::default(), 1);
    //     cli.start().unwrap();
    // }
    //tester  le sold
}
