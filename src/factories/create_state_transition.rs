use dpp::identifier::Identifier;
use dpp::platform_value::Bytes32;
use dpp::state_transition::documents_batch_transition::{DocumentsBatchTransition, DocumentsBatchTransitionV0};
use dpp::state_transition::documents_batch_transition::document_transition::DocumentTransition;
use dpp::state_transition::StateTransition;
use rand::rngs::StdRng;
use crate::factories::Factories;

impl Factories {
    pub fn create_state_transition(
        owner_id: Identifier,
        transitions: Vec<DocumentTransition>) -> StateTransition {

        let documents_batch_state_transition = DocumentsBatchTransition::V0(
            DocumentsBatchTransitionV0 {
                owner_id,
                transitions,
                user_fee_increase: 0,
                signature_public_key_id: 0,
                signature: Default::default(),
            });

        StateTransition::from(documents_batch_state_transition)
    }
}