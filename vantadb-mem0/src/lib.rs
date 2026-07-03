#![warn(missing_docs)]

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
pub use python::*;
