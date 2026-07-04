//! Role-based access control with scoped API tokens.

use std::collections::HashSet;

/// Permission levels for API operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    Metrics,
}

/// A role is a named set of permissions.
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

/// A scoped API token tied to a role and optional namespace filter.
#[derive(Debug, Clone)]
pub struct ApiToken {
    pub id: String,
    pub role: Role,
    pub allowed_namespaces: Option<Vec<String>>,
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
