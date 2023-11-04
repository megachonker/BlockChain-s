use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fmt::{self, write},
    hash::{BuildHasherDefault, Hash, Hasher},
};

use super::blockchain::{Balance, Blockchain};
use super::block::MINER_REWARD;

#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Utxo {
    pub hash: u64,
    pub onwer: u64,
    pub ammount: u64,
    pub come_from: u64, //the hash of the utxo which come from (permit to the utxo to unique), hash of the list of transactions validated if it is the utxo create by miner.
}

impl Hash for Utxo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onwer.hash(state);
        self.ammount.hash(state);
        self.come_from.hash(state);
    }
}
impl Utxo {
    fn check(&self) -> bool {
        self.hash == self.hash()
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.onwer.hash(&mut hasher);
        self.ammount.hash(&mut hasher);
        self.come_from.hash(&mut hasher);

        hasher.finish()
    }

    fn new(ammount: u64, owner: u64, come_from: u64) -> Utxo {
        let mut utxo = Self {
            hash: 0,
            onwer: owner,
            ammount,
            come_from,
        };
        utxo.hash = utxo.hash();
        utxo
    }
}

//do no show the come_from (useless to show)
impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|#{}->({},{}$)|", self.hash, self.onwer, self.ammount)
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Hash, Clone, Eq, PartialEq)]
pub struct Transaction {
    pub rx: Vec<Utxo>, //the utxo to be used
    pub tx: Vec<Utxo>, //the utxo created
                   //add signature of the sender
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for transrx in &self.rx {
            writeln!(f, "{} ", transrx)?;
        }
        write!(f, "==> ").unwrap();
        for transtx in &self.tx {
            write!(f, "{} ", transtx)?;
        }
        write!(f, "]")
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
    pub fn check(&self, balence: &Balance) -> bool {
        let mut ammount: i128 = 0;
        if self.rx.len() == 0 && self.tx.len() == 1 {
            return self.tx[0].check() ;
        }

        for utxo in self.rx.iter() {
            if !balence.valid(&utxo) || !utxo.check() {
                return false;
            }
            ammount += utxo.ammount as i128;
        }

        let mut hasher = DefaultHasher::new();
        self.rx.hash(&mut hasher);
        let hash_come_from = hasher.finish();

        for utxo in self.tx.iter() {
            if !utxo.check() || hash_come_from != utxo.come_from {
                return false;
            }
            ammount -= utxo.ammount as i128;
        }

        ammount >= 0
    }

    ///create a new Transition with the given argument. Does not check : can create invalid Transaction
    pub fn new(rx: Vec<Utxo>, tx: Vec<Utxo>) -> Transaction {
        Transaction { rx, tx }
    }

    /// get the hash id of the transaction
    /// it used inside utxo to refer correct transa in block
    /// auto self hash
    /// not efficient because init the hasher manualy
    /// tradoff is that it getting simpler
    /* ub fn hash_id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    } */

    /// create a Utxo from a know output transaction in the transa
    /// we Create a utxo from it and we need the block hash
    /// transaction don't know where it is in blockain it need to ask
    /// then create the "fake" utxo
    /// it important to remember that the value is PASTED HERE
    /// BUT NO NEEDED
    /// in reality we need to ask blockain about the amount every time we
    /// face a utxo it take time
    ///
    ///
    pub fn find_created_utxo(&self) -> Vec<Utxo> {
        self.tx.clone()
    }

    /// fin utxo taken at input in the block
    ///
    /// it better to use & but it simplify to no
    pub fn find_used_utxo(&self) -> Vec<Utxo> {
        self.rx.clone()
    }

    /// Use the blockaine to find money and send it
    /* pub fn new_online(
        blockchain: &Blockchain,
        source: u64,
        amount: u128,
        destination: u64,
    ) -> Option<Self> {
        let utxos = blockchain.filter_utxo(source);

        //not optimal but i is a NP problem see bag problem
        let (rx, resend) = Self::select_utxo_from_vec(&utxos, amount)?;

        Some(Self {
            rx,
            tx: vec![resend, amount],
            target_pubkey: destination,
        })
    } */

