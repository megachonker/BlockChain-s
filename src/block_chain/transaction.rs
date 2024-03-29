use anyhow::{anyhow, ensure, Context, Ok, Result};
use dryoc::{
    sign::{IncrementalSigner, PublicKey, SecretKey, Signature, SignedMessage, SigningKeyPair},
    types::{ByteArray, Bytes},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    default,
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
    pub fn to_utxo(&self, balance: &Balance) -> Option<Utxo> {
        balance.txin_to_utxo(&self).cloned()
    }

    // fn get_pubkey(&self, balance: &Balance) {
    //     self.to_utxo(balance)?.target
    // }
}

impl UtxoValidator<&Balance> for TxIn {
    fn valid(&self, balence: &Balance) -> Option<bool> {
        //check if possible to convert
        //check if already spend

        let res = balence.get(self.to_owned());
        Some(res.is_some() && res.unwrap().1.is_valid())

        // Some(self.to_owned().to_utxo(balence).is_some())
    }
}

impl Display for TxIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {}", self.location)
    }
}

#[derive(Clone)]
enum ComeFromID {
    BlockHeigt(u64),
    TxIn(Vec<TxIn>),
}

impl Hash for Utxo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.amount.hash(state);
        let target = bincode::serialize(&self.target).unwrap();
        target.hash(state);
        self.come_from.hash(state);
    }
}

/// # Unspend transaction Output
///
/// - need to be unique
/// - can be spend once
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Utxo {
    /// quantity of money
    amount: Amount,

    /// who can spend utxo
    /// public key destination
    target: PublicKey,

    /// make the Utxo UNIQUE
    /// sum of all Utxin
    come_from: HashValue,
}

/// Generate a "Valid" utxo if the asociated Balance was create by default
impl Default for Utxo {
    fn default() -> Self {
        Self {
            // asociated to the block 0
            come_from: 0,
            amount: 1,
            target: Default::default(),
        }
    }
}

impl Utxo {
    pub fn to_txin(&self) -> TxIn {
        TxIn {
            location: self.get_hash(),
        }
    }

    /// get the target key that need to be used in the transaction
    /// to proof the owner
    pub fn get_pubkey(&self) -> PublicKey {
        self.target.clone()
    }

    /// get the value of the token
    pub fn get_amount(&self) -> Amount {
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

impl Hash for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rx.hash(state);
        self.tx.hash(state);
        let signature: Vec<u8> = bincode::serialize(&self.signatures).unwrap();
        signature.hash(state);
    }
}

