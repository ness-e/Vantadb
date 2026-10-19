---
title: "TASK MEMG-04: Multi-tenant (enforcement + cuotas)"
kind: task
description: "Barrera de aislamiento por tenant (namespace=tenant): roles ns-scoped desde RbacCfg.roles + enforcement body-declared (records/search/export/import) en middleware + cuota opt-in de records por namespace en el write path del core (error explícito + audit). Boundary billing registrado. Test no-cruce 2 tenants por superficie."
---

# TASK MEMG-04: Multi-tenant (enforcement + cuotas)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 56, bloque F4 L1608-1635)
- **Fuente:** plan Task 56 + Backlog `MEMG-04` (fila L138) + FIND-301 (`Backlog:449` — frontera declarada) + verificación CÓDIGO-REAL del bloque (l1608-1635)
- **Esfuerzo:** 🔴 2-3sem | **Appetite:** max 1mes | **Stop (plan L1619):** 2-3sem sin contrato → entregar enforcement mínimo (barrera por tenant + test de no-cruce) + FIND de cuotas/billing restante → **este run entrega enforcement (a) completo + cuota (b) mínima real + boundary billing (c) registrado + FIND-304 de residuales**
- **Prioridad:** 🔴
- **Tipo:** Rust — core `src/server/{middleware,router,state}.rs` + `src/rbac.rs` (consumo) + `src/config.rs` (`RbacCfg`/`Config`) + `src/sdk/api/memory.rs` (cuota write path) + `tests/rbac_namespace.rs` + `tests/quota_records.rs` (nuevo) + docs
- **Turns estimados:** 10-14 (una sesión de sub-agente con corte declarado)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ✅ COMPLETADO — S0-S6 ✅; commit `aa79532f`; review P2-01 APPROVE (vanta-review, 3 rondas: changes-required → fixes C1/R1-R5 → APPROVE)
- **Incógnitas (uphill, 2 del plan — RESUELTAS en DISCOVERY):**
  (a) **¿tenant = namespace vs entidad nueva?** → **RESUELTA: namespace = tenant** (evidencia: `memory_node_id(ns,key)` particiona el keyspace `src/sdk/serialization/mod.rs:77-83`; `validate_namespace` fija charset `:108-129`; RBAC `NamespaceRead/Write` + `can_access_namespace` ya existen `src/rbac.rs:17-19,70-85`; entidades user/team son registro de auth, no barrera; ADR innecesario — no cambia el modelo de autorización, lo EXTIENDE a las superficies donde el propio SRV-05 ya prometía cobertura — ver §Spec D1/D9).
  (b) **¿punto de intercepción barato de cuotas?** → **RESUELTA: write path del core (`put_one`/`put_batch_inner`/`put_record_exact`), contador mantenido existente = text-index namespace stats `doc_count`** (`load_text_namespace_stats`, cacheado en `engine.cache.text_ns`, mantenido +1/−1 por put/delete/expurge en `text_index_ops_for_replace` — `impl_text_index.rs:293-299,269`). Coste medido por diseño: 1 lectura cache/KV solo en inserts nuevos y solo con cuota opt-in (`None` default = cero coste); no se interfiere con `put` hot path por defecto. El contador por namespace NO se re-inventa (regla memory-budget: reusar caps existentes).
