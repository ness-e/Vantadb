---
title: "MGR-04 — Policy engine: namespaces trusted/tainted + RBAC por acción (spec + slice MGR-04/MEMG-10)"
kind: research
description: "Modelo de confianza por namespace (trusted/tainted) con gate de inyección en las superficies VER-04, y modelo RBAC por acción (read/write/delete namespace-scoped) con audit de denials. Slice implementado: trust gate end-to-end (proxy/MCP) + auth_rbac; enforcement por acción restante → FIND-301."
---

# MGR-04 — Policy engine: namespaces trusted/tainted + RBAC por acción

- **Fecha:** 2026-10-05 · **Tipo:** research/spec + slice implementado (task MEMG-10, plan Task 54)
- **Contrato (plan L1557):** "spec (namespaces trusted/tainted + RBAC por acción + integración con retrieval/inyección) + implementación sobre la base VER-04: clase de confianza por namespace aplicada en retrieval/inyección (tainted no inyecta por defecto) + RBAC por acción con tests por acción + audit; sin romper ACL/namespace actuales (suite verde); modelo documentado (ADR si cambia semántica de auth)"
- **Origen:** plan `2026-10-04-master-plan-0.9.0.md` Task 54 · Backlog MGR-04 (fila L942) · `docs/dev/research/mgr-13-cuarentena.md` §4.1/§8 (deuda declarada) · Notion: páginas "Gobernanza del ciclo de vida de la memoria" y "Seguridad (área)" (fetch vía Notion MCP, 2026-10-05 — la forma exacta de MGR-04 era **PROPUESTA sin spec**: se diseña acá con evidencia de repo, precedente MEMG-09)
- **Estado:** ✅ spec completa + slice (clase mínima + audit RBAC). Lo restante (enforcement por acción) → **FIND-301** (stop condition L1559)
- **Fuera de alcance de este documento:** motor ABAC genérico; detección de poisoning por ML (v1.0); fusión con la máquina de cuarentena T1–T5 (SCH-05, ya especificada — se cita); auth de red (auth L1/L2/L3 existente, MEM-05 — se cita)

## §0. Resumen ejecutivo

El sistema hoy protege **quién** escribe y **desde qué namespace** (auth 3-capas + RBAC namespace-scoped en records/search/list, SRV-05) y **qué registro está bajo revisión** (cuarentena T1–T5, SCH-05), pero **no clasifica el contenido almacenado por confianza**: `tainted` = 0 hits en `src/` y no existe un gate que impida que contenido de un namespace dudoso se convierta en instrucción para el agente (el fallo típico del área Seguridad: "recuperar contenido malicioso dentro de una posición de autoridad"). MGR-04 cierra ese residual en dos dimensiones ortogonales:

1. **Trusted/tainted por namespace (contenido).** Clase de confianza declarada por prefijo de namespace; un namespace **tainted no inyecta por defecto** en las superficies de inyección gobernadas — las 4 de VER-04 (recall → prompt), con opt-in explícito para workflows de revisión. Implementado sobre la base VER-04 (política compartida). *Alcance: la superficie L3 del pipeline queda fuera de este corte — ver §5.*
2. **RBAC por acción (principal).** Modelo de acciones namespace-scoped `read|write|delete`; hoy DELETE mapea a write y los roles namespace-scoped no son configurables. Este run implementa el **audit de denials por acción** (aditivo; antes el 403 era silencioso) y especifica el enforcement restante → FIND-301.

Lo que este documento **NO** propone: un motor ABAC, promoción automática por ML, ni gates de retrieval explícito (HTTP search/list) por defecto — ver §5.

## §1. Gap verificado (código real — HEAD `0905e3c5`, 2026-10-05)

