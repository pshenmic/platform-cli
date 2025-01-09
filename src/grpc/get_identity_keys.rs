use dapi_grpc::platform::v0::{AllKeys, get_identity_keys_request, GetIdentityKeysRequest, KeyRequestType};
use dapi_grpc::platform::v0::get_identity_keys_request::GetIdentityKeysRequestV0;
use dapi_grpc::platform::v0::get_identity_keys_response::{get_identity_keys_response_v0, Version};
use dapi_grpc::platform::v0::key_request_type::Request;
use dpp::identifier::Identifier;
use dpp::identity::{Identity, IdentityPublicKey};
use dpp::serialization::PlatformDeserializable;
use rs_dapi_client::{AddressListError, DapiClientError, DapiRequestExecutor, RequestSettings};
use rs_dapi_client::transport::TransportError;
use tonic::Code;
use crate::errors::dapi_response_error::DapiResponseError;
use crate::errors::Error;
use crate::errors::identity_not_found_error::IdentityNotFoundError;
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn get_identity_keys(&self, identifier: Identifier) -> Result<Vec<IdentityPublicKey>, Error> {
        let request = GetIdentityKeysRequest {
            version: Some(get_identity_keys_request::Version::V0(GetIdentityKeysRequestV0 {
                identity_id: identifier.to_vec(),
                request_type: Some(KeyRequestType { request: Some(Request::AllKeys(AllKeys {})) }),
                limit: None,
                offset: None,
                prove: false,
            }))
        };

        let execution_result = self.dapi_client.execute(request, RequestSettings::default()).await;

        let result = execution_result
            .map(|execution_response| {
                let get_identity_keys_response = execution_response.inner;

                match get_identity_keys_response.version.unwrap() {
                    Version::V0(get_identity_keys_response_v0) => {
                        let result = get_identity_keys_response_v0.result.unwrap();

                        match result {
                            get_identity_keys_response_v0::Result::Keys(keys) => {
                                return keys.keys_bytes
                                    .into_iter()
                                    .map(|key| {
                                        IdentityPublicKey::deserialize_from_bytes(key.as_slice()).unwrap()
                                    })
                                    .collect::<Vec<IdentityPublicKey>>()
                                    .try_into()
                                    .unwrap()
                            }
                            get_identity_keys_response_v0::Result::Proof(_) => {
                                panic!("We don't expect proofs")
                            }
                        }
                    }
                };
            })
            .map_err(|execution_error| {
                match execution_error.inner {
                    DapiClientError::Transport(transport_error) => { match transport_error {
                        TransportError::Grpc(status) => {
                            if status.code() == Code::NotFound {
                                return Error::IdentityNotFoundError(IdentityNotFoundError::from(identifier));
                            }

                            return Error::DapiResponseError(DapiResponseError::from(format!("Unknown DAPI Response, status code: {}, message: {}", status.code(), status.message()).as_str()));
                        }
                    }
                    }
                    DapiClientError::NoAvailableAddresses => {
                        return Error::DapiResponseError(DapiResponseError::from("No available addresses"));
                    }
                    DapiClientError::AddressList(addresses) => {
                        return match addresses {
                            AddressListError::InvalidAddressUri(url) => {
                                Error::DapiResponseError(DapiResponseError::from(format!("Invalid DAPI endpoint address {}", url.to_string()).as_str()))
                            }
                        };
                    }
                    DapiClientError::Mock(_) => {
                        return Error::DapiResponseError(DapiResponseError::from("Mock dapi client response is not supported"));
                    }
                }
            });

        result
    }
}