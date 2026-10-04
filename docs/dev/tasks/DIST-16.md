---
title: "TASK DIST-16: `verify` de certificados vía MCP (VER-02, cierre del loop de evidencia)"
kind: task
description: "Tool MCP `memory_verify_certificate`: verifica certificados de purga VER-02 contra la DB viva (mismo SDK que el CLI), resultado tipado + paridad CLI, perfiles completos, docs re-baselineadas. WAL-chain (VER-01) → FIND-266"
---

# TASK DIST-16: `verify` de certificados vía MCP (VER-02, cierre del loop de evidencia)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 24, Wave F1)
- **Fuente:** plan Task 24 (bloque expandido F0) + Gate D (question owner 2026-10-04)
- **Esfuerzo:** 🟢 1d | **Appetite:** max 1d
- **Prioridad:** 🟡
- **Tipo:** MCP server (feature-add — superficie pública nueva)
- **Turns estimados:** 5-10
- **Creado:** 2026-10-04T21:45Z | **last-synced:** 2026-10-04T23:55Z
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 — scope/name/input/perfiles/output confirmados por Gate D (question 2026-10-04); SDK y CLI ya exponen el verify (código-real)
- **Pendientes (downhill):** 4 steps (RED → GREEN+counts → docs → cierre)
- **Campaign ID:** master-plan-0.9.0-20261004 · **campaign taskId:** `24`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `handle_tools_list` (`vantadb-mcp/src/handlers/tools.rs:81`) — todos los clientes MCP (`vanta-cli server --mcp`, launchers); `handle_tools_call` (`:1295`) — dispatch; `profile_allowed_tools` (`:1047`) — gate `tools/list` + `tools/call` (4 perfiles); tests `vantadb-mcp/tests/mcp_tests.rs` (conteos 79/37/36/20 + annotations). |
| Callees | `Embedded::verify_purge_certificate` (`src/sdk/api/memory.rs:987`) → `attestation::verify_certificate` (`src/attestation.rs:513`) → lecturas de `StorageEngine` (store/shred/HNSW/versions/namespace-index). Cero dependencias nuevas. |
| Implicaciones | **Contrato público:** tool MCP nueva (aditiva) en la familia memory; counts re-baselineados (80 listed / 86 defined; memory 21, agent 38, dev 37, full 80). Rust API sin cambios (`vantadb-mcp` es `publish=false`, binario). **Comportamiento:** solo dispatch nuevo; tools existentes intactas. **Performance:** no hot path — el verify es un re-scan de superficies acotadas (mismo costo que el CLI). **Migración:** ninguna (certificado JSON ya existente). **Docs:** `MCP.md` (tabla + counts + cross-link) + `AI_IDES.md` + `EXPERIMENTAL_FEATURES.md` + `Backlog-negocio.md` (DX-10 count) + FIND-266. |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, `scripts/install.ps1` (DX-12 en vuelo), `vantadb-python/**`, `vantadb-ts/**`, `vantadb-wasm/**` (DIST-15 en vuelo). WIP ajeno del árbol (`vantadb-node/Cargo.lock`, master plan, opencode.jsonc) no se stagea.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos/secciones funcionales):**
  - `vantadb-mcp/src/handlers/tools.rs` (4.3k líneas) — header annotations registry (`:19-76`), `handle_tools_list` (`:81-1044`: definiciones base + strictness `additionalProperties` `:1011-1017` + extensiones `:1022-1030` + filtro por perfil `:1032-1041`), `profile_allowed_tools` (`:1047-1254`: memory set compartido `:1052-1073`, Agent/Dev/Full), `absorbed_canonical` (`:1261-1277`), `profile_allows_call`/`tool_is_known` (`:1279-1292`), `handle_tools_call` (`:1295+`: gate de perfil `:1314-1320`, arm `memory_delete` `:1549-1584`, fallthrough `Tool not found` `:3165`). Punto de inserción: definición tras `memory_delete` (`:183`), arm tras `:1584`, entrada en `memory_tools` (`:1052`).
  - `vantadb-mcp/src/validation.rs` — helpers de respuesta: `serialize_content` `:386`, `text_content` `:393`, `structured_text_content` `:400`, `text_content_structured` `:409`, `error_content_vanta` `:506`, `error_content_mcp` `:515`, `json_value_type_name` `:485`, `validate_payload` `:84`. Todos `pub(crate)` ya importados vía `use crate::validation::*`.
  - `vantadb-mcp/src/error.rs` — `McpError::from_domain` (`:93-120`): `VANTADB_VALIDATION_ERROR` → JSON-RPC `-32009` + `data{code, retriable, hint?}`; `to_json` (`:123`).
  - `vantadb-mcp/src/config.rs` — `McpConfig` (`:67-123`), defaults (`:125-153`, `max_payload_length=1 MiB`), rustdoc de perfiles (`:6-35`, counts stale).
  - `src/sdk/api/memory.rs:950-995` — `delete_certified` (emite VER-02) y `verify_purge_certificate` (`:987`: parse → `verify_certificate`; caveat documentado `:984`: el certificado no está atado a una DB).
  - `src/attestation.rs:80-157,513-634` — `PurgeCertificate`/`PurgeCertificateVerification` (Serializable), `verify_certificate` (schema + hash + re-scan + veredicto residues-aware H-1), `chain_evidence` (`:148`: `verify_command: "vanta-cli verify"` — VER-01 citado, no duplicado).
  - `src/cli_handlers/crud.rs:554-607` — `cmd_certificate_verify` (paridad de referencia: acepta certificado crudo **o** envelope `{deleted, certificate}` `:561-570`, exit 0/1).
  - `src/cli.rs:472-483` — `CertificateCommand::Verify { file }`; `:196-201` — `verify` WAL (VER-01, fuera de scope → FIND-266).
  - `vantadb-mcp/tests/mcp_tests.rs` — setup (`:21-33`), `test_memory_delete_attest_emits_purge_certificate` (`:637-689`, patrón de test a espejar), `test_mcp_tools_list` (`:342-418`), annotations coverage (`:4656-4756`: 79 exacto), `test_mcp_tool_profiles` (`:4760-4940`: 79/37/36/20), agent default surface (`:5034-5050`: 37), gate de perfiles (`:4960-5029`).
  - `docs/api/MCP.md` — families/counts (`:196-212`), perfiles (`:231-256`), Core Tools (`:278`), Memory CRUD (`:280-292`), coverage note (`:650-652`).
