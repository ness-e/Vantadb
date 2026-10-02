---
title: "SCH-05: Cuarentena operativa + abstención + trust-aware retrieval (slice 0.8.0)"
kind: task
description: Matriz de cierre — default-exclude, gates de inyección, transiciones T1/T1c/T1d/T2/T4, abstención con señal en el wire
---

# SCH-05: Cuarentena operativa + abstención + trust-aware retrieval (slice 0.8.0)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 30 (F3) · **Origen:** plan L787-811; ADR-0046 §D2/§D5/§D5d/§D5e; MGR-13 §3/§4/§5/§7
- **Fuente del prompt:** sub-agente vanta-worker (orquestador pipeline) — wave F3.3b (única en vuelo); branch `develop`
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** feature-add (estado operativo + wire + gates de seguridad)
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ⏳ IN PROGRESS — implementación + verify mecánico ✅ + **batch post-review (F1–F6/N1–N3) ✅**; pendiente **re-review P2-01 + commit (LEAD)**
- **Incógnitas (uphill):** 0 · **Pendientes (downhill):** 0 steps de ejecución (7/7 ✅)

## Contrato (verbatim del prompt de tarea)
> "cuarentena operativa: contenido `quarantined` EXCLUIDO por defecto de search/list/retrieval (include opt-in; `auto_recall`/`inject_context` nunca inyectan cuarentenado) Y transiciones con dueño+trigger (entrada write-time, promoción/expiración) Y abstención selectiva: con umbral de confianza configurado, una consulta sin candidatos suficientes devuelve señal `abstained` explícita en el wire (nunca resultados silenciosamente degradados; default OFF) Y retrieval trust-aware respeta la clase asserted/derived (semántica ADR SCH-01) Y test de contención verde (dudoso no inyectado por defecto) + threat model de MGR-13 citado por superficie"

**Cláusulas a verificar (matriz de cierre):**

| # | Cláusula | Superficie | Evidencia esperada |
|---|----------|-----------|--------------------|
| C1 | Default-exclude: `quarantined` fuera de search/list; `include_quarantined` opt-in; `get` devuelve con estado visible | `page.rs`/`namespaces.rs`/`memory.rs` | tests `quarantine_containment` (search + list + get + opt-in) |
| C2 | Gates duros: `auto_recall`/`memory_recall`/`context_assemble` nunca inyectan cuarentenado | `l1_reader.rs` (choke point) + vanta-memory test | test vanta-memory `recall` (L1 quarantined no aparece) |
| C3 | Transiciones T1 (flag write-time), T1c (import opt-in), T1d (apply), T2 (promote), T4 (reject) con dueño+trigger; T3 deadline 30d default `keep` (nunca auto-promoción, I1); sticky (I2) | `memory.rs`/`impl_export.rs`/`config.rs` | tests transiciones + sticky + deadline |
| C4 | Abstención: con `confidence_threshold` configurado y 0 candidatos → `abstained: true` + `abstention_reason` (`no_candidates_above_threshold`/`all_quarantined`); default OFF | `page.rs`/`vector_types.rs`/`config.rs` | test señal + default OFF no cambia comportamiento |
| C5 | Trust-aware: `min_confidence` (SCH-04) aplica junto al default-exclude; clase asserted/derived respetada por el score (ADR §D4) | `page.rs` | test combinado min_confidence+quarantine |
| C6 | Threat model MGR-13 citado por superficie (API/dream/import/MCP) | task file + comentarios de gate | §Threat model abajo + tests por entrada |

