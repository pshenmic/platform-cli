use dapi_grpc::platform::v0::{get_identity_contract_nonce_request, GetIdentityContractNonceRequest, get_identity_contract_nonce_response};
use dapi_grpc::platform::v0::get_identity_contract_nonce_request::GetIdentityContractNonceRequestV0;
use dapi_grpc::platform::v0::get_identity_contract_nonce_response::get_identity_contract_nonce_response_v0;
use dpp::identifier::Identifier;
use dpp::prelude::IdentityNonce;
use rs_dapi_client::{DapiRequestExecutor, RequestSettings};
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn get_identity_contract_nonce(&self, identifier: Identifier, data_contract_identifier: Identifier) -> IdentityNonce {
        let request = GetIdentityContractNonceRequest {
            version: Some(get_identity_contract_nonce_request::Version::V0(GetIdentityContractNonceRequestV0 {
                identity_id: identifier.to_vec(),
                contract_id: data_contract_identifier.to_vec(),
                prove: false,
            }))
        };

        let response = self.dapi_client.execute(request, RequestSettings::default()).await.unwrap();

        let data = response.version.unwrap();

        let identity_nonce: IdentityNonce = match data {
            get_identity_contract_nonce_response::Version::V0(v0) => {
                let result = v0.result.unwrap();

                match result {
                    get_identity_contract_nonce_response_v0::Result::IdentityContractNonce(nonce) => {
                        IdentityNonce::from(nonce)
                    }
                    get_identity_contract_nonce_response_v0::Result::Proof(_) => {
                        panic!("We don't expect proofs")
                    }
                }
            }
        };

        identity_nonce
    }
}