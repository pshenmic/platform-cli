use std::fs;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use clap::{ Parser};
use dpp::dashcore::hashes::Hash;
use dpp::dashcore::PrivateKey;
use dpp::dashcore::secp256k1::Secp256k1;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::data_contract::{DataContract, JsonValue};
use dpp::data_contract::conversion::value::v0::DataContractValueConversionMethodsV0;
use dpp::identifier::Identifier;
use dpp::identity::accessors::IdentityGettersV0;
use dpp::identity::hash::IdentityPublicKeyHashMethodsV0;
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
use log::info;
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
use crate::utils::MyDefaultEntropyGenerator;
use regex::Regex;
use crate::MockBLS;

/// Register an Identity Name in the Dash Platform DPNS system.
#[derive(Parser)]
pub struct RegisterDPNSNameCommand {
    /// DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443
    #[clap(long, default_value(""))]
    dapi_url: String,

    /// Identity address that registers a name
    #[clap(long, default_value(""))]
    identity: String,

    /// Identity private key in WIF format
    #[clap(long, default_value(""))]
    private_key: String,

    /// Name to register (excluding .dash)
    #[clap(long, default_value(""))]
    name: String,
}

impl RegisterDPNSNameCommand {
    pub async fn run(&self) -> Result<(), Error> {
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

        let secp = Secp256k1::new();

        let private_key_data = fs::read_to_string(&self.private_key).expect("Unable to read file");
        let private_key = PrivateKey::from_wif(&private_key_data).expect("Could not load private key from WIF");
        let public_key = private_key.public_key(&secp);
        let identifier = Identifier::from_string(&self.identity, Base58).unwrap();

        let dpns_data_contract_data = fs::read_to_string("dpns_contract.json").expect("Unable to read file");
        let json_value = JsonValue::from_str(&dpns_data_contract_data).expect("Could not decode DPNS data contract json");
        let raw_data_contract: Value = Value::from(json_value);
        let dpns_contract = DataContract::from_value(raw_data_contract, true, PlatformVersion::latest()).unwrap();

        let platform_grpc_client = PlatformGRPCClient::new(&self.dapi_url);

        let identity = platform_grpc_client
            .get_identity_by_identifier(identifier).await?;

        let identity_public_keys = platform_grpc_client
            .get_identity_keys(identity.id()).await;

        let identity_public_key = identity_public_keys
            .iter()
            .filter(|key|  key.public_key_hash().unwrap() == <[u8; 20] as Into<[u8;20]>>::into(public_key.pubkey_hash().to_byte_array()))
            .collect::<Vec<&IdentityPublicKey>>()
            .first()
            .ok_or(Error::IdentityPublicKeyHashMismatchError(IdentityPublicKeyHashMismatchError::from((identifier, public_key.pubkey_hash()))))?
            .clone();

        let identity_contract_nonce = platform_grpc_client.get_identity_contract_nonce(identity.id(), dpns_contract.id()).await;

        let mut rng = StdRng::from_entropy();

        let salt: [u8; 32] = rng.gen();

        let mut salted_domain_buffer: Vec<u8> = vec![];
        salted_domain_buffer.extend(salt);
        salted_domain_buffer.extend((convert_to_homograph_safe_chars(&self.name) + ".dash").as_bytes());

        let salted_domain_hash = hash_double(salted_domain_buffer);

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

        preorder_state_transition.sign(identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();

        let preorder_buffer = preorder_state_transition.clone().serialize_to_bytes().unwrap();

        platform_grpc_client.broadcast_state_transition(preorder_state_transition).await;

        info!("Successfully broadcasted preorder document, waiting 15s for confirmation, tx hash: {}", hex::encode(digest(preorder_buffer)));

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
              "normalizedLabel": convert_to_homograph_safe_chars(&self.name),
              "parentDomainName": "dash",
              "normalizedParentDomainName": "dash"
        }), Vec::from(entropy));


        let re = Regex::new(r"^[a-zA-Z01-]{3,19}$").unwrap();

        let prefunding_voting_balance = match !re.is_match(&self.name) {
            true => {Some((String::from("parentNameAndLabel"), VOTE_RESOLUTION_FUND_FEES_VERSION1.contested_document_vote_resolution_fund_required_amount))},
            false => None
        };

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

        domain_state_transition.sign(identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();

        let domain_buffer = domain_state_transition.clone().serialize_to_bytes().unwrap();

        platform_grpc_client.broadcast_state_transition(domain_state_transition).await;

        info!("Successfully broadcasted domain document, tx hash: {}", hex::encode(digest(domain_buffer)));

        Ok(())
    }
}