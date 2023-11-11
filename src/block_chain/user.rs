use std::fmt::Display;

use super::{
    node::network::Packet,
    transaction::{Transaction, Utxo},
};
use anyhow::{Context, Error, Result};
use dryoc::{keypair, sign::*, types::StackByteArray, auth::Key};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToSave {
    wallet: Vec<Utxo>,
    privkey: SecretKey,
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Path: {}", self.path)?;
        writeln!(f, "Key: {}", self.keypair)?;
        writeln!(f, "Miner fee: {}%", self.miner_fee)?;
        writeln!(f, "Wallet:")?;
        for utxo in &self.wallet {
            writeln!(f, "{}", utxo)?;
        }
        write!(f, "")
    }
}

#[derive(Debug,PartialEq,Clone,Default)]
pub  struct Keypair(SigningKeyPair<PublicKey, SecretKey>);

impl Display for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,"Pub  Key:{:x?}",self.0.public_key)?;
        writeln!(f,"Priv Key:{:x?}",self.0.secret_key)
    }
}

/// Enable into to passe from SigningKeyPair to Keypair
impl From<SigningKeyPair<PublicKey, SecretKey>> for Keypair {
    fn from(keypair: SigningKeyPair<PublicKey, SecretKey>) -> Self {
        Keypair(keypair)
    }
}

impl Into<PublicKey> for Keypair {
    fn into(self) -> PublicKey {
        self.0.public_key.clone()
    }
}

impl Into<Keypair> for User {
    fn into(self) -> Keypair {
        self.keypair
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct User {
    /// path were stored wallet
    path: String,
    /// fee to give to miner
    pub miner_fee: f64,
    /// buch of non used transaction
    pub wallet: Vec<Utxo>,
    /// stuff to sign
    keypair: Keypair,
}

impl TryFrom<&str> for User {
    type Error = Error;
    fn try_from(path: &str) -> std::result::Result<Self, Self::Error> {
        User::load(path)
    }
}




impl User {
    pub fn get_key(&self) -> &Keypair {
        &self.keypair //double clone
    }

    pub fn new_user(path: &str) -> Self {
        Self {
            path: path.to_string(),
            keypair: SigningKeyPair::gen_with_defaults().into(),
            miner_fee:0.1,
            ..Default::default()
        }
    }

    pub fn refresh_wallet(&mut self, wallet: Vec<Utxo>) {
        self.wallet = wallet
    }

    pub fn load(path: &str) -> Result<Self> {
        //need  err handling
        let conf = std::fs::read(path).context("impossible de lire la conf")?;
        let user: ToSave = serde_json::from_slice(&conf).context("la conf lut est broken")?;
        let keypair: Keypair =
            SigningKeyPair::from_secret_key(user.privkey).into();
        Ok(Self {
            path: path.to_string(),
            wallet: user.wallet,
            keypair,
            miner_fee:0.1,
            ..Default::default()
        })
    }

    pub fn save(self) -> Result<()> {
        let tosave = ToSave {
            wallet: self.wallet,
            privkey: self.keypair.0.secret_key.to_owned(),
        };
        let contents =
            serde_json::to_string(&tosave).context("serialisation de la conf user imposible")?;
        std::fs::write(self.path, contents).context("imposible d'écrire la conf user")?;
        Ok(())
    }

    fn sign_transa(&self, transa: Transaction) -> SignedMessage<StackByteArray<64>, Vec<u8>> {
        let data = bincode::serialize(&transa).unwrap();
        let res = self.keypair.0.sign_with_defaults(data).unwrap();
        res
    }
}

#[cfg(test)]
mod test {
    use super::User;

    #[test]
    fn serialize_unserialize_key() {
        let user1 = User::new_user("test.usr");
        user1.clone().save().unwrap();
        let user2 = User::load("test.usr").unwrap();
        assert_eq!(user1, user2)
    }

    #[test]
    fn sign_transaction_verrify() {}
}
