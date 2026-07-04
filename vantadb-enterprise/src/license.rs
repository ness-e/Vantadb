//! License verification for enterprise features.

/// License status.
pub enum LicenseStatus {
    Valid {
        features: Vec<String>,
        expires_at: u64,
        max_nodes: Option<u64>,
    },
    Invalid(String),
    Expired,
}

/// Verify an enterprise license key.
pub fn verify_license(_license_key: &str) -> LicenseStatus {
    // TODO: cryptographic signature verification
    LicenseStatus::Invalid("not implemented".into())
}
