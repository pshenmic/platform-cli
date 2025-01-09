use std::collections::BTreeMap;
use dpp::identifier::Identifier;
use dpp::identity::{Identity, IdentityFacade, KeyID};
use dpp::prelude::IdentityPublicKey;
use crate::factories::Factories;
use dpp::version::PlatformVersion;

impl Factories {
    pub fn create_identity(
        identity_id: Identifier,
        public_keys: BTreeMap<KeyID, IdentityPublicKey>
    ) -> Identity {
        IdentityFacade::new(PlatformVersion::latest().protocol_version)
            .create(identity_id, public_keys).unwrap()
    }
}