use std::fmt;
use std::fmt::Formatter;
use dpp::dashcore::{PubkeyHash};
use dpp::identifier::Identifier;
use dpp::platform_value::string_encoding::Encoding::Base58;

#[derive(Debug)]
pub struct IdentifierAndPublicKeyHash {
    identifier: Identifier,
    pub_key_hash: PubkeyHash
}
#[derive(Debug)]
pub struct IdentityPublicKeyHashMismatchError(IdentifierAndPublicKeyHash);

impl fmt::Display for IdentityPublicKeyHashMismatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Could not find public key {} in the identity {}", self.0.pub_key_hash.to_hex(), self.0.identifier.to_string(Base58))
    }
}


impl From<(Identifier, PubkeyHash)> for IdentityPublicKeyHashMismatchError {
    fn from(value: (Identifier, PubkeyHash)) -> Self {
        let (identifier, pub_key_hash) = value;
        return IdentityPublicKeyHashMismatchError(IdentifierAndPublicKeyHash{ identifier, pub_key_hash });
    }
}
