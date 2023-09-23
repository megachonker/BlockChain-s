mod friendly_name;

mod block_chain{
    pub mod shared;
    pub mod block;
    pub mod node; 
}

use block_chain::node::{ client::Client,miner::Miner,network::Network,NewNode};
use std::net::{IpAddr, Ipv4Addr};

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

    node.start(); // don't care what we start just starting it
}

// s'ocupe de faire une logique des argument
fn parse_args(cli: Cli) -> NewNode {
    // check un bootstrap spésifier
    let bootstrap;
    if cli.bind.is_none() {
        bootstrap = Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    } else {
        bootstrap = cli.bootstrap
    }

    // create bind address if needed
    let binding;
    if cli.bind.is_none() {
        binding = Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    } else {
        binding = cli.bind
    }

    //create Networking worker
    let networking = Network::new(bootstrap.unwrap(), binding.unwrap());

    // si doit send
    if cli.ammount.is_normal() || !cli.secret.is_empty() || cli.destination != 0 {
        // si manque un arg pour send
        if !(cli.ammount.is_normal() && !cli.secret.is_empty() && cli.destination != 0) {
            panic!("missing amount, secret or destination")
        }
        //create client worker
        //pourait être une action ici si lancer en interpréteur
        //ça serait pas un new mais client::newaction(action)
        return NewNode::Cli(Client::new(
            networking,
            cli.destination,
            cli.secret,
            cli.ammount,
        ));
    } else {
        //create server worker
        return NewNode::Srv(Miner::new(networking));
    }
}

//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {}
