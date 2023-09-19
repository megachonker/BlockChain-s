
mod friendly_name;

mod block_chain {
    pub mod block;
    pub mod kademlia;
    pub mod node;
    pub mod shared;
}


use std::net::IpAddr;

use block_chain::node::Node;
use block_chain::shared;

#[derive(Parser)]
struct Cli {
    /// Addresse ip: du serveur a utiliser pour boostrap
    bootstrap: Option<IpAddr>,
    
    /// Addresse ip: adresse a utiliser
    bind: Option<IpAddr>,
    
    /// Address reception: addresse contenant le virement
    #[arg(short, long)]
    destination: u64,
    
    /// Montant: nombre de crédit a donner
    #[arg(short, long)]
    ammount: f64,
    
    /// Key file: fichier contenant la clef privée
    #[arg(short, long)]
    secret: String,
}


use clap::{arg, ArgAction, ArgMatches, Command, Parser};


fn main() {
    let arg = Cli::parse();
    let node = parse_args();
    // node.start();
}


// s'ocupe de faire une logique des argument
fn parse_args() -> Node{

    Node::new()
}
//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {

}