/// # Verification
#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Transaction {
    pub rx: Vec<TxIn>,
    pub tx: Vec<Utxo>,
    // Wasm challenge
    // wasm:Vec<u8>,
    /// signature of all  field
    pub signatures: Vec<Signature>,
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
        let mut need_pubkey = HashSet::new();
        for utxo in &self.rx {
            let tmp = utxo.clone();
            let encoded = tmp
                .to_utxo(balance)
                .context("cannot convert txin to utxo")?
                .target;

            need_pubkey.insert(encoded.as_array().clone());
        }

        // let vall = need_pubkey.iter().next().unwrap().clone();
        // let a = PublicKey::from(vall);

        //find key in common
        let r = key
            .iter()
            .filter(|keypair| {
                need_pubkey
                    .iter()
                    .any(|a_key| a_key.clone().clone().eq(keypair.0.public_key.as_array()))
            })
            .cloned()
            .collect();
        Ok(r)
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
    ) -> Result<Self> {
        let total_ammount = amount + acount.miner_fee;
        let (selected, sendback) = acount
            .select_utxo(total_ammount)
            .context("imposible d'avoir la some demander")?;

        let rx: Vec<TxIn> = selected.iter().map(|utxo| utxo.to_txin()).collect();
        let cum = ComeFromID::TxIn(rx.clone());
        let tx = if sendback > 0 {
            vec![
                //transaction
                Utxo::new(amount, destination, cum.clone()),
                //fragment de transaction a renvoyer a l'envoyeur
                Utxo::new(sendback, acount.get_pubkey(), cum),
            ]
        } else {
            vec![
                //transaction
                Utxo::new(amount, destination, cum.clone()),
                //fragment de transaction a renvoyer a l'envoyeur
            ]
        };

        let sigining_key: Vec<Keypair> = acount
            .get_keypair(&selected)
            .context("imposible de trouver les paire de clef")?;

        /*         //first signature we signe transa
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
        */

        let mut signatures = vec![];

        for key in sigining_key {
            let mut signer = IncrementalSigner::new();
            signer.update(&bincode::serialize(&rx).context("imposible de serialiser")?);
            signer.update(&bincode::serialize(&tx).context("imposible de serialiser")?);
            signatures.push(signer.finalize(&key.0.secret_key)?);
        }

        let transaction = Self { rx, tx, signatures };

        // Update wallet
        // can triguerre here a hanndler to know were transa done
        acount.wallet.retain(|transa| !selected.contains(&transa));

        Ok(transaction)
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
    pub fn check_sign(&self, balance: &Balance) -> Result<()> {
        let mut public_keys = vec![];
        for txin in &self.rx {
            public_keys.push(
                txin.to_utxo(balance)
                    .context("convestion failed")?
                    .get_pubkey(),
            );
        }

        ensure!(public_keys.len() == self.signatures.len(),"le nombre de clef pour ouvrire la transaction ne match pas avec le nombre de signature");

        // Ici, on utilise zip pour itérer simultanément sur les signatures et les clés publiques.
        for (signature, public_key) in self.signatures.iter().zip(public_keys.iter()) {
            let mut signer = IncrementalSigner::new();
            signer.update(&bincode::serialize(&self.rx).context("imposible de serialiser")?);
            signer.update(&bincode::serialize(&self.tx).context("imposible de serialiser")?);
            signer
                .verify(signature, public_key)
                .context("signature fausse")?;
        }
        Ok(())

        /*         let pubkeys: Vec<PublicKey> = self.rx.iter().map(|i| i.to_utxo(balance)?.target).collect();
        let pubkeys: Vec<PublicKey> = pubkeys.reverse();

        let signature: Signature = bincode::deserialize(&self.signatures)?;
        let message = self.rx + self.tx;
        let sigmsg: SignedMessage = SignedMessage::from_parts(signature, message)?;
        sigmsg.verify(pubkeys.first()?)?; */
    }

    /// # NEED TEST
    ///
    /// ## Create a Reward transaction for miner
    ///
    pub fn transform_for_miner(
        mut transas: Vec<Transaction>,
        key: Keypair,
        block_heigt: u64,
        balance: &Balance,
    ) -> Result<Vec<Transaction>> {
        let mut miner_reward = MINER_REWARD;

        let mut place_remove = None;

        for (i, t) in transas.iter().enumerate() {
            if t.rx.is_empty() && t.tx.len() == 1 {
                place_remove = Some(i)
            } else {
                miner_reward += t.remains(balance).unwrap() as Amount;
            }
        }
        if place_remove.is_some() {
            transas.remove(place_remove.unwrap()); //reward transa already present remove it
        }

        let tx = vec![Utxo::new(
            miner_reward,
            key.clone().into(),
            ComeFromID::BlockHeigt(block_heigt),
        )];

        /* let mut signer = IncrementalSigner::new();
        signer.update(&bincode::serialize(&tx).context("can not serialiaze")?);
        let signatures = vec![signer
            .finalize(&key.0.secret_key)
            .context("wrong signature")?]; */

        transas.push(Transaction {
            rx: vec![],
            tx,
            signatures: vec![Signature::default()],
        });

        Ok(transas)
    }

    pub fn update_transa_for_miner(
        transas: &mut Vec<Transaction>,
        new_transa: &Transaction,
        balance: &Balance,
        key: &Keypair,
    ) -> Result<()> {
        let index_miner_transa = transas
            .iter()
            .enumerate()
            .find(|t| t.1.rx.is_empty() && t.1.tx.len() == 1)
            .unwrap()
            .0;

        let miner_reward =
            transas[index_miner_transa].tx[0].amount + new_transa.remains(balance).unwrap();

        transas[index_miner_transa].tx[0].amount = miner_reward;

        transas.push(new_transa.clone());

        Ok(())
    }

    /// How many remain for the miner
    /// return None if negative value
    ///
    /// Need to be CHECKED
    pub fn remains(&self, balance: &Balance) -> Result<Amount> {
        let input = self
            .rx
            .iter()
            .try_fold(0, |acc, txin| txin.to_utxo(balance).map(|f| acc + f.amount))
            .context("can not convert to to_utxo, entry missing in balence ?")?;

        let output = self.tx.iter().map(|t| t.amount).sum();

        input
            .checked_sub(output)
            .with_context(|| format!("remains:{} - {}", input, output))
    }
}

