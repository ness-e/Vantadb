---
title: "TASK MEMG-10: MGR-04 — Policy engine (trusted/tainted + RBAC por acción)"
kind: task
description: "Spec MGR-04 (trusted/tainted + RBAC por acción + integración retrieval/inyección) + slice sobre VER-04: trust gate por namespace en las 4 superficies de inyección (tainted no inyecta; opt-in), wiring proxy/MCP y audit de denials RBAC por acción. Enforcement restante → FIND-301."
---

# TASK MEMG-10: MGR-04 — Policy engine (trusted/tainted + RBAC por acción)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 54, bloque F3 L1548-1574)
- **Fuente:** plan Task 54 + Backlog `MGR-04` (fila L942: "namespaces trusted/tainted + RBAC por acción") + `docs/dev/research/mgr-13-cuarentena.md` §4.1/§8 (deuda v1.0 declarada) + Notion §Gobernanza (verificada en DISCOVERY: **PROPUESTA — research pendiente**, sin forma exacta)
- **Esfuerzo:** 🔴 2-3sem | **Appetite:** max 1mes | **Stop (plan L1559):** 1mes sin contrato → spec + clase mínima (clasificación trusted/tainted + gate de inyección por namespace) + FIND del RBAC por acción restante → **este run entrega spec + clase mínima end-to-end (proxy/MCP) + audit RBAC de denials + FIND-301 del enforcement restante**
- **Prioridad:** 🟠
- **Tipo:** Rust — `vanta-memory` (hook compartido VER-04) + `vanta-proxy` (config/governance) + `vantadb-mcp` (config) + core `src/server/middleware.rs` (audit RBAC) + `tests/rbac_namespace.rs` + docs (research-doc + PROXY.md/MCP.md)
- **Turns estimados:** 12-16 (una sesión de sub-agente con corte declarado)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `54` en el campaign server)
- **Incógnitas (uphill):** 0 abiertas —
  (a) spec MGR-04 en Notion → **RESUELTA en DISCOVERY**: página "Gobernanza del ciclo de vida de la memoria" + "Seguridad (área)" (fetch vía Notion MCP, 2026-10-05): `Namespaces trusted/tainted + RBAC por acción (MGR-04): PROPUESTA (research pendiente)` — **no existe forma exacta**; el requerimiento de la página Seguridad manda: *"el sistema debe proteger los datos y evitar que el contenido almacenado se convierta en una instrucción no confiable"* → la forma se diseña en este run con evidencia de repo (precedente MEMG-09);
  (b) solape con MEMG-09 (in-flight) → **RESUELTA**: paths disjuntos (MEMG-09 = `vantadb-mcp/src/{code_index,tools,lib}.rs` + tests; este run = `vanta-memory/**`, `vanta-proxy/**`, `vantadb-mcp/src/config.rs`, `src/server/middleware.rs`, `tests/rbac_namespace.rs`) — ver §Notas.
