use std::str::FromStr;
use rs_dapi_client::{AddressList, DapiClient, RequestSettings};

mod get_identity_by_public_key_hash;
mod get_identity_keys;
mod get_identity_nonce;
mod get_identity_contract_nonce;
mod broadcast_state_transition;
mod get_identity_identifier;
mod broadcast_core_transaction;
mod get_transaction;

pub struct PlatformGRPCClient {
    dapi_client: DapiClient,
}

impl PlatformGRPCClient {
    pub fn new(dapi_url: &str) -> PlatformGRPCClient {
        return PlatformGRPCClient {
            dapi_client: DapiClient::new(
                AddressList::from_str(dapi_url).unwrap(),
                RequestSettings::default(),
            ),
        };
    }
}

