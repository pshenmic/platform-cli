use dapi_grpc::platform::v0::{AllKeys, get_identity_keys_request, get_identity_keys_response, GetIdentityKeysRequest, KeyRequestType};
use dapi_grpc::platform::v0::get_identity_keys_request::GetIdentityKeysRequestV0;
use dapi_grpc::platform::v0::get_identity_keys_response::get_identity_keys_response_v0;
use dapi_grpc::platform::v0::key_request_type::Request;
use dpp::identifier::Identifier;
use dpp::identity::{IdentityPublicKey};
use dpp::serialization::PlatformDeserializable;
use rs_dapi_client::{DapiRequestExecutor, RequestSettings};
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn get_identity_keys(&self, identifier: Identifier) -> Vec<IdentityPublicKey> {
        let request = GetIdentityKeysRequest {
            version: Some(get_identity_keys_request::Version::V0(GetIdentityKeysRequestV0 {
                identity_id: identifier.to_vec(),
                request_type: Some(KeyRequestType { request: Some(Request::AllKeys(AllKeys {})) }),
                limit: None,
                offset: None,
                prove: false,
            }))
        };

        let response = self.dapi_client.execute(request, RequestSettings::default()).await.unwrap();

        let data = response.version.unwrap();

        let identity_public_keys: Vec<IdentityPublicKey> = match data {
            get_identity_keys_response::Version::V0(v0) => {
                let result = v0.result.unwrap();

                match result {
                    get_identity_keys_response_v0::Result::Keys(keys) => {
                        let tas = keys.keys_bytes
                            .into_iter()
                            .map(|key| {
                                IdentityPublicKey::deserialize_from_bytes(key.as_slice()).unwrap()
                            })
                            .collect::<Vec<IdentityPublicKey>>()
                            .try_into()
                            .unwrap();

                        tas
                    }
                    get_identity_keys_response_v0::Result::Proof(_) => {
                        panic!("We don't expect proofs")
                    }
                }
            }
        };

        identity_public_keys
    }
}