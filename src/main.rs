mod friendly_name;

mod block_chain {
    // pub mod shared;
    pub mod block;
    pub mod blockchain;
    pub mod node;
    pub mod transaction;
    pub mod user;
}
use anyhow::{bail, Context, Result};
use block_chain::{
    node::{
        client::{self, Client},
        network::Network,
        server::Server,
        NewNode,
    },
    user::User,
};
use clap::Parser;
use tracing::info;
use std::{
    default,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

#[derive(Parser)]
pub struct Cli {
    /// Addresse ip: du serveur a utiliser pour l'initialisation
    bootstrap: Option<IpAddr>,

    /// Addresse ip: adresse a utiliser
    bind: Option<IpAddr>,

    /// Address reception: addresse contenant le virement
    #[arg(short, long, default_value_t = u32::MIN)]
    destination: u32,

    /// Montant: nombre de crédit a donner
    #[arg(short, long, default_value_t = 0)]
    ammount: u64,

    /// Key file: fichier contenant le port money
    #[arg(short, long,default_value_t = String::from("default.usr"))]
    path: String,

    /// niveaux de verbositée 1-3
    #[arg(short,default_value_t =String::from("WARN") )]
    verbose: String,

    /// nombre de thread a utiliser
    #[arg(long, default_value_t = 1)]
    threads: u16,

    /// crée un nouveaux compte
    #[arg(long, default_value_t = false)]
    jouvance: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            bootstrap: Some(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            bind: Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            ammount: 0,
            destination: 0,
            jouvance: false,
            path: "".to_string(),
            threads: 1,
            verbose: "LOG".to_string(),
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
    if arg.jouvance {
        if arg.path.is_empty(){
            bail!("missing path for create new user !");
        }
        client::Client::new_wallet(arg.path.as_str())?;
        info!("Successfully create new user !");
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
    let user = User::load(&cli.path)?;
    
    
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
    if !cli.path.is_empty() || cli.destination != 0 {
        // si manque un arg pour send
        if cli.ammount == 0 {
            bail!("missing amount")
        }

        if cli.destination == 0 {
            bail!("missing destination for transaction")
        }

        //create client worker
        Ok(NewNode::Cli(Client::new(
            networking,
            user,
            Default::default(),
            cli.ammount,
        )))
    } else {
        //create server worker
        Ok(NewNode::Srv(Server::new(networking, cli)))
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
            ..Default::default()
        };
        parse_args(cli).unwrap();

        //client mode
        let bind = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)));
        let bootstrap = Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let cli = Cli {
            bind,
            bootstrap,
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
