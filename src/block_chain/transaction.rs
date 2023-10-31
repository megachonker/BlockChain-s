use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use super::blockchain::Blockchain;

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Utxo {
    pub block_location: u64, //dans quelle block est la transa
    pub transa_id: u64,      //hash de Transaction
    moula_id: usize,         //id mola not refering value but position in the vec
    // no value !
    // it can seem verry cringe but there only refering to actual transaction
    value: u128, //can work without but Simplify the challenge NOT NEED TO SERIALIZED
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rx: [{}=>{}=>{}] {}",
            self.block_location, self.transa_id, self.moula_id, self.value,
        )
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    rx: Vec<Utxo>,
    tx: Vec<u128>, //fist is what is send back <= changed so it now last but need impleented
    pub target_pubkey: u64,
    //add signature of the sender
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "║ transa id: {}", self.hash_id()).unwrap();
        for transrx in &self.rx {
            writeln!(f, "║{}", transrx).unwrap();
        }
        write!(f, "║Tx[").unwrap();
        for transtx in &self.tx {
            write!(f, "{},", transtx).unwrap();
        }
        write!(
            f,
            "]\n\
║dst: {}",
            self.target_pubkey,
        )
    }
}

/// Make the split of the coin
impl Transaction {
    pub fn display_for_bock(&self) -> String {
        let mut str = String::from("");
        for transrx in &self.rx {
            str += format!("{}", transrx).as_str();
        }
        str += " Tx[".to_string().as_str();
        for transtx in &self.tx {
            str += format!("{},", transtx).as_str();
        }
        str += format!("] -> {}", self.target_pubkey,).as_str();

        str
    }

    /// Sum(Rx) > Sum(Tx)
    /// don't check if transa already used
    /// check value is right
    /// check all utxo exist
    pub fn check(&self, blockaine: &Blockchain) -> bool {
        // need to be done inside the block level
        // to change <================================
        // self.tx.contains(&100).then(||println!("TRIGUERRRRRRRRRRR")); // we considere that 100 number tx is directly
        self.tx.contains(&100).then_some(true); // we considere that 100 number tx is directly
                                                //the reward of the miner

        //check all utxo is accesible
        //need to use balance
        if !self.rx.iter().all(|utxo| {
            blockaine.get_block(utxo.block_location).is_some_and(|h| {
                h.transactions
                    .iter()
                    .find(|transa| transa.hash_id() == utxo.transa_id)
                    .is_some_and(|h| h.tx.get(utxo.moula_id).is_some())
            })
        }) {
            return false;
        }

        //need to be change by using get utxo because rx no need to store value
        let sum_in: u128 = self.rx.iter().map(|utxo| utxo.value).sum();
        let sum_out: u128 = self.tx.iter().sum();
        sum_in > sum_out
    }

    pub fn new(rx: Vec<Utxo>, tx: Vec<u128>, target_pubkey: u64) -> Transaction {
        Transaction {
            rx,
            tx,
            target_pubkey,
        }
    }

    /// get the hash id of the transaction
    /// it used inside utxo to refer correct transa in block
    /// auto self hash
    /// not efficient because init the hasher manualy
    /// tradoff is that it getting simpler
    pub fn hash_id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
    /// create a Utxo from a know output transaction in the transa
    /// we Create a utxo from it and we need the block hash
    /// transaction don't know where it is in blockain it need to ask
    /// then create the "fake" utxo
    /// it important to remember that the value is PASTED HERE
    /// BUT NO NEEDED
    /// in reality we need to ask blockain about the amount every time we
    /// face a utxo it take time
    pub fn find_new_utxo(&self, block_location: u64) -> Vec<Utxo> {
        let mut no = 0;
        self.tx
            .iter()
            .map(|tx| {
                let tmp = Utxo {
                    block_location,
                    transa_id: self.hash_id(),
                    moula_id: no,
                    value: *tx,
                };
                no += 1;
                tmp
            })
            .collect()
    }

