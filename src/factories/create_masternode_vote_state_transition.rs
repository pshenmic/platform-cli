use base64::Engine;
use base64::engine::general_purpose;
use dpp::identifier::Identifier;
use dpp::platform_value::{Value};
use dpp::platform_value::string_encoding::Encoding::Base64;
use dpp::prelude::IdentityNonce;
use dpp::state_transition::masternode_vote_transition::MasternodeVoteTransition;
use dpp::state_transition::masternode_vote_transition::v0::MasternodeVoteTransitionV0;
use dpp::voting::vote_choices::resource_vote_choice::ResourceVoteChoice;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use dpp::voting::vote_polls::VotePoll;
use dpp::voting::votes::resource_vote::ResourceVote;
use dpp::voting::votes::resource_vote::v0::ResourceVoteV0;
use dpp::voting::votes::Vote;
use crate::factories::Factories;

impl Factories {
    pub fn create_masternode_vote_state_transition(pro_tx_hash: &str,
                                                   voter_identity_id: Identifier,
                                                   nonce: IdentityNonce,
                                                   data_contract_id: Identifier,
                                                   document_type_name: &str,
                                                   index_name: &str,
                                                   index_values: Vec<Value>,
                                                   choice: ResourceVoteChoice) -> MasternodeVoteTransition {
        let pro_tx_hash_buffer = hex::decode(&pro_tx_hash).unwrap();
        let identifier_string = general_purpose::STANDARD.encode(pro_tx_hash_buffer);
        let validator_identifier = Identifier::from_string(&identifier_string, Base64).unwrap();

        let vote = Vote::ResourceVote(ResourceVote::V0(ResourceVoteV0 {
            vote_poll: VotePoll::ContestedDocumentResourceVotePoll(ContestedDocumentResourceVotePoll {
                contract_id: data_contract_id,
                document_type_name: String::from(document_type_name),
                index_name: String::from(index_name),
                index_values,
            }),
            resource_vote_choice: choice,
        }));

        let masternode_vote_transition = MasternodeVoteTransitionV0 {
            pro_tx_hash: validator_identifier,
            voter_identity_id,
            vote,
            nonce,
            signature_public_key_id: 0,
            signature: Default::default(),
        };

        MasternodeVoteTransition::from(masternode_vote_transition)
    }
}