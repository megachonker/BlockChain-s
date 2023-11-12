use anyhow::Result;
use dryoc::{
    sign::{PublicKey, Signature, SignedMessage, SigningKeyPair, VecSignedMessage},
    types::{Bytes, StackByteArray},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    default,
    fmt::{self, Display},
    hash::{Hash, Hasher},
    iter::Empty,
};

use super::{acount::Acount, block::MINER_REWARD};
use super::{acount::Keypair, blockchain::Balance};

pub trait UtxoValidator {
    fn valid(&self) -> bool;
}

pub type Amount = u32;
pub type HashValue = u64;

/// Wrapper of Vec<Utxo>
#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct UtxoSet(pub Vec<Utxo>);

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct UtxoSetSigned(pub Vec<TxIn>);
impl From<Vec<VecSignedMessage>> for UtxoSetSigned {
    fn from(value: Vec<VecSignedMessage>) -> Self {
        Self(value)
    }
}

impl UtxoSet {
    pub fn value(&self) -> Amount {
        self.0.iter().map(|utxo| utxo.amount).sum()
    }

    pub fn sign_set(&self, keypair: Vec<Keypair>) -> Result<Option<UtxoSetSigned>> {
        let mut result: UtxoSetSigned = vec![].into();
        for utxo in self.0 {
            //find coresponding keypair
            let keypair = keypair.iter().find(|t| t.0.public_key == utxo.target);
            if let Some(correct_keypair) = keypair {
                let data = bincode::serialize(&utxo)?;
                let signed: VecSignedMessage = correct_keypair.0.sign_with_defaults(data)?;
                result.0.push(signed);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(result))
    }
}
impl UtxoValidator for UtxoSet {
    fn valid(&self) -> bool {
        self.0.iter().all(|utxo| utxo.valid())
    }
}

impl From<Vec<Utxo>> for UtxoSet {
    fn from(value: Vec<Utxo>) -> Self {
        Self(value)
    }
}

impl Iterator for &UtxoSet {
    type Item = Utxo;
    fn next(&mut self) -> Option<Utxo> {
        self.0.into_iter().next()
    }
}

impl Display for UtxoSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut c = 0;
        for transrx in &self.0 {
            write!(f, "{} ", transrx)?;

            c += 1;
            if c % 3 == 0 {
                write!(f, "\n║\t")?;
            }
        }
        Ok(())
    }
}

/// Designe l''utxo a prendre
type UtxoLocation = (HashValue, u16);

pub struct TxIn{
    location:UtxoLocation,
    signature:VecSignedMessage
}

/// Unspend tocken
///
/// it contain a Challenge (need implement wasm)
#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Utxo {
    pub amount: Amount,
    pub target: PublicKey, //XMRIG have  multiple targt (i gess)
    // pub come_from: HashValue,
    // targets_signature: Vec<(PublicKey,Signature)>
    //target

    // pub hash: HashValue,
    // need to hash of block
    pub come_from: u64, /////////////?????????????????????????????????????????
}

impl Utxo {
    /// use trait hash and create hash
    /// overhead cuz it init the hasher each call
    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn new(ammount: Amount, target: PublicKey, come_from: u64) -> Utxo {
        let mut utxo = Self {
            amount: ammount,
            target,
            come_from,
        };
        utxo
    }

    /// Check signature validity
    ///
    /// at term will be used to run contract
    pub fn unlock(&self, keypair: Vec<Keypair>) -> Result<VecSignedMessage> {
        let k = keypair
            .iter()
            .find(|t| t.0.public_key == self.target)
            .unwrap()
            .0;
        let data = bincode::serialize(self).unwrap();
        let res: VecSignedMessage = k.sign_with_defaults(data)?;
        Ok(res)
    }
}

impl UtxoValidator for Utxo {
    fn valid(&self) -> bool {
        self.amount > 0
    }
}

impl Hash for Utxo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target.hash(state);
        self.amount.hash(state);
        self.come_from.hash(state);
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
#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    pub rx: UtxoSetSigned,
    pub tx: UtxoSet,
    //// WASM challenge
}

impl Transaction {
    pub fn display_for_bock(&self) -> String {
        let mut str = String::from("");
        str += &format!("{}", self);
        str
    }