- **Archivos referenciados hacia dentro (imports/dependencias):** el arm nuevo usa solo lo ya importado (`serde_json::Value/json`, `McpError`, `validation::*`, `vantadb::Embedded`); sin imports nuevos.
- **Referencias entrantes (grep/CodeGraph):** `handle_tools_list`/`handle_tools_call` consumidos por `src/server/mcp.rs` (stdio loop) y tests; `verify_purge_certificate` tiene 6 callers (CLI + tests `tests/certified_delete.rs`); la tool nueva es símbolo de wire, 0 referencias previas.
- **Veredicto impacto:** **BAJO-MEDIO** — cambio aditivo de superficie MCP; sin símbolos Rust públicos nuevos (`vantadb-mcp` no publica), sin formato on-disk, sin hot path. Riesgos mitigados: (1) scope → fijado en Gate D (certificados; WAL → FIND-266); (2) caveat engine-vivo → documentado en la descripción de la tool + MCP.md; (3) gate docs → MCP.md actualizado en el mismo commit + `validate-docs-coverage.ps1`.

## Contrato

"Tool MCP `memory_verify_certificate` disponible y con smoke verde (certificado válido/inválido → resultado tipado, no string), paridad con CLI (mismo veredicto para el mismo certificado: la tool reproduce exactamente el resultado del SDK `verify_purge_certificate` que el CLI usa, acepta el envelope `{deleted, certificate}` como el CLI); `MCP.md` actualizado + `validate-docs-coverage.ps1` exit 0; conteo de tools re-baselineado (80 listed / 86 defined; memory 21, agent 38, dev 37, full 80); WAL-chain (VER-01) fuera de scope → FIND-266."

