//! Generic errors that can occur anywhere

use thiserror::Error;

/// Generic Error that have no other home. 😭
#[derive(Error, Debug)]
pub enum HermitError {
    /// This should not be used, but is a fallback.
    #[error("[Unspecified Error] {0}")]
    Unspecified(String)
}