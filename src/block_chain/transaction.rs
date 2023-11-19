use anyhow::{Context, Result};
use dryoc::{
    sign::{PublicKey, Signature, SignedMessage},
    types::Bytes,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

use super::{acount::Acount, block::MINER_REWARD, blockchain::Blockchain};
use super::{acount::Keypair, blockchain::Balance};

pub trait UtxoValidator<AdditionalArg = ()> {
    fn valid(&self, arg: AdditionalArg) -> Option<bool>;
}

pub type Amount = u32;
pub type HashValue = u64;

/// Contain hash that refere to a utxo
#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TxIn {
    /// référance vere un UTXO existant Valide
    pub location: HashValue,
}

impl TxIn {
    /// convertie en Utxo utilisant la blockaine
    pub fn to_utxo(self, blockaine: &Balance) -> Option<Utxo> {
        blockaine.txin_to_utxo(self.location)
    }
}

impl UtxoValidator<&Balance> for TxIn {
    fn valid(&self, arg: &Balance) -> Option<bool> {
        //check if possible to convert
        //check if already spend
        Some(self.to_utxo(arg))
    }
}

impl Display for TxIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxIn Location: {}", self.location.get(..5)?)
    }
}

enum ComeFromID {
    BlockHeigt(u64),
    TxIn(Vec<TxIn>),
}

/// # Unspend transaction Output
///
/// - need to be unique
/// - can be spend once
#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Utxo {
    /// quantity of money
    amount: Amount,

    /// who can spend utxo
    target: PublicKey,

    /// make the Utxo UNIQUE
    /// sum of all Utxin
    come_from: HashValue,
}

impl Utxo {
    /// get the target key that need to be used in the transaction
    /// to proof the owner
    pub fn get_pubkey(&self) {
        self.target
    }

    /// get the value of the token
    pub fn get_amount(&self) {
        self.amount
    }

