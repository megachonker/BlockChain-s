use dryoc::sign::*;
use serde::{Deserialize, Serialize};

use super::transaction::Utxo;

#[derive(Debug, Serialize, Deserialize,Default)]
pub struct ToSave{
    wallet: Vec<Utxo>,
    privkey:SecretKey,
}

pub struct User{
    data :ToSave
}

impl User{
    pub fn create() -> User{
        User { data: Default::default() }
    }

    fn new_key(path:String){
        
    }
    
    fn load_key(path: String){
        
    }
    
    fn sign_transa(){}
    
}

fn main() {

    
    // Generate a random keypair, using default types
    let keypair = SigningKeyPair::gen_with_defaults();
    let message = b"Fair is foul, and foul is fair: Hover through the fog and filthy air.";
    
    // Sign the message, using default types (stack-allocated byte array, Vec<u8>)
    let signed_message = keypair.sign_with_defaults(message).expect("signing failed");
    let (a,mut b) = signed_message.into_parts();
    b = "Fair is foul, and foul is fair: Hover through the fog and filthy air.".into();
    let signed_message = SignedMessage::from_parts(a, b);
    
    // Verify the message signature
    signed_message
        .verify(&keypair.public_key)
        .expect("verification failed");
}


#[cfg(test)]
mod test{
    #[test]
    fn serialize_unserialize_key(){

    }

    #[test]
    fn sign_transaction_verrify(){

    }
    
}