- **Pendientes (downhill):** 7 steps (S0–S6); S0 ✅ al crear este archivo.
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `56` (keyeada por el server; ⛔ NO usar el ID textual "MEMG-04")
- **Coordinación:** MEMG-05 (multi-escritor) EN VUELO — misma área de write path (`src/sdk/api/memory.rs` es zona compartida). Releer fresco antes de cada edición; pathspec SIEMPRE en el commit; conflicto real → BLOQUEO (ver §Notas).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `auth_middleware` ← `src/server/{mod,routing}.rs` (2 callers; firma intacta) · `can_access_namespace` ← middleware (1 caller) + tests de `rbac.rs` · `Rbac::add_role` ← `router.rs` (+ nuevo: roles de config) · `load_text_namespace_stats` ← `search/lexical.rs`, `search/debug.rs`, `memory.rs:1876`, `impl_text_index.rs:341` (+ nuevo: check de cuota) · `put_one`/`put_batch_inner`/`put_record_exact` ← SDK público + bindings (Python/WASM/Node por `Embedded`) — firma intacta, solo nuevo error `ResourceLimit` opt-in. |
| Callees | Sin deps nuevas. Nuevo uso: `axum::body::to_bytes` (ya en el árbol axum), `serde_json` (existente), `load_text_namespace_stats` (existente), `Error::ResourceLimit` (existente `src/error.rs:204`), `AuditEvent::new` (existente). Cero unsafe, cero migración, cero wire on-disk. |
| Implicaciones | **API pública aditiva:** `RbacCfg` gana campo `roles` + struct `RbacRoleCfg` (pub; default vacío = sin cambio); `Config` gana campo `max_records_per_namespace: Option<u64>` + setter (default `None` = sin cambio). **Semántica de auth:** se cierra el hueco de enforcement en superficies body-declared para credenciales con rol namespace-scoped/global (hoy `POST /records` sin `?namespace=` evade el check SRV-05; `writer`/`reader` hardcoded quedan deny en record endpoints — consistente con los tests SRV-05 existentes). Roles globales (admin/reader/writer) siguen operando igual donde ya operaban. **Cuota:** opt-in; default byte-idéntico; `ResourceLimit` con mensaje explícito + audit `put`/`put_batch` err con razon `quota_rejected`. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `3c4a8935` + WIP ajeno `opencode.jsonc`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/rbac.rs` (170L completo — `Permission`, `AccessMode`, `Rbac::{add_role,has_permission,can_access_namespace}`; `pub(crate)`).
  - `src/server/middleware.rs` (:1-289 — auth L1/L2/L3 + bloque RBAC de transporte :177-251: extracción path/query, action/enforced/scope + audit `auth_rbac`).
  - `src/server/state.rs` (:158-292 `ServerState`/`AuthRateLimiter`/`AuthState`; :294-434 `AuthIdentity`/`resolve_user_key`/`extract_namespace`).
  - `src/server/router.rs` (415L — :131-148 roles hardcoded + `AuthState::new`; :149-309 superficies protegidas/long-running/rate-limit; :266-284 rate limit global).
  - `src/server/handlers.rs` (:240-459 records put/batch/get/versions/delete/filter + list; :500-599 search body; :680-822 audit/export/import bodies; :843-859 graph bodies).
  - `src/config.rs` (:200-210 `RbacCfg`+alias; :497-530 domain views; :997-1440 `Default for Config` (env `VANTADB_*`); :1671-1674 `with_rbac_config`).
  - `src/sdk/api/memory.rs` (:395-595 resolve/purge helpers; :597-731 `put_one`; :761-797 `put`/`put_batch` audit wrappers; :799+ `put_batch_inner`; :1142-1174 `delete_inner`; :1253-1294 `put_record_exact`).
  - `src/sdk/serialization/impl_text_index.rs` (:230-370 `text_index_ops_for_replace` — mantenimiento `doc_count` +1/−1 por registro) + `src/sdk/api/namespaces.rs` (:315-423 `count` O(n) documentado + `namespace_stats` O(n) — descartados como fuente de cuota) + `src/text_index.rs` (:652-723 keys/encode de namespace stats).
  - `tests/rbac_namespace.rs` (382L completo — patrón in-memory + spawn + helpers HTTP + 11 tests) · `src/server/cli_server_auth_tests.rs` (:468-605 SRV-04 alt key + roles — sin regresión esperada, verificado: usan `/health` y `/records` sin body) · `vantadb-server/tests/server.rs` (:165 literal `RbacConfig`).
  - `docs/dev/tasks/MEMG-10.md` (formato canónico + frontera declarada con FIND-301) · `docs/dev/Backlog.md` (:138 MEMG-04; :449 FIND-301) · `.opencode/rules/{server-mcp,concurrency-async,core-engine,api-contract,memory-budget}.md` · `.opencode/references/clean-code-clean-architecture.md` (Apéndice V).
- **Referencias hacia dentro (imports):** `middleware.rs` → `crate::rbac::{AccessMode,Permission}`, `crate::server::state::{audit_auth,extract_namespace,...}`, `crate::audit::AuditEvent`; `router.rs` → `crate::rbac::{Permission,Rbac}`; `config.rs` → std only; `memory.rs` → `crate::text_index`, `crate::error::Error`.
- **Referencias entrantes (verificadas con codegraph/`rg`):** `can_access_namespace` = 1 caller producto (middleware) + unit tests; `add_role` = 1 caller producto (router) + tests; `extract_namespace` = 1 caller (middleware); `load_text_namespace_stats` = 4 callers producto (nuevo uso en cuota, sin cambio de firma); `RbacCfg` literales = 9 sitios (tests) — se actualizan con `..Default::default()`; `Config` = 372 callers (campo nuevo con default → sin cambio).
- **Archivos a crear/tocar (este run):** `src/config.rs`, `src/server/state.rs`, `src/server/middleware.rs`, `src/server/router.rs`, `src/sdk/api/memory.rs`, `tests/rbac_namespace.rs`, `tests/quota_records.rs` (nuevo), `docs/api/HTTP_API.md` (modelo tenant), `docs/dev/Backlog.md` (FIND-304; nota en FIND-301), `docs/dev/tasks/MEMG-04.md` (este), + `..Default::default()` en literales de tests existentes (`src/cli_server_auth_tests.rs`, `src/server/cli_server_auth_tests.rs`, `vantadb-server/tests/server.rs`).
- **Veredicto impacto:** **MEDIO-ALTO (trust boundary + hot path opt-in)** — enforcement nuevo en API (aditivo para credenciales scoped; cierra hueco de aislamiento), campos config aditivos con default sin cambio, cuota opt-in en write path con coste cero por defecto. **Gate D evaluado (DISCOVERY): NO disparado** — el contrato F0 (Gate Result ✅ DO L1616, aprobado por owner) manda la barrera + cuota + boundary y su stop condition pre-autoriza recortar a enforcement mínimo + FIND si el scope crece (precedente idéntico MEMG-10/WIRE-18/MEMG-09: "Gate D pre-respondido por el plan F0"). El contrato NO agrega símbolos públicos *nuevos de comportamiento* imprevistos: `RbacRoleCfg`/2 campos son la forma mínima de configurar tenants (sin ella no hay "2 tenants" en la superficie pública — primaria+alt); la cuota usa `Config` (mecanismo existente).

## Contrato

> Verbatim del plan (L1617 — ley):

"(a) enforcement real por tenant: una credencial/tenant no puede leer/escribir/buscar/listar/exportar datos de otro tenant (barrera en la capa declarada en DISCOVERY: storage y/o API), con test de no-cruce entre 2 tenants por cada superficie habilitada; (b) al menos una cuota por tenant aplicada y testeada (p.ej. bytes o records por namespace/tenant con error explícito y audit) — o decisión registrada + FIND si el punto de intercepción no cabe en el appetite; (c) billing boundary documentado (metrado por tenant exportable o decisión "no billing en motor" registrada); modelo de tenant documentado (¿namespace? ¿entidad `tenant`?) con ADR si cambia semántica de autorización; sin romper RBAC/namespace/flujos actuales (suite verde)."

- **Pre-mortem (plan L1618):** (1) "tenant" no modelado → decidir namespace-vs-entidad en DISCOVERY ✅ (D1: namespace); (2) solape FIND-301/MEMG-10 → frontera declarada (D9); (3) cuotas sin punto barato → interceptar put/delete + contador ✅ (D5: stats de text-index reusadas); (4) triple scope → orden enforcement → test no-cruce → cuota → boundary ✅ (Steps S1-S6).
- **Stop (plan L1619):** 2-3sem sin contrato → enforcement mínimo (barrera + test no-cruce) + FIND de cuotas/billing restante.
- **Corte declarado de este run (stop package):** enforcement (a) completo en las superficies declaradas (D4) + cuota (b) mínima REAL (records por namespace, put/put_batch/put_record_exact) + boundary (c) "no billing en motor" registrado + modelo documentado. Residuales → FIND-304: provisioning de N credenciales (>2) vía L1/L3, cuota en superficies restantes (wasm OPFS, rollback L1 vanta-memory), roles por env/TOML. No hay cambio del modelo de autorización → **no ADR** (D9).

## Spec (SDD — decisiones por evidencia)

> **Gate spec-first:** tabla de decisiones con alternativas + resolución por evidencia.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Modelo de tenant | A) **namespace = tenant** (pro: `memory_node_id(ns,key)` ya particiona el keyspace `serialization/mod.rs:77-83`; `NamespaceRead/Write` + `can_access_namespace` ya existen; `validate_namespace` da charset estable; 0 migración) / B) entidad `tenant` nueva (contra: 0 código hoy; exigiría resolver N tenants/namespace → mapeo nuevo + migración + sin consumidor en RBAC) / C) team/agent de `vanta-memory` (contra: son stamps de registro L1-L3 (scope), otro store — no la data plane HTTP) | ✅ **A** — decidido-por-evidencia: `serialization/mod.rs:77`, `rbac.rs:70`; aligned con FIND-301 ("namespace-scoped") y vanta-memory scopes |
| 2 | Capa de enforcement | A) **API (vantadb-server middleware + helper puro)** (pro: las credenciales terminan ahí; el choke point RBAC YA está en middleware; storage no tiene identidad; 0 cambios de wire) / B) storage (contra: no hay principal/credencial en el engine; requeriría prop drilling de identidad al core — imposible sin rediseño) / C) core SDK (contra: embedded es single-tenant por definición) | ✅ **A** — decidido-por-evidencia: contrato L1617 "(barrera en la capa declarada en DISCOVERY: storage y/o API)" + `middleware.rs:177-251` |
| 3 | Forma de la credencial tenant | A) **existente: Bearer L1 (api_key+alt) mapeado por `token_role_map` a un rol namespace-scoped NUEVO registrable desde `RbacCfg.roles`** (pro: reusa SRV-04/L1/RBAC; 2 tenants = primaria+alt — el mínimo que el contrato exige ("entre 2 tenants"); aditivo; N>2 → FIND-304) / B) aceptar N tokens de `token_role_map` como credenciales (contra: cambio del modelo de auth L1 → necesita diseño de seguridad/ADR y review propio; fuera del appetite) / C) wire L3 user→namespace (contra: `PermissionChecker` sin cablear es el objeto de MEMG-16) | ✅ **A** — decidido-por-evidencia: contrato pide "2 tenants"; pre-mortem 2 (frontera RBAC); `router.rs:134-137` gap (roles configurables) resuelto en forma mínima |
| 4 | Superficies habilitadas (contract "por cada superficie habilitada") | A) **data plane: records get/put/batch/delete/versions/filter-delete + list + search + export + import (inline)** — path/query (existente SRV-05) + body-declared (nuevo) (pro: cubre leer/escribir/buscar/listar/exportar del contrato; métodos con choke point identificado) / B) + IQL/graph/threads/conversations (contra: fallback coarse ya los deny para scoped (sin global perm); habilitarlos requeriría extracción de ns por endpoint — scope) / C) storage-wide labels (contra: rediseño) | ✅ **A** — decidido-por-evidencia: contrato L1617 (lista leer/escribir/buscar/listar/exportar) + `handlers.rs` shapes (ExportRequest:752-759, ImportRequest:762-770, MemoryInput/search body) |
| 5 | Punto de cuota | A) **core write path `put_one`/`put_batch_inner`/`put_record_exact` con contador mantenido existente (`doc_count` de text-index namespace stats; cache `engine.cache.text_ns`) + opt-in `Config.max_records_per_namespace`** (pro: el contador YA se mantiene +1/−1 en cada put/delete/expurge (`impl_text_index.rs:269,299`; `memory.rs:833,1876`); lectura cache/KV solo en insert nuevo y solo con cuota ON; cubre SDK+server+import; sin estructura nueva que cap-ear) / B) server-layer con `count()` (contra: O(n) por write documentado `namespaces.rs:319` — descartado por coste) / C) contador nuevo DashMap (contra: estructura residente nueva a cap-ear + drift entre paths + persistencia) / D) sin cuota + decisión/FIND (contra: contrato pide "al menos una cuota aplicada y testeada" — preferimos entregarla) | ✅ **A** — decidido-por-evidencia: `impl_text_index.rs:293-299` (mantenimiento), `load_text_namespace_stats` (cache), memory-budget regla 4 (reusar cap existente), `Error::ResourceLimit` (memory-budget regla 3) |
| 6 | Semántica de la cuota | A) **máx. records NUEVOS por namespace; updates y mismo key no cuentan; batch pre-check sin escritura parcial; error `ResourceLimit` explícito (existing+new+limit en el mensaje) + audit `put`/`put_batch` err `quota_rejected`; best-effort ante concurrencia (documentado: control de recursos, no la barrera de seguridad)** / B) bytes por namespace (contra: no hay contador de bytes mantenido; on-the-fly = peso) | ✅ **A** — decidido-por-evidencia: contrato "p.ej. bytes o records"; memory-budget R3 (mensaje con bytes usados/umbral/sugerencia); `resolve_existing_for_write` ya distingue nuevo/update (`memory.rs:429-486`); batch chunked → pre-check evita parciales |
| 7 | Enforcement body-declared | A) **middleware lee el body (solo credenciales con rol no-admin, solo rutas declaradas, solo si path/query no dio ns), extrae namespaces por ruta (preciso, NO deep-walk — evita falsos positivos por `metadata.namespace`), restaura el body y aplica fail-closed** / B) checks por-handler (contra: replicar rol/audit en 5 handlers; hueco futuro al agregar handlers) / C) denegar toda ruta body para scoped sin leer (contra: rompe writes legítimas propias del tenant) | ✅ **A** — decidido-por-evidencia: choke point central existente (`middleware.rs:180-251`); `DefaultBodyLimit` 1MB (`router.rs:298`); shapes de body por ruta (handlers) |
| 8 | Boundary de billing | A) **decisión registrada: "no billing en el motor (0.9.0)"; el metrado por tenant es exportable vía audit log existente (`audit_auth` + auditoría del SDK por namespace/op) y queda como contrato de host/proxy (vanta-proxy `cost.rs`)** (pro: contrato L1617 lo permite explícitamente; 0 código de billing; no inventar subsistema) / B) implementar metrado real por tenant en motor (contra: scope ajeno; el proxy ya tiene CostTracker) | ✅ **A** — decidido-por-evidencia: contrato L1617 "(... o decisión 'no billing en motor' registrada)"; `vanta-proxy/src/cost.rs:87` ("not billing"); audit SDK ya registra por namespace |
| 9 | ADR | A) **no ADR** (pro: no hay cambio del modelo de autorización — se EXTIENDE la cobertura del modelo SRV-05 ya adjudicado (`rbac.rs:70` + tests `ns_*`) a superficies donde el propio modelo prometía aplicar; los campos config son aditivos; decisión registrada en `campaign_memory(decisions)` + §Modelo del docs) / B) ADR (criterio del contrato "si cambia semántica de autorización"; contra: la semántica por namespace no cambia, cambia la COBERTURA — el modelo sigue siendo namespace-scoped por rol; Regla 5: ADR exige autor humano) | ✅ **A** — decidido-por-evidencia: `tests/rbac_namespace.rs:9-10` ("a role with Permission::Read MUST NOT silently read across all namespaces") — el fix es el contrato YA declarado; ver §Notas |
| 10 | Docs | A) **`docs/api/HTTP_API.md` §Multi-tenant (modelo + superficies + cuota + boundary) + task file + FIND-304 + nota de frontera en FIND-301** (pro: DoD L1632 "modelo documentado"; Regla 3) / B) solo task file (contra: operador sin referencia) | ✅ **A** — decidido-por-evidencia: DoD L1632 + Regla 3 + precedente MEMG-10 |

## Invariantes de dominio (handoff — MUST)

1. **Defaults byte-idénticos:** `RbacCfg::default()` (roles vacío) y `Config.max_records_per_namespace = None` → comportamiento EXACTO actual (suite existente verde sin cambios de semántica salvo el cierre del hueco body declarado — cubierto por tests nuevos).
2. **El aislamiento es de la credencial scoped; los roles globales (admin/reader/writer) conservan su semántica adjudicada en SRV-05:** admin bypass; reader/writer deny en rutas con namespace (path/query/body) porque no tienen `Namespace*`; global perm aplica en el fallback. No se relaja ninguna denegación existente.
3. **Fail-closed en superficies body-declared:** si el body de una credencial con rol no declara namespace (export all / import por path / array vacío) → fallback coarse (deny para scoped sin permiso global). Nunca "no encontrado = permitido".
4. **La cuota no es la barrera de seguridad:** es control de recursos best-effort; el aislamiento lo garantiza el RBAC. Documentado (concurrencia puede exceder ±transitorio).
5. **Cuota solo bloquea inserts NUEVOS:** updates al límite pasan; delete/expurge liberan; batch con exceso → rechazo sin escritura parcial.
6. **No tocar WIP ajeno:** MEMG-05 (multi-escritor) toca write path; re-releer fresco antes de editar `memory.rs`; staging quirúrgico + pathspec; conflicto real → BLOQUEO.
7. **Sin cambios de wire/serialización on-disk, sin migración, sin deps nuevas, sin `unsafe`;** `unwrap`/`expect` solo en tests con allow documentado.
8. **El audit nunca registra secretos** (ni Bearer; subject = rol).
9. **Determinismo en tests:** TempDir; sin mutación de env (override por struct); HTTP helpers existentes.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** ≤0 — aditivo (1 struct config + 2 campos + 1 setter + 3 guards opt-in en write path + extracción body en middleware + registro de roles). Sin `unsafe`, sin deps, sin estructuras residentes nuevas.
**Pago:** (1) cierra el hueco de escalación lateral por body-declared namespace (hoy `POST /records` sin `?namespace=` evade SRV-05 — un `writer` cruza tenants por body); (2) hace registrable el modelo namespace-scoped desde config (`RbacCfg.roles`), que FIND-301(b) listaba como gap; (3) añade el recurso "cuota" al engine (memory-budget R4: cap declarado) con audit.
**`ponytail:` notes:** (a) provisioning N>2 credenciales = FIND-304 (L1 acepta solo primaria+alt; extender L1 o cablear L3 PermissionChecker necesita review de seguridad propio); (b) cuota en wasm OPFS / vanta-memory L1 / rollback = FIND-304 (el core SDK cubre Python/Node/WASM por la misma vía `Embedded::put`; wasm OPFS es otro write path); (c) roles por env/TOML = FIND-304 (hoy programático, igual que `token_role_map`); (d) body-read en middleware solo con rol no-admin y ruta declarada — coste 0 para dev-mode/admin.
**`NOTICED BUT NOT TOUCHING`:** `Permission::NamespaceDelete`/separación estricta delete≠write y trust de retrieval = FIND-301 (frontera: MEMG-04 = aislamiento+cuota; RBAC por acción = MGR-04/FIND-301); L3 `PermissionChecker` sin cablear = MEMG-16.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: (a) enforcement por tenant con test de no-cruce entre 2 tenants por superficie habilitada (records read/write/batch/delete, list, search, export, import) en `tests/rbac_namespace.rs`; (b) cuota de records por namespace aplicada y testeada (`tests/quota_records.rs`: límite, update-exento, delete-libera, batch sin parciales, disable default; audit con razón) + unit tests de extracción body (`src/server/state.rs`); (c) boundary billing + modelo tenant documentados (HTTP_API.md + task file + decisión en memoria); (d) suite scoped verde + fmt/clippy; (e) FIND-304 registrado; (f) review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(security): MEMG-04 — ...` + pathspec solo de archivos propios (shared `memory.rs` con posible WIP MEMG-05 → staging quirúrgico, ver §Notas) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor (core server + SDK config) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius hecho en DISCOVERY) · `codebase-memory-mcp_check_index_coverage` (hecho: sin gaps registrados)
- `cargo nextest` scoped: `-p vantadb --features server --test rbac_namespace` · `-p vantadb --test quota_records` · `-p vantadb --lib` (unit) · suite `-p vantadb --features server` acotada · `campaign_verify_cmd`
- `cargo fmt --check` + `cargo clippy -p vantadb --all-targets --features server -- -D warnings`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — HTTP_API.md + task file + Backlog