| # | Afirmación | Evidencia |
|---|-----------|-----------|
| 1 | Sin taxonomía de trust de contenido | `tainted` = 0 hits de contenido en `src/` (los `trusted_proxies` de `src/config.rs:246` son de red); mgr-13 §1 tabla #2 re-verificado |
| 2 | La inyección no filtra por confianza | `perform_auto_recall`/`governed` leen L1/persona/escena con un solo predicado: `policy.allows()` (VER-04: ACL allowlist). No hay dimensión de trust |
| 3 | RBAC por acción: solo read/write, delete plegado a write | `src/server/middleware.rs:194-197`: `POST|PUT|PATCH|DELETE → AccessMode::Write`; `src/rbac.rs:77-81`: `AccessMode::{Read,Write}` → `NamespaceRead/NamespaceWrite`. No existe `NamespaceDelete` |
| 4 | Roles namespace-scoped no configurables | `src/server/router.rs:134-137` registra solo `admin`/`reader`/`writer` (permisos globales); `RbacCfg` (`src/config.rs:203-206`) solo mapea token→rol — `NamespaceRead/Write` existen pero ningún camino productivo los registra |
| 5 | Denials RBAC silenciosos | El 403 de `middleware.rs` (bloque :210-220, pre-MEMG-10) **no emitía evento de audit** — a diferencia de los fallos L1/L2/L3 que sí se auditan |
| 6 | La base de inyección es extensible en un solo punto | Todas las fuentes pasan por `policy.allows()`: L1 sesión `auto_recall.rs:473`, persona scoped `:500`, persona sesión `:519`, escena `:536`, L1 cross-session `:646`, path core-search `:906`; proxy: `inject.rs:173-230` (persona/escena/índice). Extender el predicado gatea **las 4 superficies** sin duplicar gates |
| 7 | El estado Notion era PROPUESTA | Página "Gobernanza del ciclo de vida de la memoria": `Namespaces trusted/tainted + RBAC por acción (MGR-04): PROPUESTA (research pendiente)`; "Seguridad (área)" fija el requerimiento pero no la forma |

## §2. Modelo trusted/tainted (contenido)

### 2.1 Definiciones

| Concepto | Semántica | Default |
|----------|-----------|---------|
| **Trusted** | El contenido del namespace puede alimentar prompts (sujeto a ACL + presupuesto). | Todo namespace, si `tainted_namespaces` está vacío |
| **Tainted** | El contenido del namespace **nunca** se inyecta por defecto; requiere opt-in explícito (`include_tainted`). El denial se reporta y se audita. | Nada — la clasificación es un acto del operador |

La clasificación es por **prefijo boundary-aware** (`l1/scratch` taintea `l1/scratch` y `l1/scratch/...`, jamás `l1/scratchy`), el mismo matcher de la ACL VER-04 (`prefix_matches`, `auto_recall.rs:221`).

### 2.2 Reglas

- **R1 — Deny by default para tainted.** Sin opt-in, `allows()` devuelve `false` para un namespace tainted aunque la ACL lo permita (`auto_recall.rs:194-205`).
- **R2 — Gates independientes (AND).** ACL y trust se evalúan por separado: para inyectar hay que pasar **ambos**. Un namespace allowlisted-pero-tainted sigue denegado; tainted-`include_tainted` puede seguir denegado por ACL.
- **R3 — Opt-in explícito.** `include_tainted = true` (por superficie) levanta el gate de trust para workflows de revisión; la ACL sigue aplicando.
- **R4 — Denial observable.** Un denial por trust entra en las listas `denied` de `RecallGovernance`/`InjectionBlock` (bounded a 16, marcador `…overflow`) y las superficies lo auditan (`op=injection`, `outcome=denied`) — nunca silencioso (mismo contrato de VER-04 F1).
- **R5 — Sin auto-promoción.** Nada promueve o degrada por reloj ni por contenido: la clasificación es un cambio de config del operador. El ataque "wait-it-out" (MINJA) no puede ganar por tiempo (I1 de mgr-13 §3.3, adoptado).
- **R6 — Defaults byte-idénticos.** Lista tainted vacía + `include_tainted=false` ⇒ comportamiento EXACTO de VER-04 (test `injection_policy_from_prefixes_equals_from_parts_defaults`).

### 2.3 Complementariedad de gates (pre-mortem 2: no duplicar)

| Gate | Dimensión | Unidad | Estado |
|------|-----------|--------|--------|
| Cuarentena T1–T5 (SCH-05) | Estado de revisión de **un registro** | registro | especificada/implementada en su carril (SCH-02/05) |
| Trust trusted/tainted (MGR-04, este doc) | Clase de confianza de **un namespace** | namespace | **este run** (superficies de inyección) |
| ACL por prefijo (VER-04) | Allowlist de lectura por **superficie** | namespace × superficie | VER-04 ✅ |

Los tres son ortogonales y se evalúan por separado (AND donde aplique): un registro puede estar cuarentenado *y* su namespace tainted *y* la superficie tener ACL restrictiva. Ninguno reemplaza a otro; no se agregan gates duplicados.

## §3. Integración con retrieval/inyección (implementado)

