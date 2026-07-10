//! CLI command handlers — fragmented submodules.

pub mod backup;
pub mod crud;
pub mod data;
pub mod db;
pub mod diagnostics;
pub mod fmt;
pub mod index;
pub mod migrate;
pub mod namespace;
pub mod search;
pub mod server;
pub mod util;

pub use backup::*;
pub use crud::*;
pub use data::*;
pub use db::*;
pub use diagnostics::*;
pub use fmt::*;
pub use index::*;
pub use migrate::*;
pub use namespace::*;
pub use search::*;
pub use server::*;
pub use util::*;

pub use crate::sdk::{
    FIELD_CREATED_AT_MS, FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD,
    FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
