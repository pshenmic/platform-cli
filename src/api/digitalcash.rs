use std::collections::HashMap;
use dpp::dashcore::{Address, InstantLock, Network, Txid};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::errors::Error;

pub struct DigitalCashAPI {
    url: String,
}

#[derive(Serialize)]
pub struct JsonRPCHashMapArguments {
    method: String,
    params: Vec<HashMap<String, Vec<String>>>
}
#[derive(Serialize)]
pub struct JsonRPCArrayArguments {
    method: String,
    params: Vec<Vec<String>>
}
#[derive(Serialize, Deserialize)]
pub struct UTXO {
    pub address: String,
    pub txid: String,
    #[serde(rename(deserialize = "outputIndex"))]
    pub output_index: u32,
    pub script: String,
    pub satoshis: u64,
    pub height: u32
}
#[derive(Serialize, Deserialize)]
pub struct GetTxChainLocksResult {
    pub height: i32,
    pub chainlock: bool,
    pub mempool: bool,
}

#[derive(Serialize, Deserialize)]
pub struct JsonResponse<T> {
    result: T,
}

impl DigitalCashAPI {
    pub fn new(network: Network) -> Self {
        let url = match network {
            Network::Dash => "https://rpc.digitalcash.dev",
            Network::Testnet => "https://trpc.digitalcash.dev",
            _ => panic!("Network {} is not supported by digitalcash.dev", network)
        };

        return DigitalCashAPI { url: String::from(url) };
    }

    pub async fn get_address_utxos(&self, address: Address) -> Result<Vec<UTXO>, Error> {
        let mut params: HashMap<String, Vec<String>> = HashMap::new();

        params.insert(String::from("addresses"), vec![address.to_string()]);

        let p = JsonRPCHashMapArguments {
            method: String::from("getaddressutxos"),
            params: vec![params],
        };

        let res = reqwest::Client::new()
            .post(&self.url)
            .json(&p)
            .send()
            .await.unwrap();

        let status_code = res.status();

        match status_code.as_u16() {
            420 => {
                panic!("Rate limit")
            },
            200 => {
                let json = res
                    .json::<JsonResponse<Vec<UTXO>>>()
                    .await.unwrap();

                Ok(json.result)
            },
            _ => {
                panic!("Unknown status code")
            }
        }
    }

    pub async fn get_tx_chain_locks(&self, txid: Txid) -> Result<GetTxChainLocksResult, Error> {
        let params: Vec<String> = vec![txid.to_hex()];

        let p = JsonRPCArrayArguments {
            method: String::from("gettxchainlocks"),
            params: vec![params],
        };

        let res = reqwest::Client::new()
            .post(&self.url)
            .json(&p)
            .send()
            .await.unwrap();

        let status_code = res.status();

        match status_code.as_u16() {
            420 => {
                panic!("Rate limit")
            },
            200 => {
                let json = res
                    .json::<JsonResponse<Vec<GetTxChainLocksResult>>>()
                    .await.unwrap();

                return Ok(json.result.into_iter().nth(0).unwrap())
            },
            _ => {
                panic!("Unknown status code")
            }
        }
    }
}