**Superficies cubiertas (las 4 de VER-04, sin tocar sus firmas):**

| Superficie | Punto de enforcement | Efecto del trust |
|------------|---------------------|------------------|
| `perform_auto_recall_governed` (hook compartido) | `policy.allows()` por fuente: L1 sesión, L1 cross-session, persona scoped/sesión, escena | Fuente tainted → no se lee, `governance.deny(ns)` |
| MCP `memory_recall` | hereda el hook gobernado | ídem; audit por memoria/denial existente |
| MCP `context_assemble` | hereda el hook gobernado | ídem |
| Proxy `<vanta-memory>` block + `mem:search` | `build_memory_block` usa `policy.allows()` por sección (`inject.rs:173-230`) | persona/escena tainted → sección omitida + `denied` auditado |

**Configuración por superficie (opt-in, patrón VER-04):**

| Superficie | Clase (tainted) | Opt-in | Evidencia |
|------------|-----------------|--------|-----------|
| Proxy | `[injection] tainted_namespaces = [...]` | `[injection] include_tainted = bool` | `vanta-proxy/src/config.rs:109-116` |
| MCP | `VANTADB_MCP_TAINTED_NAMESPACES=a/,b/` | `VANTADB_MCP_INCLUDE_TAINTED=1|true|yes` | `vantadb-mcp/src/config.rs:119-128,195-207` |

**Audit:** los denials por trust viajan por el canal `denied` existente y las superficies emiten `op=injection`/`outcome=denied` (VER-04). *Límite declarado:* la razón genérica `acl=deny` no distingue ACL de trust — el split `deny=<acl|trust>` es una mejora diferida (§5).

## §4. RBAC por acción (principal)

### 4.1 Estado actual verificado (sin cambios de este run)

- `Permission` (`src/rbac.rs:7-20`): `Read`/`Write`/`Delete` globales + `Admin` + `NamespaceRead(String)`/`NamespaceWrite(String)`.
- `can_access_namespace(role, ns, mode)` (`:70-85`) con `AccessMode::{Read, Write}`; `Admin` bypassa.
- El middleware extrae namespace de path/query (`extract_namespace`, `state.rs:412-434`) y aplica el check solo a identidades Bearer con entrada en `token_role_map` (:180-223). Fail-closed: rol desconocido o sin permiso → 403.
- **Acción real vs enforcement:** `DELETE` se enforcea como `Write` (:194-197). No hay separación delete≠write ni acciones namespace-scoped para delete.
- **Roles:** hardcoded (`router.rs:134-137`); `RbacCfg` solo mapea token→rol.

### 4.2 Modelo por acción (especificado; enforcement → FIND-301)

- **Acciones:** `read` (GET/HEAD), `write` (POST/PUT/PATCH), `delete` (DELETE) — namespace-scoped además de los globales existentes.
- **Permisos nuevos:** `Permission::NamespaceDelete(String)`; `can_access_namespace(role, ns, Action::Delete)`.
- **Compat (pre-mortem 3, aditivo/opt-in):** por defecto `NamespaceWrite` cubre `delete` (semántica actual); la **separación estricta** (delete exige `NamespaceDelete`) se activa por config opt-in. Nada cambia para roles existentes sin el flag.
- **Roles configurables:** `RbacCfg` extendido (aditivo/opt-in) para declarar roles con sus permisos namespace-scoped — hoy es el gap que hace que `NamespaceRead/Write` sean inalcanzables en producción.
- **Audit por acción (implementado en este run):** cada denial RBAC emite `auth_rbac` con `action` (la acción pedida), `enforced` (el modo real aplicado) y `scope` (`namespace|global`) — metadata only, sin token. Tests por acción: read/write/delete (`tests/rbac_namespace.rs`).

### 4.3 Cambio de semántica de auth ⇒ ADR

El enforcement por acción (4.2) **cambia semántica de autorización** para deployments que lleguen a usar roles namespace-scoped ⇒ al implementarse (FIND-301) requiere **ADR** (criterio del contrato L1557). Este run **no** cambia semántica (solo audit aditivo) ⇒ no aplica ADR ahora.

## §5. Corte de este run + lo que queda

**Implementado y verificado (corte declarado, stop L1559 "spec + clase mínima"):**

1. Spec completa (este documento).
2. Trust gate end-to-end: `TrustClass` + `InjectionPolicy::{from_parts, trust_class}` en el hook compartido (`auto_recall.rs:125,163,183,194`), enforcement en las 4 superficies, wiring proxy/MCP con knobs opt-in.
3. Audit de denials RBAC por acción (`auth_rbac`) — incremento aditivo.