- **Pendientes (downhill):** 6 steps (S0–S6); S0 ✅ al crear este archivo.
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `54`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `InjectionPolicy` ← `vanta-proxy/src/{governance,inject}.rs` + `vantadb-mcp/src/config.rs` + `vanta-memory/src/core/hooks/mod.rs` (re-export); `perform_auto_recall_governed` ← 9 callers (proxy `memory_tools`, MCP `context`/`handlers/tools`, hooks `mod.rs`) — **firma intacta**; `can_access_namespace` ← 1 caller (`src/server/middleware.rs`); `audit_auth` ← middleware (2 sitios existentes + 1 nuevo). |
| Callees | Ninguno nuevo. El slice usa: `parking_lot` (ya presente en rbac), `AuditEvent::auth` (existente; layer desconocido → `auth_<layer>`, `src/audit.rs:106-111`), `AuditLogger` (existente), serde (configs existentes). Cero deps nuevas. |
| Implicaciones | **API pública aditiva vanta-memory:** `TrustClass` (enum nuevo) + `InjectionPolicy::{from_parts, trust_class}` + campos privados nuevos (`tainted_prefixes`, `include_tainted`). `allows()` mantiene semántica cuando la lista tainted está vacía (default = byte-idéntico). **Config aditiva:** proxy TOML `[injection]` gana `tainted_namespaces` + `include_tainted` (serde default); MCP gana 2 env vars (`VANTADB_MCP_TAINTED_NAMESPACES`, `VANTADB_MCP_INCLUDE_TAINTED`). **Core:** el denial RBAC del middleware gana un evento `auth_rbac` (aditivo; sin audit configurado → no-op; sin cambio de status/semántica de autorización). Sin cambios de wire/serialización on-disk; sin migración; sin unsafe. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `8028b1bc` + WIP MEMG-08/09 en worktree).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/core/hooks/auto_recall.rs` (:116-158 `InjectionPolicy`; :330-340 `perform_auto_recall_governed`; :366-523 `perform_auto_recall_inner` — gates por fuente :405, :414-421, :432-437, :451-462, :468-476; :561-601 `read_scoped_records`; :1084-1130 unit tests de policy).
  - `vanta-memory/src/core/hooks/mod.rs` (:6-14 re-exports).
  - `vanta-memory/tests/ver04_governance.rs` (:1-120 patrón de setup: db in-memory + seed L1/persona + `perform_auto_recall_governed`).
  - `vanta-proxy/src/governance.rs` (289L — `Governance::from_config`, auditoría de block/recall, tests).
  - `vanta-proxy/src/config.rs` (:96-124 `InjectionConfig` + Default).
  - `vanta-proxy/src/server.rs` (:172-196 call site de `Governance::from_config`).
  - `vanta-proxy/src/inject.rs` (:149-246 `build_memory_block` — gate `policy.allows` por sección; :256-295 `fit_sections`).
  - `vanta-proxy/src/memory_tools.rs` (:673-674 caller de `from_config` en tests).
  - `vantadb-mcp/src/config.rs` (:95-194 `McpConfig` — `injection_namespaces`, `audit`, `from_storage` env parsing, `injection_policy`).
  - `vantadb-mcp/tests/ver04_governance.rs` (:214-249 test ACL + audit — patrón de override por `McpConfig { ..base_config() }`).
  - `src/rbac.rs` (170L completo — `Permission`/`AccessMode`/`Rbac::can_access_namespace`).
  - `src/server/middleware.rs` (:1-258 — auth L1/L2/L3 + RBAC de transporte :177-224 + audit de identidad).
  - `src/server/state.rs` (:255-334 `AuthState`/`AuthIdentity`; :350-357 `audit_auth`; :410-434 `extract_namespace`).
  - `src/server/router.rs` (:131-148 roles hardcoded + `AuthState::new`).
  - `src/audit.rs` (:1-200 `AuditEvent::{new,auth,injection}` + `AuditLogger`).
  - `tests/rbac_namespace.rs` (263L — patrón in-memory + spawn + HTTP helpers).
  - `src/config.rs` (:201-253 `RbacCfg`/`ServerCfg`; :886 flat `audit_log_path`).
  - `docs/dev/tasks/VER-04.md` (task file de la base; §Diseño + deuda) · `docs/dev/tasks/MEMG-09.md` (formato + coordinación) · `docs/dev/research/mgr-13-cuarentena.md` (§4.1/§8 — MGR-04 como deuda declarada).
  - Reglas: `.opencode/rules/api-contract.md` (R-1..R-8), `.opencode/rules/server-mcp.md` (R-1..R-3).
- **Referencias hacia dentro (imports):** `auto_recall.rs` no importa a los consumidores (los consumidores importan `vanta_memory::core::hooks::{InjectionPolicy, ...}`); `governance.rs` → `vanta_memory::core::hooks::{InjectionPolicy, RecallResult}` + `vantadb::audit`; `config.rs` (proxy) → serde; `middleware.rs` → `crate::audit::{AuditEvent}`, `crate::rbac`, `crate::server::state`.
- **Referencias entrantes (verificadas en DISCOVERY):** `InjectionPolicy` = 10 callers (proxy governance/inject, MCP config, hooks mod, tests); `from_prefixes` = 6 sitios productivos/tests; `can_access_namespace` = 1 caller (middleware) + unit tests; `audit_auth` = 2 callers existentes + 1 nuevo.
- **Archivos a crear/tocar (este run):** `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/core/hooks/mod.rs`, `vanta-memory/tests/memg10_trust_gate.rs` (nuevo), `vanta-proxy/src/{config,governance,server}.rs` (+ tests inline y de `memory_tools`), `vantadb-mcp/src/config.rs`, `src/server/middleware.rs`, `tests/rbac_namespace.rs`, `docs/dev/research/mgr-04-policy-engine.md` (nuevo), `docs/api/{PROXY,MCP}.md`, `docs/dev/Backlog.md` (FIND-301), este task file.
- **Veredicto impacto:** **MEDIO (superficie pública aditiva + trust boundary)** — `vanta-memory` gana 1 enum + 2 métodos + 2 campos privados (aditivo; defaults byte-idénticos); proxy/MCP ganan config opt-in; core gana 1 evento de audit en un path de denial (aditivo; no-op sin audit). **Gate D evaluado (DISCOVERY): NO disparado** — el contrato F0 (Gate Result ✅ DO, L1556-1557, aprobado por owner) manda literalmente la spec + "implementación sobre la base VER-04: clase de confianza por namespace aplicada en retrieval/inyección" + el corte ("spec primero con corte declarado", pre-mortem L1558) + la stop condition
pre-autoriza el FIND del RBAC por acción restante. Precedente idéntico: MEMG-09/WIRE-18/MEMG-08 ("Gate D pre-respondido por el plan F0").

## Contrato

> Verbatim del plan (L1557-1559 — ley):

"spec (namespaces trusted/tainted + RBAC por acción + integración con retrieval/inyección) + implementación sobre la base VER-04: clase de confianza por namespace aplicada en retrieval/inyección (tainted no inyecta por defecto) + RBAC por acción con tests por acción + audit; sin romper ACL/namespace actuales (suite verde); modelo documentado (ADR si cambia semántica de auth)."

- **Pre-mortem (plan L1558):** (1) scope 2-3sem → spec primero con corte declarado; (2) solape SCH-05 (registro) vs trusted/tainted (namespace) → semántica complementaria declarada, no duplicar gates; (3) cambio de semántica auth rompe clientes → aditivo/opt-in por config.
- **Stop (plan L1559):** 1mes sin contrato → spec + clase mínima (clasificación trusted/tainted + gate de inyección por namespace) + FIND del RBAC por acción restante.
- **Corte declarado de este run (stop package):** spec completa + clase mínima end-to-end (trust gate en las 4 superficies + wiring proxy/MCP) + **incremento aditivo de RBAC por acción (audit de denials con label de acción)** + **FIND-301** del enforcement restante (roles configurables + `NamespaceDelete` + separación estricta). No hay cambio de semántica de auth → **no ADR** (criterio: el modelo no altera autorización existente; el ADR se requiere cuando el enforcement por acción se implemente — queda en FIND-301). Declarado en la spec §6.

## Spec (SDD — decisiones por evidencia)

> **Gate spec-first:** `## Spec` completa (decisiones con alternativas + resolución por evidencia). La spec formal MGR-04 vive en `docs/dev/research/mgr-04-policy-engine.md`; acá se fijan las decisiones del **slice de este run**.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Corte del run | A) **spec + clase mínima end-to-end: TrustClass + gate en las 4 superficies + wiring proxy/MCP + audit RBAC de denials + FIND-301 del enforcement restante** (pro: stop L1559 lo autoriza literalmente; pre-mortem 1 "spec primero con corte declarado"; una sesión) / B) contrato completo incluido enforcement por acción (contra: 2-3sem; cambio de semántica auth exige diseño opt-in — pre-mortem 3) | ✅ **A** — decidido-por-evidencia: stop L1559 + pre-mortem 1/3 (L1558) |
| 2 | Ubicación de la clase de confianza | A) **`vanta-memory::core::hooks::InjectionPolicy` (extensión de la base VER-04)** (pro: fuente única ya consumida por proxy/MCP/memory; el enforcement YA está ruteado por `policy.allows()` en todas las fuentes (`auto_recall.rs:405-476`, `inject.rs:173-230`) → 0 gates nuevos que duplicar) / B) core `src/rbac.rs` (contra: `pub(crate)`, **no consumible** por proxy/MCP — documentado en VER-04 §Investigación "RBAC core es pub(crate) → ACL propia opt-in"; duplicaría el modelo) / C) módulo core nuevo (contra: API pública nueva sin consumidor en las superficies de inyección) | ✅ **A** — decidido-por-evidencia: VER-04 §Diseño #4 (fuente única) + `auto_recall.rs:119`; la dimensión core (principal→acción) queda en RBAC (decisión 8) |
| 3 | Semántica trusted/tainted | A) **Trusted default; tainted por prefijo boundary-aware; tainted NO inyecta por defecto; opt-in `include_tainted` por superficie; denials reportados (`governance.denied`/`block.denied`, nunca silencioso) y auditados por las superficies existentes** (pro: literal "tainted no inyecta por defecto"; aditivo — lista vacía = byte-idéntico; no-auto-promoción: el ataque wait-it-out no puede ganar por reloj, I1 de mgr-13) / B) inyección solo de trusted explícito (contra: cambia el default — breaking) | ✅ **A** — decidido-por-evidencia: contrato L1557 + pre-mortem 3 (aditivo/opt-in) + mgr-13 §3.3-I1/"Seguridad (área)" Notion ("como tainted, nunca alimenta contexto con autoridad") |
| 4 | Promoción trusted↔tainted | A) **acto explícito del operador (config) en v0; sin auto-promoción por tiempo/contenido; promoción curada con linaje = roadmap v1 (FIND-301)** (pro: I1 mgr-13; MGR-12/SCH-01 no cerrados — la señal asserted/derived vive ahí) / B) auto-promoción por señal (contra: falsos positivos; solapa MGR-12) | ✅ **A** — decidido-por-evidencia: mgr-13 §3.3-I1 + §5.4 (MGR-12 consume, no define) |
| 5 | Relación con SCH-05 (registro) y VER-04 (ACL) | A) **complementaria declarada: cuarentena = estado por REGISTRO (T1-T5); trust = clase por NAMESPACE; ACL = allowlist por SUPERFICIE; gates independientes (AND)** (pro: pre-mortem 2 "semántica complementar, no duplicar gates"; SCH-05 ya implementado en su carril) / B) unificar en un motor ABAC (contra: solapa la máquina T1-T5; scope) | ✅ **A** — decidido-por-evidencia: pre-mortem 2 (L1558) + mgr-13 §2.1/§8 |
| 6 | Superficies cubiertas | A) **las 4 de VER-04: auto_recall gobernado (cubre `memory_recall` + `context_assemble` + `perform_auto_recall`) + `build_memory_block` del proxy (cubre prompt block) + `mem:search`** / B) + HTTP `search`/`list` del server (contra: retrieval explícito de un principal autenticado es la vía de revisión; requiere store de clasificación en core + wire nuevo → roadmap/FIND) | ✅ **A** — decidido-por-evidencia: enforcement existente ruteado (`auto_recall.rs:405-476`: L1 sesión, cross-session, persona, escena; `inject.rs:173-230`: persona/escena/índice) |
| 7 | Config de la clase por superficie | A) **proxy TOML `[injection] tainted_namespaces` + `include_tainted`; MCP `VANTADB_MCP_TAINTED_NAMESPACES` + `VANTADB_MCP_INCLUDE_TAINTED`** (pro: patrón VER-04 `namespace_allow_prefixes`/`VANTADB_MCP_INJECT_NAMESPACES`; opt-in) / B) solo API programática (contra: feature muerta para operadores) | ✅ **A** — decidido-por-evidencia: `vanta-proxy/src/config.rs:103-124` + `vantadb-mcp/src/config.rs:175-186` (patrón) |
| 8 | RBAC por acción (estrategia de este run) | A) **incremento aditivo: audit de denials RBAC en el middleware con label de acción (read/write/delete) + spec del modelo completo (NamespaceRead/Write/Delete + roles configurables) + FIND-301 del enforcement restante** (pro: "+ audit" del contrato; 403 hoy es silencioso — no hay evento alguno; no cambia autorización → no rompe clientes) / B) enforcement por acción ahora (contra: requiere decisión opt-in + registry configurable; pre-mortem 3) | ✅ **A** — decidido-por-evidencia: contrato L1557 ("+ audit") + stop L1559 (FIND del restante) + middleware :210-220 (403 sin audit) + router.rs:134-137 (roles hardcoded — la configurabilidad es el gap real, spec §5) |
| 9 | Modelo de acciones (spec, para FIND-301) | A) **acciones namespace-scoped `read|write|delete`; hoy DELETE mapea a write (`middleware.rs:194-197`) → el modelo agrega `NamespaceDelete` con semántica de separación estricta opt-in (compat: write cubre delete por defecto)** / B) reemplazo directo de write→delete (contra: breaking para roles existentes) | ✅ **A** — decidido-por-evidencia: `rbac.rs:77-81` + pre-mortem 3 |
| 10 | Forma del audit del denial | op `auth_rbac` (vía `AuditEvent::auth("rbac", ...)` — layer desconocido → `auth_<layer>`, `audit.rs:106-111`); subject = role; namespace = ns extraído o `N/A`; outcome = `denied`; reason = `action=<read|write|delete>;enforced=<read|write>;scope=<namespace|global>`; request id adjunto | ✅ decidido-por-evidencia: precedente de reason `k=v;` (VER-03/VER-04) + `audit.rs:99-111`; nunca el token (solo el rol) |
| 11 | Docs | A) **research-doc `mgr-04-policy-engine.md` (spec completa) + filas en `PROXY.md`/`MCP.md` (knobs) + task file + FIND-301** (pro: DoD L1571 "modelo documentado"; R-5 estilo) / B) solo task file (contra: operadores sin referencia de knobs) | ✅ **A** — decidido-por-evidencia: DoD L1571 + VER-04 §S4 (PROXY.md/MCP.md) |