    /// auto self hash without init manualy hasher
    /// overhead cuz it init the hasher each call
    fn get_hash(&self) -> HashValue {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// forge a new utxo
    ///
    /// hash all come_from
    pub fn new(amount: Amount, target: PublicKey, come_from: ComeFromID) -> Utxo {
        // Switch Type of ID
        let come_from = match come_from {
            ComeFromID::TxIn(cf) => {
                //hash all element
                let mut hasher = DefaultHasher::new();
                //maybe convert txin to utxo ? ??
                cf.iter().for_each(|txin| txin.hash(&mut hasher));
                hasher.finish()
            }
            ComeFromID::BlockHeigt(val) => val,
        };

        Self {
            amount,
            target,
            come_from,
        }
    }
}

impl UtxoValidator<&Balance> for Utxo {
    fn valid(&self, balance: &Balance) -> Option<bool> {
        balance.valid(self)?; //normaly imposible to have error here
        Some(self.amount > 0)
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

/// # Verification
#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    pub rx: Vec<TxIn>,
    pub tx: Vec<Utxo>,
    // Wasm challenge
    // wasm:Vec<u8>,
    /// signature of all  field
    pub signatures: Vec<u8>,
}

impl Transaction {
    pub fn get_hash(&self) -> HashValue {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// output keypaire needed for sining
    pub fn select_required_keys(
        &self,
        balance: &Balance,
        key: Vec<Keypair>,
    ) -> Result<Vec<Keypair>> {
        // get all key of all TxIn to unlock inside a array of uniq key
        let mut need_pubkey: PublicKey = HashSet::new();
        for utxo in self.rx {
            let encoded = &utxo
                .to_utxo(balance)
                .context("cannot convert txin to utxo")?
                .target;
            need_pubkey.insert(encoded);
        }

        //find key in common
        key.iter()
            .map_while(|keypair| need_pubkey.eq(&keypair.0.public_key).then(|| keypair))
            .collect()
    }

    /// sign a transaction by sign rx and tx using all signature from rx
    fn sign(&mut self, key: Vec<Keypair>) -> Result<()> {
        // // get all key of all TxIn to unlock inside a array of uniq key
        // let mut need_pubkey: PublicKey = HashSet::new();
        // for utxo in self.rx {
        //     let encoded = &utxo
        //         .to_utxo(balance)
        //         .context("cannot convert txin to utxo")?
        //         .target;
        //     need_pubkey.insert(encoded);
        // }

        // let signature_data: Signature;
        // //for each key
        // for pubkey in need_pubkey {
        //     let keypairs = key
        //         .iter()
        //         .map_while(|keypair| pubkey.eq(&keypair.0.public_key).then(|| keypair));

        //     //signe the transaction with the first keypair found
        //     signature_data = keypairs
        //         .next()?
        //         .0
        //         .sign_with_defaults(self.rx + self.tx)?
        //         .into_parts()
        //         .0;

        //     //sign signature resulted for nex key
        //     for keypair in keypairs {
        //         let keypair: &Keypair = keypair;
        //         signature_data = keypair.0.sign_with_defaults(signature_data)?.into_parts().0;
        //     }
        // }

        //signe the transaction with the first keypair found

        Ok(())
    }

    /// Take money from User wallet and create transaction
    /// search a utxo combinaison from user wallet
    /// send back to owner surplus
    /// signing_key are used to signe transa and unlock all utxo
    /// ///// NEED TEST
    pub fn new_transaction(
        acount: &mut Acount,
        amount: Amount,
        destination: PublicKey,
        // sigining_key:Vec<Keypair> ,
    ) -> Option<Self> {
        let total_ammount = amount + acount.miner_fee;
        let (selected,sigining_key, sendback) = acount.select_utxo(total_ammount)?;

        let rx = selected;
        let tx = vec![
            //transaction
            Utxo::new(amount, destination, selected),
            //fragment de transaction a renvoyer a l'envoyeur
            Utxo::new(sendback, acount.get_pubkey(), selected),
        ];


        let signature_data: Signature = sigining_key
            .next()?
            .0
            .sign_with_defaults(rx + tx)?
            .into_parts()
            .0;

        //sign signature resulted for nex key
        for keypair in sigining_key {
            //fix next
            let keypair: Keypair = keypair;
            signature_data = keypair.0.sign_with_defaults(signature_data)?.into_parts().0;
        }

        let signatures = bincode::serialize(&signature_data)?;


        let mut transaction = Self {
            rx,
            tx,
            signatures,
        };

        // Update wallet
        // can triguerre here a hanndler to know were transa done
        acount.wallet.retain(|transa| !selected.contains(&transa.1));

        Some(transaction)
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
        blockaine: &Blockchain,
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

        let tx = vec![Utxo::new(miner_reward, key.into(), block_heigt)];

        let signatures =
            bincode::serialize(&key.0.sign_with_defaults(vec![] + tx)?.into_parts().0)?;

        transas.push(Transaction {
            rx: vec![],
            tx,
            signatures,
        });

        transas
    }

    /// How many remain for the miner
    /// return None if negative value
    ///
    /// Need to be CHECKED
    pub fn remains(&self, balance: &Balance) -> Option<Amount> {
        let input = self
            .rx
            .iter()
            .try_fold(0, |acc, txin| txin.to_utxo(balance).map(|f| acc + f.amount));

        let output = self.tx.iter().map(|t| t.amount).sum();
        input.and_then(|i: Amount| i.checked_sub(output))
    }
}

impl UtxoValidator<&Balance> for Transaction {
    fn valid(&self, arg: &Balance) -> Option<bool> {
        //on lose la propagation d'erreur .. ? add context ?
        let rx_status = self.rx.iter().all(|t| t.valid(arg).unwrap_or(false));
        let tx_status = self.tx.iter().all(|t| t.valid(arg).unwrap_or(false));
        let sold = self.remains(arg).map_or(false, |f| f.is_positive());
        let signature = self.check_sign(&self);

        Some(rx_status && tx_status && sold && signature)
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

    use crate::block_chain::{transaction::{Transaction, Utxo}, block::{Block, Profile}, blockchain::FIRST_DIFFICULTY};

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

    #[test]
    fn test_check() {
        let mut blockchain: Blockchain = Blockchain::new();
        let block_org = Block::new();

        //+ 100 for 1
        let block_org = block_org
            .find_next_block(vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        blockchain.try_append(&block_org); //we assume its ok

        //need to take last utxo
        let utxo_s = blockchain.filter_utxo(1);
        utxo_s.iter().for_each(|f| println!("utxo for 1 is {}", f));

        //we use latest ustxo generate by miner for the actual transaction
        //59 for 10

        //should work
        /* let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8]);
               assert!(new_transa.check(&blockchain.balance));

        //bad source
        let utxo_s = blockchain.filter_utxo(5);
        let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8], 10);
        assert!(!new_transa.check(&blockchain));

        // not enought  money in utxo
        let new_transa = Transaction::new(utxo_s, vec![80, 70, 8], 10);
        assert!(!new_transa.check(&blockchain));

               // utxo do not exist
               let new_transa = Transaction::new(Default::default(), vec![70, 8]);
               assert!(!new_transa.check(&blockchain.balance))
        */
        // println!("NEW TRANSA {}", new_transa);
        // println!("Block {}", blockchain);

        // assert!(r)
    }

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
