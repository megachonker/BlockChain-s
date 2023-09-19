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
    #[arg(short, long, default_value_t = u64::MIN)]
    destination: u64,

    /// Montant: nombre de crédit a donner
    #[arg(short, long, default_value_t = f64::NAN)]
    ammount: f64,

    /// Key file: fichier contenant la clef privée
    #[arg(short, long,default_value_t = String::new())]
    secret: String,
}

use clap::Parser;

fn main() {
    //get argument
    let arg = Cli::parse();
    //check error of logique
    let node = parse_args(arg);
    // node.start();
}

// s'ocupe de faire une logique des argument
fn parse_args(cli: Cli) -> Node {
    // check un bootstrap spésifier
    if cli.bootstrap.expect("address ip invalide").is_unspecified() {
        panic!("no valide bootstrap ip given")
    }

    // si doit send
    if cli.ammount.is_normal() || !cli.secret.is_empty() || cli.destination != 0 {
        // si manque un arg pour send
        if !(cli.ammount.is_normal() && !cli.secret.is_empty() && cli.destination == 0) {
            panic!("missing amount, secret or destination")
        }
    }
    println!("{:?}", cli.bootstrap);
    println!("{} {} {}", cli.ammount, cli.destination, cli.secret);

    Node::new()
}
//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {}