## Invariantes de dominio (handoff — MUST)

1. **Defaults byte-idénticos:** `from_prefixes(p)` ≡ `from_parts(p, [], false)`; lista tainted vacía + `include_tainted=false` → comportamiento EXACTO de VER-04 (tests existentes verdes sin cambios).
2. **Tainted nunca inyecta sin opt-in explícito en las superficies gobernadas (VER-04/MGR-04)** — la superficie L3 del pipeline (`pipeline_worker`, fuera del corte) queda declarada en la spec §5(e) + FIND-301; y el denial es visible (`RecallGovernance.denied`/`InjectionBlock.denied`) — no hay denegación silenciosa (I5 mgr-13).
3. **ACL y trust son gates independientes (AND):** un namespace allowlisted-pero-tainted sigue denegado; tainted-pero-`include_tainted` puede seguir denegado por ACL.
4. **Sin cambio de semántica de autorización HTTP:** el audit `auth_rbac` es observabilidad aditiva; el status y el predicado de autorización del middleware no cambian.
5. **No tocar WIP ajeno:** `vantadb-mcp/src/{code_index,handlers/tools,lib}.rs` + tests de MEMG-09, `src/wiki/**` (MEMG-08), filas de Backlog ajenas; commit con **pathspec**.
6. **Sin cambios de wire/serialización on-disk, sin migración, sin deps nuevas, sin `unsafe`**; `unwrap`/`expect` solo en tests con allow documentado.
7. **El audit nunca registra secretos** (ni el Bearer; subject = rol).
8. **Determinismo en tests:** BTreeMap/TempDir; sin mutación de env (override por struct `McpConfig`/`InjectionConfig`).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** ≤0 — aditivo (enum + métodos + campos privados + config opt-in + evento de audit). Sin `unsafe`, sin deps, sin allocs en hot paths nuevos (`allows()` ya era O(#prefixes) por fuente; +1 lista del mismo orden, config-time).
**Pago:** (1) los denials RBAC del middleware dejan de ser silenciosos (hoy 403 sin registro — R de STRIDE repudio); (2) el residual mgr-13 §4.1 ("la autenticación protege quién escribe, no qué se escribe; no hay clase de confianza") queda cerrado en las 4 superficies de inyección.
**`ponytail:` notes:** (a) el denial por taint reusa el canal `denied` genérico (reason `acl=deny` en las superficies) — split de razones `acl|trust` = mejora diferida (spec §7); (b) wiring de trust al HTTP search/list = roadmap (necesita store de clasificación core); (c) `is_empty()` conserva semántica "sin allowlist ACL" (test-only hoy).
**`NOTICED BUT NOT TOUCHING`:** roles hardcoded del router (`router.rs:134-137`) — la configurabilidad de roles es el heart de FIND-301; `AccessMode` queda con 2 variantes en este run (la 3ª llega con el enforcement).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: (a) spec MGR-04 en research-doc con §Gobernanza + decisiones por evidencia + roadmap; (b) `TrustClass` + `from_parts`/`trust_class` con unit tests (defaults byte-idénticos, boundary-aware, opt-in, gates independientes); (c) integración: tainted no inyecta en `perform_auto_recall_governed` y el denial queda reportado (L1/persona/escena) — test en `vanta-memory/tests/memg10_trust_gate.rs`; (d) wiring proxy (`InjectionConfig` + `Governance::from_config`) y MCP (`McpConfig` + env) con tests de la política construida; (e) audit `auth_rbac` de denials read/write con namespace+rol+action (test en `tests/rbac_namespace.rs` con audit path); (f) FIND-301 en Backlog; (g) fmt/clippy scoped + suites scoped verdes; (h) review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(security): MEMG-10 — ...` + pathspec solo de archivos propios (shared files con contenido ajeno → staging quirúrgico, ver §Notas) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor (vanta-memory + core server) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius hecho en DISCOVERY) · `codebase-memory-mcp_check_index_coverage`
- `cargo nextest` scoped por crate (`-p vanta-memory --test memg10_trust_gate`, `-p vanta-memory`, `-p vanta-proxy`, `-p vantadb-mcp`, `-p vantadb --features server --test rbac_namespace`) + `campaign_verify_cmd`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs` + `gen-index --check`) — research-doc + PROXY.md/MCP.md + task file + Backlog

**Skills cargadas (SDP v3, `campaign_discover_skills_v2` phase=BUILD):** `campaign-executor` · `progreso` · `ponytail` (base auto-MCP) · `test-driven-development` (pin) · `systematic-debugging` (pin) · `source-driven-development` · `security-and-hardening` · `rust-write-tests` · `api-and-interface-design` · `documentation-skill` (docs del run). `doubt-driven-development` → cubierto por review P2-01 adversarial (`vanta-review`); `incremental-implementation` → Steps S1-S6 con corte declarado.

## Fases explícitas — SECURITY | PERFORMANCE | CONCURRENCIA (P2-07)

- [x] **SECURITY** — trust boundary explícito: **contenido almacenado → contexto del modelo** (OWASP LLM01/LLM06/LLM08; fallo típico Notion: "recuperar contenido malicioso en posición de autoridad"). Mitigaciones: clase de confianza deny-by-default para tainted (fail-closed), matching boundary-aware (no `l1/evilish` por `l1/evil`), opt-in explícito para revisión, denials auditados (repudio), config parseada con trim/normalización, audit sin secretos (subject=rol, nunca el Bearer). Sin input nuevo de usuario final; sin deps; sin red. Checklist `security-and-hardening` aplicable: autorización (least privilege: tainted no entra a prompts), audit de eventos de seguridad (RBAC denial ahora auditado), no secrets en logs. Sin hallazgos Critical/High esperados.
- [x] **PERFORMANCE** — no dispara Regla 9: sin hot path nuevo (el gate vive en el mismo `allows()` ya invocado por fuente; +1 comparación de lista config-time). Sin claims de performance (Regla 11).
- [x] **CONCURRENCIA (Regla 8)** — evaluación: el middleware corre async pero el slice solo agrega una llamada a `audit_auth` (Mutex interno del `AuditLogger`, patrón ya usado en los sitios existentes); `InjectionPolicy` sigue siendo `Clone` inmutable post-build (los campos nuevos no agregan locks). No toca `dashmap`/`parking_lot`/Tokio del core → auditoría de concurrencia no dispara (declarado).

## Steps

### Step 0 — DISCOVERY + task file + coordinación

- **Archivos:** este task file + lectura de base (rbac/middleware/audit/VER-04/Notion) + MEMG-09 releído fresco.
- **Acción:** ✅ task file completo (Impacto Regla 0 + Spec + corte declarado) + coordinación MEMG-09 (paths disjuntos verificados en worktree) + Notion §Gobernanza verificada (PROPUESTA).
- **Verify:** ✅ task file existe; `## Spec` completa (tabla de decisiones); Gate D evaluado (pre-respondido por plan F0).
- **Evidencia:** ✅ DISCOVERY arriba; `rg tainted src/` = solo `trusted_proxies` (red); Notion fetch 2026-10-05.

### Step 1 — RED (TDD): tests del contrato (vanta-memory)

- **Archivos:** `vanta-memory/tests/memg10_trust_gate.rs` (nuevo) + unit tests en `auto_recall.rs` (`#[cfg(test)]`).
- **Acción:** tests ANTES de implementar (unit: `from_parts`/`trust_class`/boundary/opt-in/gates independientes/defaults; integración: tainted no inyecta y reporta; include_tainted opt-in; trusted intacto). Primer compile falla con "no method `from_parts`".
- **Verify:** ✅ RED confirmado — `cargo nextest run -p vanta-memory --test memg10_trust_gate` falla por API ausente (razón correcta).
- **Evidencia:** ✅ RED: `error[E0599]: no function or associated item named 'from_parts' found for struct 'InjectionPolicy'` + `error[E0432]` (import `TrustClass` sin resolver) — 5 errores, ninguno por test mal escrito. GREEN posterior: integración **4/4** · unit de policy **9/9**.

### Step 2 — GREEN: TrustClass + extensión de `InjectionPolicy`

- **Archivos:** `vanta-memory/src/core/hooks/auto_recall.rs` (impl), `vanta-memory/src/core/hooks/mod.rs` (re-export).
- **Acción:** implementación mínima: `TrustClass` (+`#[non_exhaustive]`, api-contract R-6), `from_parts`, `trust_class`, `allows()` extendido (AND de ACL y trust), normalización compartida, doc rustdoc.
- **Verify:** unit + integración GREEN: `cargo nextest run -p vanta-memory --test memg10_trust_gate` + `-p vanta-memory` (suite completa, sin regresión).
- **Evidencia:** ✅ `-p vanta-memory` **694/694** (incluye VER-04 sin cambios) · policy **9/9** · integración **4/4**. Impl: `TrustClass` (`auto_recall.rs:125`), `from_parts` (:163), `trust_class` (:183), `allows` extendido (:194), `normalize_prefixes` (:211), `prefix_matches` (:221); export en `hooks/mod.rs`.

### Step 3 — GREEN wiring: proxy + MCP (config opt-in)

- **Archivos:** `vanta-proxy/src/{config.rs,governance.rs,server.rs}` (+ call sites de tests), `vantadb-mcp/src/config.rs`.
- **Acción:** `InjectionConfig.tainted_namespaces`/`include_tainted` (serde default) → `Governance::from_config` usa `from_parts`; `McpConfig.injection_tainted`/`injection_include_tainted` + envs → `injection_policy()` usa `from_parts`. Tests de la política construida (unit proxy governance + MCP si aplica).
- **Verify:** `cargo nextest run -p vanta-proxy` + `-p vantadb-mcp` (incluye ver04 tests existentes) ✅.
- **Evidencia:** ✅ proxy **322/322** (incluye unit nuevo `tainted_namespaces_deny_injection_and_include_tainted_opts_in`; 4 literales de `tests/ver04_governance.rs` + 1 de `icp02_privacy_demo.rs` actualizados por campos nuevos) · MCP `config::tests::injection_policy_composes_acl_and_trust` **1/1** (base nueva de `config.rs` ya commiteada por MEMG-09 `0905e3c5`). Call sites: `server.rs:182`, `memory_tools.rs` test.

### Step 4 — RED→GREEN core: audit de denials RBAC por acción

- **Archivos:** `src/server/middleware.rs` (+ doc del layer en `src/audit.rs` si aplica), `tests/rbac_namespace.rs`.
- **Acción:** en el path `!permitted` emitir `AuditEvent::auth("rbac", ns|N/A, role, "denied", reason{action,enforced,scope})` con request id. RED: test nuevo falla (no hay fila de audit) → GREEN tras el cambio. Tests: denial write y denial read auditados.
- **Verify:** `cargo nextest run -p vantadb --features server --test rbac_namespace` ✅ (8 existentes + 3 nuevos → **11/11**).
- **Evidencia:** ✅ RED genuino: los 3 tests nuevos fallan con "RBAC denial must be audited" (los 8 SRV-05 pasaban). GREEN: `-p vantadb --features server --test rbac_namespace` **11/11** (8 + 3: read/write/delete) · `-E 'test(auth) or test(rbac)'` **48/48** sin regresión. Impl: `middleware.rs:203-243` (`action`/`enforced`/`scope` + `auth_rbac` denied, token nunca auditado). Nota: el binario test requiere `--features server`; un lock de `vanta-cli.exe` por procesos huérfanos de un run previo se resolvió matando los procesos debug (no es falla de código).

### Step 5 — Spec doc + docs + FIND + sync

- **Archivos:** `docs/dev/research/mgr-04-policy-engine.md` (nuevo), `docs/api/PROXY.md`, `docs/api/MCP.md`, `docs/dev/Backlog.md` (FIND-301), este task file (sync).
- **Acción:** spec completa MGR-04 (modelo, integración, complementariedad SCH-05/VER-04, RBAC por acción con gap y roadmap, corte, deuda/FINDs); filas de knobs; FIND-301; sync del task file con resultados reales.
- **Verify:** gates docs (`check-links`/`check-docs`/`gen-index --check`; si gen-index requiere regeneración → staging quirúrgico).
- **Evidencia:** ✅ spec `docs/dev/research/mgr-04-policy-engine.md` (7 secciones: gap verificado, modelo, integración, RBAC por acción, corte, verificación, deuda) · `docs/api/PROXY.md` (§Injection governance: 4 reglas + 3 filas de tabla) · `docs/api/MCP.md` (§Injection governance: 4 reglas + 2 envs) · `docs/dev/Backlog.md` (FIND-301). Gates docs → S6.

### Step 6 — Verify full + OCR + review P2-01 + commit + completed

- **Archivos:** los del run (pathspec).
- **Acción:** verify scoped completo (suites + fmt + clippy) + OCR delegation + review P2-01 (`vanta-review`) + commit local + campaign `completed` (taskId 54, payload review HARD-07). **[en progreso]**
- **Verify:** [en progreso]
- **Evidencia:** [en progreso]

## Notas (coordinación + shared files)

- **MEMG-09 en vuelo** (misma área): releído fresco en DISCOVERY (2026-10-05). Su diff toca `vantadb-mcp/src/{code_index,handlers/tools,lib}.rs`, `vantadb-mcp/tests/code_index_tests.rs`, `docs/dev/research/mgr-22-repo-map.md`, `mgr-23-24-memoria-proyecto.md`, `docs/dev/Backlog.md` (FIND-299/300), `docs/index.md`/`llms.txt` (gen-index) y el plan. **Conflicto real: ninguno** en código (paths disjuntos; `vantadb-mcp/src/config.rs` NO lo toca MEMG-09). Riesgo = **shared files**: `docs/dev/Backlog.md` (mis FIND-301 vs sus FIND-299/300) y artefactos de `gen-index` — si su contenido está sin commitear al cierre → **staging quirúrgico** del contenido propio (blob = HEAD + mi delta vía `git hash-object -w` + `git update-index --cacheinfo`) para no arrastrar su WIP; `git status`/`git diff --cached` verificado antes del commit.
- **PROHIBIDO tocar:** `opencode.jsonc`, master plan, `docs/pipeline-state.json`; WIP ajeno listado arriba.
- Disco: si el linker falla → `dev-tools/target-cleanup.ps1 -Clean -Yes`.
- **Campaign server:** taskId `54` (ya reservada ⏳; el guard WIP=3 rechazó re-marcar in-progress con 3 activas — MEMG-10 ya figura activa; se cierra con `completed` al final).

**Context Save Point (si el run se interrumpe):** estado en §Steps + recitation; trabajo parcial = git diff del worktree; NO re-hacer steps ✅; el slice es aditivo y reanudable (S1-S2 vanta-memory → S3 wiring → S4 core → S5 docs → S6 cierre).
