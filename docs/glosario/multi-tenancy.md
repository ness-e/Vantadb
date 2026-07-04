---
title: "Multi-tenancy"
type: glossary-entry
status: stable
tags: [vantadb, glosario, enterprise, arquitectura]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# Multi-tenancy

## Definición

**Multi-tenancy** es una arquitectura donde una sola instancia de software sirve a múltiples clientes (tenants), manteniendo aislamiento de datos y configuración entre ellos.

## Modelos de Multi-tenancy

### 1. Shared Database, Shared Schema

```
┌─────────────────────────────────────┐
│         Single Database              │
│  ┌─────────┬─────────┬─────────┐   │
│  │Tenant A │Tenant B │Tenant C │   │
│  │rows     │rows     │rows     │   │
│  └─────────┴─────────┴─────────┘   │
└─────────────────────────────────────┘
```

**Ventajas:** Simple, eficiente
**Desventajas:** Aislamiento débil, noisy neighbor

### 2. Shared Database, Separate Schemas

```
┌─────────────────────────────────────┐
│         Single Database              │
│  ┌─────────┐ ┌─────────┐ ┌───────┐ │
│  │Schema A │ │Schema B │ │Schema │ │
│  │(Tenant) │ │(Tenant) │ │  C    │ │
│  └─────────┘ └─────────┘ └───────┘ │
└─────────────────────────────────────┘
```

**Ventajas:** Mejor aislamiento
**Desventajas:** Complejidad de gestión

### 3. Separate Databases

```
┌─────────┐ ┌─────────┐ ┌─────────┐
│ DB A    │ │ DB B    │ │ DB C    │
│(Tenant) │ │(Tenant) │ │(Tenant) │
└─────────┘ └─────────┘ └─────────┘
```

**Ventajas:** Máximo aislamiento
**Desventajas:** Overhead de recursos

## Implementación en VantaDB (Futuro)

### Namespaces como Tenants

```rust
pub struct TenantConfig {
    tenant_id: String,
    namespace: String,
    quota: ResourceQuota,
}

pub struct ResourceQuota {
    max_documents: u64,
    max_storage_bytes: u64,
    max_vector_dimensions: usize,
    rate_limit_rps: u32,
}
```

### Aislamiento de Datos

```rust
impl VantaEmbedded {
    pub fn put(&self, tenant: &TenantConfig, node: UnifiedNode) -> Result<()> {
        // Verificar quota
        if tenant.quota.exceeded() {
            return Err(VantaError::QuotaExceeded);
        }
        
        // Forzar namespace del tenant
        let mut node = node;
        node.namespace = tenant.namespace.clone();
        
        self.engine.put(node)
    }
}
```

## Consideraciones de Seguridad

| Aspecto | Riesgo | Mitigación |
|---------|--------|------------|
| **Data Leakage** | Tenant A ve datos de Tenant B | Namespace isolation + encryption |
| **Noisy Neighbor** | Tenant A consume recursos de B | Resource quotas + rate limiting |
| **Privilege Escalation** | Tenant A accede como admin | RBAC + audit logs |

## Roadmap

| Fase | Feature | Estado |
|------|---------|--------|
| **FASE 5** | Namespaces como tenants | ⬜ Planeado |
| **FASE 6** | Resource quotas | ⬜ Planeado |
| **FASE 7** | Tenant isolation testing | ⬜ Planeado |

## Véase También

- [RBAC](RBAC.md) — Control de acceso por roles
- [Backpressure](Backpressure.md) — Control de recursos
- [File Locking](File Locking.md) — Aislamiento a nivel archivo

---

*Multi-tenancy está planeado para VantaDB Cloud, manteniendo el core embebido single-tenant.*

