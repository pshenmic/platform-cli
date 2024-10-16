use dapi_grpc::platform::v0::{get_identity_request, get_identity_response, GetIdentityRequest};
use dapi_grpc::platform::v0::get_identity_request::GetIdentityRequestV0;
use dapi_grpc::platform::v0::get_identity_response::get_identity_response_v0;
use dpp::identity::Identity;
use dpp::prelude::Identifier;
use dpp::serialization::PlatformDeserializable;
use rs_dapi_client::{DapiClientError, DapiRequestExecutor, RequestSettings};
use rs_dapi_client::address_list::AddressListError;
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

        let response = self.dapi_client.execute(request, RequestSettings::default()).await;

        let result = response
            .map(|get_identity_response|{
                let data = get_identity_response.version.unwrap();

                let identity: Identity = match data {
                    get_identity_response::Version::V0(v0) => {
                        let result = v0.result.unwrap();

                        match result {
                            get_identity_response_v0::Result::Identity(bytes) => {
                                Identity::deserialize_from_bytes(bytes.as_slice()).unwrap()
                            }
                            get_identity_response_v0::Result::Proof(_) => {
                                panic!("We don't expect proofs")
                            }
                        }
                    }
                };

                return identity
            })
            .map_err(|dapi_client_error| {
                match dapi_client_error {
                    DapiClientError::Transport(status, _) => {
                        if status.code() == Code::NotFound {
                            return Error::IdentityNotFoundError(IdentityNotFoundError::from(identifier))
                        }

                        return Error::DapiResponseError(DapiResponseError::from(format!("Unknown DAPI Response, status code: {}, message: {}", status.code(), status.message()).as_str()))
                    }
                    DapiClientError::NoAvailableAddresses => {
                        return Error::DapiResponseError(DapiResponseError::from("No available addresses"))
                    }
                    DapiClientError::AddressList(addresses) => {
                        return match addresses {
                            AddressListError::AddressNotFound(url) => {
                                Error::DapiResponseError(DapiResponseError::from(format!("Invalid DAPI endpoint address {}", url.to_string()).as_str()))
                            }
                        }
                    }
                    DapiClientError::Mock(_) => {
                        return Error::DapiResponseError(DapiResponseError::from("Mock dapi client response is not supported"))
                    }
                }
            });

        result
    }
}