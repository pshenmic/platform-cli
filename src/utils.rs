use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose;
use dpp::dashcore::{Network, PrivateKey};
use dpp::util::entropy_generator::EntropyGenerator;
use getrandom::getrandom;
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
                return Err(Error::CommandLineArgumentInvalidInput(CommandLineArgumentInvalidInput::from("Could not decode private key type from file (should be in WIF or hex)")))
            }
        };

        Ok(private_key)
    }
}