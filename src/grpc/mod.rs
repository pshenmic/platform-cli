use dpp::identity::Identity;
use rs_dapi_client::{AddressList, DapiClient, RequestSettings};

mod get_identity_by_public_key_hash;
mod get_identity_keys;
mod get_identity_nonce;
mod get_identity_contract_nonce;
mod broadcast_state_transition;
mod get_identity_identifier;

pub struct PlatformGRPCClient {
    dapi_client: DapiClient,
}

impl PlatformGRPCClient {
    pub fn new(dapi_url: &str) -> PlatformGRPCClient {
        return PlatformGRPCClient {
            dapi_client: DapiClient::new(
                AddressList::from(dapi_url),
                RequestSettings::default(),
            ),
        };
    }
}

pub trait PlatformGRPCProtocol {
    fn get_identity_by_public_key(&self) -> Identity;
}

