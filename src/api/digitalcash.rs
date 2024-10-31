use std::collections::HashMap;
use dpp::dashcore::{Address, Network};
use serde::{Deserialize, Serialize};

pub struct DigitalCashAPI {
    url: String,
}

#[derive(Serialize)]
pub struct JsonRPCArguments {
    method: String,
    params: Vec<HashMap<String, Vec<String>>>
}
#[derive(Serialize, Deserialize)]
pub struct AddressesUtxoResult {
    pub address: String,
    pub txid: String,
    #[serde(rename(deserialize = "outputIndex"))]
    pub output_index: u32,
    pub script: String,
    pub satoshis: u64,
    pub height: u32
}

#[derive(Deserialize)]
pub struct JsonResponse {
    result: Vec<AddressesUtxoResult>,
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


    pub async fn get_address_utxos(&self, address: Address) -> Vec<AddressesUtxoResult> {
        let mut params: HashMap<String, Vec<String>> = HashMap::new();

        params.insert(String::from("addresses"), vec![address.to_string()]);

        let p = JsonRPCArguments {
            method: String::from("getaddressutxos"),
            params: vec![params],
        };

        let res = reqwest::Client::new()
            .post(&self.url)
            .json(&p)
            .send()
            .await.unwrap();

        let js = res
            .json::<JsonResponse>()
            .await.unwrap();

        return js.result;
    }
}