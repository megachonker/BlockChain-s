use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
struct TxMoula {
    value: u128, //wider + simpler + undivisible + optimisation + reusing common acronyme M K
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
struct RxMoula {
    block_location: Vec<u64>, //dans quelle block est la transa
    transa_id: u64,           //hash de Transaction
    moula_id: u64,            //id mola not refering value but position in the vec
                              // no value !
                              // verry cringe but there only refering to actual transaction
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Transaction {
    rx: Vec<RxMoula>,
    tx: Vec<TxMoula>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            rx: vec![],
            tx: vec![],
        }
    }
}
