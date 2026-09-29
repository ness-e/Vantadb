---
title: "SCH-05: Cuarentena operativa + abstención + trust-aware retrieval (slice 0.8.0)"
kind: task
description: Matriz de cierre — default-exclude, gates de inyección, transiciones T1/T1c/T1d/T2/T4, abstención con señal en el wire
---

# SCH-05: Cuarentena operativa + abstención + trust-aware retrieval (slice 0.8.0)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 30 (F3) · **Origen:** plan L787-811; ADR-046 §D2/§D5/§D5d/§D5e; MGR-13 §3/§4/§5/§7
- **Fuente del prompt:** sub-agente vanta-worker (orquestador pipeline) — wave F3.3b (única en vuelo); branch `develop`
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** feature-add (estado operativo + wire + gates de seguridad)
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ⏳ IN PROGRESS · **Incógnitas (uphill):** 0 · **Pendientes (downhill):** steps abajo

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
- **Referencias hacia dentro:** ADR-046 §D2 (wire), §D5 (estado/transiciones/sticky/gates), §D5d (deadline 30d/keep), §D5e (import opt-in uniforme); MGR-13 §3.2 (T1-T4), §3.3 (I1-I6), §5.1/§5.2/§5.3 (retrieval/abstención); plan Task 30.
- **Referencias entrantes:** SCH-06 (chaos: TTL+quarantine, supersede+quarantine, migración con cuarentenados) · SCH-07 (superficies + docs/api) · VER-01 (audit hash-chain, cita) · MGR-04 (ABAC, FIND).
- **Veredicto impacto:** medio-alto — aditivo en wire, pero toca literales de 3 structs públicos + firmas de import en todo el workspace; sin cambio de ranking; gates de seguridad en un choke point de lectura (`l1_reader`). Edits quirúrgicos; sin co-batch activo (SCH-03/04 commiteados).

## Spec (feature-add — decisiones por evidencia, ADR-046 firmado)

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
- **Acción:** escribir los tests núcleo (C1-C4) que usan `quarantine`, `include_quarantined`, `quarantine_apply/promote/reject`, `abstained`
- **Verify:** `cargo nextest run -p vantadb --test quarantine_containment` → falla (API inexistente = RED documentado)
- **Estado:** ⬜ PENDING

### Step 2: Wire + config (aditivo, mecánico)
- **Archivos:** `src/sdk/types/record.rs`, `src/sdk/serialization/vector_types.rs`, `src/config.rs`
- **Acción:** `MemoryInput.quarantine`; `MemoryListOptions.include_quarantined`; `MemorySearchRequest.include_quarantined`; `AbstentionReason` + `MemorySearchPage.abstained/abstention_reason`; `ImportReport.quarantined`; config `quarantine_review_default_days` (30) + `confidence_threshold` (None)
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: GREEN — filtros + ops + abstención + import T1c
- **Archivos:** `page.rs`, `namespaces.rs`, `memory.rs`, `impl_export.rs`
- **Acción:** filtro default-exclude + fingerprint + abstención (page); list (namespaces); T1 en put/put_batch + T1d/T2/T4 + bulk (memory); T1c import (impl_export)
- **Verify:** `cargo nextest run -p vantadb --test quarantine_containment` → verde
- **Estado:** ⬜ PENDING

### Step 4: Literales/firmas workspace + bindings (fix mecánico)
- **Archivos:** cli, server, providers, py/wasm/node/ts call sites, tests existentes (guiado por compilador)
- **Verify:** `cargo check --workspace --all-targets` + `cargo check -p vantadb-python` (+ wasm32 si toca wasm)
- **Estado:** ⬜ PENDING

### Step 5: Gate de inyección vanta-memory + test de recall
- **Archivos:** `vanta-memory/src/core/record/l1_reader.rs`, `vanta-memory/tests/recall.rs`
- **RED:** test `quarantined_l1_records_are_not_recalled` (falla: hoy sí se inyecta)
- **GREEN:** `include_quarantined: false` explícito en el choke point
- **Verify:** `cargo nextest run -p vanta-memory --test recall` (scoped)
- **Estado:** ⬜ PENDING

### Step 6: MCP — flag T1 en memory_put/memory_put_batch
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (schema ×2 + handler + parse)
- **Verify:** `cargo nextest run -p vantadb-mcp --ignore-default-filter -E 'test(memory_put)'` + test de recall MCP si el harness lo permite
- **Estado:** ⬜ PENDING

### Step 7: Verify full + cierre
- **Verify:** `cargo fmt --check` · clippy `-D warnings` (crates tocadas) · `cargo nextest run --profile audit -p vantadb --build-jobs 2` (full, timeout 900; flake HNSW conocido) · `scripts/validate-docs-coverage.ps1` · OCR advisory acotado
- **Estado:** ⬜ PENDING

## Pendientes (§Pendientes)
- **docs diferidas por WIP ajeno → SCH-07:** `docs/api/` (EMBEDDED_SDK/MCP/HTTP_API/scores…) + `llms.txt` con cambios de otra sesión SIN COMMITEAR — NO editar (instrucción del orquestador). Doc de cuarentena/abstención (Regla 3) se entrega en SCH-07/LEAD.
- Superficies restantes → SCH-07: `SearchPageV2.abstained` (HTTP), args de import (MCP/HTTP/CLI), bindings Py/TS/Node/WASM (getters/knobs), `quarantine_*` ops en HTTP/MCP, `min_confidence` en `MemoryListOptions`.
- T1b (promoción derivada de dream default ON) → MEM-65 (merge real; hoy stub no muta L1).
- Métrica agregada `quarantine_overdue` → SCH-07/v1.0 (señal base = campo `quarantine_review_due_ms`).
- FIND candidato: `inject_context` L0 aislado = v1.0 (MGR-13 §4.4 residual; solapa EXE-07).

## Dependencias
- **Consume:** SCH-02 ✅ (`7af34366`; campos + sticky base) · SCH-03 ✅ (`932b1211`; AS OF) · SCH-04 ✅ (`932b1211`; `min_confidence` + fingerprint) · ADR-046 `accepted` ✅ · MGR-13 ✅ (`72f29720`).
- **Bloquea:** SCH-06 (tests chaos: TTL+quarantine, supersede+quarantine, contención). **nextTask:** SCH-06.

## Herramientas
- codegraph (index FROZEN por lock de otro proceso → lectura directa documentada) · nextest scoped `-p` + `CARGO_BUILD_JOBS=2`.
- **SDP (v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · pin `security-and-hardening` (trust boundary write-time/gates) · `test-driven-development` (pin policy) · `source-driven-development` · `systematic-debugging` (pin policy) · `incremental-implementation` · `context-engineering` · `documentation-skill` (task file).

## Progreso

| Step | Estado | Evidencia |
|------|--------|-----------|
| 1 RED test contención | ⬜ | — |
| 2 Wire + config | ⬜ | — |
| 3 GREEN filtros/ops/abstención/import | ⬜ | — |
| 4 Literales workspace | ⬜ | — |
| 5 Gate vanta-memory | ⬜ | — |
| 6 MCP flag T1 | ⬜ | — |
| 7 Verify full | ⬜ | — |

## Context Save Point
- **Discovery ✅ (2026-09-29):** estado post-SCH-02/03/04 verificado; sticky T1 ya parcial en SCH-02; gap real = operacionalizar (filtros + gates + ops + abstención + config + import). Task file creado. Gate D: símbolos públicos nuevos pre-aprobados por ADR-046 (firma owner 2026-09-28) + contrato verbatim del plan — sin ronda new question.
