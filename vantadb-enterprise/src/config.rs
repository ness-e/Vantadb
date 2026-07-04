//! Enterprise configuration.

/// Enterprise feature flags and settings.
#[derive(Debug, Clone, Default)]
pub struct EnterpriseConfig {
    /// Enterprise license key string.
    pub license_key: String,
    /// Whether encryption at rest is enabled.
    pub encryption_enabled: bool,
    /// Optional path for the audit log file.
    pub audit_log_path: Option<std::path::PathBuf>,
    /// Whether RBAC is enabled.
    pub rbac_enabled: bool,
    /// Optional replication role configuration.
    pub replication: Option<crate::replication::ReplicationRole>,
}
