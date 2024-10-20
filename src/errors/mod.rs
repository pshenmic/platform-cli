use std::fmt::{Display, Formatter};
use crate::errors::cli_argument_invalid_input::CommandLineArgumentInvalidInput;
use crate::errors::cli_argument_missing_error::CommandLineArgumentMissingError;
use crate::errors::dapi_response_error::DapiResponseError;
use crate::errors::identity_not_found_error::{IdentityNotFoundError};
use crate::errors::identity_public_key_hash_mismatch_error::IdentityPublicKeyHashMismatchError;

pub mod cli_argument_missing_error;
pub mod identity_not_found_error;
pub mod dapi_response_error;
pub mod identity_public_key_hash_mismatch_error;
pub mod cli_argument_invalid_input;


pub enum Error {
    CommandLineArgumentMissingError(CommandLineArgumentMissingError),
    CommandLineArgumentInvalidInput(CommandLineArgumentInvalidInput),
    IdentityNotFoundError(IdentityNotFoundError),
    IdentityPublicKeyHashMismatchError(IdentityPublicKeyHashMismatchError),
    DapiResponseError(DapiResponseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CommandLineArgumentMissingError(err) => {
                write!(f, "{}", err)
            }
            Error::IdentityNotFoundError(err) => {
                write!(f, "{}", err)
            }
            Error::DapiResponseError(err) => {
                write!(f, "{}", err)
            }
            Error::IdentityPublicKeyHashMismatchError(err) => {
                write!(f, "{}", err)
            }
            Error::CommandLineArgumentInvalidInput(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

