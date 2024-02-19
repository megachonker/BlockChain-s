mod friendly_name;

mod block_chain {
    // pub mod shared;
    pub mod acount;
    pub mod block;
    pub mod blockchain;
    pub mod node;
    pub mod transaction;
}
use anyhow::{bail, ensure, Context, Result};
use block_chain::{
    acount::Acount,
    node::{
        client::{self, Client},
        network::Network,
        server::Server,
        NewNode,
    },
    transaction::Amount,
};
use clap::Parser;
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};
use tracing::info;

#[derive(Parser)]
pub struct Cli {
    /// Addresse ip: du serveur a utiliser pour l'initialisation
    bootstrap: Option<IpAddr>,

    /// Addresse ip: adresse a utiliser
    bind: Option<IpAddr>,

    /// Address reception: fichier du wallet destination
    #[arg(short, long, default_value_t = String::from(""))]
    destination: String,

    /// Montant: nombre de crédit a donner
    #[arg(short, long, default_value_t = 0)]
    ammount: Amount,

    /// Key file: fichier contenant le port money
    #[arg(short, long,default_value_t = String::from("default.usr"))]
    path: String,

    /// niveaux de verbositée 1-3
    #[arg(short,default_value_t =String::from("WARN") )]
    verbose: String,

    /// nombre de thread a utiliser
    #[arg(short,long, default_value_t = 1)]
    threads: u16,

    /// crée un nouveaux compte
    #[arg(short,long, default_value_t = false)]
    create_account: bool,

    /// Stat d'un compte
    #[arg(short, long, default_value_t = false)]
    stat: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            bootstrap: Some(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            bind: Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            ammount: 0,
            destination: "".to_string(),
            create_account: false,
            path: "".to_string(),
            threads: 1,
            verbose: "LOG".to_string(),
            stat: false,
        }
    }
}

fn main() -> Result<()> {
    //get argument
    let arg = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str(&arg.verbose).unwrap_or(tracing::Level::WARN))
        .init();

    // si doit recrée un compte
    if arg.create_account {
        ensure!(!arg.path.is_empty(), "missing path for create new user !");

        client::Client::new_wallet(arg.path.as_str())?;
        info!("Successfully create new user !");
        return Ok(());
    }

    // print stat of a wallet
    // update wallet
    if arg.stat {
        let user = Acount::load(&arg.path)?;

        let networking = Network::new(arg.bootstrap.unwrap(), arg.bind.unwrap());
        if let Ok(user) =
            Client::new(networking, user.clone(), Default::default(), Default::default()).refresh_wallet()
        {
            info!("wallet updated from network");
            println!("{user}");
        }
        else {
            info!("cannot access net for update local readed");
            println!("{user}");
        }
        
        info!("Successfully print stat wallet !");
        return Ok(());
    }

    //check error of logique
    let node = parse_args(arg)?;

    // don't care what we start just starting it
    node.start()
}

/// parsing argument
fn parse_args(cli: Cli) -> Result<NewNode> {
    //create user
    let user = Acount::load(&cli.path)?;

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
    if !cli.destination.is_empty() || cli.ammount != 0 {
        // si manque un arg pour send
        ensure!(cli.ammount != 0, "missing amount");
        ensure!(
            !cli.destination.is_empty(),
            "missing destination for transaction"
        );

        let destination = Acount::load(&cli.destination).unwrap().get_pubkey();

        //create client worker
        Ok(NewNode::Cli(Client::new(
            networking,
            user,
            destination,
            cli.ammount,
        )))
    } else {
        //create server worker
        Ok(NewNode::Srv(Server::new(
            networking,
            user.get_key()
                .first()
                .context("cannot get keypair for starting server")?
                .clone(),
            cli.threads,
        )))
    }
}

//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {
    use crate::{parse_args, Cli};
    use std::net::Ipv4Addr;

    #[test]
    fn argument_lunch_server_init() {
        //server mode
        let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));

        let cli = Cli {
            bind,
            bootstrap,
            path: "test.usr".to_string(),
            ..Default::default()
        };
        parse_args(cli).unwrap();

        //client mode
        let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)));
        let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let cli = Cli {
            bind,
            bootstrap,
            path: "test.usr".to_string(),
            ..Default::default()
        };
        parse_args(cli).unwrap();
    }

    #[test]
    fn test_lunch_server_init() {
        // let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)));
        // let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        // let cli = Cli {
        //     ammount: f64::NAN,
        //     bind,
        //     bootstrap,
        //     destination: u64::MIN,
        //     secret: String::new(),
        // };
        // let a = parse_args(cli);

        // //client mode
        // let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 4)));
        // let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)));
        // let cli = Cli {
        //     ammount: f64::NAN,
        //     bind,
        //     bootstrap,
        //     destination: u64::MIN,
        //     secret: String::new(),
        // };
        // let b = parse_args(cli);

        // tokio::runtime::Builder::new_current_thread()
        //     .enable_all()
        //     .build()
        //     .unwrap()
        //     .block_on(async {
        //         // assert!(true);

        //         // futures::Future::
        //         // futures::executor::block_on(async {

        //         //seed mode

        //         let a = async {
        //             println!("start server");
        //             a.start()
        //         };
        //         let b = async {
        //             thread::sleep(Duration::from_secs(3));
        //             println!("START client");
        //             b.start()
        //         };

        //         tokio::select!
        //     });

        // tokio::task::select();

        //     // let h = thread::spawn(|| thread::sleep(Duration::from_secs(5)));
        //     // let sleep = async { h.join().unwrap() };

        //     // let my_future = join(a, b).fuse();
        //     let a = a.fuse();
        //     let b = b.fuse();

        //     pin_mut!(a, b);
        //     select! {
        //         _ = a =>{},
        //         _ = b =>{},
        //         // _ = my_future =>{},
        //         // _ = sleep.fuse() =>{},
        //     }
        // });
    }
}