**Alcance explícito:** core Rust (SDK + config + ops) + gate de vanta-memory + MCP `memory_put`/`memory_put_batch` (flag T1, MGR-13 §4.4). **Superficies restantes** (HTTP response `SearchPageV2.abstained`, MCP args de import, bindings Py/TS/Node/WASM, CLI flags, docs/api) → **SCH-07** (plan Task 32).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Edita** | `src/sdk/types/record.rs` (`MemoryInput.quarantine`, `MemoryListOptions.include_quarantined`, `ImportReport.quarantined`) · `src/sdk/serialization/vector_types.rs` (`MemorySearchRequest.include_quarantined`, `AbstentionReason`, `MemorySearchPage.abstained/abstention_reason`) · `src/sdk/search/page.rs` (filtro default + fingerprint + abstención) · `src/sdk/api/namespaces.rs` (list) · `src/sdk/api/memory.rs` (T1 + ops T1d/T2/T4 + bulk) · `src/sdk/serialization/impl_export.rs` (T1c) · `src/config.rs` (2 campos) · `vanta-memory/src/core/record/l1_reader.rs` (gate) · `vantadb-mcp/src/handlers/tools.rs` (flag T1 en `memory_put`/`memory_put_batch`) |
| **Callers** | `MemorySearchRequest`/`MemoryListOptions` = literales workspace (bindings py/ts/node/wasm, providers, cli, server, desktop, tests) → fix mecánico guiado por `cargo check`; `import_records`/`import_file` (cli, server, mcp, wasm, py, tests) → param nuevo `quarantine`; `MemorySearchPage` (page.rs ×2, tests); `ImportReport` (5 literales) |
| **Callees** | `plan_fingerprint` (hash nuevo) · `record_from_node`/`memory_record_to_node_owned` (SCH-02, campos ya mapeados) · `AuditEvent` (ops nuevas) · config (`quarantine_review_default_days`, `confidence_threshold`) |
| **Implicaciones** | Aditivo serde (`#[serde(default)]`, defaults = comportamiento actual); cambio observable **solo** para registros que 0.8.0 marque (cero efecto legacy — MGR-13 §2.4); cursor de otro `include_quarantined` ⇒ `SEARCH_CURSOR_INVALID`; firma de `import_records`/`import_file` cambia (breaking pre-launch 0.8.0, ADR §D1b — call sites actualizados en este PR); ranking sin cambios (filtros post-ranking) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/types/record.rs` (979L) · `src/sdk/serialization/vector_types.rs` (684L) · `src/sdk/search/page.rs` (426L) · `src/sdk/api/memory.rs` (1258L) · `src/sdk/api/namespaces.rs` (397L) · `src/sdk/serialization/impl_export.rs` (763L) · `src/config.rs` (2393L) · `vanta-memory/src/core/record/l1_reader.rs` (317L) · `src/audit.rs` (:1-110)
- **Archivos leídos (rangos clave — archivos gigantes, precedente SCH-04):** `vantadb-mcp/src/handlers/tools.rs` (:29-36 lista readOnly, :80-174 schemas put/batch, :560-660 schemas, :1033-1150 dispatch, :1303-1452 handler put, :1755-1864 memory_recall, :2025-2110 inject_context, :3101-3170 parse_memory_input) · `vanta-memory/src/core/hooks/auto_recall.rs` (:150-449) · `src/sdk/search/mod.rs` (:1-220) · `src/sdk/search/multi.rs` (92L) · `src/sdk/version_history.rs` (:280-389) · `src/server/handlers.rs` (:235-267, :495-577) · `vantadb-mcp/src/context.rs` (159L) · `vanta-memory/src/core/dream/mod.rs` (:484-633)
- **Referencias hacia dentro:** ADR-0046 §D2 (wire), §D5 (estado/transiciones/sticky/gates), §D5d (deadline 30d/keep), §D5e (import opt-in uniforme); MGR-13 §3.2 (T1-T4), §3.3 (I1-I6), §5.1/§5.2/§5.3 (retrieval/abstención); plan Task 30.
- **Referencias entrantes:** SCH-06 (chaos: TTL+quarantine, supersede+quarantine, migración con cuarentenados) · SCH-07 (superficies + docs/api) · VER-01 (audit hash-chain, cita) · MGR-04 (ABAC, FIND).
- **Veredicto impacto:** medio-alto — aditivo en wire, pero toca literales de 3 structs públicos + firmas de import en todo el workspace; sin cambio de ranking; gates de seguridad en un choke point de lectura (`l1_reader`). Edits quirúrgicos; sin co-batch activo (SCH-03/04 commiteados).

## Spec (feature-add — decisiones por evidencia, ADR-0046 firmado)

| # | Decisión | Alternativas | Elegido | Evidencia |
|---|----------|--------------|---------|-----------|
| 1 | Semántica default-exclude | (a) filtro en `list`/`run_search_page` junto a `exclude_superseded` (post-ranking) / (b) filtro en el índice (pre-ranking) | (a) | ADR §D5 + SCH-04 precedente (`page.rs:344`); cero cambio de índice/hot path |
| 2 | `get` por key | (a) devuelve con estado visible / (b) 404 | (a) | Q5 ratificada (ADR §D5, MGR-13 §5.1) — sin cambio de código, solo test |
| 3 | Sticky (I2) | (a) put conserva cuarentena / (b) flag lo limpia | (a) | Q4 ratificada; `QuarantineState` ya en `put_one`/`put_batch_inner` (SCH-02); falta T1 (entrada) |
| 4 | Ops de transición | (a) métodos `Embedded::quarantine_apply/promote/reject` / (b) dejar sin superficie hasta SCH-07 | (a) | MGR-13 §7-3 "ops T1d/T2/T4 + audit" = contrato SCH-05; superficies (HTTP/MCP/CLI) = SCH-07 |
| 5 | `quarantined_by` sin principal | (a) `Some("system:<op>")` / (b) `None` | (a) | MGR-13 §2.2 "o `system:<op>` (ej. `system:dream_promote`)"; principal real no existe en embedded (MGR-04 fuera del corte) |
| 6 | T3 expiración | (a) deadline 30d + `keep` (sin acción automática) / (b) purge automático | (a) | D5d firmada (I1: nunca auto-promoción); `purge` opt-in = T4 |
| 7 | Forma de la señal de abstención | (a) `abstained: bool` + `abstention_reason: Option<AbstentionReason>` en `MemorySearchPage` / (b) solo menos resultados (rechazado por contrato) | (a) | ADR §D2 verbatim (códigos estables snake_case); enum `#[non_exhaustive]` con `rename_all` |
| 8 | Disparador de abstención | (a) config `confidence_threshold` (None=OFF) filtra y abstiene si 0 quedan / (b) por request | (a) | ADR §D2: "distinto del filtro por request `min_confidence`, que no dispara abstención" |
| 9 | Motivo `all_quarantined` | (a) si el candidato-set previo quedó vacío por el filtro de cuarentena / (b) si todo hit individual era cuarentenado | (a) | MGR-13 §5.3 códigos; semántica más informativa (aislado ≠ bajo umbral) |
| 10 | T1c import | (a) param `quarantine: bool` en `import_records`/`import_file` (firma cambia, breaking 0.8.0) / (b) método nuevo dual | (a) | MGR-13 §2.2 ops "import_* (opción quarantine: bool)"; One-Version (ADR §D2): nunca API dual |
| 11 | T1 flag | (a) `MemoryInput.quarantine: bool` serde default (aditivo) / (b) metadata libre | (a) | ADR §D2 verbatim; metadata no es contractual (MGR-12 tradeoff 7) |
| 12 | Gates de inyección | (a) filtro en el choke point `read_namespace_records` (todas las lecturas L1 de auto_recall pasan ahí) / (b) filtrar en cada caller | (a) | `perform_auto_recall:212-221` + `read_scoped_records:301` + `context_assemble` (MCP) usan `read_namespace_records`; 1 solo punto |
| 13 | `inject_context` | (a) sin cambio de código (write a thread L0, no lee L1) + contención en cascada documentada / (b) filtrar algo | (a) | tools.rs:2080-2085 escribe `INSERT MESSAGE ... TO THREAD#`; sin read path L1 (MGR-13 §5.2 "su propio write sigue T1"; cascade = T1b/MEM-65) |
| 14 | T1b dream | (a) diferir con MEM-65 (merge real stubbed, `promote_dream_run:615-623` no muta L1) + citar el checkpoint / (b) implementar gate sobre un path que no muta | (a) | `dream/mod.rs:605-623` STUB + MGR-13 §4.2 "cuando MEM-65 conecte el merge real"; deuda/FIND |
| 15 | Alcance MCP | (a) flag T1 en `memory_put`/`memory_put_batch` (gate write-time agent-facing) / (b) todo MCP a SCH-07 | (a) | MGR-13 §4.4 "el tool `memory_put` acepta `quarantine`"; resto de args/import → SCH-07 |
| 16 | `count`/`delete_by_filter` | (a) internos pasan `include_quarantined: true` (semántica actual intacta; los cuarentenados siguen contables/borrables) / (b) excluir también | (a) | Contrato limita default-exclude a "search/list/retrieval"; delete debe poder tocar cuarentenados (I4/T4) |

