use dapi_grpc::core::v0::GetTransactionRequest;
use dapi_grpc::platform::v0::{get_identity_contract_nonce_request, GetIdentityContractNonceRequest, get_identity_contract_nonce_response};
use dapi_grpc::platform::v0::get_identity_contract_nonce_request::GetIdentityContractNonceRequestV0;
use dapi_grpc::platform::v0::get_identity_contract_nonce_response::get_identity_contract_nonce_response_v0;
use dpp::dashcore::Txid;
use dpp::identifier::Identifier;
use dpp::prelude::IdentityNonce;
use rs_dapi_client::{DapiRequestExecutor, RequestSettings};
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn get_transaction(&self, txid: Txid) -> IdentityNonce {
        todo!()
        // let request = GetTransactionRequest {
        //     id: txid.to_hex()
        // };
        //
        //
        // let identity_nonce: IdentityNonce = match data {
        //     get_identity_contract_nonce_response::Version::V0(v0) => {
        //         let result = v0.result.unwrap();
        //
        //         match result {
        //             get_identity_contract_nonce_response_v0::Result::IdentityContractNonce(nonce) => {
        //                 IdentityNonce::from(nonce)
        //             }
        //             get_identity_contract_nonce_response_v0::Result::Proof(_) => {
        //                 panic!("We don't expect proofs")
        //             }
        //         }
        //     }
        // };
        //
        // identity_nonce
    }
}