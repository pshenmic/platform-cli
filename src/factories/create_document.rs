use std::time::{SystemTime, UNIX_EPOCH};
use dpp::document::{Document, DocumentV0, INITIAL_REVISION};
use dpp::identifier::Identifier;
use dpp::platform_value::{Bytes32, Value};
use dpp::state_transition::documents_batch_transition::{DocumentCreateTransition, DocumentsBatchTransition, DocumentsBatchTransitionV0};
use dpp::state_transition::documents_batch_transition::document_base_transition::DocumentBaseTransition;
use dpp::state_transition::documents_batch_transition::document_base_transition::v0::DocumentBaseTransitionV0;
use dpp::state_transition::documents_batch_transition::document_create_transition::DocumentCreateTransitionV0;
use dpp::state_transition::documents_batch_transition::document_transition::DocumentTransition;
use dpp::util::entropy_generator::EntropyGenerator;
use rand::prelude::StdRng;
use rand::SeedableRng;
use crate::factories::Factories;
use crate::utils::MyDefaultEntropyGenerator;

struct DocumentFactory {
}

impl Factories {
    pub fn create_document(
        data_contract_id: Identifier,
        document_type_name: &str,
        owner_id: Identifier,
        document_properties: Value,
        entropy: Vec<u8>) -> Document {
        let now = SystemTime::now();
        let now_seconds = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let document_id = Document::generate_document_id_v0(
            &data_contract_id,
            &owner_id,
            document_type_name,
            entropy.as_slice(),
        );

        let document: Document = Document::V0(DocumentV0 {
            id: document_id,
            properties: document_properties.into_btree_string_map().unwrap(),
            owner_id,
            revision: Some(INITIAL_REVISION),
            created_at: Some(now_seconds),
            updated_at: Some(now_seconds),
            transferred_at: None,
            created_at_block_height: None,
            updated_at_block_height: None,
            transferred_at_block_height: None,
            created_at_core_block_height: None,
            updated_at_core_block_height: None,
            transferred_at_core_block_height: None,
        });

        document
    }
}