## Invariantes de dominio (handoff — MUST)
- **Aditivo puro en wire:** todo campo nuevo `#[serde(default)]`; defaults = comportamiento actual. Cero efecto legacy (MGR-13 §2.4: ningún registro pre-0.8.0 puede estar cuarentenado).
- **I1 — nunca auto-promoción:** ni TTL ni deadline promueven; solo `quarantine_promote` (explícito).
- **I2 — sticky:** `put`/`put_batch`/bulk sobre key cuarentenada conserva el estado; solo T2/T4 salen.
- **I3 — ortogonalidad:** `quarantined_*` independiente de `superseded_*`/`expires_at_ms`.
- **No tocar ranking** (`fusion.rs`, orden, `mmr`, `group_by`) ni `MemoryRecord` (solo lectura).
- **Gates duros:** `auto_recall`/`memory_recall`/`context_assemble` NO inyectan cuarentenado aunque el request pida `include_quarantined` (no hay opt-in en inyección).
- **PROHIBIDOS:** `docs/**` (salvo este task file), `Backlog.md`, `perf-bench.yml`, `opencode.jsonc`, plan file (LEAD). No commit, no self-review (LEAD).
- Regla dura `-p` + `CARGO_BUILD_JOBS=2`.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: 0.** Sin `unsafe` nuevo, sin dependencias nuevas, sin hot path nuevo (retain O(n) sobre páginas ya materializadas, espejo `exclude_superseded`/`min_confidence`). Deudas diferidas (no nuevas, ya citadas): T1b con MEM-65; T5 re-validación y métrica agregada `quarantine_overdue` → v1.0/SCH-07; hash-chain → VER-01.

