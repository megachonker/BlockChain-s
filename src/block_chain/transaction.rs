use anyhow::{bail, Context, Result};
use dryoc::{
    sign::{PublicKey, Signature, SignedMessage, SigningKeyPair, VecSignedMessage},
    types::{Bytes, StackByteArray},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    default,
    fmt::{self, Display},
    hash::{Hash, Hasher},
    iter::Empty,
};

use super::{
    acount::Acount,
    block::{Block, MINER_REWARD},
    blockchain::{self, Blockchain},
};
use super::{acount::Keypair, blockchain::Balance};

pub trait UtxoValidator<AdditionalArg = ()> {
    fn valid(&self, arg: AdditionalArg) -> Option<bool>;
}

pub type Amount = u32;
pub type HashValue = u64;

/// Alias qui permet de savoir ou chercher l'utxo
/// Hashvalue est le hash de la transaction
/// usize indice de quelle utxo prendre dans la transaction
pub type UtxoLocation = (HashValue, usize);

/// utiliser en entrée pour une transa
/// référance ver le chemain pour accédée a un utxo
/// la signature de l'utxo certifie que le signant correspond a la clef public
/// car il possède la clef privée il peut donc signer
#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TxIn {
    /// référance vere un UTXO existant Valide
    pub location: UtxoLocation,
    /// UTXO fesant référance signer
    pub signed: Vec<u8>,
}

impl TxIn {
    /// # Ce vérifie tout soeul mais a besoin de Utxo
    /// Vérifie la signature
    pub fn check_sig(&self, utxo: &Utxo) -> bool {
        let message = bincode::serialize(&utxo).unwrap();
        SignedMessage::from_parts(self.signed.clone(), message)
            .verify(&utxo.target)
            .is_ok()
    }

    /// convertie en Utxo utilisant la blockaine
    pub fn to_utxo(self, blockaine: &Blockchain) -> Option<Utxo> {
        blockaine.get_utxo_from_location(self.location)
    }
}

impl UtxoValidator<&Blockchain> for TxIn {
    fn valid(&self, arg: &Blockchain) -> Option<bool> {
        Some(self.check_sig(&self.clone().to_utxo(arg)?).to_owned().clone())
    }
}

impl Display for TxIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Location: {}.{} Signature:{:?}",
            self.location.0,
            self.location.1,
            self.signed.get(..5).unwrap()
        )
    }
}

/// # Uspend Transaction X output
///
#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Utxo {
    pub amount: Amount,
    pub target: PublicKey,
    // je pense q'on s'en branle de qui ces qui l'a émis
}

impl Utxo {

    /// # Prépart une transaction Utxo => TxIn
    ///
    /// TxIn peut être stoquer ou directement utilsier pour une transaction
    ///
    /// On connais la clef secrette a utiliser vu qu'on a déja désérialiser
    pub fn sign(&self, location: UtxoLocation, key: &Keypair) -> Option<TxIn> {
        let message = bincode::serialize(&self).unwrap();
        let signature = key
            .0
            .sign_with_defaults(message)
            .unwrap()
            .into_parts()
            .0
            .to_vec();

        Some(TxIn {
            location,
            signed: signature,
        })
    }

    /// use trait hash and create hash
    /// overhead cuz it init the hasher each call
    fn get_hash(&self) -> HashValue {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn new(ammount: Amount, target: PublicKey) -> Utxo {
        let mut utxo = Self {
            amount: ammount,
            target,
        };
        utxo
    }
}

impl UtxoValidator<&Balance> for Utxo {
    fn valid(&self, balance: &Balance) -> Option<bool> {
        balance.valid(self)?;
        Some(self.amount > 0)
    }
}

impl Hash for Utxo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.amount.hash(state);
    }
}

//do no show the come_from (useless to show)
impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{}->({:?},{}$)",
            self.get_hash(),
            self.target.to_vec().get(..5).unwrap(),
            self.amount
        )
    }
}

/// signed transaction that can be send
// struct SignedTransaction{

// }