**Skills cargadas (SDP v3, `campaign_discover_skills_v2` phase=BUILD + pin del owner):** `campaign-executor` · `progreso` · `ponytail` (base auto-MCP) · `security-and-hardening` (pin owner) · `api-and-interface-design` (pin owner) · `rust-write-tests` (pin owner) · `test-driven-development` (pin) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `context-engineering`. Docs → `documentation-skill` al tocar `.md`.

## Fases explícitas — SECURITY | PERFORMANCE | CONCURRENCIA (P2-07)

- [x] **SECURITY** — trust boundary explícito: **credencial/tenant → datos de otro tenant** (OWASP API1/BOLA — broken object level authorization; CWE-639). Amenazas STRIDE: *E*levación (cruce por body-declared ns — cerrado con fail-closed), *I*nformación (export/import cross-ns — cerrado), *T*ampering (batch mixto — rechazo total), *R*epudio (audit de denials ya existente; cuota auditada). Mitigaciones: enforcement centralizado en middleware (choke point único), extracción precisa por ruta (evita deep-walk hacia metadata), fail-closed ante ambigüedad, cuota con mensaje sin datos internos, audit sin secretos. Checklist `security-and-hardening` (autorización por recurso, least privilege, audit de eventos, sin secrets en logs). Riesgo residual declarado: N>2 credenciales (FIND-304); L3 sin cablear (MEMG-16).
- [x] **PERFORMANCE** — cuota en write path: coste 0 con default `None`; con cuota ON, +1 lectura cache/KV por insert nuevo (stats ya cacheadas; el mismo registro las re-lee en `replace_derived_indexes` — hit de cache). Middleware body-read: solo rol no-admin + ruta declarada + sin ns en path/query. Sin claims de performance (Regla 11); sin benchmark requerido (no hay optimización de hot path — Regla 9 no dispara; es funcionalidad).
- [x] **CONCURRENCIA (Regla 8)** — evaluación: el middleware lee body (await) SIN sostener locks (`can_access_namespace` toma guard transitorio, no cruza await); la cuota lee stats (cache RwLock transitorio) en funciones síncronas del write path (ya serializadas por el order de locks FND-02: el check ocurre ANTES del `insert_lock` en `put_one` — solo lectura de cache/KV; mismo patrón que `replace_derived_indexes` existente). No toca `dashmap` nuevo, ni orden de locks, ni Tokio del engine → auditoría de concurrencia **no dispara** (declarado). Cuota concurrente: best-effort documentado (invariante 4).