    /// Create transaction from the input utxo and and send the ammount to destination and send back to the owner of input the surplus
    /// miner rate is part of the ammount will be pass to the miner  
    pub fn create_transa_from(
        input: &Vec<Utxo>,
        amount: u64,
        destination: u64,
        miner_rate: f64,
    ) -> Option<Self> {
        let total_ammount = (amount as f64 * (1.0 + miner_rate)) as u64;
        let (rx, resend) = Self::select_utxo_from_vec(input, total_ammount)?;

        let mut hasher = DefaultHasher::new();
        rx.hash(&mut hasher);
        let hash_come_from = hasher.finish();

        if input.len() == 0 {
            Some(Self {
                rx,
                tx: vec![Utxo::new(amount, destination, hash_come_from)],
            })
        } else {
            //send back the money to the owner of input
            Some(Self {
                rx: rx,
                tx: vec![
                    Utxo::new(amount, destination, hash_come_from),
                    Utxo::new(resend, input[0].onwer, hash_come_from),
                ],
            })
        }
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
    fn select_utxo_from_vec(avaible: &Vec<Utxo>, amount: u64) -> Option<(Vec<Utxo>, u64)> {
        if amount == 0 {
            return Some((vec![], 0));
        }
        let mut value = 0;
        let mut vec_utxo = vec![];
        for utxo in avaible {
            value += utxo.ammount;
            vec_utxo.push(utxo.clone());
            if value >= amount {
                return Some((vec_utxo, value - amount));
            }
        }

        None
    }

    pub fn transform_for_miner(mut transas: Vec<Transaction>, miner_id: u64,block_heigt : u64) -> Vec<Transaction> {
        let mut miner_reward = MINER_REWARD;

        let mut place_remove = None;

        for (i, t) in transas.iter().enumerate() {
            if t.rx.len() == 0 && t.tx.len() == 1 {
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
            tx: vec![Utxo::new(miner_reward, miner_id, block_heigt)],
        });
        transas
    }

    pub fn remains(&self) -> u64 {
        let input: u64 = self.rx.iter().map(|u| u.ammount).sum();
        let output: u64 = self.tx.iter().map(|u| u.ammount).sum();
        input - output
    }

    
}

///calulcate the hash of the come_from for the miner transa
/* pub fn come_from_hash(transas :& Vec<Transaction>) -> u64{
    let mut transas_cpy = transas.clone();
    for t in transas{
        if t.rx.len() ==0 && t.tx.len()==1{
            transas_cpy.remove(transas_cpy.iter().position(|t_cpy| t_cpy==t).unwrap());     //remove miner_transa
        }
    }

    let mut hasher = DefaultHasher::new();
    transas_cpy.hash(&mut hasher);
    hasher.finish()



}
 */
#[cfg(test)]
mod tests {

    use rand::Rng;

    use crate::block_chain::{
        block::{Block, Profile},
        blockchain::{Blockchain, FIRST_DIFFICULTY},
        transaction::{Transaction, Utxo},
    };

    #[test]
    fn create_utxo() {
        let mut rng = rand::thread_rng();
        let utxo = Utxo::new(rng.gen(), rng.gen(), rng.gen());

        assert!(utxo.check());
    }

    #[test]
    fn test_select_utxo_from_vec() {
        let rx_7 = Utxo {
            ammount: 5,
            ..Default::default()
        };
        let rx_3 = Utxo {
            ammount: 4,
            ..Default::default()
        };
        let rx_2 = Utxo {
            ammount: 8,
            ..Default::default()
        };
        let rx_9 = Utxo {
            ammount: 9,
            ..Default::default()
        };

        let wallet = vec![rx_7, rx_3, rx_2, rx_9];

        let amount = 10;
        let (transa, sendback) = Transaction::select_utxo_from_vec(&wallet, amount).unwrap();
        transa.iter().for_each(|transa| print!("{}", transa));
        let full: u64 = transa.iter().map(|f| f.ammount).sum();
        assert!(full > amount);
        assert!(full - amount == sendback);
    }

    #[test]
    fn test_check() {
        let mut blockchain: Blockchain = Blockchain::new();
        let block_org = Block::new();

        //+ 100 for 1
        let block_org = block_org
            .find_next_block( vec![], Profile::INFINIT, FIRST_DIFFICULTY)
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
               let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8]);
               assert!(!new_transa.check(&blockchain.balance));

               // not enought  money in utxo
               let new_transa = Transaction::new(utxo_s, vec![80, 70, 8]);
               assert!(!new_transa.check(&blockchain.balance));

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