## Definition of Done
- **Task:** contrato ✅ (matriz C1-C6) + fmt/clippy/nextest verdes + tests del cambio.
- **Commit:** atómico, conventional (`feat:`), verificación mecánica — **LEAD**.
- **Release:** N/A (wave; `--workspace` = cierre de tarea acotado a crates tocadas).

## Threat model MGR-13 citado por superficie (write-time)
| Superficie | Vector (MGR-13 §4) | Mitigación en este slice |
|---|---|---|
| API HTTP / SDK (A) | Write autorizado con contenido no verificado | T1 `MemoryInput.quarantine` (serde auto en HTTP `records_put`); T1d `quarantine_apply` post-hoc; default-exclude |
| Dream (B) | Deep-poisoning vía promoción derivada | T1b diferido con MEM-65 (stub no muta L1, `dream/mod.rs:605-623`); checkpoint citado; residuo documentado |
| Import (C) | Supply chain: archivo malicioso/export comprometido | T1c `quarantine=true` opt-in (firma nueva) + `ImportReport.quarantined` + audit; roundtrip preserva estado |
| MCP agent-facing (D) | Prompt injection indirecta → writes legítimos | Flag T1 en `memory_put`/`memory_put_batch`; `memory_recall` hereda el gate de `perform_auto_recall`; `inject_context` = write L0 (cascada T1b) |
| Retrieval (todos) | Inyección a prompt de contenido aislado | Gates duros en `l1_reader` (choke point) + test de contención |

## Steps

### Step 1: RED — test de contención (campos/ops no existen ⇒ falla de compilación controlada)
- **Archivos:** `tests/quarantine_containment.rs` (nuevo)
- **Estado:** ✅ RED capturado — 21 errores de compilación (E0609 `abstained`/`abstention_reason`; E0560/E0063 campos; E0599/E0061 ops; E0432 `AbstentionReason` no exportado)

### Step 2: Wire + config (aditivo, mecánico)
- **Archivos:** `src/sdk/types/record.rs`, `src/sdk/serialization/vector_types.rs`, `src/config.rs`, exports (`sdk/types/search.rs`, `sdk/mod.rs`, `lib.rs`)
- **Estado:** ✅ `cargo check -p vantadb` (tras fixes de literales); `public-api.txt` regen deliberada (+72/-4)

### Step 3: GREEN — filtros + ops + abstención + import T1c
- **Archivos:** `page.rs`, `namespaces.rs`, `memory.rs`, `impl_export.rs`
- **Estado:** ✅ **20/20** tests `quarantine_containment` (default-exclude list/search · opt-in · get visible · sticky · T1d/T2/T4 · deadline 30d/0 · I1 · T1c import/roundtrip · bulk flag · abstención ×4 · min_confidence+quarantine · audit transiciones · fingerprint cursor) — **ampliado a 24/24 en el batch post-review (ver §Batch post-review P2-01)**