## Steps

### Step 0 — DISCOVERY + task file + coordinación

- **Archivos:** este task file + lectura de base (rbac/middleware/router/state/config/memory/impl_text_index/handlers/tests) + coordinación MEMG-05.
- **Acción:** ✅ task file completo (Impacto Regla 0 + Spec + corte declarado); incógnitas (a)/(b) resueltas con evidencia; MEMG-05 releído fresco (WIP ajeno actual: solo `opencode.jsonc` — no tocar).
- **Verify:** ✅ task file existe; `## Spec` completa (tabla de decisiones); Gate D evaluado (pre-respondido por plan F0).
- **Evidencia:** ✅ DISCOVERY arriba; `git status` = `M opencode.jsonc` (ajeno); `rg tenant` re-verificado.

### Step 1 — RED (TDD): tests del contrato

- **Archivos:** `tests/rbac_namespace.rs` (+ helpers 2 tenants + tests no-cruce), `tests/quota_records.rs` (nuevo), unit tests en `src/server/state.rs`.
- **Acción:** tests ANTES de implementar (estado-side: no-cruce 2 tenants por superficie; quota: límite/update/delete/batch/audit; unit: extracción body por ruta). Primer compile falla por símbolos ausentes (`RbacRoleCfg`, `roles`, `max_records_per_namespace`, helpers).
- **Verify:** RED confirmado — falla por API ausente (razón correcta), no por test mal escrito.
- **Evidencia:** ✅ RED genuino: `cargo nextest run --profile audit -p vantadb --features server --test rbac_namespace` → `error[E0432]: unresolved import vantadb::config::RbacRoleCfg` + `error[E0560]: struct RbacCfg has no field named roles`; `tests/quota_records.rs` compiló contra `max_records_per_namespace` ausente. Ningún error por test mal escrito. Nota de entorno: crash intermitente de rustc (STATUS_STACK_BUFFER_OVERRUN / OOM) por builds concurrentes de MEMG-05 en el mismo `target/`; resuelto con `cargo clean -p vantadb` + `CARGO_INCREMENTAL=0` + `--build-jobs 1` (no es falla de código).

