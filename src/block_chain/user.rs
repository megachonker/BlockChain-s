use ring::signature::{self, KeyPair};
use ring::rand::SystemRandom;
use ring::signature::RsaKeyPair;
use ring::signature::{RSA_PKCS1_2048_8192_SHA256, VerificationAlgorithm};
use std::fs::File;
use std::io::Read;



struct User{
    public_key : ,
    private_key : ,
}

fn generate_and_save_key_pair() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a new key pair
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;

    // Save the private key to a file
    std::fs::write("private_key.pem", &pkcs8_bytes)?;

    // Derive the public key from the private key
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    let public_key_bytes = key_pair.public_key().as_ref();

    // Save the public key to a file
    std::fs::write("public_key.pem", public_key_bytes)?;

    Ok(())
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