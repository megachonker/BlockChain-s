use std::fmt::Display;

use super::transaction::{Amount, Transaction, Utxo, TxIn, UtxoLocation};
use anyhow::{Context, Error, Result};
use dryoc::{sign::*, types::StackByteArray};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToSave {
    wallet: Vec<(Amount, TxIn)>,
    privkey: SecretKey,
}
use tracing::{debug, info, trace, warn};

impl std::fmt::Display for Acount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Path: {}", self.path)?;
        writeln!(f, "Key: {}", self.keypair)?;
        writeln!(f, "Miner fee: {}%", self.miner_fee)?;
        writeln!(f, "Wallet:")?;
        for utxo in &self.wallet {
            writeln!(f, "Value: {}, TxIn:{}", utxo.0,utxo.1)?;
        }
        write!(f, "sold: {}", self.get_sold())
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Keypair(pub SigningKeyPair<PublicKey, SecretKey>);

impl Display for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pub  Key:{:x?}", self.0.public_key)?;
        writeln!(f, "Priv Key:{:x?}", self.0.secret_key)
    }
}

/// Enable into to passe from SigningKeyPair to Keypair
impl From<SigningKeyPair<PublicKey, SecretKey>> for Keypair {
    fn from(keypair: SigningKeyPair<PublicKey, SecretKey>) -> Self {
        Keypair(keypair)
    }
}

impl From<Keypair> for PublicKey {
    fn from(val: Keypair) -> Self {
        val.0.public_key.clone()
    }
}

impl From<Acount> for Keypair {
    fn from(val: Acount) -> Self {
        val.keypair
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Acount {
    /// path were stored wallet
    path: String,
    /// fee to give to miner
    pub miner_fee: Amount,
    /// buch of non used transaction
    pub wallet: Vec<(Amount, TxIn)>,
    /// stuff to sign
    keypair: Keypair,//to change on vec cuz there can have multiple key
}

impl TryFrom<&str> for Acount {
    type Error = Error;
    fn try_from(path: &str) -> std::result::Result<Self, Self::Error> {
        Acount::load(path)
    }
}

impl Acount {
    pub fn get_key(&self) -> &Keypair {
        &self.keypair //double clone
    }

    pub fn get_pubkey(&self)->PublicKey{
        self.keypair.0.public_key.clone()
    }

    pub fn get_sold(&self) -> Amount {
        self.wallet
            .iter()
            .fold(Default::default(), |sum, x| x.0 + sum)
    }

    pub fn new_user(path: &str) -> Self {
        Self {
            path: path.to_string(),
            keypair: SigningKeyPair::gen_with_defaults().into(),
            miner_fee: 2,
            ..Default::default()
        }
    }

    pub fn refresh_wallet(&mut self, wallet: Vec<(UtxoLocation, Utxo)>) ->Result<()> {
        let mut new_wallet = vec![];
        for (position,utxo) in wallet{
            if self.get_pubkey() != utxo.target{
                warn!("missing key for: {}",utxo);
                continue;
            }
            let signed = utxo.sign(position, self.get_key()).with_context(|| format!("imposible de signer utxo {}",utxo) )?;
            new_wallet.push((utxo.amount, signed));


        }
        self.wallet = new_wallet;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self> {
        //need  err handling
        let conf = std::fs::read(path).with_context(|| {
            format!(
                "I/O impossible de charger le wallet [{}] (non existing file ?)",
                path
            )
        })?;
        let user: ToSave = serde_json::from_slice(&conf).context("la conf lut est broken")?;
        let keypair: Keypair = SigningKeyPair::from_secret_key(user.privkey).into();
        Ok(Self {
            path: path.to_string(),
            wallet: user.wallet,
            keypair,
            miner_fee: 2,
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
        self.keypair.0.sign_with_defaults(data).unwrap()
    }



    /// # find a combinaison of Utxo for a amount given
    ///
    /// ### exemple:
    /// want send 10
    ///
    /// at input there are 7 2 2 9
    ///
    /// stop at 11  
    ///
    /// 7 2 2 was selected
    ///
    /// 10 to the user and send back 1
    pub fn select_utxo(&self, amount: Amount) -> Option<(Vec<TxIn>, Amount)> {
        if amount == 0 {
            return None;
        }

        let mut value_total = 0;
        let mut vec_utxo = vec![];

        for (amount,utxo) in &self.wallet {
            value_total += amount;
            vec_utxo.push(utxo.clone());
            if value_total >= amount.clone() {
                return Some((vec_utxo, value_total - amount));
            }
        }

        None
    }



}

#[cfg(test)]
mod test {
    use super::Acount;

    #[test]
    fn serialize_unserialize_key() {
        let user1 = Acount::new_user("test.usr");
        user1.clone().save().unwrap();
        let user2 = Acount::load("test.usr").unwrap();
        assert_eq!(user1, user2)
    }

    // #[test]
    // fn sign_transaction_verrify() {}
}