## Spec (SDD — decisiones confirmadas por Gate D, question owner 2026-10-04)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Scope | A) **Solo certificados VER-02** + FIND-266 para el WAL (contrato del plan) / B) + WAL chain ahora (rompe appetite) / C) sin FIND | ✅ **A** (question, opción recomendada) — evidencia: plan Task 24 contrato; `cli.rs:196-201` vs `:474-485` |
| 2 | Nombre canónico | A) **`memory_verify_certificate`** — familia `memory_*`, pareja de `memory_delete attest:true`, sin ambigüedad con VER-01 / B) `certificate_verify` (espeja CLI, sin familia) / C) `verify` (ambiguo) | ✅ **A** (question) — evidencia: `memory_delete` ya emite `certificate` (`tools.rs:1549-1584`) |
| 3 | Input shape | A) **Objeto o string + unwrap envelope** (paridad CLI `crud.rs:563-569`) / B) solo objeto / C) solo string | ✅ **A** (question) |
| 4 | Perfiles | A) **Todos (memory set compartido)** — agent default puede emitir y verificar / B) solo full / C) full+dev | ✅ **A** (question) — evidencia: `memory_tools` `:1052-1073` incluido en los 4 perfiles |
| 5 | Output/errores | A) **Válido → `structuredContent {valid:true, verification:{...}}`; inválido → `isError:true` + envelope ERR-MCP-01 tipado** / B) siempre structuredContent con `valid:false` / C) texto plano | ✅ **A** (question) — evidencia: convención `error_content_vanta` (`validation.rs:506`) + `McpError::from_domain` (`error.rs:93`) |
| 6 | Guard de tamaño | A) **Reusar `validate_payload(max_payload_length=1 MiB)`** — consistente con `memory_put` / B) knob nuevo / C) sin guard | ✅ **A** — decidido-por-evidencia (`validation.rs:84`, `config.rs:129`); sin config nueva (ponytail) |
| 7 | Annotations | A) **readOnly true / destructive false / idempotent true / openWorld false** — verificación pura sin mutación / B) otras | ✅ **A** — decidido-por-evidencia: spec MCP 2025-06-18 (tool sin efectos persistentes) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `src/wal.rs`, `src/storage/`, `src/attestation.rs`, `src/sdk/api/memory.rs` NO se tocan (el verify del core es intocable; solo se consume).
  2. Tools existentes intactas: sin cambios de comportamiento ni de wire shape (solo adición + counts).
  3. El certificado se verifica contra la DB viva por namespace/key/node_id — no se ata a una instancia (caveat documentado, `memory.rs:984`).
  4. La cadena WAL (VER-01) se cita (`chain.verify_command`), no se re-verifica ni duplica (MCP.md §Injection governance).
  5. Sin símbolos públicos Rust nuevos (`public-api.txt` intacto; `vantadb-mcp` publish=false).
  6. Absorbed/aliases (8 nombres dispatch-only) intactos.
