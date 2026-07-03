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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkg_name_is_set() {
        assert!(!PKG_NAME.is_empty());
    }

    #[test]
    fn test_pkg_version_is_set() {
        assert!(!PKG_VERSION.is_empty());
    }

    #[test]
    fn test_display_name() {
        assert_eq!(DISPLAY_NAME, "VantaDB");
    }

    #[test]
    fn test_mcp_server_info_name() {
        assert_eq!(MCP_SERVER_INFO_NAME, "vantadb");
    }

    #[test]
    fn test_env_reported_version_constant() {
        assert_eq!(ENV_REPORTED_VERSION, "VANTADB_REPORTED_VERSION");
    }

    #[test]
    fn test_reported_version_defaults_to_pkg_version() {
        let version = reported_version();
        assert!(!version.is_empty());
        assert_eq!(version.as_ref(), PKG_VERSION);
    }

    #[test]
    fn test_version_label_format() {
        let label = version_label();
        assert!(label.starts_with('v'));
        assert_eq!(label, format!("v{}", PKG_VERSION));
    }
}
