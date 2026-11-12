use getrandom;
use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum Curve25519Error {
    FetchRandomBytes(getrandom::Error),
}

impl fmt::Display for Curve25519Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FetchRandomBytes(err) => write!(f, "Error: Failed to fetch rendom byte: {}", err),
        }
    }
}

impl std::error::Error for Curve25519Error {}
