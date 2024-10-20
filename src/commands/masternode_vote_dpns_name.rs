use std::fs;
use std::str::FromStr;
use clap::Parser;
use dpp::dashcore::hashes::Hash;
use dpp::dashcore::key::Secp256k1;
use dpp::dashcore::{Network, ProTxHash};
use dpp::identifier::{Identifier, MasternodeIdentifiers};
use dpp::identity::accessors::IdentityGettersV0;
use dpp::identity::hash::IdentityPublicKeyHashMethodsV0;
use dpp::identity::{IdentityPublicKey};
use dpp::platform_value::string_encoding::Encoding::{Base58};
use dpp::platform_value::Value;
use dpp::serialization::PlatformSerializable;
use dpp::state_transition::StateTransition;
use dpp::voting::vote_choices::resource_vote_choice::ResourceVoteChoice;
use log::{debug, info};
use sha256::digest;
use crate::errors::cli_argument_missing_error::CommandLineArgumentMissingError;
use crate::errors::Error;
use crate::errors::identity_public_key_hash_mismatch_error::IdentityPublicKeyHashMismatchError;
use crate::factories::Factories;
use crate::grpc::PlatformGRPCClient;
use crate::MockBLS;
use crate::utils::Utils;

/// Perform a masternode vote towards contested DPNS name
#[derive(Parser)]
pub struct MasternodeVoteDPNSNameCommand {
    /// Network, mainnet or testnet
    #[clap(long, default_value(""))]
    network: String,

    /// DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443
    #[clap(long, default_value(""))]
    dapi_url: String,

    /// ProTxHash of the Masternode performing a Vote, in hex
    #[clap(long, default_value(""))]
    pro_tx_hash: String,

    /// Path to file with voting (or owner) private key in WIF format
    #[clap(long, default_value(""))]
    private_key: String,

    /// Normalized label to vote upon (can be grabbed from https//dash.vote)
    #[clap(long, default_value(""))]
    normalized_label: String,

    /// The choice of the Vote.
    /// It can be an Identifier you are voting towards (ex. BMJWm8wKmbApR7nQ6q7RG3HgD8maJ8t7B4yWBKRe7aZ6), or Lock, or Abstain
    #[clap(long, default_value(""))]
    choice: String,

    /// Verbose
    #[clap(long)]
    pub verbose: bool,
}

const DPNS_DATA_CONTRACT_IDENTIFIER: &str = "GWRSAVFMjXx8HpQFaNJMqBV7MBgMK4br5UESsB4S31Ec";

impl MasternodeVoteDPNSNameCommand {
    pub async fn run(&self) -> Result<(), Error> {
        if self.network.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("network")));
        }
        if self.pro_tx_hash.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("pro_tx_hash")));
        }
        if self.normalized_label.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("normalized_label")));
        }
        if self.private_key.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("private_key")));
        }
        if self.dapi_url.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("dapi_url")));
        }
        if self.choice.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("choice")));
        }
        info!("Starting Masternode Vote on {} DPNS name process with choice {} ({})", &self.network, &self.normalized_label, &self.choice);

        let secp = Secp256k1::new();

        let network_type = Network::from_str(&self.network).expect("Could not parse network");
        let private_key_data = fs::read_to_string(&self.private_key).expect("Unable to read private key file");
        let private_key = Utils::decode_private_key_from_input_string(private_key_data.as_str(), network_type)?;
        let public_key = private_key.public_key(&secp);
        let pro_tx_hash = ProTxHash::from_hex(&self.pro_tx_hash).expect("Could not decode pro tx hash");
        let voting_address = public_key.pubkey_hash().to_byte_array();

        let buffer: [u8; 32] = <[u8; 32]>::try_from(hex::decode(&self.pro_tx_hash).unwrap()).unwrap();

        let voter_identity_id = Identifier::create_voter_identifier(&buffer, &voting_address);

        let platform_grpc_client = PlatformGRPCClient::new(&self.dapi_url);

        let identity = platform_grpc_client.get_identity_by_identifier(voter_identity_id).await?;

        debug!("Identity with identifier {} found in the network", identity.id());

        let identity_public_keys = platform_grpc_client
            .get_identity_keys(identity.id()).await;

        debug!("Finding matching IdentityPublicKey in the Identity against applied private key");

        let identity_public_key = identity_public_keys
            .iter()
            .filter(|key| key.public_key_hash().unwrap() == <[u8; 20] as Into<[u8; 20]>>::into(public_key.pubkey_hash().to_byte_array()))
            .collect::<Vec<&IdentityPublicKey>>()
            .first()
            .ok_or(Error::IdentityPublicKeyHashMismatchError(IdentityPublicKeyHashMismatchError::from((identity.id(), public_key.pubkey_hash()))))?
            .clone();

        debug!("Found matching IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
            identity_public_key.id(),
            identity_public_key.key_type(),
            identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
            identity_public_key.purpose(),
            identity_public_key.security_level());

        let choice = match self.choice.as_str() {
            "Lock" => ResourceVoteChoice::Lock,
            "Abstain" => ResourceVoteChoice::Abstain,
            _ => ResourceVoteChoice::TowardsIdentity(Identifier::from_string(&self.choice, Base58).unwrap()),
        };

        let masternode_vote_transition = Factories::create_masternode_vote_state_transition(
            &pro_tx_hash.to_hex(),
            voter_identity_id,
            Identifier::from_string(DPNS_DATA_CONTRACT_IDENTIFIER, Base58).unwrap(),
            "domain",
            "parentNameAndLabel",
            vec![
                Value::Text("dash".to_string()),
                Value::Text(self.normalized_label.clone()),
            ],
            choice,
        );

        let mut masternode_vote_state_transition = StateTransition::from(masternode_vote_transition);

        debug!("Signing IdentityCreditWithdrawal with IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
            identity_public_key.id(),
            identity_public_key.key_type(),
            identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
            identity_public_key.purpose(),
            identity_public_key.security_level());

        masternode_vote_state_transition.sign(&identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();

        let masternode_vote_buffer = masternode_vote_state_transition.clone().serialize_to_bytes().unwrap();
        let masternode_vote_hex = masternode_vote_buffer.clone();
        let masternode_vote_hash = digest(masternode_vote_buffer.clone());
        debug!("Signed MasternodeVote Transaction Hex: {}", masternode_vote_hex.to_lower_hex_string());
        info!("MasternodeVote Transaction Hash: {}", masternode_vote_hash);
        platform_grpc_client.broadcast_state_transition(masternode_vote_state_transition).await;

        println!("Masternode Vote for {} DPNS name has been sucessfully submitted", &self.normalized_label);
        info!("Please check your transaction on the Platform Explorer to make sure it finished successfully");

        Ok(())
    }
}