/// Represents a transaction involving the transfer of ownership from one set of entities to another.
///
/// # Short Description
///
/// The `Transaction` struct facilitates the transfer of Utxos (Unspent Transaction Outputs) from a group
/// of owners (Rx) to a new owner (Tx). This process involves ownership conditions, proof of ownership,
/// fee calculation, and a future challenge mechanism.
///
/// # Ownership and Conditions
///
/// - **Condition:** To consume Rx Utxos, the NextOwner of Rx must match the NewOwner of Tx.
/// - **Proof of Ownership:** Creation of new Tx Utxos requires the use of a private key, providing cryptographic
///   proof of ownership based on the validity of the Tx owner's pubkey.
///
/// ## Multiple Owners Unlocking Rx Utxo for Tx
///
/// The transaction process involves multiple owners collaborating to unlock the Rx Utxos. Each input Utxo in
/// the Rx set may require unlocking with a different public key. The transaction represents the collective
/// effort of these owners, who unlock the NextOwner of Rx to create the Tx Utxos.
///
/// - To ensure the validity of the transaction, it is essential that all Utxos in the Rx set are successfully
///   unlocked during the creation of Tx. To achieve this, private keys are employed by the respective owners.
///   Each Tx Utxo is signed with all NextOwner private keys, providing cryptographic proof of their rightful ownership.
/// - Given that there can be multiple inputs and outputs in a transaction, each input may be unlocked by a
///   different target public key. This flexibility allows for diverse ownership structures within a single
///   transaction.
///
/// ## Miner Reward
///
/// The miner receives the remaining amount of the transaction as a reward. This amount is calculated as the
/// difference between the sum of Rx Utxos and the sum of Tx Utxos, constituting the transaction fee.
///
/// // and const reward ??
///
///
/// # Verification
#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    /// on va prouver qu'on possède les clef
    pub rx: Vec<TxIn>,
    pub tx: Vec<Utxo>,
    /// use Rx's key pour signer transaction
    /// on va prouver que ces nous qui avons cref la transa
    pub signatures: Vec<u8>, //// WASM challenge is the crypto challenge
}

