mod commands;
mod grpc;

mod factories;
pub(crate) mod utils;
mod errors;
mod logger;

use std::{str::FromStr};
use std::future::Future;
use clap::{Parser, Subcommand};
use crate::commands::masternode_vote_dpns_name::MasternodeVoteDPNSNameCommand;
use crate::commands::register_dpns_name::RegisterDPNSNameCommand;
use crate::commands::withdraw::WithdrawCommand;
use log::{LevelFilter};
use crate::logger::Logger;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: MyCommand,
}

#[derive(Subcommand)]
enum MyCommand {
    Withdraw(WithdrawCommand),
    RegisterDPNSName(RegisterDPNSNameCommand),
    MasternodeVoteDPNSName(MasternodeVoteDPNSNameCommand)
}

static LOGGER: Logger = Logger;

#[tokio::main]
async fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info)).unwrap();

    let args = Args::parse();

    let result = match args.cmd {
        MyCommand::Withdraw(x) => x.run().await,
        MyCommand::RegisterDPNSName(x) => x.run().await,
        MyCommand::MasternodeVoteDPNSName(x) => x.run().await,
    };

    let a = match result {
        Ok(_) => {
            println!()
        }
        Err(err) => {
            println!("Error: {}", err)
        }
    };
}