- **Comandos de verificación:** `cargo check -p vantadb-mcp` · `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` · `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `pwsh scripts/validate-docs-coverage.ps1` · `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check`.
- **Deuda pendiente:** FIND-266 (MCP verify del WAL chain VER-01) — registrado en Backlog en DISCOVERY.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** cero o negativo — cambio aditivo (tool nueva + tests + docs); no introduce `unsafe`, clones en hot path ni abstracciones especulativas. Sin deuda nueva; FIND-266 documenta un gap pre-existente (no introducido).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Tool `memory_verify_certificate` listada en los 4 perfiles + dispatch con resultado tipado (válido → `{valid:true, verification}`; inválido → `isError` + envelope ERR-MCP-01) + paridad SDK/CLI (misma función, veredicto idéntico) + envelope aceptado + tests RED→GREEN + fmt/clippy/nextest verdes + gates docs (coverage 0 gaps) |
| **Commit** | Commit atómico conventional `feat(mcp):` + `git diff` limpio + verificación mecánica (nunca auto-reporte) + solo archivos propios |
| **Release** | n/a directo — el commit alimenta el changelog de release-plz (feature → minor); push diferido al final del plan (decisión owner) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius tools/memory/attestation — usado) + `codebase-memory-mcp_check_index_coverage` (7 paths, sin gaps registrados) + grep (counts + callers)
- `cargo check -p vantadb-mcp` + `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` (loop TDD)
- `campaign_verify_cmd` (verify mecánico) + `pwsh dev-tools/ocr-review.ps1` (OCR delegation al cierre)

**Skills cargadas (SDP v3):** `api-and-interface-design` (pinned — diseño de la superficie: nombre/input/output/errores) · `documentation-and-adrs` (pinned — MCP.md/counts) · `source-driven-development` (spec MCP 2025-06-18 para annotations/structured output) · `security-and-hardening` (trust boundary: input JSON del cliente + guard de tamaño) · `incremental-implementation` (slices RED→GREEN) · `test-driven-development` (RED primero) · `documentation-skill` (edits en `docs/**` + gates). Base auto: campaign-executor, progreso, ponytail. SDP: 7 cargadas + base.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — trust boundary: input del cliente MCP (`certificate` JSON) → parse con serde tipado (`PurgeCertificate`), guard de tamaño (`validate_payload`, 1 MiB), errores tipados sin internals; sin secretos, sin auth, sin deps nuevas. Checklist `security-and-hardening`: validación en el borde ✅ (tipo + tamaño), salida tipada ✅, sin superficie de red nueva ✅ (stdio existente).
- [ ] **PERFORMANCE** — no aplica (Regla 9): el verify es read-only, invocado explícitamente por el cliente, mismo costo que el CLI existente; no toca hot paths (search/ingest/serialización).

## Steps

### Step 1 — RED: tests de la tool (válido/paridad + inválido tipado)

- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs`
- **Acción:** anexar tras `test_memory_delete_attest_emits_purge_certificate` (`:689`): (a) `test_mcp_memory_verify_certificate_valid_typed_and_parity` — put + delete attest → cert; veredicto SDK como referencia; MCP con objeto, envelope `{deleted, certificate}` y string → `structuredContent.valid=true` + `verification` idéntico al SDK; (b) `test_mcp_memory_verify_certificate_rejects_tampered_and_garbage` — namespace editado → `isError` + `data.code=VANTADB_VALIDATION_ERROR` + JSON-RPC `-32009`; string no-JSON → `isError` con código `VANTADB_*`; argumento ausente → `Err(-32602)`.
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests -E 'test(/verify_certificate/)'` → RED esperado (tool inexistente → fallthrough `Tool not found`)
- **Evidencia:** ✅ RED verificado en HEAD pre-fix: `cargo nextest run -p vantadb-mcp --test mcp_tests --ignore-default-filter -E "test(/verify_certificate/)"` → **2 tests FAILED** con `Tool not found: memory_verify_certificate` (code -32601) — razón correcta (tool inexistente). Nota: `mcp_tests` está excluido del default-filter de nextest (heavy-cert tier) → `--ignore-default-filter` es el flag local correcto.
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: tool + perfil + dispatch + counts

- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/config.rs`, `vantadb-mcp/tests/mcp_tests.rs`
- **Acción:** definición JSON (tras `memory_delete`), entrada en `memory_tools` (4 perfiles), arm de dispatch (tras `memory_delete`), header comment re-baselineado (86 defs / 48 base / 80 listed; hints 48/65/73/84); `config.rs` rustdoc counts; tests de counts (79→80, 37→38, 36→37, 20→21; + `memory_verify_certificate` en los loops de presencia).
- **Verify:** scoped `-E 'test(/verify_certificate/)'` + `-E 'test(/tool_annotations|tool_profiles|agent_default_surface|tools_list/)'` → GREEN; `cargo check -p vantadb-mcp` exit 0
- **Evidencia:** ✅ GREEN — scoped 11/11 (2 nuevos + counts) · `mcp_tests` completo **114/114** · paquete completo `cargo nextest run -p vantadb-mcp --ignore-default-filter` **260/260** · `cargo check -p vantadb-mcp` exit 0 · `cargo fmt --check` exit 0 (3 diffs de rustfmt corregidos con `cargo fmt -p vantadb-mcp`) · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` exit 0. Implementación: definición (`tools.rs` ~L184), `memory_tools` (~L1081, 4 perfiles), arm dispatch (~L1619), header counts (86/48/80, hints 48/65/73/84), `config.rs` rustdoc, tests counts 80/38/37/21 + presencia en 4 perfiles.
- **Estado:** ✅ COMPLETED

### Step 3 — Docs: MCP.md + counts vivos + gates

- **Archivos:** `docs/api/MCP.md`, `docs/user/AI_IDES.md`, `docs/user/operations/EXPERIMENTAL_FEATURES.md`, `docs/dev/Backlog-negocio.md`
- **Acción:** MCP.md — counts (80/86, Core 48, perfiles 38/80/37/21), fila `memory_verify_certificate` en Memory CRUD (10), cross-link en `memory_delete`, nota de última sincronización; AI_IDES `:54` 37→38/79→80; EXPERIMENTAL_FEATURES `:136` 85→86/47→48; DX-10 row count 79→80/85→86.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 · `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check` exit 0
- **Evidencia:** ✅ `validate-docs-coverage.ps1` → **vantadb-mcp (tools): 48 items ok en MCP.md · 0 gaps** · `check-links.mjs` exit 0 (0 broken markdown; 20 wikilinks budgeted pre-existentes) · `check-docs.mjs` exit 0 (GATING all clear) · `gen-index.mjs --check` exit 0 (sin regeneración). MCP.md: counts 80/86/Core 48/Memory CRUD 10/perfiles 38·80·37·21 + fila `memory_verify_certificate` + cross-link en `memory_delete` + nota de sync 2026-10-04. Extra: `AI_IDES.md:54` (38/80), `EXPERIMENTAL_FEATURES.md:136` (86/48, cita re-pointada a MCP.md:198), `Backlog-negocio.md` DX-10 (80/86).
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + review P2-01 + commit local

- **Archivos:** `docs/dev/tasks/DIST-16.md`
- **Acción:** `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + nextest (`-p vantadb-mcp` scoped + `--workspace` de cierre) + gates docs; OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`); review P2-01 (tier **Fast** — paths `vantadb-mcp/**` + `docs/**` no matchean adversarial; verify fast mecánico + veredicto registrado en §Review; fork a `vanta-review` como segundo par de ojos); commit **LOCAL** `feat(mcp): DIST-16 — ...` solo con archivos propios (nunca push).
- **Verify:** `git show --stat HEAD` limitado a archivos propios; veredicto en §Review
- **Evidencia:** ✅ `cargo fmt --check` 0 · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` 0 · `cargo nextest run -p vantadb-mcp --ignore-default-filter` **260/260** (post-fix incluido) · gates docs 0 gaps · OCR delegation Group 2 (server/MCP + Rust): revisión contra reglas — 0 Critical/High (sin unwrap/expect en producción, sin errores tragados, input validado tipo+tamaño, errores tipados) · review P2-01 adversarial `vanta-review` ✅ **APPROVE** (R1; fixes Low/Nit post-verdicto, ver §Review) · commit local `feat(mcp):` (hash en RESULTADO). Nota: `campaign_verify_cmd` con spawn roto en este entorno (exitCode -1 sin output — precedente ENC-01) → verificación mecánica por shell directo.
- **Estado:** ✅ COMPLETED

## Dependencias

- Ninguna bloqueante. DIST-15 (bindings, en vuelo) no toca `vantadb-mcp/**` — disjunto.
- `verify_purge_certificate` (VER-02) y `cmd_certificate_verify` (CLI) ya existen y están testeados (`tests/certified_delete.rs`).
- nextTask: lo decide el orquestador (DIST-17 PENDING).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Adversarial** (`docs/api/**` matchea HARD-02). Sesión reviewer: `ses_ef6cc10a7ffe6zQaHRYOkvdUpO`.
- **Enfoque:** superficie nueva (definición/annotations/schema, memory set → 4 perfiles, arm tras el gate de perfil), semántica input (objeto/envelope/string/null/tipo raro/cap), paridad CLI/SDK (mismo `verify_purge_certificate`; comparación byte-exacta vs SDK en test), errores tipados (tampered -32009 / garbage -32002 / missing -32602; sin string libre), counts re-baselineados (80/86/48/38/37/21) y scope (diff = exactamente 7 archivos; core `wal/storage/attestation/sdk memory.rs` intacto; absorbed/aliases intactos; sin símbolos públicos nuevos).
- **Cómo se probó:** el reviewer corrió por su cuenta: `pwsh scripts/validate-docs-coverage.ps1` → **48 items MCP / 0 gaps** · check-links/check-docs exit 0 · `cargo fmt --check -p vantadb-mcp` 0 · scoped 6/6 · **`cargo nextest run -p vantadb-mcp --ignore-default-filter` 260/260** · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` 0. Conteo directo de 48 names en el bloque base + revisión directa del arm (`:1619-1650`) y del gate (`:1339`).
- **Hallazgos:** R1 [Low] comentario stale `≤36 tools` (`tools.rs:1137`) → **cerrado** (`≤37`). R1 [Low] divergencia de aceptación vs CLI para el envelope **como string** (el branch String pasaba verbatim; el CLI desenvuelve cualquier representación, `crud.rs:563-570`) → **cerrado** con el fix de 3 líneas recomendado por el reviewer (unwrap en el branch String, espejo del CLI) + assert nuevo (Act 4, envelope-as-string) en `test_mcp_memory_verify_certificate_valid_typed_and_parity`. R1 [Nit] `tools.rs:38` "total 115 hits" contradictorio → **cerrado** ("517 hint-literal occurrences across src", medido con `rg -o`). R1 [Nit, pre-existente] comentario "(45) and false (31)" en `mcp_tests.rs:4913` — **no tocado** (pre-existente, fuera de scope; el assert real es dinámico ≥40).
- **Veredicto:** ✅ **APPROVE** (R1) — contrato verificado mecánicamente por el reviewer; 0 Critical/High/Medium. Fixes Low/Nit incorporados **post-verdicto** con el código exacto recomendado por el reviewer (transparencia, precedente ENC-01 §Review) + re-verificación completa tras el fix (fmt 0 · clippy 0 · scoped 2/2 · paquete **260/260**).

## Notas

- **Gate D:** disparado (superficie pública nueva — tool MCP). Question batched 2026-10-04 → 5 decisiones confirmadas (scope/name/input/perfiles/output). Registro en §Spec.
- **Caveat engine-vivo:** el certificado no está atado a una DB; verificar contra una DB distinta puede dar `residues`/inconsistencia — documentado en la descripción de la tool y MCP.md.
- **FIND-266:** WAL-chain verify (VER-01) fuera de scope, registrado en Backlog en DISCOVERY.
- WIP ajeno en el árbol (master plan, opencode.jsonc, `vantadb-node/Cargo.lock`) no se stagea.

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: <pendiente — commit `feat(mcp): DIST-16` (hash en la recitation; se actualiza en el commit de cierre docs(task))>
ARCHIVOS: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/src/config.rs, vantadb-mcp/tests/mcp_tests.rs, docs/api/MCP.md, docs/user/AI_IDES.md, docs/user/operations/EXPERIMENTAL_FEATURES.md, docs/dev/Backlog-negocio.md, docs/dev/tasks/DIST-16.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:disparado V:no C:no | D:superficie pública nueva → question 5/5 confirmadas
SKILLS_CARGADAS: api-and-interface-design, documentation-and-adrs, source-driven-development, security-and-hardening, incremental-implementation, test-driven-development, documentation-skill (base auto: campaign-executor, progreso, ponytail)
```
