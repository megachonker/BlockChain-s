mod friendly_name;

mod block_chain {
    // pub mod shared;
    pub mod block;
    pub mod node;
    pub mod blockchain;
    pub mod transaction;
}

use block_chain::node::{client::Client, network::Network, server::Server, NewNode};
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
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    //get argument
    let arg = Cli::parse();

    //check error of logique
    let node = parse_args(arg);

    // don't care what we start just starting it
    node.start();
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
        return NewNode::Srv(Server::new(networking));
    }
}

//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use crate::{parse_args, Cli};

    #[test]
    fn argument_lunch_server_init() {
        //seed mode
        let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        let cli = Cli {
            ammount: f64::NAN,
            bind,
            bootstrap,
            destination: u64::MIN,
            secret: String::new(),
        };
        parse_args(cli);

        //client mode
        let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)));
        let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let cli = Cli {
            ammount: f64::NAN,
            bind,
            bootstrap,
            destination: u64::MIN,
            secret: String::new(),
        };
        parse_args(cli);
    }
}
