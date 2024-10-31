use std::fs;
use std::ops::Add;
use std::str::FromStr;
use clap::{Parser};
use dpp::dashcore::hashes::Hash;
use dpp::dashcore::{Address, Network, OutPoint, ScriptBuf, Transaction, Txid, TxIn, TxOut};
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
        let private_key = Utils::decode_private_key_from_input_string(private_key_data.as_str(), network_type)?;
        let public_key = private_key.public_key(&secp);
        let address = Address::p2pkh(&public_key, network_type);

        let rpc = DigitalCashAPI::new(network_type);
        let resp = rpc.get_address_utxos(address).await;

        let inputs: Vec<(TxIn, u64)> = resp.into_iter().map(|utxo| {
            let tx_in = TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_hex(&utxo.txid).unwrap(),
                    vout: utxo.output_index,
                },
                script_sig: ScriptBuf::from_hex(&utxo.script).unwrap(),
                sequence: 0xFFFFFFFF,
                witness: Default::default(),
            };

            return (tx_in, utxo.satoshis);
        })
            .collect::<Vec<(TxIn, u64)>>();

        let balance = inputs.iter().fold(0, |acc, (_, satoshis)| {
            return acc + satoshis;
        });

        let float = self.amount.parse::<f64>().unwrap();
        let funding_amount = (float * 10e7).floor() as u64;
        let gas_fee: u64 = 10000;

        if &balance < (&funding_amount.add(gas_fee)) {
            panic!()
        }


        println!();
        // let re = Regex::new(r"^[a-zA-Z01-]{3,19}$").unwrap();
        //
        // let normalized_name = convert_to_homograph_safe_chars(&self.name);
        // let full_domain_name = format!("{}.dash", &self.name);
        // let is_contested = re.is_match(&self.name);
        //
        // info!("Starting registering DPNS name process ({})", &self.network);
        // info!("Name: {}, Normalized Name: {}, Full Domain Name: {}, Is Contested: {}", &self.name, normalized_name.clone(), &full_domain_name, is_contested);
        //

        // let identifier = Identifier::from_string(&self.identity, Base58).unwrap();
        //
        // let dpns_contract = DataContract::from_value(Constants::dpns_data_contract_value(), true, PlatformVersion::latest()).unwrap();
        //
        // // get utxo by the address
        //
        //
        // create assetlock transaction
        enum OpCode {
            Return,
            Dup,
            Hash160,
            EqualVerify,
            CheckSig,
        }

        let (assetlock_amount, change_amount) = Utils::coin_select(inputs.clone(), funding_amount, 10000);

        // change script (to the same address)
        let (address_script, _) = inputs.clone().first().unwrap().clone();

        let assetlock_output = TxOut {
            value: assetlock_amount,
            script_pubkey: ScriptBuf::from(vec![
                0x6a,
                0x00,
            ]),
        };
        // 76a914a7b2ece31ad0d9b5857d110a1bc6447432470d4e88ac
        let change_output = TxOut { value: change_amount, script_pubkey: address_script.script_sig };

        let credit_output = TxOut { value: assetlock_amount, script_pubkey: Address::p2pkh(&public_key, network_type).script_pubkey() };

        let mut transaction = Transaction {
            version: 3,
            lock_time: 0,
            input: inputs.iter().map(|(input, _)| { input.clone() }).collect::<Vec<TxIn>>(),
            output: vec![assetlock_output, change_output],
            special_transaction_payload: Some(TransactionPayload::AssetLockPayloadType(AssetLockPayload {
                version: 1,
                credit_outputs: vec![credit_output],
            })),
        };

        println!("{}", transaction.serialize().to_lower_hex_string());
        let cache = SighashCache::new(&transaction);

        let sighashes: Vec<LegacySighash> = transaction
            .input
            .iter()
            .enumerate()
            .map(|(i, input)| {
                cache
                    .legacy_signature_hash(i, &input.script_sig, 1u32)
                    .expect("expected sighash")
            })
            .collect();

        transaction.input
            .iter_mut()
            .zip(sighashes.into_iter())
            .try_for_each(|(input, sighash)| {
                let message = Message::from_digest(sighash.into());

                // Sign the message with the private key
                let sig = secp.sign_ecdsa(&message, &private_key.inner);

                // Serialize the DER-encoded signature and append the sighash type
                let mut serialized_sig = sig.serialize_der().to_vec();

                let mut sig_script = vec![serialized_sig.len() as u8 + 1];

                sig_script.append(&mut serialized_sig);

                sig_script.push(1);

                let mut serialized_pub_key = private_key.public_key(&secp).serialize();

                sig_script.push(serialized_pub_key.len() as u8);
                sig_script.append(&mut serialized_pub_key);
                // Create script_sig
                input.script_sig = ScriptBuf::from_bytes(sig_script);
                Ok::<(), String>(())
            }).unwrap();
        println!("{}", transaction.serialize().to_lower_hex_string());

        // broadcast core transaction

        // wait for instant lock / chainlock

        // create assetLock Proof

        // create identitycreatetransition

        // sign & broadcast in the platform

        // let platform_grpc_client = PlatformGRPCClient::new(&self.dapi_url);
        //
        // let identity = platform_grpc_client
        //     .get_identity_by_identifier(identifier).await?;
        //
        // debug!("Identity with identifier {} found in the network", identity.id());
        //
        // let identity_public_keys = platform_grpc_client
        //     .get_identity_keys(identity.id()).await;
        //
        // debug!("Finding matching IdentityPublicKey in the Identity against applied private key");
        //
        // let identity_public_key = identity_public_keys
        //     .iter()
        //     .filter(|key|  key.public_key_hash().unwrap() == <[u8; 20] as Into<[u8;20]>>::into(public_key.pubkey_hash().to_byte_array()))
        //     .collect::<Vec<&IdentityPublicKey>>()
        //     .first()
        //     .ok_or(Error::IdentityPublicKeyHashMismatchError(IdentityPublicKeyHashMismatchError::from((identifier, public_key.pubkey_hash()))))?
        //     .clone();
        //
        // debug!("Found matching IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
        //     identity_public_key.id(),
        //     identity_public_key.key_type(),
        //     identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
        //     identity_public_key.purpose(),
        //     identity_public_key.security_level());
        //
        // let identity_contract_nonce = platform_grpc_client.get_identity_contract_nonce(identity.id(), dpns_contract.id()).await;
        //
        // debug!("Identity contract nonce for identifier {} is {}", identity.id(), identity_contract_nonce.clone());
        //
        // let mut rng = StdRng::from_entropy();
        //
        // let salt: [u8; 32] = rng.gen();
        //
        // let mut salted_domain_buffer: Vec<u8> = vec![];
        // salted_domain_buffer.extend(salt);
        // salted_domain_buffer.extend((normalized_name.clone() + ".dash").as_bytes());
        //
        // let salted_domain_hash = hash_double(salted_domain_buffer);
        //
        // debug!("Salted Domain Hash for {} is {}", normalized_name.clone() + ".dash", salted_domain_hash.to_lower_hex_string());
        //
        // let generator = MyDefaultEntropyGenerator{};
        // let entropy = generator.generate().unwrap();
        //
        // let pre_order_document = Factories::create_document(dpns_contract.id(),
        //                                                     "preorder",
        //                                                     identity.id(),
        //                                                     platform_value!(
        //    {
        //        "saltedDomainHash": Value::Bytes32(salted_domain_hash)
        //     }
        // ), Vec::from(entropy));
        //
        // let pre_order_transition = Factories::document_create_transition(pre_order_document, "preorder", dpns_contract.id(), identity_contract_nonce.add(1), Vec::from(entropy), None);
        // let mut preorder_state_transition = StateTransition::from(IdentityStateTransition{
        //     identity: identity.id(),
        //     transitions: vec![pre_order_transition]
        // });
        //
        // debug!("Signing preorder transaction with IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
        //     identity_public_key.id(),
        //     identity_public_key.key_type(),
        //     identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
        //     identity_public_key.purpose(),
        //     identity_public_key.security_level());
        // preorder_state_transition.sign(identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();
        //
        // let preorder_buffer = preorder_state_transition.clone().serialize_to_bytes().unwrap();
        // let preorder_hex = preorder_buffer.clone();
        // let preorder_hash = digest(preorder_buffer.clone());
        //
        // debug!("Signed Preorder Transaction Hex: {}", preorder_hex.to_lower_hex_string());
        // info!("Preorder Transaction Hash: {}", preorder_hash);
        //
        // platform_grpc_client.broadcast_state_transition(preorder_state_transition).await;
        //
        // info!("Preorder document has been successfully sent into the network");
        //
        // info!("Waiting 20s for a confirmation in the network");
        // sleep(Duration::from_millis(20000)).await;
        //
        // let domain_document = Factories::create_document(dpns_contract.id(), "domain", identity.id(),
        //                                                  platform_value!(
        //   {
        //       "label": &self.name,
        //       "records": {
        //         "identity": identity.id(),
        //       },
        //       "preorderSalt": Value::Bytes32(salt),
        //       "subdomainRules": {
        //         "allowSubdomains": false
        //       },
        //       "normalizedLabel": normalized_name.clone(),
        //       "parentDomainName": "dash",
        //       "normalizedParentDomainName": "dash"
        //     }
        //  ), Vec::from(entropy));
        //
        // let prefunding_voting_balance = match is_contested {
        //     true => {Some((String::from("parentNameAndLabel"), VOTE_RESOLUTION_FUND_FEES_VERSION1.contested_document_vote_resolution_fund_required_amount))},
        //     false => None
        // };
        //
        // if !prefunding_voting_balance.is_none() {
        //     info!("Chosen name was detected as a contested resource, including 0.2 Dash in credits as a prefund for voting process");
        // }
        //
        // let domain_document_transition = Factories::document_create_transition(
        //     domain_document,
        //     "domain",
        //     dpns_contract.id(),
        //     identity_contract_nonce.add(2),
        //     Vec::from(entropy), prefunding_voting_balance);
        //
        // let mut domain_state_transition = StateTransition::from(IdentityStateTransition{
        //     identity: identity.id(),
        //     transitions: vec![domain_document_transition]
        // });
        //
        // debug!("Signing domain transaction with IdentityPublicKey id: {}, key_type: {}, pubkeyhash: {}, purpose: {}, security_level: {}",
        //     identity_public_key.id(),
        //     identity_public_key.key_type(),
        //     identity_public_key.public_key_hash().unwrap().to_lower_hex_string(),
        //     identity_public_key.purpose(),
        //     identity_public_key.security_level());
        // domain_state_transition.sign(identity_public_key, private_key.to_bytes().as_slice(), &MockBLS{}).unwrap();
        //
        // let domain_buffer = domain_state_transition.clone().serialize_to_bytes().unwrap();
        // let domain_hex = domain_buffer.clone();
        // let domain_hash = digest(domain_buffer.clone());
        // debug!("Signed Domain Transaction Hex: {}", domain_hex.to_lower_hex_string());
        // info!("Domain Transaction Hash: {}", domain_hash);
        //
        // platform_grpc_client.broadcast_state_transition(domain_state_transition).await;
        //
        // info!("Successfully registered DPNS Name {} for Identity {}", full_domain_name, identity.id().to_string(Base58));
        // info!("Please check your transactions on the Platform Explorer to make sure they all finished successfully");
        //
        // if is_contested {
        //     info!("Your name was registered through the contested resource process, please check if your name appears on the https://dash.vote now");
        // }

        Ok(())
    }
}

