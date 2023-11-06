use dryoc::{keypair, sign::*, types::StackByteArray};
use serde::{Deserialize, Serialize};

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

impl From<&str> for User {
    fn from(path: &str) -> Self {
        User::load(path)
    }
}

impl User {
    pub fn new_user(path:&str) -> Self {
        Self {
            path: path.to_string(),
            wallet: vec![],
            keypair: SigningKeyPair::gen_with_defaults(),
        }
    }

    pub fn load(path: &str) -> Self { //need  err handling
        let conf = std::fs::read(path).unwrap();
        let user: ToSave = serde_json::from_slice(&conf).unwrap();
        let keypair: SigningKeyPair<PublicKey, SecretKey> =
            SigningKeyPair::from_secret_key(user.privkey);
        Self {
            path: path.to_string(),
            wallet: user.wallet,
            keypair,
        }
    }

    pub fn save(self) {
        let tosave = ToSave {
            wallet: self.wallet,
            privkey: self.keypair.secret_key.to_owned(),
        };
        let contents = serde_json::to_string(&tosave).unwrap();
        std::fs::write(self.path, contents).unwrap();
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
        user1.clone().save();
        let user2 = User::load("test.usr");
        assert_eq!(user1, user2);
    }

    #[test]
    fn sign_transaction_verrify() {}
}
