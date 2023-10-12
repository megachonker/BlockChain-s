use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    default, fmt,
    hash::{Hash, Hasher},
};
use tokio::sync::mpsc::Receiver;

use super::block::Block;
use super::blockchain::{self, Blockchain};

use tokio::select;
use tokio::sync::{mpsc, RwLock};

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Utxo {
    block_location: u64, //dans quelle block est la transa
    transa_id: u64,      //hash de Transaction
    moula_id: usize,     //id mola not refering value but position in the vec
    // no value !
    // it can seem verry cringe but there only refering to actual transaction
    value: u128, //can work without but Simplify the challenge NOT NEED TO SERIALIZED
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
║Rx: [{}=>{}=>{}] {}",
            self.block_location, self.transa_id, self.moula_id, self.value,
        )
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    rx: Vec<Utxo>,
    tx: Vec<u128>, //fist is what is send back
    pub target_pubkey: u64,
    //add signature of the sender
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for transrx in &self.rx {
            writeln!(f, "║{}", transrx).unwrap();
        }
        write!(f,"║Tx[").unwrap();
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
    pub fn new(rx: Vec<Utxo>, tx: Vec<u128>, target_pubkey: u64) -> Transaction {
        Transaction {
            rx,
            tx,
            target_pubkey,
        }
    }

    /// create a Utxo from a know output transaction in the transa
    /// we Create a utxo from it and we need the block hash
    /// transaction don't know where it is in blockain it need to ask
    /// then create the "fake" utxo
    /// it important to remember that the value is PASTED HERE
    /// BUT NO NEEDED
    /// in reality we need to ask blockain about the amount every time we
    /// face a utxo it take time
    ///
    /// because utxo gen here cannot be a & (i think need box)
    pub fn find_new_utxo(&self, block_location: u64) -> Vec<Utxo> {
        let mut s = DefaultHasher::new();
        let mut no = 0;
        self.tx
            .iter()
            .map(|tx| {
                no += 1;
                self.hash(&mut s);

                Utxo {
                    block_location,
                    transa_id: s.finish(),
                    moula_id: no,
                    value: tx.clone(),
                }
            })
            .collect()
    }

    /// fin utxo taken at input in the block
    ///
    /// it better to use & but it simplify to no
    pub fn find_used_utxo(&self) -> Vec<Utxo> {
        self.rx.clone()
    }

    /// Invalidate Transaction from blockaine
    /// Import new transa from network
    /// write a RwLock the updated vec transa
    async fn runner(
        mut from_network: Receiver<Transaction>,
        mut from_block: Receiver<Transaction>,
        shared_var: Arc<RwLock<Vec<Transaction>>>,
    ) {
        let mut transaction_register: HashMap<Transaction, bool> = HashMap::new();

        loop {
            select! {
                transa_from_net = from_network.recv() => {
                    if let Some(transaction) = transa_from_net {
                        transaction_register.entry(transaction).or_insert(true);
                    } else {
                        break; // Network channel closed
                    }
                },
                transa_from_block = from_block.recv() => {
                    if let Some(transaction) = transa_from_block {
                        transaction_register.insert(transaction, false);
                    } else {
                        break; // Block channel closed
                    }
                }
            }

            let valid_transactions: Vec<Transaction> = transaction_register
                .iter()
                .filter_map(|(k, v)| if *v { Some(k.clone()) } else { None })
                .collect();

            let mut shared_data = shared_var.write().await;
            *shared_data = valid_transactions;
        }
    }

    pub fn check(&self) -> bool {
        !self.rx.is_empty()
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
    /// whant send 10
    ///
    /// with fee it need to search for 11 for give 1 to miner
    ///
    /// at input there 7 3 2 9
    ///
    /// stop at 12  
    ///
    /// it select 7 3 2
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

    //generate transaction
    // fn  fuzzy_transa(number:u32,){

    // }
    //input number of transaction
    //output
}

#[cfg(test)]
mod tests {
    use std::{
        default,
        sync::{Arc, Barrier},
        thread,
        time::Duration,
    };

    use tokio::sync::{mpsc::channel, RwLock};

    use crate::block_chain::{
        block::{Block, Profile},
        blockchain::Blockchain,
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
        let full: u128 = transa.iter().map(|f| f.value).into_iter().sum();
        let total_cost = full - 10;
        println!(
            "\nneed to send back:{}, total spend with fee:{}",
            sendback, total_cost
        );
        assert_eq!(sendback, 6);
        assert_eq!(total_cost, 7)
    }

    #[tokio::test]
    async fn runner_lunch() {
        //setup
        let (from_network_tx, from_network_rx) = channel(10);
        let (from_block_tx, from_block_rx) = channel(10);
        let valid = Arc::new(RwLock::new(vec![Transaction::default()]));

        //put the runner spawned to the Futures of the async
        tokio::task::spawn(Transaction::runner(
            from_network_rx,
            from_block_rx,
            valid.clone(),
        ));

        //create fake transaction
        let t1 = Transaction::new_offline(&Default::default(), 0, 1).unwrap();
        let t2 = Transaction::new_offline(&Default::default(), 0, 2).unwrap();
        let t3 = Transaction::new_offline(&Default::default(), 0, 3).unwrap();
        let t4 = Transaction::new_offline(&Default::default(), 0, 4).unwrap();

        //adding transaction from the network
        from_network_tx.send(t1.clone()).await.unwrap();
        from_network_tx.send(t2.clone()).await.unwrap();
        from_network_tx.send(t4.clone()).await.unwrap();

        //invalidate the t1 transaction
        from_block_tx.send(t1.clone()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(1)).await; // wait that runner process transaction
        let stor = valid.read().await;
        assert!(!stor.contains(&t1));
        assert!(stor.contains(&t2));
        assert!(!stor.contains(&t3));
        assert!(stor.contains(&t4))
    }

    #[test]
    fn test_new_online() {
        // let mut blockchain = Blockchain::new();
        // let miner_self_transa = Transaction::new(Default::default(), vec![100], 1);

        // //forge teh fist block
        // let org_block = Block::new().find_next_block(1, vec![miner_self_transa],Profile::INFINIT).unwrap();

        // //append fist block with original money
        // let (block,nhsh) = blockchain.append(&org_block);

        // // create random transaction
        // let transactions = vec![
        //     Transaction::new_online(&blockchain, 1, 25, 10).unwrap(),
        //     Transaction::new_online(&blockchain, 1, 25, 10).unwrap(),
        //     Transaction::new_online(&blockchain, 1, 25, 11).unwrap(),
        // ];


        // //mine the next block with the new transaction
        // let block = block.unwrap().find_next_block(1, transactions,Profile::INFINIT).unwrap();

        // //add it to the blockaine
        // let (block,nhsh) = blockchain.append(&block);

        // println!("{} {:?}", block.unwrap(),nhsh);
        // assert!(true)
    }
}
