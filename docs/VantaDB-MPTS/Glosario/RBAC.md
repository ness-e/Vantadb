---
type: glossary-entry
status: stable
tags: [vantadb, glosario, enterprise, seguridad]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# RBAC (Role-Based Access Control)

## Definición

**RBAC** (Control de Acceso Basado en Roles) es un modelo de seguridad donde los permisos se asignan a roles, y los usuarios heredan permisos al ser asignados a roles.

## Modelo para VantaDB

### Roles Predefinidos

| Rol | Permisos |
|-----|----------|
| **admin** | Todo: crear/leer/escribir/eliminar + gestión de usuarios |
| **writer** | Crear, leer, escribir, eliminar (sin gestión) |
| **reader** | Solo lectura |
| **auditor** | Lectura + logs de auditoría |

### Permisos Granulares

```rust
pub enum Permission {
    // Namespace level
    NamespaceCreate,
    NamespaceDelete,
    NamespaceList,
    
    // Document level
    DocumentRead,
    DocumentWrite,
    DocumentDelete,
    
    // Index level
    IndexRebuild,
    IndexQuery,
    
    // Admin level
    UserManage,
    ConfigModify,
    AuditRead,
}
```

## Configuración (Futuro)

```python
# Crear rol personalizado
db.create_role(
    name="data_scientist",
    permissions=[
        "namespace:list",
        "document:read",
        "document:write",
        "index:query"
    ]
)

# Asignar usuario a rol
db.assign_role(
    user="alice@example.com",
    role="data_scientist",
    namespace="research"
)
```

## Integración con Autenticación

```rust
pub struct AuthContext {
    user_id: String,
    roles: Vec<Role>,
    namespace: String,
}

impl StorageEngine {
    pub fn put(&self, ctx: &AuthContext, node: UnifiedNode) -> Result<()> {
        // Verificar permiso
        if !ctx.has_permission(Permission::DocumentWrite) {
            return Err(VantaError::PermissionDenied);
        }
        
        // Verificar namespace
        if ctx.namespace != node.namespace {
            return Err(VantaError::NamespaceMismatch);
        }
        
        // Proceder
        self.engine.put(node)
    }
}
```

## Auditoría

```rust
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

pub struct AuditEntry {
    timestamp: u64,
    user_id: String,
    action: String,
    resource: String,
    result: AuditResult,  // Success/Denied
    ip_address: String,
}
```

## Roadmap

| Fase | Feature | Estado |
|------|---------|--------|
| **FASE 5** | RBAC básico | ⬜ Planeado |
| **FASE 6** | Auditoría completa | ⬜ Planeado |
| **FASE 7** | Integración LDAP/SAML | ⬜ Planeado |

## Véase También

- [Multi-tenancy](Multi-tenancy.md) — Aislamiento de datos por tenant
- [File Locking](File Locking.md) — Seguridad a nivel de archivo

---

*RBAC está planeado para versiones enterprise futuras de VantaDB.*

