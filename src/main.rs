mod commands;
mod grpc;

mod factories;
pub(crate) mod utils;
mod errors;
mod logger;

use clap::{Parser, Subcommand};
use dpp::{BlsModule, ProtocolError, PublicKeyValidationError};
use crate::commands::masternode_vote_dpns_name::MasternodeVoteDPNSNameCommand;
use crate::commands::register_dpns_name::RegisterDPNSNameCommand;
use crate::commands::withdraw::WithdrawCommand;
use log::{LevelFilter};
use crate::logger::Logger;

pub struct MockBLS {

}

impl BlsModule for MockBLS {
    fn validate_public_key(&self, pk: &[u8]) -> Result<(), PublicKeyValidationError> {
        panic!("BLS signatures are not implemented");
    }

    fn verify_signature(&self, signature: &[u8], data: &[u8], public_key: &[u8]) -> Result<bool, ProtocolError> {
        panic!("BLS signatures are not implemented");
    }

    fn private_key_to_public_key(&self, private_key: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        panic!("BLS signatures are not implemented");
    }

    fn sign(&self, data: &[u8], private_key: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        panic!("BLS signatures are not implemented");
    }
}


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

    match result {
        Ok(_) => (),
        Err(err) => {
            println!("Error: {}", err)
        }
    };
}