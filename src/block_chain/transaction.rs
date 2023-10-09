use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use super::blockchain::{self, Blockchain};

/// can be used iside smart contract
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
struct TxUtxo {
    pub value: u128, //wider + simpler + undivisible + optimisation + reusing common acronyme M K
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct RxUtxo {
    block_location: u64, //dans quelle block est la transa
    transa_id: u64,      //hash de Transaction
    moula_id: usize,     //id mola not refering value but position in the vec
    // no value !
    // it can seem verry cringe but there only refering to actual transaction
    value: u128, //can work without but Simplify the challenge NOT NEED TO SERIALIZED
}

impl fmt::Display for RxUtxo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
║Rx: [{}=>{}=>{}] {}",
            self.block_location, self.transa_id, self.moula_id, self.value,
        )
    }
}

impl RxUtxo {
    /// find the value of a utcxo for a given RxMoula
    fn get_utcxo<'a>(&self, blockchain: &'a Blockchain) -> Option<&'a TxUtxo> {
        let mut s = DefaultHasher::new();
        blockchain
            .get_chain()
            .iter()
            .find(|block| block.block_id == self.block_location)
            .and_then(|block| {
                block
                    .transactions
                    .iter()
                    .find(|transa| {
                        transa.hash(&mut s);
                        s.finish() == self.transa_id
                    })
                    .and_then(|good_transa| good_transa.tx.get(self.moula_id))
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Transaction {
    rx: Vec<RxUtxo>,
    tx: Vec<TxUtxo>, //fist is what is send back
    pub target_pubkey: u64,
    //add signature of the sender
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╔════════════════RX═════════════════════╗").unwrap();
        for transrx in &self.rx {
            writeln!(f, "║{}", transrx).unwrap();
        }
        writeln!(f, "╠════════════════TX═════════════════════╣").unwrap();
        for transtx in &self.tx {
            writeln!(f, "║{}", transtx.value).unwrap();
        }
        write!(
            f,
            "\
╠═══════════════════════════════════════╣
║Sender PubKey: {}
╚═══════════════════════════════════════╝",
            self.target_pubkey,
        )
    }
}

/// Make the split of the coin
impl Transaction {
    /// Use the blockaine to find money and send it
    pub fn new_online(
        blockchain: &Blockchain,
        source: u64,
        amount: u128,
        destination: u64,
    ) -> Self {
        let utxos = blockchain.filter_utxo(source);

        //not optimal but i is a NP problem see bag problem
        let (rx, resend) = Self::select_utxo_from_vec(&utxos, amount);

        Self {
            rx,
            tx: vec![TxUtxo { value: resend }, TxUtxo { value: amount }],
            target_pubkey: destination,
        }
    }

    /// ofline use actual wallet and create transa
    pub fn new_offline(input: &Vec<RxUtxo>, amount: u128, destination: u64) -> Transaction {
        let (rx, resend) = Self::select_utxo_from_vec(input, amount);

        Self {
            rx,
            tx: vec![TxUtxo { value: resend }, TxUtxo { value: amount }],
            target_pubkey: destination,
        }
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
    fn select_utxo_from_vec(avaible: &Vec<RxUtxo>, amount: u128) -> (Vec<RxUtxo>, u128) {
        let fee = amount / 10;
        let mut sum = 0;
        let r: Vec<RxUtxo> = avaible
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
        let to_send_back = sum - (amount + fee) ;
        (r, to_send_back)
    }

    pub fn get_utxos(&self, block_location: u64) -> Vec<RxUtxo> {
        let mut s = DefaultHasher::new();
        let mut no = 0;
        self.tx
            .iter()
            .map(|tx| {
                no += 1;
                self.hash(&mut s);

                RxUtxo {
                    block_location,
                    transa_id: s.finish(),
                    moula_id: no,
                    value: tx.value,
                }
            })
            .collect()
    }
}



#[cfg(test)]
mod tests {
    use crate::block_chain::transaction::{RxUtxo, Transaction};



    #[test]
    fn test_select_utxo_from_vec(){
        let rx_7 = RxUtxo{block_location:0,transa_id:0,moula_id:0,value:5};
        let rx_3 = RxUtxo{block_location:0,transa_id:0,moula_id:0,value:4};
        let rx_2 = RxUtxo{block_location:0,transa_id:0,moula_id:0,value:8};
        let rx_9 = RxUtxo{block_location:0,transa_id:0,moula_id:0,value:9};

        let wallet = vec![rx_7,rx_3,rx_2,rx_9];
        
        let (transa,sendback) = Transaction::select_utxo_from_vec(&wallet,10);
        transa.iter().for_each(|transa|print!("{}",transa));
        let full:u128 = transa.iter().map(|f|f.value).into_iter().sum();
        let total_cost = full-10;
        println!("\nneed to send back:{}, total spend with fee:{}",sendback,total_cost);
        assert_eq!(sendback,6);
        assert_eq!(total_cost,7)
    }
}  