### Step 4: Literales/firmas workspace + bindings (fix mecánico)
- **Estado:** ✅ `cargo check --workspace --all-targets` clean · wasm32 clean · python/node/server (manifest-path) clean · 9 snapshots `search_request_*` regen deliberada (+1 línea c/u)

### Step 5: Gate de inyección vanta-memory + test de recall
- **Archivos:** `vanta-memory/src/core/record/l1_reader.rs`, `vanta-memory/tests/recall.rs`
- **Estado:** ✅ nuevo test `quarantined_l1_records_are_never_recalled`; recall **16/16**; suite vanta-memory **550/550**

### Step 6: MCP — flag T1 en memory_put/memory_put_batch
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (schema ×2 + handler + `parse_optional_bool`)
- **Estado:** ✅ test `test_mcp_memory_put_quarantine_flag_isolates_record` PASS; suite MCP completa **244/244**

### Step 7: Verify full + cierre
- **Estado:** ✅ mecánico (ver §Verificación final) — **pendiente review P2-01 + commit (LEAD)**

## Verificación final (2026-09-29 — evidencia mecánica)

| Comando | Resultado |
|---|---|
| `cargo nextest run -p vantadb --test quarantine_containment` | ✅ **20/20** (RED→GREEN documentado) |
| `cargo nextest run --profile audit -p vantadb --build-jobs 2 --no-fail-fast` | ✅ **2458/2458** (2 skipped; 393s) — incluye `public_api` re-snapshot y snapshots regen |
| `cargo nextest run -p vanta-memory` (completa) | ✅ **550/550** |
| `cargo nextest run -p vantadb-mcp --ignore-default-filter` (completa) | ✅ **244/244** |
| `cargo nextest run -p vantadb-server -E 'binary(e2e)' --ignore-default-filter -j 2` | ✅ **19/19** (default parallelism = contención de recursos de la máquina; `-j 2` estable) |
| `cargo check --workspace --all-targets` | ✅ exit 0 |
| `cargo check -p vantadb-wasm --target wasm32-unknown-unknown --all-targets` | ✅ exit 0 |
| `cargo check` python/node/server (manifest-path) | ✅ exit 0 |
| `cargo clippy -p vantadb --all-targets -- -D warnings` (comando REAL del gate; re-corrido en batch post-review) | ✅ exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` (forma CI; batch post-review) | ✅ exit 0 |
| `cargo clippy` (vanta-memory · vantadb-mcp · server · wasm · python · node, `-D warnings`, `--all-targets`) | ✅ 0 warnings |
| `cargo fmt --all -- --check` (+ python/wasm manifest) | ✅ 0 diffs |
| `cargo test --doc -p vantadb` | ✅ 13 passed · 1 ignored (pre-existente) |
| `tests/api/public-api.txt` | ✅ regenerado y verificado (solo símbolos SCH-05; firmas import con `bool`) |
| `pwsh scripts/validate-docs-coverage.ps1` | ⚠️ 2 gaps docs (`confidence_threshold`, `quarantine_review_default_days` en `docs/api/CONFIGURATION.md`) — **diferido SCH-07 por WIP ajeno (prohibido editar docs/**)**; resto 0 gaps |
| OCR advisory (`dev-tools/ocr-review.ps1 -Format json`) | ✅ spec generado; pase cognitivo acotado a archivos de la task: sin Critical/High (sin `unsafe`/`unwrap` nuevos en producción; lock pattern preexistente REVIEW-13; validación de reason code + trust boundary MCP tipada) |

**Contrato (matriz):** C1 ✅ · C2 ✅ · C3 ✅ · C4 ✅ · C5 ✅ · C6 ✅ (threat model por superficie arriba).

### Batch post-review P2-01 (2026-09-29) — evidencia por finding

| # | Fix | Evidencia (comando → resultado) |
|---|-----|---------------------------------|
| F1 | `tests/quarantine_containment.rs:534` sin borrow redundante (`format!` pasado directo) | `cargo clippy -p vantadb --all-targets -- -D warnings` → **exit 0**; forma CI `cargo clippy --workspace --all-targets --all-features -- -D warnings` → **exit 0** (FIND-184 no bloqueó: no se usó el scoped multi-crate combinado) |
| F2 | `resolve_axioms(storage, include_quarantined)`: `read_axioms`→`false`; next-id de `write_axiom`→`true` | test nuevo `test_mcp_quarantined_axiom_hidden_but_counts_for_next_id` → PASS; suite MCP **245/245** |
| F3 | `quarantine_enter` por registro en `put_batch` (post-commit) + audit en `import_records` directo + `bulk_import` + `BulkImportReport.quarantined` | tests `batch_import_and_bulk_quarantine_entries_are_audited` y `bulk_import_honors_the_quarantine_write_flag` (contador) → **24/24** |
| F4 | Sticky en transporte raw: `put_record_exact` preserva cuarentena existente; `bulk_import_stream` idem (lectura metadata-only) | tests `import_plain_over_quarantined_key_preserves_state` + `bulk_plain_over_quarantined_key_preserves_state` → **24/24**; wording del task file corregido (§Pendientes) |
| F5 | Doc-only: señal `abstained` solo en el wire SDK; HTTP/MCP/bindings no la propagan | nota en `src/config.rs` (`confidence_threshold`) + §Pendientes (no habilitar en server/MCP hasta SCH-07) |
| F6 | `quarantine_reject` toma `supersede_lock` y re-chequea bajo guard | `quarantine_reject_deletes_record_and_is_auditable_transition` → **24/24** |
| N1 | `ImportReport.quarantined` cuenta solo puts persistidos | `import_quarantined_count_only_includes_persisted_records` → **24/24** |
| N2 | resuelto por F3 | — |
| N3 | doc: set de reason codes abierto (formato lowercase snake validado) | `src/sdk/types/record.rs` doc |
| — | Re-snapshot `public-api.txt` (`BulkImportReport::quarantined`, campo público) | `VANTADB_PUBLIC_API_UPDATE=1 … public_api` → PASS; luego full |
| — | Re-verificación full | `cargo nextest run --profile audit -p vantadb --build-jobs 2 --no-fail-fast` → **2464/2464** (2 skipped, 272s); `-E 'test(quarantine) or test(abstain) or test(axiom)'` → 23/23; MCP 245/245; `cargo fmt --all -- --check` → 0 diffs; `validate-docs-coverage.ps1` → solo los 2 gaps declarados (`confidence_threshold`, `quarantine_review_default_days` → SCH-07) |

## Pendientes (§Pendientes)
- **docs diferidas por WIP ajeno → SCH-07:** `docs/api/` (`CONFIGURATION.md` 2 campos, EMBEDDED_SDK/MCP/HTTP_API/scores…) + `llms.txt` con cambios de otra sesión SIN COMMITEAR — NO editar (instrucción del orquestador). Doc de cuarentena/abstención (Regla 3) se entrega en SCH-07/LEAD.
- Superficies restantes → SCH-07: `SearchPageV2.abstained` (HTTP), args de import (MCP `import` tool/HTTP/CLI), bindings Py/TS/Node/WASM (getters/knobs/stubs), `quarantine_*` ops en HTTP/MCP, `min_confidence` en `MemoryListOptions`, `include_quarantined` en MCP list/search args.
- T1b (promoción derivada de dream default ON) → MEM-65 (merge real; hoy stub no muta L1).
- Métrica agregada `quarantine_overdue` → SCH-07/v1.0 (señal base = campo `quarantine_review_due_ms`).
- **F4 aplicado (batch post-review):** sticky también en el transporte raw — `put_record_exact` preserva la cuarentena existente cuando el registro entrante no trae estado, y `bulk_import_stream` la preserva (lectura metadata-only por registro; cierra el bypass de I2 vía `import`/bulk). **Nota ADR-0046 §D5e (enmienda menor: sticky en raw transport) → registrar en SCH-07/cierre de ADR** (no editar el ADR ahora: `docs/**` prohibido por WIP ajeno).
- **F5 aplicado (doc-only):** `abstained`/`abstention_reason` solo viajan por el wire del SDK; HTTP `SearchPageV2`, MCP y bindings no propagan la señal aún → **no habilitar `VANTADB_CONFIDENCE_THRESHOLD` en server/MCP hasta SCH-07** (nota en `config.rs`).
- Hallazgo entorno (no-FIND sin fila, anotado): e2e server con paralelismo default falla por contención de recursos en máquina cargada; `-j 2` estable — no es regresión de este diff.
- FIND candidato: `inject_context` L0 aislado = v1.0 (MGR-13 §4.4 residual; solapa EXE-07).

## Dependencias
- **Consume:** SCH-02 ✅ (`7af34366`; campos + sticky base) · SCH-03 ✅ (`932b1211`; AS OF) · SCH-04 ✅ (`932b1211`; `min_confidence` + fingerprint) · ADR-0046 `accepted` ✅ · MGR-13 ✅ (`72f29720`).
- **Bloquea:** SCH-06 (tests chaos: TTL+quarantine, supersede+quarantine, contención). **nextTask:** SCH-06.

## Herramientas
- codegraph (index FROZEN por lock de otro proceso → lectura directa documentada) · nextest scoped `-p` + `CARGO_BUILD_JOBS=2`.
- **SDP (v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · pin `security-and-hardening` (trust boundary write-time/gates) · `test-driven-development` (pin policy) · `source-driven-development` · `systematic-debugging` (pin policy) · `incremental-implementation` · `context-engineering` · `documentation-skill` (task file).

## Progreso

| Step | Estado | Evidencia |
|------|--------|-----------|
| 1 RED test contención | ✅ | 21 errores de compilación (API inexistente) |
| 2 Wire + config | ✅ | `cargo check -p vantadb`; exports + public-api regen |
| 3 GREEN filtros/ops/abstención/import | ✅ | `quarantine_containment` 20/20 |
| 4 Literales workspace | ✅ | workspace/wasm32/python/node/server check clean; 9 snaps regen |
| 5 Gate vanta-memory | ✅ | recall 16/16; vanta-memory 550/550 |
| 6 MCP flag T1 | ✅ | test MCP PASS; MCP 244/244 |
| 7 Verify full | ✅ | full core 2458/2458; fmt/clippy/doc-test verdes |

## Review (GATE — agente distinto, P2-01; tier adversarial por `src/sdk/**`)
> **Revisor:** `ses_f13840ac8ffemmiGx03HvEyn17` (fresco ≠ autor `ses_f14231baaffeN8BlPkEGhshYcr`) — ronda 1 **❌ CHANGES REQUIRED** (F1 Critical clippy; F2/F3 Required; F4-F6 Optional; N1-N3) → batch aplicado → **delta ✅** (clippy exit 0; containment 24/24; axiom test PASS; sin regresión en lo aprobado). Residuos declarados: full suites del worker (2464/2464 · MCP 245/245 · vanta-memory 550/550); nit de trazabilidad import-refresh → SCH-06/v1.0.
- **Ronda 1 (2026-09-29): ❌ CHANGES REQUIRED** — F1 Critical + F2/F3 Required + F4–F6 Optional + N1–N3. **Batch completo aplicado** (evidencia por finding en §Batch post-review P2-01).
- **Estado:** ⏳ pendiente **re-review P2-01 (contexto fresco)** + commit (LEAD). Diff sigue tocando `src/sdk/**` → tier adversarial.
- **Insumos para el revisor:** §Batch post-review (comando → resultado por finding), full **2464/2464**, MCP **245/245**, clippy scoped + CI-form exit 0, `public-api.txt` re-snapshot (`BulkImportReport::quarantined`).

## Context Save Point
- **Discovery ✅ (2026-09-29):** gap real = operacionalizar (filtros + gates + ops + abstención + config + import); sticky T1 base ya existía de SCH-02. Task file creado. Gate D: símbolos públicos nuevos pre-aprobados por ADR-0046 (firma owner 2026-09-28) + contrato verbatim del plan.
- **Implementación completa (2026-09-29):** Steps 1-7 ✅ con verificación mecánica por crate (tabla §Verificación final). Red/Green por test (RED compilación controlada; GREEN 20/20).
- **Batch post-review ✅ (2026-09-29):** F1–F6 + N1–N3 aplicados (evidencia por finding en §Batch post-review); tests `quarantine_containment` 24/24, MCP 245/245, full core 2464/2464, clippy scoped + CI-form exit 0, `public-api.txt` re-snapshot.
- **Pendiente (LEAD):** re-review P2-01 adversarial (contexto fresco) + commit local (nada de push). **Docs/api diferidas** (WIP ajeno) → SCH-07; nota F4 (sticky raw transport) → enmienda menor ADR-0046 §D5e a registrar en SCH-07/cierre. `nextTask`: SCH-06.