**Diferido (FIND-301):**

- (a) Enforcement por acción: `NamespaceDelete` + separación estricta opt-in.
- (b) Roles namespace-scoped configurables (`RbacCfg` extendido).
- (c) Trust aplicado al **retrieval HTTP explícito** (search/list): hoy lectura explícita de un principal autenticado — es la vía de revisión (mgr-13 §5.1/I5); filtrarla por defecto necesita un store de clasificación core y cambia wire semantics ⇒ decidir en su propio slice.
- (d) Promoción curada trusted (con linaje/revisión, alineada con MGR-12/SCH-01) y split de razón de denial `deny=<acl|trust>`.
- (e) **Superficie no gobernada en este corte (review P2-01 R1):** `vanta-memory/src/services/pipeline_worker.rs:717` (`run_context_assembly` → `perform_auto_recall` sin policy, allow-all) persiste `context/<session>/__assembled` (enabled por default :195-198; scheduler 60 s en `vantadb-server/src/scheduler.rs:39`), y los entry points que usan la firma ungoverned (desktop/Python) quedan igual. Decisión pendiente: dotar al worker de una policy (config del servicio, por sesión/global) o declarar el límite en docs — alineado con FIND-301 (mismo principio: no cambiar firma/estado global sin diseño).

## §6. Verificación del slice (evidencia)

| Ítem | Comando → resultado |
|------|---------------------|
| Unit trust (7) | `cargo nextest run -p vanta-memory -E 'test(injection_policy)'` → **9/9 ✅** (2 pre-existentes + 7 nuevos) |
| Integración trust (4) | `cargo nextest run -p vanta-memory --test memg10_trust_gate` → **4/4 ✅** (tainted no inyecta y reporta; opt-in; persona/escena; only-tainted) |
| Regresión vanta-memory | `cargo nextest run -p vanta-memory` → **694/694 ✅** (VER-04 intacto) |
| Wiring proxy | `cargo nextest run -p vanta-proxy` → **322/322 ✅** (incluye unit `tainted_namespaces_deny_injection_and_include_tainted_opts_in`) |
| Wiring MCP | `cargo nextest run -p vantadb-mcp -E 'test(injection_policy)'` → **1/1 ✅** |
| Audit RBAC por acción (3) | `cargo nextest run -p vantadb --features server --test rbac_namespace` → **11/11 ✅** (8 SRV-05 + 3 nuevos: read/write/delete) |
| Auth/RBAC scoped | `cargo nextest run -p vantadb --features server -E 'test(auth) or test(rbac)'` → **48/48 ✅** |

## §7. Deuda, límites y notas

- **Deuda v1.0 / FIND-301:** ver §5 — enforcement por acción + roles configurables + trust en retrieval HTTP + promoción curada.
- **`ponytail:` límites declarados:** (1) denial por trust reusa el canal/razón `denied`/`acl=deny` genérico (split por clase = mejora diferida); (2) `is_empty()` conserva semántica "sin allowlist ACL" (hoy test-only); (3) enforcement de acciones: DELETE→write sigue siendo el default compat.
- **Invariantes que NO se pueden romper al continuar:** defaults byte-idénticos; tainted nunca inyecta sin opt-in **en las superficies gobernadas (VER-04/MGR-04)** — la superficie L3 no gobernada está declarada en §5(e); gates independientes (AND); sin cambio de semántica de auth hasta el ADR de FIND-301; el audit nunca registra tokens.
- **Fuentes internas (evidencia):** `src/rbac.rs`, `src/server/middleware.rs`, `src/server/router.rs`, `src/server/state.rs`, `src/audit.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-proxy/src/{config,governance,inject}.rs`, `vantadb-mcp/src/config.rs`, `tests/rbac_namespace.rs`, `vanta-memory/tests/memg10_trust_gate.rs`; `docs/dev/tasks/VER-04.md`, `docs/dev/tasks/MEMG-10.md`, `docs/dev/research/mgr-13-cuarentena.md`; Notion (títulos citados en el encabezado).
- **Skills (SDP v3):** `test-driven-development` · `systematic-debugging` (pins) · `source-driven-development` · `security-and-hardening` · `rust-write-tests` · `api-and-interface-design` · `documentation-skill` + base `campaign-executor`/`progreso`/`ponytail`.
