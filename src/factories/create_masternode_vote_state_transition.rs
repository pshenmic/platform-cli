use std::time::{SystemTime, UNIX_EPOCH};
use base64::Engine;
use base64::engine::general_purpose;
use dpp::document::{Document, DocumentV0, INITIAL_REVISION};
use dpp::identifier::Identifier;
use dpp::platform_value::{Bytes32, Value};
use dpp::platform_value::string_encoding::Encoding::Base64;
use dpp::state_transition::documents_batch_transition::{DocumentCreateTransition, DocumentsBatchTransition, DocumentsBatchTransitionV0};
use dpp::state_transition::documents_batch_transition::document_base_transition::DocumentBaseTransition;
use dpp::state_transition::documents_batch_transition::document_base_transition::v0::DocumentBaseTransitionV0;
use dpp::state_transition::documents_batch_transition::document_create_transition::DocumentCreateTransitionV0;
use dpp::state_transition::documents_batch_transition::document_transition::DocumentTransition;
use dpp::state_transition::masternode_vote_transition::MasternodeVoteTransition;
use dpp::state_transition::masternode_vote_transition::v0::MasternodeVoteTransitionV0;
use dpp::state_transition::StateTransitionType::MasternodeVote;
use dpp::util::entropy_generator::EntropyGenerator;
use dpp::voting::vote_choices::resource_vote_choice::ResourceVoteChoice;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use dpp::voting::vote_polls::VotePoll;
use dpp::voting::votes::resource_vote::ResourceVote;
use dpp::voting::votes::resource_vote::v0::ResourceVoteV0;
use dpp::voting::votes::Vote;
use rand::prelude::StdRng;
use rand::SeedableRng;
use crate::factories::Factories;
use crate::utils::MyDefaultEntropyGenerator;

impl Factories {
    pub fn create_masternode_vote_state_transition(pro_tx_hash: &str,
                                                   voter_identity_id: Identifier,
                                                   data_contract_id: Identifier,
                                                   document_type_name: &str,
                                                   index_name: &str,
                                                   index_values: Vec<Value>,
                                                   choice: ResourceVoteChoice) -> MasternodeVoteTransition {
        let pro_tx_hash_buffer = hex::decode(&pro_tx_hash).unwrap();
        let identifier_string = general_purpose::STANDARD.encode(pro_tx_hash_buffer);
        let validator_identifier = Identifier::from_string(&identifier_string, Base64).unwrap();

        let vote = Vote::ResourceVote((ResourceVote::V0(ResourceVoteV0 {
            vote_poll: VotePoll::ContestedDocumentResourceVotePoll(ContestedDocumentResourceVotePoll {
                contract_id: data_contract_id,
                document_type_name: String::from(document_type_name),
                index_name: String::from(index_name),
                index_values,
            }),
            resource_vote_choice: choice,
        })
        ));
        // let vote = Vote::ResourceVote(ResourceVote(ResourceVoteV0{
        //     vote_poll: VotePoll::ContestedDocumentResourceVotePoll(ContestedDocumentResourceVotePoll {
        //         contract_id: data_contract_id,
        //         document_type_name: String::from(document_type_name),
        //         index_name: String::from(index_name),
        //         index_values,
        //     }) ),
        //     resource_vote_choice: choice,
        // });

        let masternode_vote_transition = MasternodeVoteTransitionV0 {
            pro_tx_hash: validator_identifier,
            voter_identity_id,
            vote,
            nonce: 0,
            signature_public_key_id: 0,
            signature: Default::default(),
        };

        MasternodeVoteTransition::from(masternode_vote_transition)
    }
}