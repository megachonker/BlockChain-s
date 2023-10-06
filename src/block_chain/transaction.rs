use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use super::blockchain::{self, Blockchain};

/// can be used iside smart contract
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
struct TxMoula {
    pub value: u128, //wider + simpler + undivisible + optimisation + reusing common acronyme M K
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
struct RxMoula {
    block_location: u64, //dans quelle block est la transa
    transa_id: u64,      //hash de Transaction
    moula_id: usize,     //id mola not refering value but position in the vec
                         // no value !
                         // it can seem verry cringe but there only refering to actual transaction
}

impl RxMoula {
    /// find the value of a utcxo for a given RxMoula
    fn get_utcxo(&self, blockchain: &Blockchain) -> Option<&TxMoula> {
        let mut s = DefaultHasher::new();
        blockchain
            .blocks
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
    rx: Vec<RxMoula>,
    pub tx: Vec<TxMoula>,
    pub target_pubkey: u64,
    //add signature of the sender
}

/// Make the split of the coin
impl Transaction {
    /// Use the blockaine to find money and send it
    pub fn new_online(blockchain: &Blockchain,src:u64, amount: u128, destination: u64) -> Self {
        //not optimal but i is a NP problem see bag problem 
        let (rx,mut unspend) = Self::select_utcxo(src, blockchain, amount);
        
        // Triks compilator to make fee 10%
        unspend*=9;
        unspend/=10;
        
        Self {
            rx,
            tx: vec![TxMoula{value:unspend}],
            target_pubkey: destination,
        }
    }

    /// ofline use actual wallet and create transa
    fn new_offline(input: Vec<RxMoula>, amount: u128, destination: u64) {}

    /// output optimal utxo for a amount given
    /// return a list of utxo used to make transaction
    fn select_utcxo(src:u64,blockchain: &Blockchain,amount:u128) -> (Vec<RxMoula>, u128) {
        //need to gpt
        let mut output = vec![];
        let mut counting = 0;
        let mut s = DefaultHasher::new();

        for block in blockchain.blocks{
            for transa in block.transactions{
                let mut cringe_counter = 0;
                for utxo in transa.tx{
                    cringe_counter+=1;

                    counting += utxo.value;
                    if counting >= amount{
                        break;
                    }   
                    transa.hash(&mut s); 
                    let transaid = s.finish();
                    let rxtransa = RxMoula{block_location:block.block_id,transa_id:transaid,moula_id:cringe_counter};
                    output.push(rxtransa);
                }
            }
        }
        (output,counting - amount)
    }
}
