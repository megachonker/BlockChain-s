use std::fmt::Display;

use super::transaction::{Amount, Transaction, TxIn, Utxo};
use anyhow::{bail, Context, Error, Result};
use dryoc::{sign::*, types::StackByteArray};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToSave {
    wallet: Vec<Utxo>,
    privkey: Vec<SecretKey>,
}

impl std::fmt::Display for Acount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Path: {}", self.path)?;
        for k in &self.keypair {
            writeln!(f, "Keys: {}", k)?;
        }
        writeln!(f, "Miner fee: {}%", self.miner_fee)?;
        writeln!(f, "Wallet:")?;
        for utxo in &self.wallet {
            writeln!(f, "{}", utxo)?;
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

impl From<Acount> for Vec<Keypair> {
    fn from(val: Acount) -> Self {
        val.keypair
    }
}

impl Default for Acount {
    fn default() -> Self {
        Self {
            miner_fee: 1,
            keypair: vec![SigningKeyPair::gen_with_defaults().into()],
            path: Default::default(),
            wallet: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Acount {
    /// path were stored wallet
    path: String,
    /// fee to give to miner
    pub miner_fee: Amount,
    /// buch of non used transaction
    pub wallet: Vec<Utxo>,
    /// stuff to sign
    keypair: Vec<Keypair>, //to change on vec cuz there can have multiple key
}

impl TryFrom<&str> for Acount {
    type Error = Error;
    fn try_from(path: &str) -> std::result::Result<Self, Self::Error> {
        Acount::load(path)
    }
}

impl Acount {
    pub fn get_key(&self) -> &Vec<Keypair> {
        &self.keypair //double clone
    }

    /// get one of public authentity
    pub fn get_pubkey(&self) -> PublicKey {
        self.keypair.first().unwrap().0.public_key.clone()
    }

    pub fn get_sold(&self) -> Amount {
        self.wallet
            .iter()
            .fold(Default::default(), |sum, x| x.get_amount() + sum)
    }

    pub fn new_user(path: &str) -> Self {
        Self {
            path: path.to_string(),
            ..Default::default()
        }
    }
    pub fn refresh_wallet(&mut self, wallet: Vec<Utxo>) -> Result<()> {
        // Nouveau portefeuille pour stocker les UTXOs valides
        let mut new_wallet = Vec::new();

        // Parcourir chaque UTXO dans le portefeuille fourni
        for utxo in wallet {
            // Vérifier si l'UTXO peut être débloqué avec une des clés disponibles
            let can_unlock = self
                .keypair
                .iter()
                .any(|ctx| ctx.0.public_key == utxo.get_pubkey());

            // Si l'UTXO peut être débloqué, l'ajouter au nouveau portefeuille
            if can_unlock {
                new_wallet.push(utxo);
            } else {
                // Si aucun clé ne correspond, signaler une erreur
                bail!("missing key for unlocking utxo => {}", utxo);
            }
        }

        // Mettre à jour le portefeuille avec les UTXOs valides
        self.wallet = new_wallet;
        debug!("refreshed wallet:");
        for e in &self.wallet {
            debug!("wallet entry:{}", e);
        }
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

        let mut keypair: Vec<Keypair> = vec![];
        for k in user.privkey {
            keypair.push(SigningKeyPair::from_secret_key(k).into());
        }
        Ok(Self {
            path: path.to_string(),
            wallet: user.wallet,
            keypair,
            ..Default::default()
        })
    }

    pub fn save(self) -> Result<()> {
        let tosave = ToSave {
            wallet: self.wallet,
            privkey: self
                .keypair
                .iter()
                .map(|k| k.0.secret_key.to_owned())
                .collect(),
        };
        let contents =
            serde_json::to_string(&tosave).context("serialisation de la conf user imposible")?;
        std::fs::write(self.path, contents).context("imposible d'écrire la conf user")?;
        Ok(())
    }

    // fn sign_transa(&self, transa: Transaction) -> SignedMessage<StackByteArray<64>, Vec<u8>> {
    //     let data = bincode::serialize(&transa).unwrap();
    //     self.keypair.0.sign_with_defaults(data).unwrap()
    // }

    pub fn get_keypair(&self, utxo: &Vec<Utxo>) -> Option<Vec<Keypair>> {
        utxo.iter()
            .map(|utxo| {
                self.keypair
                    .iter()
                    .find(|k| k.0.public_key == utxo.get_pubkey())
                    .cloned()
            })
            .collect()
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
    pub fn select_utxo(&self, amount: Amount) -> Option<(Vec<Utxo>, Amount)> {
        if amount == 0 {
            return None;
        }

        let mut value_total = 0;
        let mut vec_utxo = vec![];

        for utxo in &self.wallet {
            value_total += utxo.get_amount();
            vec_utxo.push(utxo.clone());
            if value_total >= amount {
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
