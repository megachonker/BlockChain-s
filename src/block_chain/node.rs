use client::Client;
use server::Server;

pub mod client;
pub mod network;
pub mod server;
use anyhow::{Context, Result};

pub enum NewNode {
    Srv(Server),
    Cli(Client),
}

impl NewNode {
    pub fn start(self) -> Result<()> {
        match self {
            Self::Cli(cli) => cli.start(),
            Self::Srv(srv) => srv.start(),
        }
    }
}
