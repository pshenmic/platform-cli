use std::str::FromStr;
use dapi_grpc::core::v0::BroadcastTransactionRequest;
use dpp::dashcore::psbt::serialize::Serialize;
use dpp::dashcore::{Transaction, Txid};
use rs_dapi_client::{DapiRequestExecutor, RequestSettings};
use crate::grpc::{PlatformGRPCClient};

impl PlatformGRPCClient {
    pub async fn broadcast_core_transaction(&self, transaction: Transaction) -> Txid {
        let buffer = transaction.serialize();

        let broadcast_req = BroadcastTransactionRequest {
            transaction: buffer,
            allow_high_fees: false,
            bypass_limits: false,
        };

        let execution_response = self.dapi_client.execute(broadcast_req, RequestSettings::default()).await.unwrap();

        let txid = execution_response.inner.transaction_id;

        return Txid::from_str(&txid).unwrap();
    }
}