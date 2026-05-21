//! Product identifiers and versioning.
//!
//! Canonical version is **`CARGO_PKG_VERSION`** from the root `Cargo.toml` at compile time.
//! Optionally override what user-facing surfaces report with **`VANTADB_REPORTED_VERSION`**
//! (e.g. `1.2.3` or `1.2.3-rc1`).

use std::borrow::Cow;

/// Crate name from `Cargo.toml` (e.g. `vantadb`).
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Semantic version from the root crate `Cargo.toml`.
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Display name shown in banners and prose (not necessarily equal to crate name).
pub const DISPLAY_NAME: &str = "VantaDB";

/// Lower-case identifier for MCP `serverInfo.name` style consumers.
pub const MCP_SERVER_INFO_NAME: &str = "vantadb";

/// Override reported version when set to a non-empty string (banner, MCP, diagnostics).
pub const ENV_REPORTED_VERSION: &str = "VANTADB_REPORTED_VERSION";

/// Version string exposed to banners and MCP. Uses [`ENV_REPORTED_VERSION`] when valid.
#[inline]
pub fn reported_version() -> Cow<'static, str> {
    match std::env::var(ENV_REPORTED_VERSION) {
        Ok(s) => {
            let trimmed = s.trim().to_owned();
            if trimmed.is_empty() {
                Cow::Borrowed(PKG_VERSION)
            } else {
                Cow::Owned(trimmed)
            }
        }
        Err(_) => Cow::Borrowed(PKG_VERSION),
    }
}

/// Label like `"v0.1.1"` for compact UI strings.
#[inline]
pub fn version_label() -> String {
    format!("v{}", reported_version())
}
