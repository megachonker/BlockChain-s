use ring::signature::{self, KeyPair};
use ring::rand::SystemRandom;
use ring::signature::RsaKeyPair;
use ring::signature::{RSA_PKCS1_2048_8192_SHA256, VerificationAlgorithm};
use std::fs::File;
use std::io::Read;



//enum Node::Client 
//stuct client{
// User
// Network
//}

/*
node en mode Client peut
- send de l'argent
- inspecter la blockaine
- voir le compte du client
*/


// donc client va utiliser User comme truc de base 
//user doit pouvoir implementer masse function comme compaerer UserA et UserB pour les balance


// est possédée par la structure client 
struct User{
    //struct Kriptography <== pour avoir une abstraction 
    public_key : ,
    private_key : ,
}

//Kriptography::load <= file ou clef
//Kriptography::gen
//Kriptography::signe ?
//Kriptography::check <= avec le trai

//Transaction::check(transaction) <== doit vérifier kripto si ces bon

//User::balance <== balance utilisera blockaine
//User::send <== quand on send on fera des op de crypto derrierre
//User::Contract <== faire des contract need structure

//dans les transa contra on fait référance a des user ? 

//ces le truc que j'avait trouver, j'ai gélérée comme une chienne pour trouver une lib qui tien la route gg
//pour moi faut faire une abstra cripto
fn generate_and_save_key_pair() -> Result<(), Box<dyn std::error::Error>> {//obliger de box ? //pourquoi présice pas erro?
    // Generate a new key pair
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?; /////need to update kargooooooo

    // Save the private key to a file
    std::fs::write("private_key.pem", &pkcs8_bytes)?;

    // Derive the public key from the private key
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    let public_key_bytes = key_pair.public_key().as_ref();

    // Save the public key to a file
    std::fs::write("public_key.pem", public_key_bytes)?;

    Ok(())//je fait pas ça mais ces pas si con
}


fn load_and_use_key_pair() -> Result<(), Box<dyn std::error::Error>> {
    let rng = SystemRandom::new();

    // Load the private key from a file
    let mut private_key_file = File::open("private_key.pem")?;
    let mut private_key_bytes = Vec::new();
    private_key_file.read_to_end(&mut private_key_bytes)?;

    // Load the public key from a file
    let mut public_key_file = File::open("public_key.pem")?;
    let mut public_key_bytes = Vec::new();
    public_key_file.read_to_end(&mut public_key_bytes)?;

    // Create key pair from the loaded private key bytes
    let key_pair = RsaKeyPair::from_pkcs8(&RSA_PKCS1_2048_8192_SHA256, private_key_bytes.as_ref())?;

    // Encrypt a message using the public key
    let message = b"Hello, world!";
    let mut ciphertext = vec![0; key_pair.public_modulus_len()];
    key_pair.public_encrypt(&rng, message, &mut ciphertext)?;

    // Decrypt the ciphertext using the private key
    let mut plaintext = vec![0; key_pair.public_modulus_len()];
    key_pair.private_decrypt(ciphertext.as_ref(), &mut plaintext)?;

    // Verify the decrypted message matches the original message
    assert_eq!(&plaintext[..message.len()], message);

    Ok(())
}