impl Transaction {
    pub fn get_hash(&self) -> HashValue {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// need to have multiple key
    fn sign(&mut self,blockaine: &Blockchain,key:Keypair)-> Result<Signature>{
        // get all needed key
        // let mut need_pubkey = HashSet::new();
        // for utxo in self.rx{
        //     let encoded = bincode::serialize(&utxo.to_utxo(blockaine).context("cannot convert utxo")?.target).unwrap();
        //     need_pubkey.insert(encoded);
        // }

        // for signature in need_pubkey{

        // }

        let s = bincode::serialize(&self.tx).unwrap();
        let b = key.0.sign_with_defaults(s).unwrap().into_parts().0;

        Ok(b)
    }

    /// Take money from User wallet and create transaction
    /// search a utxo combinaison from user wallet
    /// send back to owner surplus
    /// ///// NEED TEST
    pub fn new_transaction(
        acount:&mut Acount,
        amount: Amount,
        destination: PublicKey,
    ) -> Option<Self> {
        let total_ammount = amount + acount.miner_fee;
        let (selected, sendback) = acount.select_utxo(total_ammount)?;

        
        let rx = selected.clone();
        let tx = vec![
            Utxo::new(amount, destination),         //transa
            Utxo::new(sendback, acount.get_pubkey()), //retour
        ];

        let s = bincode::serialize(&tx).unwrap();
        let signatures = acount.get_key().0.sign_with_defaults(s).unwrap().into_parts().0;

        let mut transaction = Self { rx, tx, signatures: bincode::serialize(&signatures).unwrap() };


        // Update wallet
        // can triguerre here a hanndler to know were transa done
        acount.wallet.retain(|transa| !selected.contains(&transa.1));

        Some(transaction)
    }

    /// if we just have TxIn bacily we are server
    pub fn sign_set(
        utxos: Vec<(UtxoLocation,Utxo)>,
        blockaine: Blockchain,
        keypair: Vec<Keypair>,
    ) -> Result<Option<Vec<TxIn>>> {
        let mut result = vec![];
        for (postision,utxo) in utxos {
            //find coresponding keypair
            let keypair = keypair.iter().find(|t| t.0.public_key == utxo.target);
            if let Some(correct_keypair) = keypair {
                utxo.sign(postision, correct_keypair);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(result))
    }

    pub fn display_for_bock(&self) -> String {
        let mut str = String::from("");
        str += &format!("{}", self);
        str
    }

    // can create transa from multiple user

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
    fn select_utxo_from_vec(avaible: &Vec<Utxo>, amount: Amount) -> Option<(Vec<Utxo>, Amount)> {
        if amount == 0 {
            return None;
        }

        let mut value = 0;
        let mut vec_utxo = vec![];

        for utxo in avaible {
            value += utxo.amount;
            vec_utxo.push(utxo.clone());
            if value >= amount {
                return Some((vec_utxo, value - amount));
            }
        }

        None
    }

    /// # NEED TEST
    ///
    /// ## Create a Reward transaction for miner
    ///
    pub fn transform_for_miner(
        mut transas: Vec<Transaction>,
        key: Keypair,
        block_heigt: u64,
        blockaine: &Blockchain
    ) -> Vec<Transaction> {
        let mut miner_reward = MINER_REWARD;

        let mut place_remove = None;

        for (i, t) in transas.iter().enumerate() {
            if t.rx.is_empty() && t.tx.len() == 1 {
                place_remove = Some(i)
            } else {
                miner_reward += t.remains(blockaine).unwrap() as Amount;
            }
        }
        if place_remove.is_some() {
            transas.remove(place_remove.unwrap()); //reward transa already present remove it
        }

        transas.push(Transaction {
            rx: vec![].into(),
            tx: vec![Utxo::new(miner_reward, key.into())].into(), //blocke heigh ??
            ..Default::default()///////////need implemented
        });
        transas
    }

    /// Combien Input Utcxo - OutputUtxo => Pour  le miner
    ///
    /// Need to be opti
    pub fn remains(&self, blockaine: &Blockchain) -> Option<i128> {
        let input = self.rx.iter().try_fold(0, |acc, txin| {
            txin.clone().to_utxo(blockaine).map(|f| acc + f.amount as i128)
        });

        let output: Amount = self.tx.iter().map(|t| t.amount).sum();
        let a = input.and_then(|i: i128| Some(i - output as i128));
        a
    }
}

impl UtxoValidator<(&Blockchain, &Balance)> for Transaction {
    fn valid(&self, arg: (&Blockchain, &Balance)) -> Option<bool> {
        //on lose la propagation d'erreur .. ?
        let rx_status = self.tx.iter().all(|t| t.valid(arg.1).unwrap_or(false));
        let tx_status = self.tx.iter().all(|t| t.valid(arg.1).unwrap_or(false));
        let sold = self.remains(arg.0).map_or(false, |f| f.is_positive());

        Some(rx_status && tx_status && sold)
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        write!(f, "Hash:{}", hash)?;
        write!(f, "\n║Input:\t")?;
        for rx in &self.rx {
            write!(f, "{}", rx)?;
        }
        write!(f, "\n║Output:\t")?;
        for tx in &self.tx {
            write!(f, "{}", tx)?;
        }
        // write!(f, "For the miner: {}", self.remains())?;
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {

    use crate::block_chain::transaction::{Transaction, Utxo};
    use rand::Rng;

    use super::*;

    // #[test]
    // fn create_utxo() {
    //     let mut rng = rand::thread_rng();
    //     let utxo = Utxo::new(rng.gen(), Default::default(), rng.gen());

    //     assert!(utxo.valid());
    // }

    #[test]
    fn test_select_utxo_from_vec() {
        let rx_7 = Utxo {
            amount: 5,
            ..Default::default()
        };
        let rx_3 = Utxo {
            amount: 4,
            ..Default::default()
        };
        let rx_2 = Utxo {
            amount: 8,
            ..Default::default()
        };
        let rx_9 = Utxo {
            amount: 9,
            ..Default::default()
        };

        let wallet = vec![rx_7, rx_3, rx_2, rx_9];

        let amount = 10;
        let (transa, sendback) = Transaction::select_utxo_from_vec(&wallet, amount).unwrap();
        transa.iter().for_each(|transa| print!("{}", transa));
        let full: Amount = transa.iter().map(|f| f.amount).sum();
        assert!(full > amount);
        assert!(full - amount == sendback);
    }

    // #[test]
    // fn test_check() {
    //     let mut blockchain: Blockchain = Blockchain::new();
    //     let block_org = Block::new();

    //     //+ 100 for 1
    //     let block_org = block_org
    //         .find_next_block(vec![], Profile::INFINIT, FIRST_DIFFICULTY)
    //         .unwrap();
    //     blockchain.try_append(&block_org); //we assume its ok

    //     //need to take last utxo
    //     let utxo_s = blockchain.filter_utxo(1);
    //     utxo_s.iter().for_each(|f| println!("utxo for 1 is {}", f));

    //     //we use latest ustxo generate by miner for the actual transaction
    //     //59 for 10

    //     //should work
    //     /* let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8]);
    //            assert!(new_transa.check(&blockchain.balance));

    //     //bad source
    //     let utxo_s = blockchain.filter_utxo(5);
    //     let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8], 10);
    //     assert!(!new_transa.check(&blockchain));

    //     // not enought  money in utxo
    //     let new_transa = Transaction::new(utxo_s, vec![80, 70, 8], 10);
    //     assert!(!new_transa.check(&blockchain));

    //            // utxo do not exist
    //            let new_transa = Transaction::new(Default::default(), vec![70, 8]);
    //            assert!(!new_transa.check(&blockchain.balance))
    //     */
    //     // println!("NEW TRANSA {}", new_transa);
    //     // println!("Block {}", blockchain);

    //     // assert!(r)
    // }

    /* #[test]
    /// need to be finished
    fn test_new_online() {
        let mut blockchain = Blockchain::new();

        //forge teh fist block
        let org_block = Block::new()
            .find_next_block(1, vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();

        //append fist block with original money
        let (block, _) = blockchain.try_append(&org_block);

        // create random transaction
        let transa = vec![Transaction::new_online(&blockchain, 1, 25, 10).unwrap()];

        //mine the next block with the new transaction
        let block = block
            .unwrap()
            .find_next_block(1, transa, Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();

        //add it to the blockaine
        let (_block, _) = blockchain.try_append(&block);

        println!("{}", blockchain);
        assert!(true)
    } */
}

// need to test:
// merge 3:2
// transition 2:2
// split 2:3
