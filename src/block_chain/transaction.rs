use dryoc::{
    sign::{PublicKey},
    types::{Bytes},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fmt::{self},
    hash::{Hash, Hasher},
};

use super::{blockchain::{Balance}, acount::Keypair};
use super::{block::MINER_REWARD, acount::Acount};

pub type Amount = u32;
pub  type HashValue = u64;



#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Utxo {
    pub hash: HashValue,
    pub onwer: PublicKey,
    pub amount: Amount,

    // need to hash of block
    pub come_from: HashValue, //the hash of the utxo which come from (permit to the utxo to unique), hash of the list of transactions validated if it is the utxo create by miner.
}

impl Hash for Utxo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onwer.hash(state);
        self.amount.hash(state);
        self.come_from.hash(state);
    }
}
impl Utxo {
    fn check(&self) -> bool {
        self.hash == self.hash() && self.amount > 0
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.onwer.hash(&mut hasher);
        self.amount.hash(&mut hasher);
        self.come_from.hash(&mut hasher);

        hasher.finish()
    }

    fn new(ammount: Amount, owner: PublicKey, come_from: u64) -> Utxo {
        let mut utxo = Self {
            hash: 0,
            onwer: owner,
            amount: ammount,
            come_from,
        };
        utxo.hash = utxo.hash();
        utxo
    }
}

//do no show the come_from (useless to show)
impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}->({:?},{}$)", self.hash,self.onwer.to_vec().get(..5).unwrap(), self.amount)
    }
}



#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
/// Structure That Be Signed
pub struct Transaction {
    pub rx: Vec<Utxo>,
    pub tx: Vec<Utxo>,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();

        write!(f,"Hash:{}",hash)?;
        write!(f, "\n║Input:\t")?;
        let mut c = 0;
        for transrx in &self.rx {
            write!(f, "{} ", transrx)?;
            c+=1;
            if c == 3 {
                write!(f,"\n║\t")?;
                c=0;

            }
        }
        write!(f, "\n║Output:\t")?;
        c = 0;
        for transtx in &self.tx {
            write!(f, "{} ", transtx)?;
            c+=1;
            if c == 3 {
                write!(f,"\n║\t")?;
                c=0;
            }
        }
        write!(f, "")
    }
}

/// Make the split of the coin
impl Transaction {
    pub fn display_for_bock(&self) -> String {
        let mut str = String::from("");
        str += &format!("{}", self);
        str
    }

    ///Check if the transaction is valid :    
    /// all utxo is valid, the rx is present in the balence (can be use) and the ammont is positive
    pub fn check_utxo_valid(&self, balence: &Balance) -> bool {
        for utxo in self.rx.iter() {
            if !balence.valid(utxo) {
                return false;
            }
        }
        true
    }
    pub fn check(&self) -> bool {
        let mut ammount: i128 = 0;
        if self.rx.is_empty() && self.tx.len() == 1 {
            return self.tx[0].check();
        }

        for utxo in self.rx.iter() {
            if !utxo.check() {
                return false;
            }
            ammount += utxo.amount as i128;
        }

        let mut hasher = DefaultHasher::new();
        self.rx.hash(&mut hasher);
        let hash_come_from = hasher.finish();

        for utxo in self.tx.iter() {
            if !utxo.check() || hash_come_from != utxo.come_from {
                print!("Ici");
                return false;
            }
            ammount -= utxo.amount as i128;
        }

        ammount >= 0
    }

    ///create a new Transition with the given argument. Does not check : can create invalid Transaction
    pub fn new(rx: Vec<Utxo>, tx: Vec<Utxo>) -> Transaction {
        Transaction { rx, tx }
    }

    pub fn find_created_utxo(&self) -> Vec<Utxo> {
        self.tx.clone()
    }

    /// fin utxo taken at input in the block
    pub fn find_used_utxo(&self) -> Vec<Utxo> {
        self.rx.clone()
    }

    // can create transa from multiple user
    // todo!()

    /// Take money from User wallet and create transaction
    /// search a utxo combinaison from user wallet
    /// introduce miner fee
    /// send back to owner surplus
    /// ///// NEED TEST
    pub fn create_transa_from(user: &mut Acount, amount: Amount, destination: PublicKey) -> Option<Self> {
        let total_ammount = amount + user.miner_fee;//// on veuux pas taxer sur des pourcent mais pour pas abu
        // je send 1 milliard si je me fait taxer 10% le miner recois 10Million autant faire moi meme un noeud lol
        let (selected, sendback) = Self::select_utxo_from_vec(&user.wallet, total_ammount)?;

        let mut hasher = DefaultHasher::new();
        selected.hash(&mut hasher);
        let hash_come_from = hasher.finish();

        let mut transaction = Self {
            rx: selected.clone(),///
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
        transaction
            .tx
            .push(Utxo::new(sendback, user.wallet[0].onwer.clone(), hash_come_from));
        Some(transaction)
    }

    /// ## find a combinaison
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

    /// NEED TEST
    pub fn transform_for_miner(
        mut transas: Vec<Transaction>,
        key:Keypair,
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
            rx: vec![],
            tx: vec![Utxo::new(miner_reward, key.into(), block_heigt)],
        });
        transas
    }

    /// Combien Input Utcxo - OutputUtxo => Pour  le miner
    pub fn remains(&self) -> Amount {
        let input: Amount = self.rx.iter().map(|u| u.amount).sum();
        let output: Amount = self.tx.iter().map(|u| u.amount).sum();
        input - output
    }
}

#[cfg(test)]
mod tests {

    use crate::block_chain::{
        transaction::{Transaction, Utxo},
    };
    use rand::Rng;

    use super::*;

    #[test]
    fn create_utxo() {
        let mut rng = rand::thread_rng();
        let utxo = Utxo::new(rng.gen(), Default::default(), rng.gen());

        assert!(utxo.check());
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
