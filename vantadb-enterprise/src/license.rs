//! License verification for enterprise features.

/// License status.
pub enum LicenseStatus {
    /// License is valid with the given feature set.
    Valid {
        /// List of enabled enterprise features.
        features: Vec<String>,
        /// Unix timestamp when the license expires.
        expires_at: u64,
        /// Maximum number of nodes allowed, if limited.
        max_nodes: Option<u64>,
    },
    /// License key is invalid with an error message.
    Invalid(String),
    /// License has expired.
    Expired,
}

/// Verify an enterprise license key.
pub fn verify_license(_license_key: &str) -> LicenseStatus {
    // TODO: cryptographic signature verification
    LicenseStatus::Invalid("not implemented".into())
}