impl UtxoValidator<&Balance> for Transaction {
    fn valid(&self, balence: &Balance) -> Option<bool> {
        //on lose la propagation d'erreur .. ? add context ?
        let rx_status = self.rx.iter().all(|t| t.valid(balence).unwrap_or(false));
        // let tx_status = self.tx.iter().all(|t| t.valid(arg).unwrap_or(false));       //not necessary to be in balence
        let sold = self.remains(balence).is_ok();
        let signature = if !(self.rx.is_empty() && self.tx.len() == 1) {
            !self.check_sign(&balence).is_err()
        } else {
            true
        };
        Some(rx_status && sold && signature)
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

    use rand::Rng;

    use crate::block_chain::{
        block::{self, Block, Profile},
        blockchain::{self, FIRST_DIFFICULTY},
        transaction::{Transaction, Utxo},
    };

    use super::*;

    #[test]
    fn create_utxo() {
        fn check_failure(input: Utxo) {
            assert!(!input.valid(&Default::default()).unwrap_or(false));
        }
        let incorect_amount = Utxo::new(2, Default::default(), ComeFromID::BlockHeigt(0));
        let incorect_block = Utxo::new(1, Default::default(), ComeFromID::BlockHeigt(1));
        let from_somthing_not_exist =
            Utxo::new(2, Default::default(), ComeFromID::TxIn(Default::default()));
        let correct = Utxo::new(1, Default::default(), ComeFromID::BlockHeigt(0));

        check_failure(incorect_amount);
        check_failure(incorect_block);
        check_failure(from_somthing_not_exist);

        assert!(correct.valid(&Default::default()).unwrap_or(false));
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
        let utxo_s = blockchain.filter_utxo(Default::default());
        ///1
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

    #[test]
    /// test if the default utxo + Balance is working
    /// by default it create one utxo that can be spended
    fn spend_default_utxo() {
        let utxo: Utxo = Default::default();
        assert!(utxo.valid(&Default::default()).unwrap_or(false))
    }

    #[test]
    fn simple_transaction() {
        fn update(c: &mut Acount, b: &Balance) {
            c.refresh_wallet(b.filter_utxo(c.get_pubkey())).unwrap();
        }

        fn mine(h: u64, block: Block, c: &Acount, b: &mut Balance, t: Vec<Transaction>) -> Block {
            let transactions =
                Transaction::transform_for_miner(t, c.get_signkeypair(), h, b).unwrap();
            let block_1 = block
                .find_next_block(transactions, Profile::INFINIT, FIRST_DIFFICULTY)
                .unwrap();
            // balance.add(&block).unwrap();
            b.add(&block_1).unwrap();
            block_1
        }

        let mut balance = Balance::default();

        let mut compt_user = Acount::default();
        let mut compt_miner = Acount::default();

        assert_eq!(compt_miner.get_sold(), 0);
        assert_eq!(compt_user.get_sold(), 0);

        let block = Block::default();

        let block_1 = mine(1, block, &compt_miner, &mut balance, vec![]);

        update(&mut compt_miner, &balance);
        update(&mut compt_user, &balance);
        assert_eq!(compt_miner.get_sold(), 1);
        assert_eq!(compt_user.get_sold(), 0);

        let block_2 = mine(2, block_1, &compt_miner, &mut balance, vec![]);

        update(&mut compt_miner, &balance);
        update(&mut compt_user, &balance);
        assert_eq!(compt_miner.get_sold(), 2);
        assert_eq!(compt_user.get_sold(), 0);

        let mine_to_comt =
            Transaction::new_transaction(&mut compt_miner, 1, compt_user.get_pubkey()).unwrap();

        let block_3 = mine(3, block_2, &compt_miner, &mut balance, vec![mine_to_comt]);

        update(&mut compt_miner, &balance);
        update(&mut compt_user, &balance);
        assert_eq!(compt_miner.get_sold(), 2);
        assert_eq!(compt_user.get_sold(), 1);
        println!("{block_3}");
        println!("{compt_user}");
        println!("{compt_miner}");
        println!("{balance}");
        
    }

    #[test]
    fn signature_multiple_keypair() {
        let recv = Acount::default();

        //new account
        let mut acc_sending = Acount::default();
        let acc_sending_bis = Acount::default();

        let utxo_a = Utxo::new(10, acc_sending.get_pubkey(), ComeFromID::BlockHeigt(1));
        let utxo_b = Utxo::new(2, acc_sending_bis.get_pubkey(), ComeFromID::BlockHeigt(0));

        //add moula
        let moulat = vec![utxo_a, utxo_b.clone()];
        acc_sending.wallet = moulat.clone();

        let balance = &Balance::new(moulat);

        //check if balance not enought
        assert!(Transaction::new_transaction(&mut acc_sending, 12, recv.get_pubkey()).is_err());

        //check when missing key
        assert!(Transaction::new_transaction(&mut acc_sending, 11, recv.get_pubkey()).is_err());

        // add keypair of bis to signe
        acc_sending.add_key(
            acc_sending_bis
                .get_keypair(&vec![utxo_b])
                .unwrap()
                .first()
                .unwrap()
                .clone(),
        );

        //create transaciton
        let mut transa =
            Transaction::new_transaction(&mut acc_sending, 11, recv.get_pubkey()).unwrap();

        assert!(transa.check_sign(balance).is_ok());
        println!("{transa}");

        //check if signature altered
        let e = transa.signatures.first_mut().unwrap();
        *e = Signature::default();
        assert!(transa.check_sign(balance).is_err());

        //check if no signature
        transa.signatures = vec![];
        assert!(transa.check_sign(balance).is_err())
    }

    /*
    #[test]
    fn simple_signature() {
        fn update(c: &mut Acount, b: &Balance) {
            c.refresh_wallet(b.filter_utxo(c.get_pubkey())).unwrap();
        }

        fn mine(h: u64, block: Block, c: &Acount, b: &mut Balance, t: Vec<Transaction>) -> Block {
            let transactions =
                Transaction::transform_for_miner(t, c.get_signkeypair(), h, b).unwrap();
            let block_1 = block
                .find_next_block(transactions, Profile::INFINIT, FIRST_DIFFICULTY)
                .unwrap();
            // balance.add(&block).unwrap();
            b.add(&block_1).unwrap();
            block_1
        }

        let mut balance = Balance::default();
        let mut compt_user = Acount::default();
        let mut compt_miner = Acount::default();
        let block = Block::default();

        let block_1 = mine(1, block.clone(), &compt_miner, &mut balance, vec![]);
        block_1.check(&balance).unwrap();

        let block_2 = mine(2, block_1, &compt_miner, &mut balance, vec![]);

        update(&mut compt_miner, &balance);
        // block_2.check(&balance).unwrap();

        let mine_to_comt =
            Transaction::new_transaction(&mut compt_miner, 1, compt_user.get_pubkey()).unwrap();
        let block_3 = mine(3, block_2, &compt_miner, &mut balance, vec![mine_to_comt]);

        block_3.check(&balance).unwrap();

        println!("{compt_user}");
        println!("{balance}");
    } */
}

// need to test:
// merge 3:2
// transition 2:2
// split 2:3
