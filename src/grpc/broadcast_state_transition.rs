use dapi_grpc::platform::v0::{BroadcastStateTransitionRequest};
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