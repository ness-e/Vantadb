---
title: "TASK MEMG-16: Compartir/colaboración multi-agente (scopes + permisos + revocación)"
kind: task
description: "Cablear el PermissionChecker (existe completo y testeado, sin superficie de producto) a una superficie SDK: grants/revocación sobre ACL + entidades existentes, scopes org/team/proyecto mapeados, ops chequeadas get/put_shared, doc del modelo (revocación = acceso futuro, no purga) y FIND de propagación/EXE-07."
---

# TASK MEMG-16: Compartir/colaboración multi-agente (scopes + permisos + revocación)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 58, bloque F4 L1665-1691)
- **Fuente:** plan Task 58 + Backlog `MEMG-16` (fila L142) + verificación CÓDIGO-REAL del bloque (l1671) + EXE-07 (`Backlog:852`)
- **Esfuerzo:** 🔴 1-2sem | **Appetite:** max 1mes | **Stop (plan L1676):** 1-2sem sin contrato → scopes + grants + revoke mínimo sobre el checker existente + doc del modelo + FIND (propagación/EXE-07) → **este run entrega exactamente ese paquete mínimo completo**
- **Prioridad:** 🟠
- **Tipo:** Rust — core `src/sdk/api/sharing.rs` (nuevo) + `src/sdk/mod.rs`/`src/sdk/api.rs` (re-exports) + `src/sdk/api/sharing_tests.rs` (nuevo) + `docs/api/SHARING.md` (nuevo) + `docs/dev/Backlog.md` (FIND-305)
- **Turns estimados:** 8-12 (una sesión de sub-agente con corte declarado)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ✅ COMPLETADO — S0-S6 ✅; commits `8a039119` + `97571f3f`; review P2-01 APPROVE (vanta-review, 2 rondas: CHANGES-REQUIRED → fixes R1/R2 → APPROVE)
- **Incógnitas (uphill, 2 del plan — RESUELTAS en DISCOVERY):**
  (a) **¿Superficie de cableado?** → **RESUELTA: SDK memory API (`Embedded`), módulo nuevo `src/sdk/api/sharing.rs`** (evidencia: `Embedded` es el choke point que consumen TODOS los demás planos — bindings Python/Node/WASM (vía `Embedded`), server (`src/server/state.rs:297` declara la intención "authorize against the resolved principal (e.g. with PermissionChecker)"), MCP (`vantadb-mcp/src/handlers/tools.rs`), vanta-memory (`profile_sync.rs:16` `use vantadb::sdk::{Embedded, ...}`); `checker.rs` + `EntityStore` viven en el core; `api-contract.md` R-8: la lógica vive en el core y los bindings son glue. Server/MCP exponen después consumiendo los mismos métodos → FIND-305).
  (b) **¿Semántica de revocación?** → **RESUELTA: revocación de ACCESO (futuro), NO purga de datos** (evidencia: contrato L1674 (b) "tras revocar, el acceso deja de permitirse en el **siguiente acceso**"; el checker lee entidades vivas en cada llamada (sin cache) → delete de ACL / `status=removed` de membresía es efectivo inmediatamente; la purga de lo compartido es erasure MEMG-17 — frontera declarada en `docs/api/SHARING.md`; convención existente `checker_tests.rs:134` (`removed_member_denied`, status "removed")). Registrado en `campaign_memory(decisions)`.
