use dapi_grpc::platform::v0::{BroadcastStateTransitionRequest, get_identity_keys_response, get_identity_nonce_request, get_identity_nonce_response, GetIdentityNonceRequest};
use dapi_grpc::platform::v0::get_identity_keys_response::get_identity_keys_response_v0;
use dapi_grpc::platform::v0::get_identity_nonce_request::GetIdentityNonceRequestV0;
use dapi_grpc::platform::v0::get_identity_nonce_response::get_identity_nonce_response_v0;
use dpp::identifier::Identifier;
use dpp::prelude::IdentityNonce;
use dpp::serialization::PlatformSerializable;
use dpp::state_transition::StateTransition;
use rs_dapi_client::{DapiRequestExecutor, RequestSettings};
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn broadcast_state_transition(&self, state_transition: StateTransition) -> () {
        let buffer = state_transition.serialize_to_bytes().expect("Could not serialize state transition to buffer");

        let broadcast_req = BroadcastStateTransitionRequest {
            state_transition: buffer,
        };

        self.dapi_client.execute(broadcast_req, RequestSettings::default()).await.unwrap();
    }
}