### Step 2 — GREEN config: `RbacCfg.roles` + `Config.max_records_per_namespace`

- **Archivos:** `src/config.rs` (+ literales de tests existentes con `..Default::default()`).
- **Acción:** `RbacRoleCfg` (namespace_read/namespace_write) + campo `roles` + `From<&Config>`; campo `max_records_per_namespace` + env `VANTADB_MAX_RECORDS_PER_NAMESPACE` (0=unlimited) + setter.
- **Verify:** `cargo check -p vantadb --features server` + `cargo nextest run -p vantadb --lib -E 'test(config)'`.
- **Evidencia:** ✅ `cargo check -p vantadb --features server` verde (solo dead_code warnings de los helpers nuevos hasta S4, luego desaparecidos). Impl: `RbacCfg.roles` + `RbacRoleCfg` (`src/config.rs:203-235`), `From<&Config>` (+roles), `Config.max_records_per_namespace` + env `VANTADB_MAX_RECORDS_PER_NAMESPACE` (0=unlimited) + `with_max_records_per_namespace`; 7 literales `RbacConfig` de tests actualizados con `..Default::default()`.

### Step 3 — GREEN cuota: write path + audit

- **Archivos:** `src/sdk/api/memory.rs`.
- **Acción:** helper `check_namespace_quota` + guard en `put_one`/`put_record_exact` (insert nuevo) + pre-check batch en `put_batch_inner` (sin parciales) + audit `quota_rejected` en wrappers `put`/`put_batch`.
- **Verify:** `cargo nextest run -p vantadb --test quota_records` GREEN + `-p vantadb --lib` sin regresión.
- **Evidencia:** ✅ GREEN: `cargo nextest run --profile audit -p vantadb --test quota_records` → **8/8** (default unlimited; ResourceLimit explícito; update exento; delete libera; per-namespace; batch sin parciales; batch de updates no cuenta; audit con razón `quota_rejected`). Impl: `check_namespace_quota` + `check_batch_namespace_quota` + guards en `put_one`/`put_batch_inner`/`put_record_exact` + audit reason en `put`/`put_batch` (`src/sdk/api/memory.rs`).

