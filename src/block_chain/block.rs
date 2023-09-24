use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash,Hasher};
use std::sync::{Arc, Mutex};

const HASH_MAX: u64 = 1000000000000;

#[derive(Debug, Serialize, Deserialize , Clone)]
pub struct Block {
    pub block_id: u64,                  //the hash of whole block
    block_height: u64,              //the number of the current block
    parent_hash: u64,               //the id of last block (block are chain with that)
    transactions: Vec<Transaction>, //the vector of all transaction validated with this block
    miner_hash: u64,                //Who find the answer
    nonce: u64,                     //the answer of the defi
    quote : String,
}
#[derive(Debug, Hash, Serialize, Deserialize, Clone)]
pub struct Transaction {////////////////////on peut implémenter des **TRAI** de transaction ici   
    src: u64,  //who send coin
    dst: u64,  //who recive
    qqty: u32, //the acount
}

pub fn hash<T: Hash>(value: T) -> u64 {
    //return the hash of the item (need to have Hash trait)
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl Block {
    /// create the first block full empty
    pub fn new() -> Block {
        let mut block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: vec![],
            nonce: 0,
            miner_hash: 0,
            quote : String::from(""),
        };
        block.nonce = 0;
        block.block_id = hash(&block); //the
        block
    }

    //PARDON ? ces pas clean ??
    pub fn new_wrong(value: u64) -> Block {
        let mut block = Block {
            block_height: 0,
            block_id: 0,
            parent_hash: 0,
            transactions: vec![],
            nonce: value, //for the block zero the nonce indique the status of the block (use to response to GetBlock(i))
            miner_hash: 0,
            quote : String::from(""),

        };
        block.block_id = hash(&block); //the
        block
    }

    pub fn get_height_nonce(&self) -> (u64, u64) {
        (self.block_height, self.nonce)
    }

    //la structure transaction peut faire un check sur le block donc pourait être un trait requi d'une transaction  --> une transaction est verifier surtout par les mineurs, pas vraiment duarnt la creation mais plutot dans l'interegration dans un bloc 
    //une transaction peut utiliser le trait check pour check si le node est correct (last version blockaine)       --> comment ca un node correct, pas un block plutot ? 
    //la transaction peut check check si le compte est bon si on fait une structure compte on peut metre le trait check  --> Une struct compte peut être une bonne idée mais elle serait pour quoi ? Parce que si on tien a jour tout les compte ca peut faire beaucoup (en gros en soit a chaque transa un regarde si c'est valid ou alors on tiens les comptes a jours) 
    pub fn check(&self) -> bool {
        let mut hasher = DefaultHasher::new(); //why don't use hash fun ? hash(self) ?? like in last commit  -> je pense faut refaire un peu les hash (nottament il faut que le hash prennent en compte plus de chose comme l'id du hasheur pour la securité)

        //playload of block to hash
        // self.block_height.hash(&mut hasher);
        self.parent_hash.hash(&mut hasher);
        // self.transactions.hash(&mut hasher);     //tres variable donc osef
        // self.miner_hash.hash(&mut hasher);
        // self.quote.hash(&mut hasher);
        self.nonce.hash(&mut hasher);

        let answer = hasher.finish();
        answer < HASH_MAX && hash(self) == self.block_id && self.quote.len() < 100
    }

    

    pub fn generate_block(&self, finder: u64,transactions:Vec<Transaction>, mut quote : &str,should_stop: &Arc<Mutex<bool>>) ->Option<Block>{
        //wesh ces l'enfer ça 
        //si tu check comme ça ces que le buffer peut être gros
        //faut check si ces pas des carac chelou
        if quote.len() >100{
            quote = "";
        }

        let mut new_block = Block {
            block_height: self.block_height + 1,
            block_id: 0,
            parent_hash: self.block_id,
            transactions, //put befort because the proof of work are link to transaction 
            nonce: 0,
            miner_hash: finder,//j'aime pas 
            quote : String::from(quote),
        };
        new_block.nonce = mine(&new_block, should_stop)?; //putain...
        new_block.block_id = hash(&new_block); //set the correct id
        Some(new_block)
    }

}

impl Hash for Block {
    //implement the Hash's trait for Block
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_height.hash(state);
        self.parent_hash.hash(state);
        self.transactions.hash(state);
        self.miner_hash.hash(state);
        self.quote.hash(state);
        self.nonce.hash(state);
    }
}

//exelent!
impl PartialEq for Block {
    fn eq(&self, o: &Block) -> bool {
        self.block_id == o.block_id
    }
}


//comment ça ?
pub fn mine(block: &Block, should_stop: &Arc<Mutex<bool>>) -> Option<u64> {
    let mut rng = rand::thread_rng(); //to pick random value
    let mut hasher = DefaultHasher::new();

    //playload of block to hash
    // block.block_height.hash(&mut hasher);
    block.parent_hash.hash(&mut hasher);
    block.transactions.hash(&mut hasher); //on doit fixer la transaction a avoir 
    // block.miner_hash.hash(&mut hasher);
    // block.quote.hash(& mut hasher); 

    let mut nonce_to_test = rng.gen::<u64>();

    loop {
        let mut to_hash = hasher.clone(); //save l'état du hasher
        nonce_to_test.hash(&mut to_hash);

        let answer = to_hash.finish();

        if answer < HASH_MAX {
            return Some(nonce_to_test);
        }
        nonce_to_test = nonce_to_test.wrapping_add(1);
        if nonce_to_test % 100000 == 0 {
            //test not all time (mutex has big complexity)
            {
                let mut val = should_stop.lock().unwrap();
                //each time making hashing make a comparaison for debug not cool
                if *val {
                    *val = false;
                    return None;
                }
            }
        }
    }
}

//pourait être dans un autre fichier car les transaction travaille sur la BLOCKCHAINE qui elle meme a des transa  --> les transa on peut les faire autre part
//en gros une transaction peut être un TRAI a blockchaine est a block  -> je comprend pas bien la phrase, et je vois pas comment une transa peut être un trait ? 
// ce trait poura avoir un for et spécifier ce qu'on veux faire transiter
impl Transaction {
    pub fn new(src: u64, dst: u64, qqt: u32) -> Transaction {
        Transaction {
            src,
            dst,
            qqty: qqt,
        }
    }
}

#[cfg(test)]
mod tests {
    
}
