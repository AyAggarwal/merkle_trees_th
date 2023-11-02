use std::fmt::{self};

use hex::FromHexError;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    BelowZero,
    Invalid,
}

#[derive(Debug, PartialEq)]
pub enum MerkleError {
    EncodeError(FromHexError),
    InvalidBytes,
    MaxDepthExceeded,
    InvalidIndex,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValidationError::BelowZero => write!(f, "Input can only accept positive values"),
            ValidationError::Invalid => write!(f, "Input is invalid"),
        }
    }
}

impl fmt::Display for MerkleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MerkleError::EncodeError(e) => write!(f, "{}", e),
            MerkleError::InvalidBytes => write!(f, "leaf must be 32 byte hex string"),
            MerkleError::MaxDepthExceeded => write!(f, "depth must be less than 30"),
            MerkleError::InvalidIndex => write!(f, "index is out of bounds"),
        }
    }
}

impl From<FromHexError> for MerkleError {
    fn from(err: FromHexError) -> MerkleError {
        MerkleError::EncodeError(err)
    }
}
