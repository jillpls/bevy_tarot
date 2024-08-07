#![warn(missing_docs)]
//! Random generation and utility.

use bevy_tarot_hermit::HermitError;
use thiserror::Error;

/// Alias because "WheelOfFortune" is very long.
pub type WOFError = WheelOfFortuneError;

/// Random generation errors.
#[derive(Error, Debug)]
pub enum WheelOfFortuneError {
    /// Generic error
    #[error("<Hermit Error> {0}")]
    HermitError(HermitError)
}