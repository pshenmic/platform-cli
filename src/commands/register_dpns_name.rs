use std::fs;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use clap::{ Parser};
use dpp::dashcore::hashes::Hash;
use dpp::dashcore::{Network};
use dpp::dashcore::secp256k1::hashes::hex::DisplayHex;
use dpp::dashcore::secp256k1::Secp256k1;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::data_contract::{DataContract};
use dpp::data_contract::conversion::value::v0::DataContractValueConversionMethodsV0;
use dpp::identifier::Identifier;
use dpp::identity::accessors::IdentityGettersV0;
use dpp::identity::hash::IdentityPublicKeyHashMethodsV0;
use dpp::identity::identity_public_key::accessors::v0::IdentityPublicKeyGettersV0;
use dpp::identity::IdentityPublicKey;
use dpp::platform_value::string_encoding::Encoding::Base58;
use dpp::platform_value::{platform_value, Value};
use dpp::serialization::PlatformSerializable;
use dpp::state_transition::StateTransition;
use dpp::util::entropy_generator::EntropyGenerator;
use dpp::util::hash::hash_double;
use dpp::util::strings::convert_to_homograph_safe_chars;
use dpp::version::fee::vote_resolution_fund_fees::v1::VOTE_RESOLUTION_FUND_FEES_VERSION1;
use dpp::version::PlatformVersion;
use log::{debug, info};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sha256::digest;
use tokio::time::sleep;
use crate::errors::cli_argument_missing_error::CommandLineArgumentMissingError;
use crate::errors::Error;
use crate::errors::identity_public_key_hash_mismatch_error::IdentityPublicKeyHashMismatchError;
use crate::factories::create_documents_batch::IdentityStateTransition;
use crate::factories::Factories;
use crate::grpc::PlatformGRPCClient;
use crate::utils::{MyDefaultEntropyGenerator, Utils};
use regex::Regex;
use crate::constants::Constants;
use crate::{MockBLS};

/// Register an Identity Name in the Dash Platform DPNS system.
#[derive(Parser)]
pub struct RegisterDPNSNameCommand {
    /// Network, mainnet or testnet
    #[clap(long, default_value(""))]
    network: String,

    /// DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443
    #[clap(long, default_value(""))]
    dapi_url: String,

    /// Identity address that registers a name
    #[clap(long, default_value(""))]
    identity: String,

    /// Path to file with private key from Identity in WIF format
    #[clap(long, default_value(""))]
    private_key: String,

    /// Name to register (excluding .dash)
    #[clap(long, default_value(""))]
    name: String,

    /// Enable verbose logging for a debugging
    #[clap(long)]
    pub verbose: bool,
}

