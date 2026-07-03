#![allow(dead_code)]
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub(crate) enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    NamespaceRead(String),
    NamespaceWrite(String),
}

#[derive(Clone)]
pub(crate) struct RoleConfig {
    pub permissions: Vec<Permission>,
}

pub(crate) struct Rbac {
    roles: RwLock<HashMap<String, RoleConfig>>,
}

impl Rbac {
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_role(&self, name: &str, permissions: Vec<Permission>) {
        self.roles
            .write()
            .insert(name.to_string(), RoleConfig { permissions });
    }

    pub fn has_permission(&self, role: &str, permission: &Permission) -> bool {
        let roles = self.roles.read();
        match roles.get(role) {
            Some(config) => {
                if config.permissions.contains(&Permission::Admin) {
                    return true;
                }
                config.permissions.contains(permission)
            }
            None => false,
        }
    }

    pub fn can_access_namespace(&self, role: &str, namespace: &str, write: bool) -> bool {
        let roles = self.roles.read();
        match roles.get(role) {
            Some(config) => {
                if config.permissions.contains(&Permission::Admin) {
                    return true;
                }
                let ns_perm = if write {
                    Permission::NamespaceWrite(namespace.to_string())
                } else {
                    Permission::NamespaceRead(namespace.to_string())
                };
                config.permissions.contains(&ns_perm)
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_rbac() -> Rbac {
        let rbac = Rbac::new();
        rbac.add_role("admin", vec![Permission::Admin]);
        rbac.add_role("reader", vec![Permission::Read]);
        rbac.add_role("writer", vec![Permission::Read, Permission::Write]);
        rbac.add_role(
            "ns_admin",
            vec![
                Permission::NamespaceRead("team".into()),
                Permission::NamespaceWrite("team".into()),
            ],
        );
        rbac
    }

    #[test]
    fn test_rbac_new_has_no_permissions() {
        let rbac = Rbac::new();
        assert!(!rbac.has_permission("anyone", &Permission::Read));
    }

    #[test]
    fn test_rbac_admin_has_all_permissions() {
        let rbac = setup_rbac();
        assert!(rbac.has_permission("admin", &Permission::Read));
        assert!(rbac.has_permission("admin", &Permission::Write));
        assert!(rbac.has_permission("admin", &Permission::Delete));
        assert!(rbac.has_permission("admin", &Permission::Admin));
    }

    #[test]
    fn test_rbac_reader_has_read_only() {
        let rbac = setup_rbac();
        assert!(rbac.has_permission("reader", &Permission::Read));
        assert!(!rbac.has_permission("reader", &Permission::Write));
    }

    #[test]
    fn test_rbac_writer_has_read_and_write() {
        let rbac = setup_rbac();
        assert!(rbac.has_permission("writer", &Permission::Read));
        assert!(rbac.has_permission("writer", &Permission::Write));
        assert!(!rbac.has_permission("writer", &Permission::Delete));
    }

    #[test]
    fn test_rbac_unknown_role_denies() {
        let rbac = setup_rbac();
        assert!(!rbac.has_permission("unknown", &Permission::Read));
    }

    #[test]
    fn test_rbac_can_access_namespace_read() {
        let rbac = setup_rbac();
        assert!(rbac.can_access_namespace("ns_admin", "team", false));
        assert!(!rbac.can_access_namespace("ns_admin", "other", false));
    }

    #[test]
    fn test_rbac_can_access_namespace_write() {
        let rbac = setup_rbac();
        assert!(rbac.can_access_namespace("ns_admin", "team", true));
        assert!(!rbac.can_access_namespace("ns_admin", "other", true));
    }

    #[test]
    fn test_rbac_admin_can_access_any_namespace() {
        let rbac = setup_rbac();
        assert!(rbac.can_access_namespace("admin", "anything", true));
        assert!(rbac.can_access_namespace("admin", "anything", false));
    }

    #[test]
    fn test_rbac_reader_cannot_access_namespace() {
        let rbac = setup_rbac();
        assert!(!rbac.can_access_namespace("reader", "team", false));
    }
}