### Step 4 — GREEN enforcement: middleware body-declared + registro de roles

- **Archivos:** `src/server/state.rs` (helpers puros), `src/server/middleware.rs`, `src/server/router.rs`.
- **Acción:** `is_body_namespace_route`/`body_namespaces_for_route` (puros + unit tests); middleware: body-read condicional con restauración + fail-closed + audit reusado; router: registro de `RbacCfg.roles` (built-ins no overridables).
- **Verify:** `cargo nextest run -p vantadb --features server --test rbac_namespace` GREEN + unit `-p vantadb --lib`.
- **Evidencia:** ✅ GREEN: `-p vantadb --features server --test rbac_namespace` → **22/22** (11 SRV-05/MEMG-10 existentes + 11 nuevos de no-cruce MEMG-04: read, write body, query-mask, batch, search, list/list-all, delete, export/export-all, import/path, fail-closed sin ns, audit de denial body). Unit helpers: `-p vantadb --features server --lib -E 'test(body_namespaces) or test(body_route)'` → **7/7**. Impl: `is_body_namespace_route`/`body_namespaces_for_route` (`src/server/state.rs`), `read_body_json` + unión path/query∪body + fail-closed en `auth_middleware` (`src/server/middleware.rs`), registro de `RbacCfg.roles` (`src/server/router.rs`).