impl RegisterDPNSNameCommand {
    pub async fn run(&self) -> Result<(), Error> {
        if self.network.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("network")));
        }

        if self.private_key.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("private_key")));
        }

        if self.identity.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("identity")));
        }

        if self.name.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("name")));
        }

        if self.dapi_url.is_empty() {
            return Err(Error::CommandLineArgumentMissingError(CommandLineArgumentMissingError::from("dapi_url")));
        }

        let re = Regex::new(r"^[a-zA-Z01-]{3,19}$").unwrap();

        let normalized_name = convert_to_homograph_safe_chars(&self.name);
        let full_domain_name = format!("{}.dash", &self.name);
        let is_contested = re.is_match(&self.name);

        info!("Starting registering DPNS name process ({})", &self.network);
        info!("Name: {}, Normalized Name: {}, Full Domain Name: {}, Is Contested: {}", &self.name, normalized_name.clone(), &full_domain_name, is_contested);

        let secp = Secp256k1::new();

        let network = if &self.network == "mainnet" { "dash" } else { &self.network  };
        let network_type = Network::from_str(network).expect("Could not parse network");
        let private_key_data = fs::read_to_string(&self.private_key).expect("Unable to read private key file");
        let private_key = Utils::decode_private_key_from_input_string(private_key_data.as_str(), network_type)?;
        let public_key = private_key.public_key(&secp);
        let identifier = Identifier::from_string(&self.identity, Base58).unwrap();

        let dpns_contract = DataContract::from_value(Constants::dpns_data_contract_value(), true, PlatformVersion::latest()).unwrap();

        let platform_grpc_client = PlatformGRPCClient::new(&self.dapi_url);

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
            .ok_or(Error::IdentityPublicKeyHashMismatchError(IdentityPublicKeyHashMismatchError::from((identifier, public_key.pubkey_hash()))))?
            .clone();

        debug!("Found matching IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
            identity_public_key.id(),
            identity_public_key.key_type(),
            identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
            identity_public_key.purpose(),
            identity_public_key.security_level());

        let identity_contract_nonce = platform_grpc_client.get_identity_contract_nonce(identity.id(), dpns_contract.id()).await;

        debug!("Identity contract nonce for identifier {} is {}", identity.id(), identity_contract_nonce.clone());

        let mut rng = StdRng::from_entropy();

        let salt: [u8; 32] = rng.gen();

        let mut salted_domain_buffer: Vec<u8> = vec![];
        salted_domain_buffer.extend(salt);
        salted_domain_buffer.extend((normalized_name.clone() + ".dash").as_bytes());

        let salted_domain_hash = hash_double(salted_domain_buffer);

        debug!("Salted Domain Hash for {} is {}", normalized_name.clone() + ".dash", salted_domain_hash.to_lower_hex_string());

        let generator = MyDefaultEntropyGenerator{};
        let entropy = generator.generate().unwrap();

        let pre_order_document = Factories::create_document(dpns_contract.id(),
                                                            "preorder",
                                                            identity.id(),
                                                            platform_value!(
           {
               "saltedDomainHash": Value::Bytes32(salted_domain_hash)
            }
        ), Vec::from(entropy));

        let pre_order_transition = Factories::document_create_transition(pre_order_document, "preorder", dpns_contract.id(), identity_contract_nonce.add(1), Vec::from(entropy), None);
        let mut preorder_state_transition = StateTransition::from(IdentityStateTransition{
            identity: identity.id(),
            transitions: vec![pre_order_transition]
        });

        debug!("Signing preorder transaction with IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
            identity_public_key.id(),
            identity_public_key.key_type(),
            identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
            identity_public_key.purpose(),
            identity_public_key.security_level());
        preorder_state_transition.sign(identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();

        let preorder_buffer = preorder_state_transition.clone().serialize_to_bytes().unwrap();
        let preorder_hex = preorder_buffer.clone();
        let preorder_hash = digest(preorder_buffer.clone());

        debug!("Signed Preorder Transaction Hex: {}", preorder_hex.to_lower_hex_string());
        info!("Preorder Transaction Hash: {}", preorder_hash);

        platform_grpc_client.broadcast_state_transition(preorder_state_transition).await;

        info!("Preorder document has been successfully sent into the network");

        info!("Waiting 20s for a confirmation in the network");
        sleep(Duration::from_millis(20000)).await;

        let domain_document = Factories::create_document(dpns_contract.id(), "domain", identity.id(),
                                                         platform_value!(
          {
              "label": &self.name,
              "records": {
                "identity": identity.id(),
              },
              "preorderSalt": Value::Bytes32(salt),
              "subdomainRules": {
                "allowSubdomains": false
              },
              "normalizedLabel": normalized_name.clone(),
              "parentDomainName": "dash",
              "normalizedParentDomainName": "dash"
            }
         ), Vec::from(entropy));

        let prefunding_voting_balance = match is_contested {
            true => {Some((String::from("parentNameAndLabel"), VOTE_RESOLUTION_FUND_FEES_VERSION1.contested_document_vote_resolution_fund_required_amount))},
            false => None
        };

        if !prefunding_voting_balance.is_none() {
            info!("Chosen name was detected as a contested resource, including 0.2 Dash in credits as a prefund for voting process");
        }

        let domain_document_transition = Factories::document_create_transition(
            domain_document,
            "domain",
            dpns_contract.id(),
            identity_contract_nonce.add(2),
            Vec::from(entropy), prefunding_voting_balance);

        let mut domain_state_transition = StateTransition::from(IdentityStateTransition{
            identity: identity.id(),
            transitions: vec![domain_document_transition]
        });

        debug!("Signing domain transaction with IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
            identity_public_key.id(),
            identity_public_key.key_type(),
            identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
            identity_public_key.purpose(),
            identity_public_key.security_level());
        domain_state_transition.sign(identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();

        let domain_buffer = domain_state_transition.clone().serialize_to_bytes().unwrap();
        let domain_hex = domain_buffer.clone();
        let domain_hash = digest(domain_buffer.clone());
        debug!("Signed Domain Transaction Hex: {}", domain_hex.to_lower_hex_string());
        info!("Domain Transaction Hash: {}", domain_hash);

        platform_grpc_client.broadcast_state_transition(domain_state_transition).await;

        info!("Successfully registered DPNS Name {} for Identity {}", full_domain_name, identity.id().to_string(Base58));
        info!("Please check your transactions on the Platform Explorer to make sure they all finished successfully");

        if is_contested {
            info!("Your name was registered through the contested resource process, please check if your name appears on the https://dash.vote now");
        }

        Ok(())
    }
}

