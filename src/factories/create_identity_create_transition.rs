use dpp::identifier::Identifier;
use dpp::prelude::AssetLockProof;
use dpp::state_transition::identity_create_transition::IdentityCreateTransition;
use dpp::state_transition::identity_create_transition::v0::IdentityCreateTransitionV0;
use dpp::state_transition::public_key_in_creation::IdentityPublicKeyInCreation;
use crate::factories::Factories;

impl Factories {
    pub fn create_identity_create_transition(identity_id: Identifier, asset_lock_proof: AssetLockProof, public_keys: Vec<IdentityPublicKeyInCreation>) -> IdentityCreateTransition {
        let mut identity_create_transition = IdentityCreateTransition::V0(IdentityCreateTransitionV0 {
            public_keys,
            asset_lock_proof,
            user_fee_increase: 0,
            signature: Default::default(),
            identity_id,
        });

        identity_create_transition
    }
}