    /// fin utxo taken at input in the block
    ///
    /// it better to use & but it simplify to no
    pub fn find_used_utxo(&self) -> Vec<Utxo> {
        self.rx.clone()
    }

    /// Use the blockaine to find money and send it
    pub fn new_online(
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
    }

    /// ofline use actual wallet and create transa
    pub fn new_offline(input: &Vec<Utxo>, amount: u128, destination: u64) -> Option<Self> {
        let (rx, resend) = Self::select_utxo_from_vec(input, amount)?;

        Some(Self {
            rx,
            tx: vec![resend, amount],
            target_pubkey: destination,
        })
    }

    /// ## find a combinaison
    /// want send 10
    ///
    /// with fee it need to search for 11 for give 1 to miner
    ///
    /// at input there are 7 3 2 9
    ///
    /// stop at 12  
    ///
    /// 7 3 2 was selected
    ///
    /// it need to give 1 to miner give implicitly, 10 to the user and send back 1
    fn select_utxo_from_vec(avaible: &Vec<Utxo>, amount: u128) -> Option<(Vec<Utxo>, u128)> {
        let fee = amount / 10;
        let mut sum = 0;
        let r: Vec<Utxo> = avaible
            .iter()
            .take_while(|utxo| {
                if sum <= amount + fee {
                    sum += utxo.value;
                    true
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        let to_send_back = sum.checked_sub(amount + fee);
        to_send_back.map(|val| (r, val))
    }
}

#[cfg(test)]
mod tests {

    use crate::block_chain::{
        block::{Block, Profile},
        blockchain::{Blockchain, FIRST_DIFFICULTY},
        transaction::{Transaction, Utxo},
    };

    #[test]
    fn test_select_utxo_from_vec() {
        let rx_7 = Utxo {
            value: 5,
            ..Default::default()
        };
        let rx_3 = Utxo {
            value: 4,
            ..Default::default()
        };
        let rx_2 = Utxo {
            value: 8,
            ..Default::default()
        };
        let rx_9 = Utxo {
            value: 9,
            ..Default::default()
        };

        let wallet = vec![rx_7, rx_3, rx_2, rx_9];

        let (transa, sendback) = Transaction::select_utxo_from_vec(&wallet, 10).unwrap();
        transa.iter().for_each(|transa| print!("{}", transa));
        let full: u128 = transa.iter().map(|f| f.value).sum();
        let total_cost = full - 10;
        println!(
            "\nneed to send back:{}, total spend with fee:{}",
            sendback, total_cost
        );
        assert_eq!(sendback, 6);
        assert_eq!(total_cost, 7)
    }

    #[test]
    fn test_check() {
        let mut blockchain = Blockchain::new();
        let block_org = Block::new();

        //+ 100 for 1
        let block_org = block_org
            .find_next_block(1, vec![], Profile::INFINIT, FIRST_DIFFICULTY)
            .unwrap();
        blockchain.try_append(&block_org); //we assume its ok

        //need to take last utxo
        let utxo_s = blockchain.filter_utxo(1);
        utxo_s.iter().for_each(|f| println!("utxo for 1 is {}", f));

        //we use latest ustxo generate by miner for the actual transaction
        //59 for 10

        //should work
        let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8], 10);
        assert!(new_transa.check(&blockchain));

        //bad source
        let utxo_s = blockchain.filter_utxo(5);
        let new_transa = Transaction::new(utxo_s.clone(), vec![1, 50, 8], 10);
        assert!(!new_transa.check(&blockchain));

        // not enought  money in utxo
        let new_transa = Transaction::new(utxo_s, vec![80, 70, 8], 10);
        assert!(!new_transa.check(&blockchain));

        // utxo do not exist
        let new_transa = Transaction::new(Default::default(), vec![70, 8], 10);
        assert!(!new_transa.check(&blockchain))

        // println!("NEW TRANSA {}", new_transa);
        // println!("Block {}", blockchain);

        // assert!(r)
    }

    #[test]
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
    }
}
