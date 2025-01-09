use std::{fs, thread};
use std::ops::{Add, Mul};
use std::str::FromStr;
use std::time::Duration;
use clap::{Parser};
use dashcore_rpc::{Auth, Client, json, RpcApi};
use dpp::dashcore::hashes::Hash;
use dashcore::hash_types::Txid;
use dashcore_rpc::dashcore::address::AddressEncoding;
use dashcore_rpc::jsonrpc::Response;
use dpp::dashcore::{Address, Network, OutPoint, PrivateKey, ScriptBuf, Transaction, TxIn, TxOut};
use dpp::dashcore::psbt::serialize::Serialize;
use dpp::dashcore::secp256k1::hashes::hex::DisplayHex;
use dpp::dashcore::secp256k1::{Message, Secp256k1};
use dpp::dashcore::sighash::{LegacySighash, SighashCache};
use dpp::dashcore::transaction::special_transaction::asset_lock::AssetLockPayload;
use dpp::dashcore::transaction::special_transaction::TransactionPayload;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::serialization::PlatformSerializable;
use crate::errors::cli_argument_missing_error::CommandLineArgumentMissingError;
use crate::errors::Error;
use crate::utils::{MyDefaultEntropyGenerator, Utils};
use crate::api::digitalcash::DigitalCashAPI;
use dpp::dashcore;
use dpp::ed25519_dalek::ed25519::signature::SignerMut;
use dpp::identifier::Identifier;
use dpp::identity::{KeyType, Purpose, SecurityLevel};
use dpp::identity::state_transition::asset_lock_proof::chain::ChainAssetLockProof;
use dpp::platform_value::BinaryData;
use dpp::platform_value::string_encoding::Encoding::Base58;
use dpp::prelude::{AssetLockProof, Identity, IdentityPublicKey};
use dpp::state_transition::public_key_in_creation::accessors::IdentityPublicKeyInCreationV0Setters;
use dpp::state_transition::public_key_in_creation::IdentityPublicKeyInCreation;
use dpp::state_transition::public_key_in_creation::v0::IdentityPublicKeyInCreationV0;
use dpp::state_transition::StateTransition;
use log::debug;
use serde_json::{json, Value};
use sha256::digest;
use crate::factories::Factories;
use crate::grpc::PlatformGRPCClient;
use crate::MockBLS;
use rust_decimal::prelude::*;
use tokio::task::id;

/// Register an Identity Name in the Dash Platform DPNS system.
#[derive(Parser)]
pub struct RegisterIdentityCommand {
    /// Network, mainnet or testnet
    #[clap(long, default_value(""))]
    network: String,

    /// DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443
    #[clap(long, default_value(""))]
    dapi_url: String,

    /// Path to file with private key from Identity in WIF format
    #[clap(long, default_value(""))]
    private_key: String,

    /// Amount in DASH
    #[clap(long, default_value(""))]
    amount: String,

    /// Enable verbose logging for a debugging
    #[clap(long)]
    pub verbose: bool,
}

// create assetlock (
// create identitycreate transition

