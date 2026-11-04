//! Opt-in anonymous operational telemetry (TEC-09).
//!
//! # Privacy and Governance Guarantees
//! - **Strictly opt-in**: Telemetry is disabled by default. It only activates when
//!   `VANTADB_TELEMETRY=1` or `VANTADB_TELEMETRY=true` is explicitly set.
//! - **Do-Not-Track respect**: If `DO_NOT_TRACK=1` is set in the environment, telemetry
//!   is forcefully disabled, overriding any opt-in flag.
//! - **Zero data payload leakage**: Never inspects, extracts, stores, or transmits
//!   record payloads, vectors, namespaces, metadata keys, or user identifiers.
//! - **Operational aggregates only**: Captures runtime environment metadata (OS, architecture,
//!   engine version, storage backend type, and process uptime).

use serde::{Deserialize, Serialize};

/// Operational telemetry payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryPayload {
    /// Schema version for telemetry records.
    pub schema_version: &'static str,
    /// VantaDB engine crate version.
    pub engine_version: &'static str,
    /// Host operating system (`linux`, `windows`, `macos`, etc.).
    pub os: &'static str,
    /// Host CPU architecture (`x86_64`, `aarch64`, etc.).
    pub arch: &'static str,
    /// Active storage engine backend (`fjall`, `memory`, `rocksdb`).
    pub storage_backend: String,
    /// Total process uptime in seconds.
    pub uptime_seconds: u64,
}

/// Pure evaluation of telemetry opt-in status from explicit environment values.
///
/// Returns `true` ONLY IF:
/// 1. `vantadb_telemetry` is `"1"` or `"true"` (case-insensitive).
/// 2. `do_not_track` is NOT `"1"` or `"true"`.
pub fn is_telemetry_enabled_from(
    do_not_track: Option<&str>,
    vantadb_telemetry: Option<&str>,
) -> bool {
    if let Some(dnt) = do_not_track {
        let clean = dnt.trim().to_lowercase();
        if clean == "1" || clean == "true" {
            return false;
        }
    }
    match vantadb_telemetry {
        Some(v) => {
            let clean = v.trim().to_lowercase();
            clean == "1" || clean == "true"
        }
        None => false,
    }
}

/// Evaluates whether operational telemetry is active under current process environment.
pub fn is_telemetry_enabled() -> bool {
    let dnt = std::env::var("DO_NOT_TRACK").ok();
    let tel = std::env::var("VANTADB_TELEMETRY").ok();
    is_telemetry_enabled_from(dnt.as_deref(), tel.as_deref())
}

/// Builds an anonymous telemetry report given explicit opt-in inputs.
pub fn build_telemetry_payload_from(
    do_not_track: Option<&str>,
    vantadb_telemetry: Option<&str>,
    backend: &str,
    uptime_seconds: u64,
) -> Option<TelemetryPayload> {
    if !is_telemetry_enabled_from(do_not_track, vantadb_telemetry) {
        return None;
    }

    Some(TelemetryPayload {
        schema_version: "1.0.0",
        engine_version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        storage_backend: backend.to_string(),
        uptime_seconds,
    })
}

/// Builds an anonymous telemetry report if telemetry is enabled in the current environment.
/// Returns `None` if disabled or blocked by Do-Not-Track.
pub fn build_telemetry_payload(backend: &str, uptime_seconds: u64) -> Option<TelemetryPayload> {
    let dnt = std::env::var("DO_NOT_TRACK").ok();
    let tel = std::env::var("VANTADB_TELEMETRY").ok();
    build_telemetry_payload_from(dnt.as_deref(), tel.as_deref(), backend, uptime_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_default_disabled() {
        assert!(!is_telemetry_enabled_from(None, None));
        assert!(build_telemetry_payload_from(None, None, "fjall", 100).is_none());
    }

    #[test]
    fn test_telemetry_opt_in_activation() {
        assert!(is_telemetry_enabled_from(None, Some("1")));
        assert!(is_telemetry_enabled_from(None, Some("true")));
        assert!(is_telemetry_enabled_from(None, Some("TRUE")));

        let payload = build_telemetry_payload_from(None, Some("1"), "fjall", 42)
            .expect("should produce payload");
        assert_eq!(payload.schema_version, "1.0.0");
        assert_eq!(payload.storage_backend, "fjall");
        assert_eq!(payload.uptime_seconds, 42);
        assert_eq!(payload.os, std::env::consts::OS);
    }

    #[test]
    fn test_telemetry_honors_do_not_track() {
        // DO_NOT_TRACK takes strict precedence over opt-in
        assert!(!is_telemetry_enabled_from(Some("1"), Some("1")));
        assert!(!is_telemetry_enabled_from(Some("true"), Some("1")));
        assert!(!is_telemetry_enabled_from(Some("1"), Some("true")));
        assert!(build_telemetry_payload_from(Some("1"), Some("1"), "fjall", 100).is_none());
    }

    #[test]
    fn test_telemetry_disabled_explicit_zero() {
        assert!(!is_telemetry_enabled_from(None, Some("0")));
        assert!(!is_telemetry_enabled_from(None, Some("false")));
        assert!(!is_telemetry_enabled_from(None, Some("random_str")));
        assert!(build_telemetry_payload_from(None, Some("0"), "fjall", 100).is_none());
    }

    #[test]
    fn test_telemetry_payload_has_no_sensitive_fields() {
        let payload = TelemetryPayload {
            schema_version: "1.0.0",
            engine_version: "0.9.0",
            os: "linux",
            arch: "x86_64",
            storage_backend: "fjall".to_string(),
            uptime_seconds: 120,
        };
        let json_str = serde_json::to_string(&payload).unwrap();
        assert!(!json_str.contains("payload"));
        assert!(!json_str.contains("vector"));
        assert!(!json_str.contains("namespace"));
        assert!(!json_str.contains("key"));
    }
}
