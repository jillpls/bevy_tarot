#![warn(missing_docs)]
//! Utility functions and macros

/// Math utility
pub mod math;
pub mod error;

use std::fmt::Debug;
pub use error::HermitError;

/// Helper method to not call `format!("{:?}", _)`
pub trait SimpleToString: Debug {
    /// Calls `format!("{:?}", self)`
    fn sstr(&self) -> String {
        format!("{:?}", self)
    }
}

impl<T: Debug> SimpleToString for T {}

/// Unwraps $q.get_single() or returns.
#[macro_export]
macro_rules! get_single {
    ($q:expr) => {
        match $q.get_single() {
            Ok(m) => m,
            _ => return,
        }
    };
}

/// Unwraps $q.get_single_mut() or returns.
#[macro_export]
macro_rules! get_single_mut {
    ($q:expr) => {
        match $q.get_single_mut() {
            Ok(m) => m,
            _ => return,
        }
    };
}

/// Unwraps Option or continues, with optional warning.
#[macro_export]
macro_rules! unwrap_option_continue {
    ($q:expr) => {
        match $q {
            Some(m) => m,
            _ => {
                continue;
            }
        }
    };
    ($q: expr, $warn: expr) => {
        match $q {
            Some(m) => m,
            _ => {
                warn!("{}", $warn)
                continue;
            }
        }
    }
}

/// Unwraps Result or continues, with optional warning.
#[macro_export]
macro_rules! unwrap_result_continue {
    ($q:expr) => {
        match $q {
            Ok(m) => m,
            _ => {
                continue;
            }
        }
    };
    ($q: expr, $warn: expr) => {
        match $q {
            Ok(m) => m,
            _ => {
                warn!("{}", $warn);
                continue;
            }
        }
    };
}

/// Unwraps Option or returns, with optional warning.
#[macro_export]
macro_rules! unwrap_option {
    ($q:expr) => {
        match $q {
            Some(m) => m,
            _ => return,
        }
    };
    ($q: expr, $warn: expr) => {
        match $q {
            Some(m) => m,
            _ => {
                warn!("{}", $warn);
                return;
            }
        }
    };
}

/// Unwraps Result or returns, with optional warning.
#[macro_export]
macro_rules! unwrap_result {
    ($q:expr) => {
        match $q {
            Ok(m) => m,
            _ => return,
        }
    };
    ($q: expr, $warn: expr) => {
        match $q {
            Some(m) => m,
            _ => {
                warn!("{}", $warn);
                continue;
            }
        }
    };
}

/// Simple default comparison for serde.
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}
