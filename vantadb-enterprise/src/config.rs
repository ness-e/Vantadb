//! Enterprise configuration.

/// Enterprise feature flags and settings.
#[derive(Debug, Clone)]
pub struct EnterpriseConfig {
    pub license_key: String,
    pub encryption_enabled: bool,
    pub audit_log_path: Option<std::path::PathBuf>,
    pub rbac_enabled: bool,
    pub replication: Option<crate::replication::ReplicationRole>,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            license_key: String::new(),
            encryption_enabled: false,
            audit_log_path: None,
            rbac_enabled: false,
            replication: None,
        }
    }
}