impl RegisterIdentityCommand {
    pub async fn run(&self) -> Result<(), Error> {
        if self.network.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("network")));
        }

        if self.dapi_url.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("dapi_url")));
        }

        if self.private_key.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("private_key")));
        }

        let secp = Secp256k1::new();
        let network_type: Network = Network::from_str(&self.network).expect("Could not parse network");
        let private_key_data = fs::read_to_string(&self.private_key).expect("Unable to read private key file");

        let funding_private_key = Utils::decode_private_key_from_input_string(private_key_data.as_str(), network_type)?;
        let funding_public_key = funding_private_key.public_key(&secp);
        let funding_address = Address::p2pkh(&funding_public_key, network_type);

        let duffs_amount: u64 = Decimal::from_str(&self.amount).unwrap().mul(Decimal::new(100000000, 0)).to_u64().unwrap();
        let core_tx_fee: u64 = 10000;

        let asset_lock_onetime_private_key_bytes: [u8; 32] = rand::random();
        let asset_lock_onetime_private_key = PrivateKey::from_slice(asset_lock_onetime_private_key_bytes.as_slice(), network_type).unwrap();
        let asset_lock_onetime_public_key = asset_lock_onetime_private_key.public_key(&secp);

        let platform_grpc_client = PlatformGRPCClient::new(&self.dapi_url);
        let rpc = DigitalCashAPI::new(network_type);
        let core_client = Client::new(
            &"127.0.0.1:19998",
            Auth::UserPass(
                String::from("dashmate"),
                String::from("nQf1xDUKsEkG"),
            ),
        )
            .ok().unwrap();

        println!("Funding address {}", &funding_address.to_string());

        let inputs: Vec<(TxIn, u64)> = Utils::wait_for_balance(&rpc, funding_address, duffs_amount).await?;

        let core_transaction = Factories::create_assetlock_transaction(inputs,
                                                                       duffs_amount,
                                                                       core_tx_fee,
                                                                       funding_private_key,
                                                                       asset_lock_onetime_public_key,
                                                                       network_type);

        let vout = core_transaction.output.iter().position(|tx_out|{tx_out.script_pubkey.is_op_return()}).unwrap() as u32;
        let txid = platform_grpc_client.broadcast_core_transaction(core_transaction.clone()).await;

        println!("Core Transaction AssetLock tx hash {}", txid.to_hex());

        let asset_lock_proof = Utils::wait_for_asset_lock_proof(&rpc, &core_client, core_transaction, vout).await?;
        let identity_id = asset_lock_proof.create_identifier().unwrap();

        let auth_master_private_key_bytes: [u8; 32] = rand::random();
        let auth_master_private_key = PrivateKey::from_slice(auth_master_private_key_bytes.as_slice(), network_type).unwrap();
        let auth_master_public_key = auth_master_private_key.public_key(&secp);

        println!("Auth Master Private Key: {}", auth_master_private_key.to_string());

        let mut auth_master_identity_public_key = IdentityPublicKeyInCreation::V0(IdentityPublicKeyInCreationV0 {
            id: 0,
            key_type: KeyType::ECDSA_SECP256K1,
            purpose: Purpose::AUTHENTICATION,
            security_level: SecurityLevel::MASTER,
            contract_bounds: None,
            read_only: false,
            data: auth_master_public_key.to_bytes().into(),
            signature: Default::default(),
        });


        let auth_high_private_key_bytes: [u8; 32] = rand::random();
        let auth_high_private_key = PrivateKey::from_slice(auth_high_private_key_bytes.as_slice(), network_type).unwrap();
        let auth_high_public_key = auth_high_private_key.public_key(&secp);

        println!("Auth High Private Key: {}", auth_high_private_key.to_string());

        let mut auth_high_identity_public_key = IdentityPublicKeyInCreation::V0(IdentityPublicKeyInCreationV0 {
            id: 1,
            key_type: KeyType::ECDSA_SECP256K1,
            purpose: Purpose::AUTHENTICATION,
            security_level: SecurityLevel::HIGH,
            contract_bounds: None,
            read_only: false,
            data: auth_high_public_key.to_bytes().into(),
            signature: Default::default(),
        });

        let auth_critical_private_key_bytes: [u8;32] = rand::random();
        let auth_critical_private_key = PrivateKey::from_slice(auth_critical_private_key_bytes.as_slice(), network_type).unwrap();
        let auth_critical_public_key = auth_critical_private_key.public_key(&secp);

        println!("Auth Critical Private Key: {}", auth_critical_private_key.to_string());

        let mut auth_critical_identity_public_key = IdentityPublicKeyInCreation::V0(IdentityPublicKeyInCreationV0 {
            id: 2,
            key_type: KeyType::ECDSA_SECP256K1,
            purpose: Purpose::AUTHENTICATION,
            security_level: SecurityLevel::CRITICAL,
            contract_bounds: None,
            read_only: false,
            data: auth_critical_public_key.to_bytes().into(),
            signature: Default::default(),
        });

        let transfer_critical_private_key_bytes: [u8;32]  = rand::random();
        let transfer_critical_private_key = PrivateKey::from_slice(transfer_critical_private_key_bytes.as_slice(), network_type).unwrap();
        let transfer_critical_public_key = transfer_critical_private_key.public_key(&secp);

        println!("Transfer Critical Private Key: {}", transfer_critical_private_key.to_string());

        let mut transfer_critical_identity_public_key = IdentityPublicKeyInCreation::V0(IdentityPublicKeyInCreationV0 {
            id: 3,
            key_type: KeyType::ECDSA_SECP256K1,
            purpose: Purpose::TRANSFER,
            security_level: SecurityLevel::CRITICAL,
            contract_bounds: None,
            read_only: false,
            data: transfer_critical_public_key.to_bytes().into(),
            signature: Default::default(),
        });

        let mut identity_create_transition = Factories::create_identity_create_transition(identity_id, asset_lock_proof.clone(), vec![
            auth_master_identity_public_key.clone(),
            auth_high_identity_public_key.clone(),
            auth_critical_identity_public_key.clone(),
            transfer_critical_identity_public_key.clone()
        ]);

        let mut state_transition = StateTransition::from(identity_create_transition.clone());
        state_transition.sign_by_private_key(auth_master_private_key_bytes.as_slice(), KeyType::ECDSA_SECP256K1, &MockBLS {}).unwrap();
        let mut signature = state_transition.signature().clone();
        auth_master_identity_public_key.set_signature(signature);
        state_transition.set_signature(Default::default());

        state_transition = StateTransition::from(identity_create_transition.clone());
        state_transition.sign_by_private_key(auth_critical_private_key_bytes.as_slice(), KeyType::ECDSA_SECP256K1, &MockBLS {}).unwrap();
        signature = state_transition.signature().clone();
        auth_critical_identity_public_key.set_signature(signature);
        state_transition.set_signature(Default::default());

        state_transition = StateTransition::from(identity_create_transition.clone());
        state_transition.sign_by_private_key(auth_high_private_key_bytes.as_slice(), KeyType::ECDSA_SECP256K1, &MockBLS {}).unwrap();
        signature = state_transition.signature().clone();
        auth_high_identity_public_key.set_signature(signature);
        state_transition.set_signature(Default::default());

        state_transition = StateTransition::from(identity_create_transition.clone());
        state_transition.sign_by_private_key(transfer_critical_private_key_bytes.as_slice(), KeyType::ECDSA_SECP256K1, &MockBLS {}).unwrap();
        signature = state_transition.signature().clone();
        transfer_critical_identity_public_key.set_signature(signature);
        state_transition.set_signature(Default::default());

        identity_create_transition = Factories::create_identity_create_transition(identity_id, asset_lock_proof, vec![
            auth_master_identity_public_key.clone(),
            auth_high_identity_public_key.clone(),
            auth_critical_identity_public_key.clone(),
            transfer_critical_identity_public_key.clone()
        ]);

        state_transition = StateTransition::from(identity_create_transition.clone());

        state_transition.sign_by_private_key(asset_lock_onetime_private_key.to_bytes().as_slice(), KeyType::ECDSA_SECP256K1, &MockBLS {}).unwrap();

        let state_transition_buffer = state_transition.clone().serialize_to_bytes().unwrap();
        let state_transition_hash = digest(state_transition_buffer.clone());
        println!("Signed IdentityCreate Transaction Hash: {}", state_transition_hash);
        println!("Signed IdentityCreate Transaction Hex: {}", state_transition_buffer.to_lower_hex_string());
        platform_grpc_client.broadcast_state_transition(state_transition).await;
        println!("");
        Ok(())
    }
}