### Step 5 — Docs + FIND + sync

- **Archivos:** `docs/api/HTTP_API.md`, `docs/dev/Backlog.md` (FIND-304 + nota FIND-301), este task file, `campaign_memory_write(decisions)`.
- **Acción:** §Multi-tenant (modelo namespace=tenant, credenciales scoped, superficies, cuota, boundary billing "no billing en motor"); FIND-304 residuales; nota de frontera en FIND-301 (b parcialmente resuelto); sync del task file.
- **Verify:** gates docs (`check-links`/`check-docs`/`gen-index --check`).
- **Evidencia:** ✅ `docs/api/HTTP_API.md` §Multi-tenant (modelo namespace=tenant, credenciales, tabla de superficies habilitadas, cuota, boundary "no billing en motor") · `docs/dev/Backlog.md` FIND-304 (residuales: N>2 credenciales, cuota OPFS/L1, roles env/TOML) + nota en FIND-301 (ítem (b) parcialmente resuelto por MEMG-04) · decisiones en `campaign_memory(decisions)` (modelo + criterio no-ADR) · gates docs ejecutados en S6.

### Step 6 — Verify full + OCR + review P2-01 + commit + completed

- **Archivos:** los del run (pathspec; shared files verificados).
- **Acción:** suites scoped + fmt + clippy ✅ · OCR delegation (`dev-tools/ocr-review.ps1 -Format json` → Rule Groups aplicados manualmente a los archivos del run: 0 Critical/High; nits menores: `contains` lineal en unión de namespaces ≤ batch, aceptado) · review P2-01 adversarial por agente distinto (`vanta-review`, 3 rondas) · commit local `feat(security):` · campaign `completed` (taskId 56, payload review HARD-07).
- **Verify:** ✅ v2 final (post-fixes): `--test rbac_namespace --test quota_records` **34/34** · `--lib -E 'not test(concurrent_insert_preserves_hnsw_invariants)'` **2445/2445** + HNSW stress aislado **1/1** (en suite completa excede el `terminate-after=180s` del perfil audit bajo carga concurrente — artefacto de entorno documentado, no regresión) · `--test request_id` **3/3** · fmt ✅ · clippy ✅ (hook exacto `-p vantadb -j 2 -- -D warnings` incluido) · `vantadb-server --tests` compila ✅ · docs: check-links 0 · check-docs all clear · gen-index --check pendiente del lead (MEMG-04.md nuevo; los índices regenerados en el árbol incluyen ambas tasks y quedan para el commit del lead — precedente MEMG-10).
- **Evidencia:** ⏳ commit + hash (se actualiza en el commit `docs(task):` inmediato; ver §RESULTADO). Entorno: 3 crashes intermitentes de rustc (STATUS_STACK_BUFFER_OVERRUN/OOM) por builds concurrentes en el `target/` compartido; mitigados con `cargo clean -p vantadb` + `CARGO_INCREMENTAL=0` + `--build-jobs 1` + lock `heavy-test-lock.ps1` (regla owner 2026-10-05).

## Iteración post-review P2-01 (adversarial, reviewer distinto)

> Review ronda 1: **changes-required** (1 Critical + 4 Required + 1 Required doc). Todos resueltos antes del commit:

| Finding | Severidad | Resolución |
|---------|-----------|------------|
| C1 — bypass cross-tenant en `POST /search`: `?namespace=own` + body blank → handler hace `search_all` → devolvía registros de otro tenant (200) | 🔴 Critical | `require_coarse` en middleware: body 0-ns en ruta body-declared exige ADEMÁS el permiso coarse global (sin borrar el check path/query — preserva la semántica MEMG-10 del writer+query). Test `tenant_search_query_namespace_cannot_mask_blank_body`. Reviewer verificó variante whitespace-only + control sin over-deny → CERRADO |
| R1 — bulk import `.vdbdump` exento de cuota Y del contador `doc_count` (subvaluado); docs sobredeclaraban | 🟠 Required | Decisión registrada: bulk exento en 0.9.0; FIND-304 ítem (b) extendido; claims corregidos (`config.rs` + `HTTP_API.md`). Reviewer → CERRADO |
| R2 — `writer`/`reader` globales cambian de comportamiento en body-declared (pre: 201 por coarse; post: 403) | 🟠 Required | Adjudicado como tightening intencional (consistencia SRV-05); documentado con upgrade note en `HTTP_API.md` + test que fija la semántica (`global_writer_role_denied_on_body_namespace`). Reviewer → CERRADO |
| R3 — namespace con `/` percent-encoded no matcheaba el grant en path | 🟠 Required | `extract_namespace` decodifica el segmento antes del compare RBAC + test `tenant_reads_own_namespace_with_encoded_slash`. Reviewer → CERRADO |
| R4 — rol scoped solo-read no podía BUSCAR (POST→Write) | 🟠 Required | `POST /search` mapea a `AccessMode::Read` (+ action "read"); test `read_only_scoped_role_can_search_own_namespace`. Reviewer → CERRADO |
| R5 — task file citaba FIND-303 (de MEMG-05) en vez de FIND-304 | 🟠 Required (doc) | 13 ocurrencias corregidas + 2 residuales (`HTTP_API.md:49`, `rbac_namespace.rs:390`) en ronda 2. Reviewer → CERRADO |
| Nit — wording "discards" vs mecanismo real (exige coarse además) | 🟢 Nit | Alineado en ronda 3 ("additionally requires the coarse global permission"). Nit "Two clarifications / tres bullets" no aplicado (cosmético, opcional) |

