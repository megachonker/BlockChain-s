
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


//il existe un autre parsing qui utilise une structure au lieux de .arg .arg qui est moin lisible  -> sur la même lib ou une autre ?
fn parse_args() -> ArgMatches {
    Command::new("NIC")
        .version("1.0")
        .author("Thompson")
        .about("A great Block Chain")
        .arg(
            arg!(-p --ip <IP> "Your IP:port for bind the socket")
                .required(false)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-r --receive <num> "The id of the receiver ")
                .required(false)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-s --sender  <num> "Your Id")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            arg!(-m --mode  <MODE> "Wich mode (send, mine) ")
                .required(false)
                .action(ArgAction::Set)
                .default_value("mine"),
        )
        .arg(
            arg!(-g --gate <IP> "The IP:port of the entry point")
                .required(false)
                .action(ArgAction::Set)
                .default_value("0.0.0.0:6021"),         //First node 
        )
        .arg(
            arg!(-c --count <count> "The value amount for the  transaction")
                .required(false)
                .action(ArgAction::Set)
                .default_value("0"),
        )
        .get_matches()
}

fn main() {
    let matches = parse_args();
    Node::start(matches);
}


//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {

}
