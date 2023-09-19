mod block_chain {
    pub mod block;
    pub mod kademlia;
    pub mod node;
    pub mod shared;
}


use block_chain::node::Node;
use block_chain::shared;



use clap::{arg, ArgAction, ArgMatches, Command};


//il existe un autre parsing qui utilise une structure au lieux de .arg .arg qui est moin lisible
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


//des scénario de test avec 2 node par ex
#[cfg(test)]
mod tests {

}
