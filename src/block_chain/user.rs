use dryoc::{keypair, sign::*, types::StackByteArray};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result, Error};
use super::{
    node::network::Packet,
    transaction::{Transaction, Utxo},
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToSave {
    wallet: Vec<Utxo>,
    privkey: SecretKey,
}

#[derive(Debug, PartialEq, Clone)]
pub struct User {
    path: String,
    wallet: Vec<Utxo>,
    keypair: SigningKeyPair<PublicKey, SecretKey>,
}

impl TryFrom<&str> for User {
    type Error = Error;
    fn try_from(path: &str) -> std::result::Result<Self, Self::Error> {
        User::load(path)
    }
}

impl User {
    pub fn get_key(&self)-> &SigningKeyPair<PublicKey, SecretKey> {
        &self.keypair
    }

    pub fn new_user(path:&str) -> Self {
        Self {
            path: path.to_string(),
            wallet: vec![],
            keypair: SigningKeyPair::gen_with_defaults(),
        }
    }

    pub fn  refresh_wallet(&mut self,wallet:Vec<Utxo>){
        self.wallet = wallet
    }

    pub fn load(path: &str) -> Result<Self> { //need  err handling
        let conf = std::fs::read(path).context( "impossible de lire la conf")?;
        let user: ToSave = serde_json::from_slice(&conf).context("la conf lut est broken")?;
        let keypair: SigningKeyPair<PublicKey, SecretKey> =
            SigningKeyPair::from_secret_key(user.privkey);
        Ok(Self {
            path: path.to_string(),
            wallet: user.wallet,
            keypair,
        })
    }

    pub fn save(self) -> Result<()> {
        let tosave = ToSave {
            wallet: self.wallet,
            privkey: self.keypair.secret_key.to_owned(),
        };
        let contents = serde_json::to_string(&tosave).context("serialisation de la conf user imposible")?;
        std::fs::write(self.path, contents).context("imposible d'écrire la conf user")?;
        Ok(())
    }

    fn sign_transa(&self, transa: Transaction) -> SignedMessage<StackByteArray<64>, Vec<u8>> {
        let data = bincode::serialize(&transa).unwrap();
        let res = self.keypair.sign_with_defaults(data).unwrap();
        res
    }
}

fn main() {
    // Generate a random keypair, using default types
    let keypair = SigningKeyPair::gen_with_defaults();
    let message = b"Fair is foul, and foul is fair: Hover through the fog and filthy air.";

    // Sign the message, using default types (stack-allocated byte array, Vec<u8>)
    let signed_message = keypair.sign_with_defaults(message).expect("signing failed");
    let (a, mut b) = signed_message.into_parts();
    b = "Fair is foul, and foul is fair: Hover through the fog and filthy air.".into();
    let signed_message = SignedMessage::from_parts(a, b);

    // Verify the message signature
    signed_message
        .verify(&keypair.public_key)
        .expect("verification failed");
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
