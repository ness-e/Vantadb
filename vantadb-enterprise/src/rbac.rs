//! Role-based access control with scoped API tokens.

use std::collections::HashSet;

/// Permission levels for API operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Read-only access to data.
    Read,
    /// Write access to create and update data.
    Write,
    /// Delete access to remove data.
    Delete,
    /// Administrative access for configuration changes.
    Admin,
    /// Access to operational metrics.
    Metrics,
}

/// A role is a named set of permissions.
#[derive(Debug, Clone)]
pub struct Role {
    /// Human-readable role name.
    pub name: String,
    /// Set of permissions granted to this role.
    pub permissions: HashSet<Permission>,
}

/// A scoped API token tied to a role and optional namespace filter.
#[derive(Debug, Clone)]
pub struct ApiToken {
    /// Unique token identifier.
    pub id: String,
    /// Role associated with this token.
    pub role: Role,
    /// Optional namespace scope restriction.
    pub allowed_namespaces: Option<Vec<String>>,
    /// Optional expiration time (Unix timestamp).
    pub expires_at: Option<u64>,
}

/// Validates whether a token has permission for an operation.
pub fn check_permission(token: &ApiToken, required: &Permission, namespace: &str) -> bool {
    if !token.role.permissions.contains(required) {
        return false;
    }
    if let Some(ref namespaces) = token.allowed_namespaces {
        if !namespaces.contains(&namespace.to_string()) {
            return false;
        }
    }
    true
}
