//! Slabbable Error

use core::fmt;
use core::fmt::Display;

use std::error::Error;

/// Slabbable error types are shared across the implementations. Every implementation must use the
/// shared error type so the implementation can be changed easily around without switching error type.
#[derive(Debug, PartialEq)]
pub enum SlabbableError {
    /// At capacity, not able to take more
    AtCapacity(usize),
    /// Invalid index referred in the request
    InvalidIndex(usize),
    /// This is a bug and should not happen.
    Bug(&'static str),
}

impl Display for SlabbableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AtCapacity(s) => write!(f, "At maximum fixed capacity: {}", s),
            Self::InvalidIndex(s) => write!(f, "Invalid slot: {}", s),
            Self::Bug(s) => write!(f, "BUG: Please report this bug: {}", s),
        }
    }
}

impl Error for SlabbableError {}
