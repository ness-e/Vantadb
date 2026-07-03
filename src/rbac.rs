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
