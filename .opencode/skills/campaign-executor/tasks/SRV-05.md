# TASK-SRV-05: RBAC scoping por namespace

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (Wave 6)
- **Creado:** 2026-08-29T22:00
- **last-synced:** 2026-08-29T22:30
- **Estado:** ⏳ IN PROGRESS (verify OK, sin commit — BLOQUEO para vanta-lead)

## Spec

| Decisión | Justificación por evidencia |
|----------|----------------------------|
| Granularidad: namespace × role × mode (r/w) | Patrón qdrant v1.9 per-collection + weaviate roles; ya implementado en `src/rbac.rs:17-19` (`NamespaceRead(String)`, `NamespaceWrite(String)`) |
| Entry point: `Rbac::can_access_namespace(role, ns, write)` | Único método; ya usado por `cli_server.rs:876` |
| Extracción de namespace: helper `extract_namespace(path, query)` | Ya implementado `cli_server.rs:570-592` (path `/api/v2/records/{ns}/{key}` o query `?namespace=`/`?ns=`) |
| Admin bypass | Ya implementado `rbac.rs:67` — Admin siempre pasa |
| Fallback a Permission global cuando ns no aplica | Ya implementado `cli_server.rs:877-885` — endpoints no-records usan `Read`/`Write` |
| Integration test obligatorio `tests/rbac_namespace.rs` | Plan file contrato: `cargo test --test rbac_namespace` ≥1 PASS |
| Cobertura del integration test | Pre-mortem plan: privilege escalation entre roles → exhaustivo; backwards compat con HTTP method → 1 test |

## Blast Radius

### Implementación previa (existente en HEAD)
- `src/rbac.rs` (`Permission::NamespaceRead/Write`, `Rbac::can_access_namespace`)
- `src/cli_server.rs:570-592` (`extract_namespace`)
- `src/cli_server.rs:866-885` (authorize path para record/search/list)

### Esta tarea agrega (NO toca código de producto)
- `tests/rbac_namespace.rs` (nuevo, 226 líneas, 8 tests)
- `Cargo.toml` (1 bloque `[[test]]` con `required-features = ["server"]`)

### Invariantes
- `Rbac`/`Permission` siguen `pub(crate)` → no rompe encapsulación
- `can_access_namespace` es la única entry point → ya consumido por `cli_server.rs:876`
- `extract_namespace` extrae de path params + query params → contrato estable

## Contrato
```
cargo test -p vantadb --test rbac_namespace 2>&1 | Select-String "ok|PASS" | Measure-Object | Select-Object Count
```
**Resultado esperado:** Count >= 1
**Resultado obtenido:** Count = 1 ✅ (1 línea = "test result: ok")

## Herramientas
- cargo, nextest (tests)

## Steps

### Step 1: Crear integration test `tests/rbac_namespace.rs` ✅
- **Archivos:** `tests/rbac_namespace.rs` (nuevo)
- **Acción:** 8 tests cubriendo pre-mortem exhaustivo
- **Verify:** 8/8 OK ✅
- **Estado:** ✅ COMPLETED

### Step 2: Registrar test en `Cargo.toml` ✅
- **Archivos:** `Cargo.toml` (+5 líneas)
- **Acción:** Agregado `[[test]] name = "rbac_namespace"` con `required-features = ["server"]`
- **Verify:** `cargo test -p vantadb --test rbac_namespace` compila ✅
- **Estado:** ✅ COMPLETED

### Step 3: Contrato del plan file ✅
- **Acción:** Ejecutado comando del contrato
- **Verify:** Count = 1 ✅
- **Estado:** ✅ COMPLETED

### Step 4: Verify full ✅
- `cargo fmt --check` → 0 ✅
- `cargo clippy -p vantadb --features "server cli" --tests -- -D warnings` → 0 warnings ✅
- `cargo test -p vantadb --lib --features "server cli" rbac` → 10/10 ✅ (no regresión unit tests)
- `cargo test -p vantadb --test server_auth_rotation` → 2/2 ✅ (no regresión)
- `cargo test -p vantadb --test request_id` → 3/3 ✅ (no regresión)
- **Estado:** ✅ COMPLETED

## 8 tests implementados (cubren pre-mortem)

1. `ns_admin_role_can_access_any_namespace_record` — admin bypass en `/api/v2/records/{ns}/{key}`
2. `ns_reader_role_cannot_access_namespaced_record` — reader sin NamespaceRead → 403 (privilege escalation)
3. `ns_writer_role_cannot_write_namespaced_record_without_namespace_perm` — writer sin NamespaceWrite → no 2xx
4. `ns_non_record_endpoint_uses_global_reader_permission` — fallback a Permission::Read en /api/v2/health
5. `ns_query_param_namespace_is_respected` — `?namespace=` extraído correctamente
6. `ns_reader_role_cannot_access_namespaced_list_query` — reader con `?namespace=` → 403
7. `ns_bearer_without_role_entry_falls_through_to_transport` — Bearer sin role entry → 200
8. `ns_path_namespace_403_for_reader_role` — 4 namespaces distintos → todos 403 (no cross-namespace leakage)

## Dependencias
- (ninguna — SRV-05 no depende de otras tareas W6)

## Notas para vanta-lead (commit pendiente)

### Commit message
```
feat: SRV-05 — RBAC scoping por namespace (r/w per-collection)
```

### Archivos para `git add`
```
tests/rbac_namespace.rs           (nuevo, 226 líneas)
Cargo.toml                         (+5 líneas, bloque [[test]])
.opencode/skills/campaign-executor/tasks/SRV-05.md  (task file)
```

### NO stagear (de OTROS workers Wave 6)
- `docs/api/HTTP_API.md` (modificado por SRV-02 u otro worker)
- `docs/operations/SECURITY.md` (modificado por SRV-08 u otro worker)
- `docs/pipeline-state.json` (auto-update de pipeline)
- `docs/plans/2026-08-28-backlog-triage.md` y `docs/plans/2026-08-29-full-backlog-parallel.md` (mod por vanta-lead)

### Contexto pre-commit
- Verificado: `cargo fmt --check` 0, `cargo clippy -- -D warnings` 0, integration test 8/8 + unit 10/10 sin regresión.
- NO hubo cambios de dependencias → `cargo audit` no requerido.
- NO se tocó `src/wal.rs`, `src/vector/`, `src/storage/` (out of scope para SRV-05).

## Context Save Point
- **Fecha:** 2026-08-29T22:30
- **Branch:** develop
- **CI pendiente:** ninguno — verificación local exhaustiva antes de commit
- **Decisiones:** integración test con roles pre-registrados (admin/reader/writer) + unit test exhaustivo en rbac.rs cubre custom roles; no se modificó `app_with_cors` (sería invasivo); pre-mortem de privilege escalation cubierto por reader/writer denial + admin bypass
- **Problemas conocidos:** ninguno
- **Próxima tarea:** SRV-07 / SRV-08 (paralela a esta)