- **Pendientes (downhill):** 6 steps (S1–S6); S0 ✅ al crear este archivo.
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `58` (keyeada por el server; ⛔ NO usar el ID textual "MEMG-16")
- **Coordinación:** VER-10 (attestation) EN VUELO — área disjunta (`src/attestation.rs`, `src/wal.rs`, `src/cli.rs`, `src/sdk/api/memory.rs`, `docs/api/CERTIFIED_DELETE.md`). Este run NO toca ninguno de esos archivos (sharing.rs es archivo nuevo; `src/sdk/api.rs` solo gana una línea `pub mod sharing;`; `src/sdk/mod.rs` solo re-exports). Releer fresco antes de cada edición; pathspec SIEMPRE en el commit; conflicto real → BLOQUEO (ver §Notas).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno existente afectado: módulo nuevo + re-exports aditivos (`src/sdk/mod.rs`). `Embedded` gana 8 métodos nuevos (no cambia firma de ninguno existente). |
| Callees | Reuso puro: `crate::entity::{EntityStore, EntityWrite, EntityPage}` + `crate::entity::checker::{PermissionChecker, Action, TeamRole, Visibility, PermDecision}` (existentes, cero cambios) + `self.put/get` (existentes) + `self.audit`/`AuditEvent` (existentes) + `validate_key` implícito vía EntityStore. **Cero deps nuevas, cero `unsafe`, cero wire on-disk, cero migración.** |
| Implicaciones | **API pública aditiva:** 1 módulo (`sdk::api::sharing`, re-exportado en `vantadb::sdk`) + 7 tipos (`ShareAssetInput`, `TeamMemberInput`, `GrantInput`, `Subject`, `AccessQuery`, `SharedOutcome`, + re-exports `Action`/`TeamRole`/`Visibility`/`PermDecision`) + 8 métodos `Embedded::{share_asset, add_team_member, revoke_team_member, grant_access, revoke_access, check_access, get_shared, put_shared}`. **Comportamiento default byte-idéntico:** nada cambia para quien no llame los métodos nuevos; los métodos gestionan entidades `entity:{ns}:{collection}` (partición `InternalMetadata` ya existente) y leen con el checker existente. Sin cambios de semántica en rutas existentes. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `65144964` + WIP ajeno `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` + `opencode.jsonc` + untracked `dev-tools/heavy-test-lock.ps1`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/entity/checker.rs` (342L completo — `Action`/`Visibility`/`TeamRole`/`PermDecision`, `PermissionChecker` cadena allow-only 7 pasos `:122-210`, `membership` `:213`, `acl_allows` `:229`, `role_default_covers` `:267`).
  - `src/entity/checker_tests.rs` (:1-379 — seeds `team_member`/`asset`/`acl`, tests de cadena; `removed_member_denied:134` = convención status "removed"; `acl_agent_subject_requires_agent_id:340`).
  - `src/entity/mod.rs` (:1-318 — `EntityRepository` trait `:85`, `EntityStore` `:113`, `Entity` `:47`, `EntityWrite` `:71`, `entity_key`/`validate_scope`/`validate_key` `:278-312`; charset id: sin `{`,`}`,`:`).
  - `src/sdk/builder.rs` (:12-70 struct `Embedded` + `from_engine`; `:141 engine_handle()` `pub(crate)`; `:147 audit()` `pub(crate)`).
  - `src/sdk/api.rs` (:1-90 — módulos `admin/graph/memory/namespaces/search`, re-export `BulkImportReport`, patrón test `make_embedded` in-memory `:34-42`).
  - `src/sdk/mod.rs` (39L completo — `mod api` privado, re-exports `types::*`/`merge`).
  - `src/sdk/api/namespaces.rs` (:1-40 — patrón `impl Embedded` por dominio, REVIEW-12).
  - `src/sdk/api/memory.rs` (secciones funcionales: `put` `:836-853` con audit; `get` `:1156-1170`; `delete`/`delete_inner` `:1231-1263`; `put_record_exact` `:1342`).
  - `src/sdk/types/record.rs` (:176-222 `MemoryInput` campos).
  - `src/sdk/serialization/mod.rs` (:108-153 `validate_namespace`/`validate_key` del SDK).
  - `src/error.rs` (:137-300 enum `Error` — sin variante de permiso; patrón decisión-dato).
  - `src/audit.rs` (:41-66 `AuditEvent::new(op, namespace, key, outcome, reason)`).
  - `vanta-memory/src/core/profile/profile_sync.rs` (:1-120 — scope `team:{t}|agent:{a}`, `ProfileIsolation`).
  - `vanta-memory/src/core/hooks/auto_recall.rs` (:630-682 — filtro D22 `RecallScope::{Session,Agent,Team}`).
  - `vanta-memory/src/core/abstractions/types.rs` (:106-114 — `team_id`/`agent_id` por registro).
  - `src/server/state.rs` (:294-311 `AuthIdentity` + comentario `:297`), `src/server/middleware.rs` (:178-190 comentario L3→PermissionChecker).
  - `docs/dev/Backlog.md` (:130-143 MEMG-16; `:852` EXE-07), plan (L1665-1691), `docs/dev/tasks/MEMG-04.md` (formato canónico + precedente Gate D), `docs/dev/tasks/MEMG-05.md` (coordinación).
  - `.opencode/rules/api-contract.md` + `.opencode/rules/core-engine.md` + `.opencode/references/clean-code-clean-architecture.md` (Apéndice V) + `.opencode/task-system/prompts/findings.md`.
- **Referencias hacia dentro (imports):** `sharing.rs` → `crate::entity::{EntityStore, EntityWrite}`, `crate::entity::checker::{...}`, `super::super::builder::Embedded`, `crate::error::{Error, Result}`, `crate::node::FieldValue`, `crate::audit::AuditEvent` (todos existentes).
- **Referencias entrantes (verificadas con codegraph/rg):** `PermissionChecker` = solo `checker.rs` + `checker_tests.rs` (+ 2 comentarios `middleware.rs:179`, `state.rs:297`) → **0 callers de producto hoy (el hueco que esta tarea cierra)**; `EntityStore` = callers en `src/server/*` + `src/entity/*` (sin cambio); `Embedded::put/get` = callers masivos (sin cambio de firma); `sdk/mod.rs` re-exports = consumo vía `vantadb::sdk::*` (aditivo).
- **Archivos a crear/tocar (este run):** `src/sdk/api/sharing.rs` (nuevo), `src/sdk/api/sharing_tests.rs` (nuevo), `src/sdk/api.rs` (+1 línea `pub mod sharing;`), `src/sdk/mod.rs` (+1 bloque `pub use`), `docs/api/SHARING.md` (nuevo), `docs/dev/Backlog.md` (FIND-305), `docs/dev/tasks/MEMG-16.md` (este).
- **Veredicto impacto:** **MEDIO (API pública aditiva, trust boundary de autorización)** — superficie nueva de gestión de permisos que reutiliza la cadena de decisión existente; sin cambio de comportamiento default; sin wire/migración/deps/unsafe. **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato L1674 ordena "scopes + grants + revocación + doc" y el pre-mortem L1675 ordena "decidir superficie en DISCOVERY"; cualquier superficie elegida agrega símbolos públicos (es la forma mínima del mandato — sin métodos públicos no hay "un grant habilita acceso entre 2 agentes/usuarios" en superficie de producto). Precedente idéntico aprobado: MEMG-04 (`MEMG-04.md:52`, Gate Result ✅ DO L1616) y MEMG-10/WIRE-18/MEMG-09 ("Gate D pre-respondido por el plan F0"). La decisión de superficie está registrada en §Spec D1 + `campaign_memory(decisions)`.

## Contrato

> Verbatim del plan (L1674 — ley):

"memoria compartida con scopes (org/team/proyecto → mapeo declarado a namespace/entidades) + grants y revocación efectiva sobre las operaciones declaradas (read/write/use/recall entre agentes y usuarios): (a) un grant habilita acceso compartido entre 2 agentes/usuarios y (b) tras revocar, el acceso deja de permitirse en el siguiente acceso (test); (c) modelo de datos compartidos documentado (aislamiento, propagación/contagio, revocación, qué pasa con lo ya recordado — frontera con erasure MEMG-17 y cuarentena SCH-05/MEMG-10); reusa `PermissionChecker`/ACL y scopes existentes (no duplicar); containment ante error de un agente declarado (enlace o límite explícito con EXE-07); suites verdes."

- **Pre-mortem (plan L1675):** (1) checker sin cablear → superficie decidida ✅ (D1: SDK); (2) "revocación" ambigua → semántica declarada ✅ (D3: acceso futuro, no purga; frontera MEMG-17); (3) propagación/contagio (MAST) → scope explícito (D6: lo que se propaga = acceso, no copias; containment mínimo = revocación; EXE-07 → FIND-305); (4) EXE-07 (P50) pendiente → no bloquea: containment mínimo propio (revocación inmediata) + dependencia declarada (FIND-305).
- **Stop (plan L1676):** 1-2sem sin contrato → scopes + grants + revoke mínimo sobre el checker existente + doc del modelo + FIND (propagación/EXE-07). **Este run = ese paquete, completo.**
- **Corte declarado de este run (stop package):** grants (asset/membership/ACL) + revocación (delete ACL / status=removed) + `check_access` + ops chequeadas `get_shared`/`put_shared` + scopes declarados + doc del modelo (`docs/api/SHARING.md`) + FIND-305 (exposición server/MCP, propagación recall/vanta-memory, EXE-07). Residuales → FIND-305: wire server/MCP (consumen los métodos), recall de vanta-memory filtrado por grant (hoy filtro D22 por label), cascada multi-agente EXE-07.

## Spec (SDD — decisiones por evidencia)

> **Gate spec-first:** tabla de decisiones con alternativas + resolución por evidencia.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Superficie de cableado | A) **SDK memory API (`Embedded`), módulo `src/sdk/api/sharing.rs`** (pro: choke point de todos los planos — bindings/server/MCP/vanta-memory consumen `Embedded`; checker+EntityStore viven en core; R-8 api-contract: lógica en core, bindings glue; wiring único → cero duplicación; embedded-first) / B) server HTTP (contra: MEMG-04 ya cubrió tenant RBAC coarse; endpoints+handlers+auth+docs = scope mayor; consume `Embedded` igual → se expone después) / C) MCP tools (contra: consume `Embedded` igual; no sirve a vanta-memory ni al SDK) / D) vanta-memory (contra: capa de producto sobre el SDK; duplicaría el acceso al checker para server/MCP) | ✅ **A** — decidido-por-evidencia: `state.rs:297` (intención server), `profile_sync.rs:16` (vanta-memory usa `vantadb::sdk`), `api-contract.md` R-8; pre-mortem "reutilizar, no re-implementar" |
| 2 | Estrategia de reuso | A) **`EntityStore` (CRUD entidades) + `PermissionChecker` (cadena allow-only) tal cual — cero re-implementación, cero estructuras nuevas** / B) nueva capa de grants paralela (contra: duplica ACL/checker — prohibido por contrato "no duplicar") | ✅ **A** — decidido-por-evidencia: `checker.rs:122-257` ya resuelve la decisión; `EntityStore` ya persiste las 3 colecciones (`asset`/`team_member`/`acl`) |
| 3 | Semántica de revocación | A) **Acceso futuro: `revoke_access` = delete de la entidad ACL; `revoke_team_member` = `status=removed`; el checker lee vivo → el siguiente acceso deniega** (pro: contrato (b) dice "el siguiente acceso"; sin cache; preserva data; frontera limpia con erasure) / B) purga de lo compartido (contra: es MEMG-17 erasure; borrar data del receptor por revocar un grant = pérdida de datos; requiere DEK/crypto-shredding ajeno) | ✅ **A** — decidido-por-evidencia: contrato L1674(b); `checker.rs` sin cache; `checker_tests.rs:134` (convención "removed"); frontera MEMG-17 declarada en doc |
| 4 | Mapeo de scopes | A) **org→namespace (tenant MEMG-04), proyecto→sub-namespace `{org}/{proyecto}` (convención ya usada: `l1/{session}`, `profile/{scope}`), team→entidades `team_member` + `team_id` en asset, asset→entidad `asset`+`acl`** (pro: 0 código nuevo de scopes; reusa aislamiento por namespace existente; declarado en doc + module docs) / B) entidad `org`/`project` nueva (contra: 0 consumidor; migración; scope) | ✅ **A** — decidido-por-evidencia: `validate_namespace` acepta `/` (`serialization/mod.rs:121-129`); MEMG-04 namespace=tenant; convención `l1/`/`profile/` en vanta-memory |
| 5 | Modelo de grants | A) **Subjects del checker tal cual: `user` / `team_role` / `agent`; acciones `Read/Write/Assign/Share/Use`; `grant_access` upserta ACL `effect=allow` (clave `{asset}.{subject_type}.{subject_id}.{action}`); `add_team_member`/`revoke_team_member` para membresía** (pro: paridad exacta con `acl_allows`/`membership`; los tests del checker ya fijan el formato de claves) / B) subjects nuevos (contra: tocar el checker → riesgo) | ✅ **A** — decidido-por-evidencia: `checker.rs:229-257` (clave ACL), `:213-226` (clave membresía), `checker_tests.rs:340-366` (agent subject) |
| 6 | Operaciones chequeadas | A) **`check_access` (primitiva, cualquier acción) + `get_shared`/`put_shared` (check + op; devuelven `PermDecision` + registro solo si allowed)** (pro: "efectiva sobre las operaciones declaradas" con punto de enforcement real: read→get_shared, write→put_shared, use/recall→check_access declarado; record ausente si deny = sin leak) / B) solo `check_access` (contra: "efectiva" queda como obligación del consumidor sin punto demostrable en producto) | ✅ **A** — decidido-por-evidencia: contrato "read/write/use/recall"; patrón decisión-dato de `rbac.rs` (bool → caller 403) |
| 7 | Superficie de denegación | A) **`PermDecision` como dato (sin variante de error nueva; `record: None` cuando deny)** (pro: sin cambio de enum público `Error`; sin mapping nuevo en bindings/server/MCP; precedente `can_access_namespace` → bool → 403 del caller; sin oráculo de existencia) / B) `Error::PermissionDenied` nueva variante (contra: enum público + `is_retriable`/`recovery_hint` + 4 bindings + server mapping; sin precedente) | ✅ **A** — decidido-por-evidencia: `error.rs:137+` (sin variante; patrón `ChainedError`/reasons), `rbac.rs:70-85` (decisión como valor) |
| 8 | Identidad del asset | A) **`asset_id` = key del registro (convención declarada en doc); keys con `{`/`}`/`:` no son válidas como asset id (charset `EntityStore::validate_key`, `entity/mod.rs:301-312`) → error de validación explícito** (pro: mapping 1:1 key↔asset; sin encoding; simple) / B) encoding/hash del key (contra: opaco, doble mapping, sin necesidad hoy) | ✅ **A** — decidido-por-evidencia: `entity/mod.rs:301-312`; `validate_key` SDK permite `:` en keys (`serialization/mod.rs:133-153`) → límite v1 documentado (ponytail: encoding si aparece la necesidad) |
| 9 | Audit | A) **5 ops de gestión (`share`/`grant`/`revoke`/`member_add`/`member_revoke`) emiten `AuditEvent` (Repudiation — security skill); ops chequeadas no (ruido de lectura)** (pro: trazabilidad de quién otorgó/revocó; audit opt-in existente; coste 0 default) / B) sin audit (contra: eventos de seguridad sin registro) | ✅ **A** — decidido-por-evidencia: `audit.rs:43-59`; MEMG-04 auditó cuota; skill `security-and-hardening` (audit logging) |
| 10 | Docs | A) **`docs/api/SHARING.md` (modelo: scopes, grants, revocación, propagación/contagio, fronteras MEMG-17/SCH-05/MEMG-10, containment/EXE-07, trust model) + task file + FIND-305** (pro: DoD L1688 "modelo documentado"; Regla 3) / B) solo task file (contra: sin referencia de producto) | ✅ **A** — decidido-por-evidencia: contrato L1674(c) + DoD |
| 11 | ADR | A) **no ADR** (pro: extiende el modelo de autorización ya adjudicado (MEM-04/MEM-05 checker+ACL) a su superficie de producto; semántica de revocación registrada en doc + `campaign_memory(decisions)`; Regla 5: ADR exige autor humano) / B) ADR (contra: no cambia el modelo, lo EXPONE; precedente MEMG-04 "no ADR") | ✅ **A** — decidido-por-evidencia: precedente `MEMG-04.md:78` (D9); el modelo checker/ACL es preexistente |

## Invariantes de dominio (handoff — MUST)

1. **Defaults byte-idénticos:** nada cambia para quien no llame los métodos nuevos; sin tocar rutas existentes de `put`/`get`/`search`/etc.
2. **La cadena de decisión es la del checker existente — no se toca `checker.rs`.** La tarea solo la expone; cualquier cambio de semántica del checker = otra tarea.
3. **Revocación = acceso futuro, no purga:** `revoke_access` borra la ACL; `revoke_team_member` marca `status=removed` (la entidad queda para auditoría). Lo ya entregado/copiado no se retira — eso es erasure MEMG-17 (frontera declarada en doc).
4. **Fail-closed:** asset ausente/archivado → deny `asset_not_available`; sin regla que permita → deny `no_permission` (allow-only del checker, intacto).
5. **Deny no filtra el registro:** `get_shared`/`put_shared` devuelven `record: None` cuando `allowed=false`.
6. **Charset de entidades:** asset_id/membership ids deben cumplir `validate_key` de `EntityStore` (sin `{`,`}`,`:`); error explícito, nunca panic.
7. **No tocar WIP ajeno:** VER-10 (attestation) toca `src/attestation.rs`/`src/wal.rs`/`src/cli.rs`/`src/sdk/api/memory.rs` — este run no edita ninguno; re-releer fresco antes de editar; staging quirúrgico + pathspec; conflicto real → BLOQUEO.
8. **Sin cambios de wire/serialización on-disk, sin migración, sin deps nuevas, sin `unsafe`;** `unwrap`/`expect` solo en tests con allow documentado.
9. **Audit sin secretos:** op/namespace/key/outcome/reason; nunca tokens.
10. **Determinismo en tests:** `Embedded` in-memory (`storage_path: ":memory:"` + `BackendKind::InMemory`), sin mutación de env, sin timing.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** ≤0 — aditivo (1 módulo nuevo + 7 tipos + 8 métodos que delegan en el checker/EntityStore existentes). Sin `unsafe`, sin deps, sin estructuras residentes nuevas.
**Pago:** (1) cablea a producto el `PermissionChecker` que llevaba desde MEM-04 sin superficie (hueco explícito del plan L1671: "no está cableada a ninguna superficie de producto"); (2) expone grants/revocación reutilizando el modelo ACL existente (0 duplicación); (3) registra la semántica de revocación (acceso vs purga) que estaba ambigua — pre-mortem #2.
**`ponytail:` notes:** (a) ids con `.`/`{`/`}`/`:` no son compartibles como asset/membership (separadores de claves compuestas + charset EntityStore) — encoding si aparece la necesidad; (b) ops chequeadas = `get_shared`/`put_shared` (no se instrumentan search/list/delete — mismo `check_access` disponible para consumidores; FIND-305 si se piden); (c) sin jerarquía de grants por proyecto a nivel ACL (proyecto = sub-namespace, D4) — subject `project` solo si aparece el caso.
**Deuda declarada del review P2-01 (O1/O2):** O1 — el template de claves ACL/membresía se reconstruye en `sharing.rs` (duplicado de `checker.rs:214,248`; pinneado por tests cruzados grant→check — builder compartido si aparece drift); O2 — `PermDecision.reason` distingue deny reasons (oráculo de estado para callers no confiables): colapsar a 403/404 uniforme al cablear server/MCP (prerequisito del slice (a) de FIND-305).
**`NOTICED BUT NOT TOUCHING`:** `Permission::NamespaceDelete`/trust de retrieval = FIND-301 (MEMG-04); recall de vanta-memory filtrado por label D22 sin grant → FIND-305; cascada multi-agente EXE-07 (P50) → FIND-305.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: (a) grant habilita acceso entre 2 agentes/usuarios (test: usuario→usuario y agent subject con `agent_id`); (b) tras revocar, el siguiente acceso se deniega (test: revoke ACL + revoke membresía); (c) `docs/api/SHARING.md` con modelo (scopes, propagación, revocación, fronteras MEMG-17/SCH-05/MEMG-10, containment/EXE-07, trust model); (d) reuso del checker (cero re-implementación — `rg` de claves ACL/membresía); (e) suites scoped verdes + fmt/clippy; (f) FIND-305 registrado; (g) review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(memory): MEMG-16 — ...` + pathspec solo de archivos propios (zona VER-10 evitada; staging quirúrgico) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor (core SDK surface) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius hecho en DISCOVERY) · `codebase-memory-mcp_check_index_coverage` (a ejecutar sobre archivos tocados)
- `cargo nextest` scoped CON LOCK PESADO (`pwsh dev-tools/heavy-test-lock.ps1 acquire` → correr → `release`): `-p vantadb --lib -E 'test(sharing)'` · suite `-p vantadb --lib` al cierre
- `cargo check -p vantadb` + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`) — SHARING.md + task file + Backlog

**Skills cargadas (SDP v3, `campaign_discover_skills_v2` phase=BUILD + pin del owner):** `campaign-executor` · `progreso` · `ponytail` (base auto-MCP) · `security-and-hardening` (pin owner) · `api-and-interface-design` (pin owner) · `rust-write-tests` (pin owner) · `test-driven-development` (pin) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `context-engineering` · `documentation-skill` (docs).

## Fases explícitas — SECURITY | PERFORMANCE | CONCURRENCIA (P2-07)

- [x] **SECURITY** (✅ revisión aplicada — 0 hallazgos Critical/High/Medium en archivos propios; fail-closed y no-oráculo verificados por tests) — trust boundary explícito: **principal → asset ajeno** (OWASP API1/BOLA; CWE-639). Amenazas STRIDE: *E*levación (grant sin membresía → deny `not_team_member`; visibilidad private/restricted → deny), *I*nformación (deny devuelve `record: None`; sin oráculo), *R*epudio (audit de gestión), *T*ampering (upsert de ACL solo `effect=allow`; el checker ya ignora deny como "no allow" y `restricted_acl_deny_effect_wins` cubre el efecto). Mitigaciones: cadena allow-only del checker intacta (fail-closed), revocación efectiva al siguiente acceso, management plane con el mismo trust que el resto del SDK embedded (documentado en SHARING.md §trust model — el server gatea al exponer). Checklist `security-and-hardening` ✅ al cierre. Riesgo residual declarado: management plane sin auth propia (FIND-305 al exponer server/MCP); propagación recall (FIND-305).
- [x] **PERFORMANCE** (✅ sin claims; Regla 9 no dispara — funcionalidad, no optimización) — coste: management ops = 1 escritura de entidad (partición `InternalMetadata`, ya existente) + audit opt-in; `check_access`/`get_shared`/`put_shared` = 1-3 lecturas KV (mismo orden que `get`) + (put) el write existente. Sin hot path de search/ingestión; sin claims de performance (Regla 11); Regla 9 no dispara (no hay optimización — funcionalidad nueva).
- [x] **CONCURRENCIA (Regla 8)** (✅ no dispara auditoría — sin locks/async/multi-índice nuevos; `EntityStore` usa el write path existente) — evaluación: sin `dashmap`/`parking_lot`/Tokio/orden de locks nuevo; `EntityStore` usa el mismo `write_backend_batch`/`put_to_partition` que el resto del SDK (serialización del engine intacta); checker = lecturas puntuales sin cache ni locks propios. **No dispara auditoría de concurrencia** (declarado; sin paths multi-índice). Revocación concurrente con un acceso en vuelo: semántica de "siguiente acceso" (documentada) — el acceso ya iniciado puede completar; consistente con contrato.

## Steps

### Step 0 — DISCOVERY + task file + coordinación

- **Archivos:** este task file + lectura de base (checker/entity/sdk/error/audit/vanta-memory/server comments/Backlog/plan/rules/clean-code).
- **Acción:** ✅ task file completo (Impacto Regla 0 + Spec + corte declarado); incógnitas (a)/(b) resueltas con evidencia; VER-10 releído (área disjunta — sin WIP en archivos propios); heavy-test-lock verificado (LIBRE).
- **Verify:** ✅ task file existe; `## Spec` completa (tabla de decisiones); Gate D evaluado (pre-respondido por plan F0, precedente MEMG-04).
- **Evidencia:** ✅ DISCOVERY arriba; `git status` = `M docs/dev/plans/...` (ajeno) + `M opencode.jsonc` (ajeno) + `?? dev-tools/heavy-test-lock.ps1`; `rg PermissionChecker` = solo checker(+tests)+2 comentarios.

### Step 1 — RED (TDD): tests del contrato

- **Archivos:** `src/sdk/api/sharing_tests.rs` (nuevo; 437L tras el test de regresión R1) + `src/sdk/api/sharing.rs` (módulo mínimo: docs + wiring de tests) + `src/sdk/api.rs` (+`pub mod sharing;`).
- **Acción:** 9 tests escritos ANTES de implementar (contrato (a)/(b) user/agent, membresía, fail-closed, no-oráculo, upsert, validación, team_role); +1 test de regresión R1 tras el review P2-01 → 10.
- **Verify:** ✅ RED genuino — compile falla por API ausente, no por tests mal escritos.
- **Evidencia:** ✅ `cargo nextest run --profile audit -p vantadb --lib -E 'test(sharing)'` (CON LOCK) → `error[E0432]: unresolved imports super::{AccessQuery, Action, GrantInput, ...}` + 14× `error[E0599]: no method named {share_asset, grant_access, check_access, put_shared, get_shared, revoke_team_member} found` + E0609 (campos de `()`); `could not compile (lib test) due to 23 previous errors` (exit 101).

### Step 2 — GREEN gestión de entidades: `share_asset` + membresía

- **Archivos:** `src/sdk/api/sharing.rs` (tipos + `share_asset`/`add_team_member`/`revoke_team_member`), `src/sdk/api.rs`, `src/sdk/mod.rs` (re-exports).
- **Acción:** upsert de `asset` (`team_id`/`owner_user_id`/`visibility`/`status=active`) y `team_member` (`role`/`status=active`); `revoke_team_member` = `status=removed` (preserva role); audit `share`/`member_add`/`member_revoke`; validación boundary de ids (`validate_entity_id` — renombrado en el fix R1).
- **Verify:** ✅ compila + tests de gestión verdes (GREEN único con S3/S4).
- **Evidencia:** ✅ `sharing.rs:169` (`share_asset`), `:216` (`add_team_member`), `:256` (`revoke_team_member`), helpers `:181/:228/:273/:463`.

### Step 3 — GREEN grants: `grant_access` + `revoke_access` + `check_access`

- **Archivos:** `src/sdk/api/sharing.rs`.
- **Acción:** upsert ACL allow (clave `{asset}.{subject_type}.{subject_id}.{action}`, campos effect/permission/subject_*) + delete de ACL + `check_access` → `PermissionChecker::can_access_asset` (reuso directo); audit `grant`/`revoke` con razón `subject_type:subject_id:action`.
- **Verify:** ✅ tests (a)/(b)/(c)/(d)/(e) verdes.
- **Evidencia:** ✅ `sharing.rs:304` (`grant_access`), `:356` (`revoke_access`), `:391` (`check_access`), helpers `:316/:368/:457`.

### Step 4 — GREEN ops chequeadas: `get_shared` + `put_shared`

- **Archivos:** `src/sdk/api/sharing.rs`.
- **Acción:** check + `self.get`/`self.put`; `SharedOutcome { decision, record }` con `record: None` en deny (sin lectura ni escritura en deny).
- **Verify:** ✅ 9/9 tests verdes en el primer GREEN completo.
- **Evidencia:** ✅ `cargo nextest run --profile audit -p vantadb --lib -E 'test(sharing)'` → 9/9 en el GREEN inicial y **10/10 tras el fix R1** (`Summary: 10 tests run: 10 passed`); `sharing.rs:409` (`get_shared`), `:434` (`put_shared`).

### Step 5 — VERIFY + docs: SHARING.md + FIND-305/306 + gates

- **Archivos:** `docs/api/SHARING.md` (nuevo; 166L con los fixes R1/N3/O4), `docs/dev/Backlog.md` (FIND-305 + FIND-306), `docs/user/operations/CONFIGURATION.md` (fix colateral de link roto pre-existente), `docs/index.md` + `docs/api/index.md` (gen-index).
- **Acción:** doc del modelo (scopes D4, grants, revocación D3, propagación, fronteras MEMG-17/SCH-05/MEMG-10, containment/EXE-07, trust model, límites); FIND-305 (exposición server/MCP + recall + EXE-07); FIND-306 (clippy drift pre-existente); fix del link gating roto; `gen-index --write`.
- **Verify:** ✅ fmt --check (exit 0); ✅ `cargo clippy -p vantadb --all-targets -- -D warnings` **exit 0 sin diagnósticos** (los 2 ajenos de la corrida anterior quedaron resueltos por sus sesiones: VER-10 cerró su doc lint y el fix de `merge_tests` aterrizó); ✅ `-p vantadb --lib` completo **2359/2359** (CON LOCK, 219s, tras fixes R1); ✅ check-links (0 links gating rotos) + check-docs (sin violaciones nuevas) + validate-docs-coverage 0 gaps.
- **Evidencia:** ✅ comandos/resultados arriba; OCR delegation (advisory) aplicada sobre `sharing.rs` + `sharing_tests.rs`: 0 Critical/High/Medium.

### Step 6 — CIERRE: OCR + review P2-01 + commit + campaign

- **Archivos:** commits `8a039119` (docs: fix link) + `97571f3f` (feat: MEMG-16).
- **Acción:** ✅ OCR delegation (0 Critical/High/Medium) → review P2-01 ronda 1 `CHANGES-REQUIRED` (R1 colisión de claves compuestas + R2 staging) → fixes (`validate_entity_id` + test de regresión + staging selectivo) → ronda 2 `APPROVE` → commits LOCALES (pathspec estricto, sin WIP VER-10) → `campaign_update_task_state(completed)` → skill progreso.
- **Verify:** ✅ hashes `8a039119` + `97571f3f`; veredicto APPROVE (sesión re-review `ses_ef1030fa9ffeMYBrzFuqL4KoXl`); campaign updated.
- **Evidencia:** ✅ §Review abajo; re-verify post-fix: fmt 0 / clippy 0 / scoped 10-10 / lib 2359-2359.

## Review P2-01 (fork vanta-review — adversarial, paths `src/sdk/**`)

- **Ronda 1 (CHANGES-REQUIRED):** R1 — colisión de claves compuestas: ids con `.` aceptados en el boundary colisionaban las claves del checker (`{team}.{user}`, `{asset}.{subject_type}.{subject}.{action}`; precondición `checker.rs:25`). R2 — el commit planificado arrastraba WIP ajeno (FIND-307/308 + WRITE_RECEIPTS en índices). Nits N1-N3 + optional O1-O4.
- **Fixes aplicados:** `validate_entity_id` (rechaza `.`/`{`/`}`/`:`; aplicado a todos los ids compuestos + `agent_id`) + test `rejects_ids_with_composite_key_separators` + `SHARING.md §Limits` + module docs; staging selectivo verificado (`git show --name-only 97571f3f` sin VER-10; Backlog solo FIND-305/306; índices regenerados en worktree temporal HEAD+docs propios, auto-consistentes para el árbol del commit); N1 (`:856`) + N2 (clippy verde re-corrido) + N3/O3/O4 aplicados; O1/O2 declarados en §Deuda.
- **Ronda 2:** ✅ **APPROVE** (verificación del revisor: `git show` de ambos commits anti-fuga VER-10, contrato scoped 10/10 re-ejecutado, fmt/clippy/docs gates exit 0). Revisor ≠ autor (fork fresco, P2-01).

## Notas

- **Coordinación VER-10 (actualizado 2026-10-05):** el WIP de VER-10 apareció EN VUELO durante este run (`src/attestation.rs` +495, `src/sdk/api/memory.rs` +47, `tests/write_receipts.rs`, `docs/api/WRITE_RECEIPTS.md`, `docs/dev/tasks/VER-10.md` — también `docs/api/CERTIFIED_DELETE.md`/`WAL_INTEGRITY.md`). Sin solape con mis paths (`sharing.rs`/`sharing_tests.rs` nuevos, `src/sdk/api.rs` +1 línea, `src/sdk/mod.rs` re-exports). Commit con pathspec ESTRICTO — nada de VER-10 se stagea. Su WIP rompe `clippy` crate-wide (`attestation.rs:664` doc lint) — ajeno, se auto-resuelve en su cierre; registrado en el recitation.
- **Colaterales:** (a) `src/sdk/merge_tests.rs` tenía `match_like_matches_macro` (pre-existente de MEMG-05) → FIND-306; otra sesión lo estaba corrigiendo en vuelo al momento del cierre (no se toca); (b) `docs/user/operations/CONFIGURATION.md:78` tenía el link gating roto `../api/HTTP_API.md` (de `65144964`, MEMG-04) → fix colateral `../../api/HTTP_API.md` en commit `docs:` separado (desbloquea el gate docs para todos); (c) `docs/index.md`/`docs/api/index.md` regenerados incluyen la entrada del task file de VER-10 (estado real del árbol; su sesión lo commitea).
- **Regla nueva owner 2026-10-05 (pruebas pesadas serializadas):** `pwsh dev-tools/heavy-test-lock.ps1 acquire` → correr → `release`; TTL 45min; retry 60s si tomado. Usado en: RED, GREEN, clippy+lib suite (una sesión ajena lo retuvo ~5min — espera respetada).
- **Review P2-01 tier:** paths del diff = `src/sdk/**` → **Adversarial** (tabla pipeline-full L167) → `vanta-review` obligatorio (fork) o degraded con waiver.
- **Memoria de decisiones:** registrar semántica revocación (D3) + superficie (D1) en `campaign_memory(file="decisions")` al cierre.

## RESULTADO (§7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7 (S0-S6)
PROXIMO_STEP: ninguno
COMMIT_HASH: 97571f3f (+ 8a039119 docs)
ARCHIVOS: src/sdk/api/sharing.rs · src/sdk/api/sharing_tests.rs · src/sdk/api.rs · src/sdk/mod.rs · docs/api/SHARING.md · docs/dev/Backlog.md · docs/dev/tasks/MEMG-16.md · docs/user/operations/CONFIGURATION.md · docs/index.md · docs/api/index.md · llms.txt
VERIFY_CONTRATO: pasa (fmt 0 / clippy 0 / scoped 10-10 / lib 2359-2359 / docs gates 0)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | D pre-respondido por F0 (contrato manda superficie); sin stalls ni colaterales bloqueantes
SKILLS_CARGADAS: security-and-hardening · api-and-interface-design · rust-write-tests · incremental-implementation · documentation-skill (+ base auto: campaign-executor/progreso/ponytail) | SDP v3: campaign-executor, progreso, writing-guidelines, writing-plans, incremental-implementation, test-driven-development, context-engineering, source-driven-development
```
