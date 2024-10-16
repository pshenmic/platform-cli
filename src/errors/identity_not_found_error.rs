use std::fmt;
use std::fmt::Formatter;
use dpp::dashcore::{PubkeyHash};
use dpp::identifier::Identifier;
use dpp::platform_value::string_encoding::Encoding::Base58;

#[derive(Debug)]
pub enum IdentifierOrPublicKey {
    Identifier(Identifier),
    PublicKeyHash(PubkeyHash),
}

#[derive(Debug)]
pub struct IdentityNotFoundError(IdentifierOrPublicKey);

impl fmt::Display for IdentityNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            IdentifierOrPublicKey::Identifier(identifier) => {
                write!(f, "Identity with identifier {} not found", identifier.to_string(Base58))
            }
            IdentifierOrPublicKey::PublicKeyHash(pub_key_hash) => {
                write!(f, "Identity with public key hash {} not found", pub_key_hash.to_hex())
            }
        }
    }
}

impl From<PubkeyHash> for IdentityNotFoundError {
    fn from(public_key_hash: PubkeyHash) -> Self {
        return IdentityNotFoundError(IdentifierOrPublicKey::PublicKeyHash(public_key_hash));
    }
}

impl From<Identifier> for IdentityNotFoundError {
    fn from(identifier: Identifier) -> Self {
        return IdentityNotFoundError(IdentifierOrPublicKey::Identifier(identifier));
    }
}