**Re-verify post-fixes (ronda 2/3):** `--test rbac_namespace --test quota_records` **34/34** fresco · `--lib` 2445/2445 + HNSW 1/1 · fmt/clippy 0 · grep FIND-303 → 0. **Veredicto final ronda 3: ✅ APPROVE.**

## Review (P2-01)

- **Revisor:** `vanta-review` (sesión `ses_ef17bff35ffe7smEq6532CUS0T`, contexto distinto al autor `vanta-worker`).
- **Enfoque:** revisión adversarial en contexto fresco con probes independientes pre/post fix (search query-mask cross-tenant, bulk quota, slash-ns, read-only search, empty-body variants en /export /import /batch /records) + re-ejecución de la evidencia mecánica.
- **Veredicto:** ✅ **APPROVE** (3 rondas: changes-required → fixes → APPROVE).

## RESULTADO (sección 7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: aa79532f (feat(security): MEMG-04 — 13 archivos, +1473/−71)
ARCHIVOS: src/config.rs · src/server/{state,middleware,router}.rs · src/sdk/api/memory.rs (hunks propios) · tests/rbac_namespace.rs · tests/quota_records.rs (nuevo) · docs/api/HTTP_API.md · docs/dev/Backlog.md (FIND-304 + nota FIND-301) · docs/dev/tasks/MEMG-04.md (nuevo) · src/cli_server_auth_tests.rs · src/server/cli_server_auth_tests.rs · vantadb-server/tests/server.rs (literales RbacConfig)
VERIFY_CONTRATO: pasa (RED→GREEN enforcement 34/34 + cuota 8/8 + lib 2445/2445 + HNSW 1/1 + request_id 3/3 + fmt/clippy + docs gates; review P2-01 APPROVE)
BLOQUEO: ninguno (entorno: crashes rustc por builds concurrentes mitigados con clean+incremental=0+jobs1+heavy lock)
GATES_EVALUADOS: P:no(pre-respondido por plan F0 — corte declarado L1617-19) D:no(pre-respondido por plan F0) V:no C:no | review adversarial P2-01: C1+4R resueltos
SKILLS_CARGADAS: campaign-executor · progreso · ponytail (base auto-MCP) · security-and-hardening · api-and-interface-design · rust-write-tests · test-driven-development · source-driven-development · doubt-driven-development · incremental-implementation · context-engineering · documentation-skill
```

**Review P2-01:** ✅ APPROVE (fresh, `vanta-review` sesión `ses_ef17bff35ffe7smEq6532CUS0T` ≠ autor; 3 rondas).

## Notas (coordinación + shared files)

- **MEMG-05** (multi-escritor, EN VUELO): write path `src/sdk/api/memory.rs` es zona compartida — re-leer fresco antes de cada edición; pathspec SIEMPRE; conflicto real → BLOQUEO (el agent no pisa WIP ajeno).
- **Frontera FIND-301 (MGR-04):** MEMG-04 = barrera de aislamiento (quién entra a qué namespace) + cuota; FIND-301 = autorización por acción (delete≠write, roles por env/TOML, trust de retrieval, promoción). `RbacCfg.roles` es configurabilidad mínima de aislamiento — la forma completa (acciones + env) queda en FIND-301/FIND-304.
- **ADR (D9):** no se crea — no hay cambio del modelo de autorización (extensión de cobertura del modelo SRV-05 ya adjudicado); Regla 5 (ADR = autor humano) hace inviable redactarlo por IA en este run; decisión registrada en memoria + docs. Si el owner considera que la cobertura nueva SÍ cambia semántica → ADR a posteriori con este task file como evidencia.
- **PROHIBIDO tocar:** `opencode.jsonc`, master plan, `docs/pipeline-state.json`.
- Disco: si el linker falla → `dev-tools/target-cleanup.ps1 -Clean -Yes`.
- **Campaign server:** taskId `56` — cierre `completed` con payload review HARD-07.

**Context Save Point (si el run se interrumpe):** estado en §Steps + recitation; trabajo parcial = git diff del worktree; NO re-hacer steps ✅; el slice es aditivo y reanudable (S1 tests → S2 config → S3 cuota → S4 enforcement → S5 docs → S6 cierre).
