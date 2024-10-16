use std::collections::{HashMap, HashSet};
use dpp::document::{Document, DocumentV0Getters};
use dpp::fee::Credits;
use dpp::identifier::Identifier;
use dpp::platform_value::Bytes32;
use dpp::prelude::IdentityNonce;
use dpp::state_transition::documents_batch_transition::{DocumentCreateTransition, DocumentsBatchTransition, DocumentsBatchTransitionV0};
use dpp::state_transition::documents_batch_transition::document_base_transition::DocumentBaseTransition;
use dpp::state_transition::documents_batch_transition::document_base_transition::v0::DocumentBaseTransitionV0;
use dpp::state_transition::documents_batch_transition::document_create_transition::DocumentCreateTransitionV0;
use dpp::state_transition::documents_batch_transition::document_transition::DocumentTransition;
use dpp::state_transition::StateTransition;
use dpp::util::entropy_generator::EntropyGenerator;
use dpp::version::fee::vote_resolution_fund_fees::v1::VOTE_RESOLUTION_FUND_FEES_VERSION1;
use rand::prelude::StdRng;
use rand::SeedableRng;
use crate::factories::Factories;

impl Factories {
    pub fn document_create_transition(
        document: Document,
        document_type_name: &str,
        data_contract_id: Identifier,
        identity_contract_nonce: IdentityNonce,
        entropy: Vec<u8>,
        prefunded_voting_balance: Option<(String, Credits)>) -> DocumentTransition {

        let transition: DocumentTransition = DocumentTransition::Create(DocumentCreateTransition::V0(DocumentCreateTransitionV0 {
            base: DocumentBaseTransition::V0(DocumentBaseTransitionV0 {
                id: document.id(),
                identity_contract_nonce,
                document_type_name: String::from(document_type_name),
                data_contract_id,
            }),
            entropy: entropy.as_slice().try_into().unwrap(),
            data: document.properties().clone(),
            prefunded_voting_balance,
        }));

        transition
    }
}

pub struct IdentityStateTransition {
    pub(crate) identity: Identifier,
    pub(crate) transitions: Vec<DocumentTransition>
}

impl From<IdentityStateTransition> for StateTransition {
    fn from(value: IdentityStateTransition) -> Self {
        let documents_batch_state_transition = DocumentsBatchTransition::V0(
            DocumentsBatchTransitionV0 {
                owner_id: value.identity,
                transitions: value.transitions,
                user_fee_increase: 0,
                signature_public_key_id: 0,
                signature: Default::default(),
            });

        StateTransition::from(documents_batch_state_transition)
    }
}