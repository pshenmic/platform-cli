use dashcore_rpc::dashcore::key::Secp256k1;
use dashcore_rpc::dashcore::psbt::serialize::Serialize;
use dashcore_rpc::dashcore::secp256k1::hashes::hex::DisplayHex;
use dashcore_rpc::dashcore::secp256k1::Message;
use dashcore_rpc::dashcore::sighash::{LegacySighash, SighashCache};
use dashcore_rpc::dashcore::{Address, PrivateKey, PublicKey, ScriptBuf, Transaction, TxOut};
use dashcore_rpc::dashcore::transaction::special_transaction::asset_lock::AssetLockPayload;
use dashcore_rpc::dashcore::transaction::special_transaction::TransactionPayload;
use dpp::dashcore::{Network, TxIn};
use crate::factories::Factories;
use crate::utils::Utils;

impl Factories {
    pub fn create_assetlock_transaction(inputs: Vec<(TxIn, u64)>,
                                        funding_amount: u64,
                                        fee: u64,
                                        private_key: PrivateKey,
                                        asset_lock_public_key: PublicKey,
                                        network_type: Network) -> Transaction {
        let secp = Secp256k1::new();
        enum OpCode {
            Return,
            Dup,
            Hash160,
            EqualVerify,
            CheckSig,
        }

        let (assetlock_amount, change_amount) = Utils::coin_select(inputs.clone(), funding_amount, 10000);

        let assetlock_output = TxOut {
            value: assetlock_amount,
            script_pubkey: ScriptBuf::from(vec![
                0x6a,
                0x00,
            ]),
        };

        let change_output = TxOut {
            value: change_amount,
            script_pubkey: ScriptBuf::new_p2pkh(&private_key.public_key(&secp).pubkey_hash())
        };

        let credit_output = TxOut { value: assetlock_amount, script_pubkey: Address::p2pkh(&asset_lock_public_key, network_type).script_pubkey() };

        let mut transaction = Transaction {
            version: 3,
            lock_time: 0,
            input: inputs.iter().map(|(input, _)| { input.clone() }).collect::<Vec<TxIn>>(),
            output: vec![assetlock_output, change_output],
            special_transaction_payload: Some(TransactionPayload::AssetLockPayloadType(AssetLockPayload {
                version: 1,
                credit_outputs: vec![credit_output],
            })),
        };

        println!("{}", transaction.serialize().to_lower_hex_string());
        let cache = SighashCache::new(&transaction);

        let sighashes: Vec<LegacySighash> = transaction
            .input
            .iter()
            .enumerate()
            .map(|(i, input)| {
                cache
                    .legacy_signature_hash(i, &input.script_sig, 1u32)
                    .expect("expected sighash")
            })
            .collect();

        transaction.input
            .iter_mut()
            .zip(sighashes.into_iter())
            .try_for_each(|(input, sighash)| {
                let message = Message::from_digest(sighash.into());

                // Sign the message with the private key
                let sig = secp.sign_ecdsa(&message, &private_key.inner);

                // Serialize the DER-encoded signature and append the sighash type
                let mut serialized_sig = sig.serialize_der().to_vec();

                let mut sig_script = vec![serialized_sig.len() as u8 + 1];

                sig_script.append(&mut serialized_sig);

                sig_script.push(1);

                let mut serialized_pub_key = private_key.public_key(&secp).serialize();

                sig_script.push(serialized_pub_key.len() as u8);
                sig_script.append(&mut serialized_pub_key);
                // Create script_sig
                input.script_sig = ScriptBuf::from_bytes(sig_script);
                Ok::<(), String>(())
            }).unwrap();

        println!("AssetLock TX {}", transaction.serialize().to_lower_hex_string());

        transaction
    }
}