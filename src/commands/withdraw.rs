use std::fs;
use std::str::FromStr;
use clap::Parser;
use dpp::dashcore::{Network};
use dpp::dashcore::hashes::Hash;
use dpp::dashcore::secp256k1::hashes::hex::DisplayHex;
use dpp::dashcore::secp256k1::Secp256k1;
use dpp::identifier::Identifier;
use dpp::identity::accessors::IdentityGettersV0;
use dpp::identity::core_script::CoreScript;
use dpp::identity::hash::IdentityPublicKeyHashMethodsV0;
use dpp::identity::identity_public_key::accessors::v0::IdentityPublicKeyGettersV0;
use dpp::identity::IdentityPublicKey;
use dpp::platform_value::string_encoding::Encoding::{Base58};
use dpp::serialization::{PlatformSerializable};
use dpp::state_transition::identity_credit_withdrawal_transition::v1::IdentityCreditWithdrawalTransitionV1;
use dpp::state_transition::StateTransition;
use dpp::withdrawal::Pooling;
use log::{debug, info};
use sha256::digest;
use crate::errors::cli_argument_missing_error::CommandLineArgumentMissingError;
use crate::errors::identity_public_key_hash_mismatch_error::IdentityPublicKeyHashMismatchError;
use crate::errors::Error;
use crate::grpc::PlatformGRPCClient;
use crate::MockBLS;
use crate::utils::Utils;

/// Withdraw credits from the Identity to the L1 Core chain
#[derive(Parser)]
pub struct WithdrawCommand {
    /// Network, mainnet or testnet
    #[clap(long, default_value(""))]
    network: String,

    /// DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443
    #[clap(long, default_value(""))]
    dapi_url: String,

    /// Identity address, that initiate withdrawal
    #[clap(long, default_value(""))]
    identity: String,

    /// Path to file with private key from Identity in WIF format
    #[clap(long, default_value(""))]
    private_key: String,

    /// Core withdrawal address (P2PKH / P2SH)
    #[clap(long, default_value(""))]
    withdrawal_address: String,

    /// Amount of credits to withdraw
    #[clap(long, default_value("0"))]
    amount: u64,

    /// Verbose
    #[clap(long)]
    pub verbose: bool,
}

impl WithdrawCommand {
    pub async fn run(&self) -> Result<(), Error> {
        if self.network.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("network")));
        }

        if self.dapi_url.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("dapi_url")));
        }

        if self.identity.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("identity")));
        }

        if self.private_key.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("private_key")));
        }

        if self.withdrawal_address.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("withdrawal_address")));
        }

        if self.amount == 0 {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("amount")));
        }

        info!("Starting Identity Credits Withdrawal from {} {} CREDITS ({} Dash) to {} Dash address ({})", &self.identity, &self.amount, (u64::from(self.amount.clone()) as f64 / 10e10 as f64), &self.withdrawal_address, &self.network);

        let secp = Secp256k1::new();

        let network_type = Network::from_str(&self.network).expect("Could not parse network");
        let private_key_data = fs::read_to_string(&self.private_key).expect("Unable to read private key file");
        let private_key = Utils::decode_private_key_from_input_string(private_key_data.as_str(), network_type)?;
        let public_key = private_key.public_key(&secp);

        let platform_grpc_client = PlatformGRPCClient::new(&self.dapi_url);

        let identifier = Identifier::from_string(&self.identity, Base58).unwrap();

        let identity = platform_grpc_client
            .get_identity_by_identifier(identifier).await?;

        debug!("Identity with identifier {} found in the network", identity.id());

        let identity_public_keys = platform_grpc_client
            .get_identity_keys(identity.id()).await;

        debug!("Finding matching IdentityPublicKey in the Identity against applied private key");

        let identity_public_key = identity_public_keys
            .iter()
            .filter(|key|  key.public_key_hash().unwrap() == <[u8; 20] as Into<[u8;20]>>::into(public_key.pubkey_hash().to_byte_array()))
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

        let nonce = platform_grpc_client.get_identity_nonce(identity.id()).await;

        debug!("Identity nonce for identifier {} is {}", identity.id(), nonce.clone());

        let output_script = CoreScript::new_p2pkh(public_key.pubkey_hash().into());

        let identity_credit_withdrawal_transition = IdentityCreditWithdrawalTransitionV1 {
            identity_id: identifier,
            amount: self.amount,
            core_fee_per_byte: 1,
            pooling: Pooling::Never,
            output_script: Some(output_script),
            nonce: &nonce + 1,
            user_fee_increase: 0,
            signature_public_key_id: 0,
            signature: Default::default(),
        };

        let mut state_transition = StateTransition::from(identity_credit_withdrawal_transition);

        debug!("Signing IdentityCreditWithdrawal with IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
            identity_public_key.id(),
            identity_public_key.key_type(),
            identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
            identity_public_key.purpose(),
            identity_public_key.security_level());
        state_transition.sign(&identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();

        let buffer = state_transition.serialize_to_bytes().unwrap();
        let tx_hash = digest(buffer.clone());

        debug!("Signed IdentityCreditWithdrawal Hex: {}", buffer.to_lower_hex_string());
        info!("IdentityCreditWithdrawal Transaction Hash: {}", tx_hash);

        platform_grpc_client.broadcast_state_transition(state_transition).await;

        info!("Successfully sent IdentityCreditWithdrawal transaction for {} CREDITS from Identity {}", self.amount, identity.id().to_string(Base58));

        Ok(())
    }
}