# REV-016: `vantadb-enterprise` premature abstraction audit

**Date:** 2026-07-14
**Method:** ponytail-audit (over-engineering / complexity scope only)
**Crate:** `vantadb-enterprise/` — 7 Rust source files, 267 lines total

## Method

Per ponytail-audit rules: scan all code for premature abstraction (interfaces
with one implementation, factories with one product, dead config, TODO shells,
dead code with no callers). Correctness bugs and performance are out of scope.

## Findings (ranked by impact)

```
delete    encryption.rs (26 lines). All TODO stubs, zero real implementation.
          Pass-through no-ops. [vantadb-enterprise/src/encryption.rs]

delete    audit.rs (52 lines). AuditLogger has no fields, log() is a no-op,
          query() returns empty vec, AuditFilter only used as stub param.
          [vantadb-enterprise/src/audit.rs]

delete    replication.rs (48 lines). ReplicationManager has no fields,
          ship_wal_segment() no-op, health_check() returns hardcoded true.
          [vantadb-enterprise/src/replication.rs]

delete    license.rs (24 lines). verify_license() always returns
          Invalid("not implemented"). Dead code. [vantadb-enterprise/src/license.rs]

delete    rbac.rs (53 lines). check_permission() has real logic but zero
          callers — ApiToken is never created, stored, or loaded by any code path.
          [vantadb-enterprise/src/rbac.rs]

yagni     config.rs (16 lines). Fields reference features that don't work:
          license_key → license always invalid; encryption_enabled → encryption
          is no-op; audit_log_path → logger is no-op.
          [vantadb-enterprise/src/config.rs]

yagni     Cargo.toml features (encryption, audit-log, rbac, replication).
          Defined but no #[cfg] gates use them anywhere in the crate or dependents.
          [vantadb-enterprise/Cargo.toml]
```

## Integration check

```
grep -r "vantadb_enterprise" vantadb/src/  → 0 matches
grep -r "vantadb-enterprise" vantadb-*/Cargo.toml  → 0 matches (except own Cargo.toml)
```

The main `vantadb` crate never imports `vantadb_enterprise`. The enterprise crate
is completely disconnected from the product.

## Summary

| Metric | Value |
|--------|-------|
| Total lines | 267 |
| Lines with real logic | ~10 (`rbac::check_permission`) |
| Lines that are TODO/placeholder | ~257 |
| Files that are entirely stubs | 5/7 |
| Features defined but unused | 4/4 |
| Integration into main crate | none |

### Recommendation

Delete the entire `vantadb-enterprise/` crate. It is 96% placeholder code.
Enterprise features (encryption, audit, RBAC, replication) should be built
incrementally when a concrete customer requirement exists, not pre-emptively
as empty abstractions. The spec and API surface can be re-derived at that
point from actual usage data.

## Net

`-267 lines, -2 deps possible (serde, serde_json become unused with crate gone).`
