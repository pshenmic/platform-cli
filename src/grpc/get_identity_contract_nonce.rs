use dapi_grpc::platform::v0::{get_identity_contract_nonce_request, GetIdentityContractNonceRequest, get_identity_contract_nonce_response};
use dapi_grpc::platform::v0::get_identity_contract_nonce_request::GetIdentityContractNonceRequestV0;
use dapi_grpc::platform::v0::get_identity_contract_nonce_response::{get_identity_contract_nonce_response_v0, Version};
use dpp::identifier::Identifier;
use dpp::identity::IdentityPublicKey;
use dpp::prelude::IdentityNonce;
use rs_dapi_client::{AddressListError, DapiClientError, DapiRequestExecutor, RequestSettings};
use rs_dapi_client::transport::TransportError;
use tonic::Code;
use crate::errors::dapi_response_error::DapiResponseError;
use crate::errors::Error;
use crate::errors::identity_not_found_error::IdentityNotFoundError;
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn get_identity_contract_nonce(&self, identifier: Identifier, data_contract_identifier: Identifier) -> Result<IdentityNonce, Error> {
        let request = GetIdentityContractNonceRequest {
            version: Some(get_identity_contract_nonce_request::Version::V0(GetIdentityContractNonceRequestV0 {
                identity_id: identifier.to_vec(),
                contract_id: data_contract_identifier.to_vec(),
                prove: false,
            }))
        };

        let execution_result = self.dapi_client.execute(request, RequestSettings::default()).await;

        let result = execution_result
            .map(|execution_response| {
                let get_identity_contract_nonce_response = execution_response.inner;

                match get_identity_contract_nonce_response.version.unwrap() {
                    Version::V0(get_identity_contract_nonce_response_v0) => {
                        let result = get_identity_contract_nonce_response_v0.result.unwrap();

                        match result {
                            get_identity_contract_nonce_response_v0::Result::IdentityContractNonce(nonce) => {
                                return IdentityNonce::from(nonce)
                            }
                            get_identity_contract_nonce_response_v0::Result::Proof(_) => {
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