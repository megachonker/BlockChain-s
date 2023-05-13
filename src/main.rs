use block_chain::{Transaction,Block,hash};

// fn main() {
//     //guy's
//     let maximator = hash("uss");
//     let neeto = hash("neeto");
//     let chonker = hash("chonker");

//     let transaction_a = Transaction::new(maximator,chonker, 100);
//     let transaction_b = Transaction::new(chonker,neeto, 10);
    
//     let origin_block = Block::new(vec![transaction_a]);
//     assert!(origin_block.check());

//     let block_1 = origin_block.new_block(vec![transaction_b],chonker);
//     assert!(block_1.check());
//     println!("last block:\n{:?}\nnew block:\n{:?}",origin_block,block_1);
// }


use w3f_bls::{Keypair,ZBLS,Message,Signed};


fn main() {

    let mut keypair = Keypair::<ZBLS>::generate(::rand::thread_rng());
    let message = Message::new(b"Some context",b"Some message");
    let sig = keypair.sign(&message);
    assert!( sig.verify(&message,&keypair.public) );
}