    ///Check if the transaction is valid :    
    /// all utxo is valid, the rx is present in the balence (can be use) and the ammont is positive
    pub fn check_utxo_valid(&self, balence: &Balance) -> bool {
        for utxo in self.rx.0.iter() {
            if !balence.valid(utxo) {
                return false;
            }
        }
        true
    }

    pub fn check(&self) -> bool {
        let mut ammount: i128 = 0;
        if self.rx.is_empty() && self.tx.len() == 1 {
            return self.tx[0].valid();
        }

        for utxo in self.rx.iter() {
            if !utxo.valid() {
                return false;
            }
            ammount += utxo.amount as i128;
        }

        let mut hasher = DefaultHasher::new();
        self.rx.hash(&mut hasher);
        let hash_come_from = hasher.finish();

        for utxo in self.tx.iter() {
            if !utxo.valid() || hash_come_from != utxo.come_from {
                print!("Ici");
                return false;
            }
            ammount -= utxo.amount as i128;
        }

        ammount >= 0
    }

    pub fn find_created_utxo(&self) -> UtxoSet {
        self.tx.clone()
    }

    /// fin utxo taken at input in the block
    pub fn find_used_utxo(&self) -> UtxoSetSigned {
        self.rx.clone()
    }

    // can create transa from multiple user
    // todo!()

    /// Take money from User wallet and create transaction
    /// search a utxo combinaison from user wallet
    /// introduce miner fee
    /// send back to owner surplus
    /// ///// NEED TEST
    pub fn create_transa_from(
        user: &mut Acount,
        amount: Amount,
        destination: PublicKey,
    ) -> Option<Self> {
        let total_ammount = amount + user.miner_fee; //// on veuux pas taxer sur des pourcent mais pour pas abu
                                                     // je send 1 milliard si je me fait taxer 10% le miner recois 10Million autant faire moi meme un noeud lol
        let (selected, sendback) = Self::select_utxo_from_vec(&user.wallet, total_ammount)?;

        let mut hasher = DefaultHasher::new();
        selected.hash(&mut hasher);
        let hash_come_from = hasher.finish();

        let mut transaction = Self {
            rx: selected.clone(),
            ///
            tx: vec![Utxo::new(amount, destination, hash_come_from)],
        };

        // Update wallet
        // can triguerre here a hanndler to know were transa done
        user.wallet.retain(|transa| !selected.contains(transa));

        // if ? ? ?
        if user.wallet.is_empty() || sendback == 0 {
            return Some(transaction);
        }

        //send back the money to the owner of input
        transaction.tx.push(Utxo::new(
            sendback,
            user.wallet[0].target.clone(),
            hash_come_from,
        ));
        Some(transaction)
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
    fn select_utxo_from_vec(avaible: &UtxoSet, amount: Amount) -> Option<(UtxoSet, Amount)> {
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
    ) -> Vec<Transaction> {
        let mut miner_reward = MINER_REWARD;

        let mut place_remove = None;

        for (i, t) in transas.iter().enumerate() {
            if t.rx.is_empty() && t.tx.len() == 1 {
                place_remove = Some(i)
            } else {
                miner_reward += t.remains();
            }
        }
        if place_remove.is_some() {
            transas.remove(place_remove.unwrap()); //reward transa already present remove it
        }

        transas.push(Transaction {
            rx: vec![].into(),
            tx: vec![Utxo::new(miner_reward, key.into(), block_heigt)].into(),
        });
        transas
    }

    /// Combien Input Utcxo - OutputUtxo => Pour  le miner
    pub fn remains(&self) -> i128 {
        self.rx.value() as i128 - self.tx.value() as i128
    }
}

impl UtxoValidator for Transaction {
    fn valid(&self) -> bool {
        self.rx.valid() && self.tx.valid() && self.remains().is_positive()
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        write!(f, "Hash:{}", hash)?;
        write!(f, "\n║Input:\t")?;
        write!(f, "{:?}", self.rx.to_vec().get(..5).unwrap())?;
        write!(f, "\n║Output:\t")?;
        write!(f, "{}", self.tx)?;
        write!(f, "For the miner: {}", self.remains())?;
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {

    use crate::block_chain::transaction::{Transaction, Utxo};
    use rand::Rng;

    use super::*;

    #[test]
    fn create_utxo() {
        let mut rng = rand::thread_rng();
        let utxo = Utxo::new(rng.gen(), Default::default(), rng.gen());

        assert!(utxo.valid());
    }

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
