use std::thread;
use std::time::Duration;
use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose;
use dashcore_rpc::{Client, RpcApi};
use dashcore_rpc::dashcore::{Address, OutPoint, Script, ScriptBuf, Txid};
use dashcore_rpc::dashcore::consensus::Decodable;
use dpp::dashcore::{InstantLock, Network, PrivateKey, Transaction, TxIn};
use dpp::identity::state_transition::asset_lock_proof::chain::ChainAssetLockProof;
use dpp::identity::state_transition::asset_lock_proof::InstantAssetLockProof;
use dpp::prelude::AssetLockProof;
use dpp::util::entropy_generator::EntropyGenerator;
use getrandom::getrandom;
use crate::api::digitalcash::DigitalCashAPI;
use crate::errors::cli_argument_invalid_input::CommandLineArgumentInvalidInput;
use crate::errors::Error;


pub struct MyDefaultEntropyGenerator;

impl EntropyGenerator for MyDefaultEntropyGenerator {
    fn generate(&self) -> anyhow::Result<[u8; 32]> {
        let mut buffer = [0u8; 32];
        getrandom(&mut buffer).context("generating entropy failed").unwrap();
        Ok(buffer)
    }
}

pub struct Utils;

impl Utils {
    pub fn decode_private_key_from_input_string(input: &str, network: Network) -> Result<PrivateKey, Error> {
        let trimmed_input = input.replace("\n", "");

        let base58: Vec<u8> = match PrivateKey::from_wif(&trimmed_input) {
            Ok(private_key) => private_key.to_bytes(),
            Err(_) => Vec::from([])
        };
        let hex: Vec<u8> = hex::decode(&trimmed_input).unwrap_or(Vec::from([]));
        let base64: Vec<u8> = general_purpose::STANDARD.decode(hex::decode(&trimmed_input).unwrap_or(Vec::from([]))).unwrap_or(Vec::from([]));

        let private_key: PrivateKey = {
            if base58.len() > 0 {
                PrivateKey::from_wif(&trimmed_input).expect("Unexpected error, could not construct private key from hex after validation")
            } else if hex.len() > 0 {
                PrivateKey::from_slice(hex.as_slice(), network).expect("Unexpected error, could not construct private key from hex after validation")
            } else if base64.len() > 0 {
                PrivateKey::from_slice(base64.as_slice(), network).expect("Unexpected error, could not construct private key from base64 after validation")
            } else {
                return Err(Error::CommandLineArgumentInvalidInput(CommandLineArgumentInvalidInput::from("Could not decode private key type from file (should be in WIF or hex)")));
            }
        };

        Ok(private_key)
    }
    pub fn coin_select(inputs: Vec<(TxIn, u64)>, needed_amount: u64, fee: u64) -> (u64, u64) {
        return inputs.into_iter().fold((0, 0), |(output_amount, change_amount), (_, satoshis)| {
            if output_amount + satoshis + fee >= needed_amount {
                let change_amount = (output_amount + satoshis) - needed_amount - fee;

                return (needed_amount, change_amount);
            }

            return (output_amount + satoshis, 0);
        });
    }
    pub async fn wait_for_balance(rpc: &DigitalCashAPI, address: Address, amount: u64) -> Result<Vec<(TxIn, u64)>, Error> {
        loop {
            let resp = rpc.get_address_utxos(address.clone()).await?;

            let inputs = resp.into_iter()
                .filter(|utxo| {
                    let script_buf = ScriptBuf::from_hex(&utxo.script).unwrap();
                    script_buf.is_p2pkh()
                })
                .map(|utxo| {
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

            if balance > amount {
                println!("Got the balance");
                return Ok(inputs);
            }

            thread::sleep(Duration::from_millis(15000));
        }
    }
    pub fn wait_for_tx_mined(inputs: Vec<(TxIn, u64)>, needed_amount: u64, fee: u64) -> (u64, u64) {
        return inputs.into_iter().fold((0, 0), |(output_amount, change_amount), (_, satoshis)| {
            if output_amount + satoshis + fee >= needed_amount {
                let change_amount = (output_amount + satoshis) - needed_amount - fee;

                return (needed_amount, change_amount);
            }

            return (output_amount + satoshis, 0);
        });
    }
    pub async fn wait_for_chain_lock(rpc: &DigitalCashAPI, txid: Txid) -> Result<u32, Error> {
        loop {
            let chainlock = rpc.get_tx_chain_locks(txid).await?;

            if chainlock.chainlock {
                return Ok(chainlock.height as u32);
            }

            thread::sleep(Duration::from_millis(15000));
        }
    }
    pub async fn wait_for_asset_lock_proof(rpc: &DigitalCashAPI, core_client: &Client, transaction: Transaction, vout: u32) -> Result<AssetLockProof, Error> {
        loop {
            // check instantsend
            let raw_instant_locks = core_client.get_raw_instant_locks(vec![&transaction.txid()]).unwrap();
            let raw_instant_locks_value = raw_instant_locks.into_iter().nth(0).unwrap();

            let raw_instant_lock: Option<String> = if raw_instant_locks_value == "None" { None } else { Some(raw_instant_locks_value) };

            if raw_instant_lock.is_some() {
                let instant_lock_bytes = hex::decode(raw_instant_lock.unwrap()).unwrap();
                let instant_lock = InstantLock::consensus_decode(&mut instant_lock_bytes.as_slice()).unwrap();

                let asset_lock_proof = AssetLockProof::Instant(InstantAssetLockProof{
                    instant_lock,
                    transaction: transaction.clone(),
                    output_index: vout,
                });

                return Ok(asset_lock_proof);
            }

            // check chainlock
            let chainlock = rpc.get_tx_chain_locks(transaction.txid()).await?;

            if chainlock.chainlock {
                let core_chain_locked_height = chainlock.height as u32;

                let out_point = OutPoint {
                    txid: transaction.txid(),
                    vout,
                };

                return Ok(AssetLockProof::Chain(ChainAssetLockProof{ core_chain_locked_height, out_point }))
            }

            thread::sleep(Duration::from_millis(5000));
        }
    }
}