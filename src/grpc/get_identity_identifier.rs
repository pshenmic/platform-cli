use dapi_grpc::platform::v0::{get_identity_request, get_identity_response, GetIdentityRequest};
use dapi_grpc::platform::v0::get_identity_request::GetIdentityRequestV0;
use dapi_grpc::platform::v0::get_identity_response::{get_identity_response_v0, Version};
use dpp::identity::Identity;
use dpp::prelude::Identifier;
use dpp::serialization::PlatformDeserializable;
use rs_dapi_client::{DapiClientError, DapiRequestExecutor, ExecutionError, ExecutionResult, RequestSettings};
use rs_dapi_client::address_list::AddressListError;
use rs_dapi_client::transport::TransportError;
use tonic::{Code};
use crate::errors::dapi_response_error::DapiResponseError;
use crate::errors::Error;
use crate::errors::identity_not_found_error::IdentityNotFoundError;
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn get_identity_by_identifier(&self, identifier: Identifier) -> Result<Identity, Error> {
        let request = GetIdentityRequest {
            version: Some(get_identity_request::Version::V0(GetIdentityRequestV0 {
                id: identifier.to_vec(),
                prove: false,
            }))
        };

        let execution_result = self.dapi_client.execute(request, RequestSettings::default()).await;

        let result = execution_result
            .map(|execution_response| {
                let get_identity_response = execution_response.inner;

                match get_identity_response.version.unwrap() {
                    Version::V0(get_identity_response_v0) => {
                        let result = get_identity_response_v0.result.unwrap();

                        match result {
                            get_identity_response_v0::Result::Identity(bytes) => {
                                return Identity::deserialize_from_bytes(bytes.as_slice()).unwrap()
                            }
                            get_identity_response_v0::Result::Proof(_) => {
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