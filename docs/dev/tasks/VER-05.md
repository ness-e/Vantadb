---
title: "VER-05: Importadores Mem0/Zep/Letta→VantaDB + formato de intercambio documentado"
kind: task
description: "3 importadores (Mem0 memories JSON, Zep/Graphiti episodios+facts, Letta .af bloques+mensajes) → MemoryExportLine v2 con procedencia y descartes declarados, roundtrip estable, doc del formato en docs/api y guía de migración por sistema."
---

# VER-05: Importadores Mem0/Zep/Letta→VantaDB + formato de intercambio documentado

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 36, Fase F4)
- **Fuente:** Backlog:927 (P52) + plan Task 36
- **Esfuerzo:** 🟢 1-2d
- **Prioridad:** 🟠
- **Tipo:** Rust (+ Docs)
- **Turns estimados:** 8-10
- **Creado:** 2026-09-29T15:05
- **last-synced:** 2026-09-29T17:55
- **Estado:** ⏳ IN PROGRESS — review-fix batch ✅ (R1+O1+O2+O3+N1; N2 doc); cierre (commit + re-review P2-01) = LEAD
- **Incógnitas (uphill):** 0 abiertas (formatos verificados con fuentes oficiales; R1 grounded en `loop.af` real)
- **Pendientes (downhill):** 0 steps de ejecución (10/10 ✅ + batch de review ✅); queda re-review + commit (LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno nuevo: `src/sdk/importers/` es aditivo (módulo + re-export en `src/sdk/mod.rs`). Consumidores futuros: ICP-03 (F5), guías de migración. |
| Callees | `src/sdk/types.rs` (`MemoryExportLine`, `MemoryRecord`, `Value`, `Fields`, `ConfidenceClass`, `DERIVATION_DISCOUNT`, `default_confidence`), `src/sdk/serialization/mod.rs` (`record_from_export_line`, `EXPORT_SCHEMA_VERSION`), `crate::error::Error`, `serde_json`, `chrono`, `twox_hash`. |
| Implicaciones | Cero cambios de wire (no se toca serialización on-disk). Cero migración de datos. API pública **aditiva** (módulo nuevo + 4 símbolos públicos) → doc `docs/api/` en el mismo PR (Regla 3). `src/cli.rs` NO se toca (co-batch VER-01). Performance: conversión O(n) offline, no hot path. Tests existentes intactos (solo se agrega `tests/importers.rs` + fixtures). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/serialization/mod.rs` (regiones 51-85, 345-800, 1690-1770), `src/sdk/serialization/impl_export.rs` (255-448), `src/sdk/types/record.rs` (380-560), `src/sdk/types.rs` (100-159), `src/sdk/api/memory.rs` (790-909), `src/sdk/mod.rs`, `src/cli.rs` (100-128), `src/cli_handlers/data.rs` (140-219), `Cargo.toml` (lints/deps), `tests/memory_export_import.rs` (1-120), `docs/api/EMBEDDED_SDK.md` (505-574), `docs/user/tutorials/03-migrating-from-chromadb.md`, `scripts/validate-docs-coverage.ps1`.
- **Archivos referenciados hacia dentro (imports/dependencias):** `src/sdk/mod.rs` re-exporta `serialization`/`types` (público); `impl_export.rs` usa `record_from_export_line`; `vantadb-python/vantadb-python/src/lib.rs` y `vantadb-wasm/src/lib.rs` consumen `Embedded::import_records/import_file` (sin cambios).
- **Archivos que referencian a los editados (referencias entrantes):** `rg 'sdk::importers|importers::'` = 0 (nuevo); `EXPORT_SCHEMA_VERSION` solo lo usa `src/sdk/serialization/mod.rs` (2 usos: export + validación) → promover a `pub(crate)` no cambia semántica.
- **Veredicto impacto:** 🟢 bajo — aditivo puro. `src/sdk/mod.rs` gana 1 línea (`pub mod importers;`); `serialization/mod.rs` gana `pub(crate)` (1 palabra). Nada se rompe si se revierte el módulo nuevo.

## Contrato

(verbatim del plan Task 36 — ley)

"3 importadores (Mem0 memories JSON, Zep/Graphiti episodios+facts, Letta archivos/bloques) que mapean a `MemoryExportLine` preservando metadata y procedencia, cada uno con test sobre fixture real Y roundtrip estable (import → export v2 → diff) Y formato de intercambio VantaDB documentado (JSONL v2 + semántica temporal/dedup/conflicto + límites de lo preservable) en `docs/api/` Y guía de migración por sistema publicada en `docs/user/tutorials/` (solo archivos de export; sin credenciales)"

## Spec (SDD — feature-add: símbolos públicos nuevos)

> Gate P/D: el plan ya autorizó la familia (Gate Result ✅ DO, destinos prescritos: `nuevo src/sdk/importers/`, fixtures, guías). Decisiones abiertas resueltas por evidencia (docs oficiales de los 3 formatos) — ver Investigation Notes.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Superficie pública | (A) módulo `sdk::importers` con `convert_str`/`convert_file` por fuente + `Conversion`/`ConversionStats` / (B) flag CLI `--format mem0|zep|letta` (requiere `src/cli.rs`) / (C) script Python | **A** — prescrito por plan; CLI bloqueado por co-batch VER-01 (deuda → ICP-03/VER-06) | ✅ decidido-por-evidencia (plan Task 36 archivos clave; ctx F4.1 prohíbe `src/cli.rs`) |
| 2 | Namespace destino | (A) derivado del scope del export (`mem0/<user_id>`, `zep/<user_id\|graph_id>`, `letta/<agent>/<blocks\|messages>`), saneado al charset (`A-Za-z0-9._/-`, ≤128B) / (B) parametrizable por argumento | **A** — las líneas son editables antes de importar; B agrega API sin caso de uso | ✅ decidido-por-evidencia (`validate_namespace` serialization/mod.rs:108-131) |
| 3 | Claves (dedup/idempotencia) | (A) id del sistema origen / (B) content-hash siempre / (C) id o content-hash si falta id | **C** — re-import con mismo id = update (no duplica); exports sin id quedan idempotentes por hash (`twox_hash::XxHash3_128`, patrón `memory_node_id` :77-83 y `md_import` MEM-39) | ✅ decidido-por-evidencia (pre-mortem F3 del plan; `md_import.rs:208-241`) |
| 4 | Temporalidad | (A) `created_at`/`updated_at` del origen + Zep `valid_at`/`invalid_at` → campos v2 / (B) ignorar timestamps | **A**; Zep `expired_at` (transaction-time) SIN destino v2 → descartado explícito (contador + doc) | ✅ decidido-por-evidencia (Zep docs: 4 timestamps; v2 fields record.rs:477-483) |
| 5 | Procedencia Zep facts | (A) `Derived` + `derived_from`=uuids de episodios + `confidence = default_confidence()×DERIVATION_DISCOUNT` / (B) `Asserted` | **A** — Zep facts son extraídos de episodios: el v2 schema tiene los campos exactos (fidelidad máxima) | ✅ decidido-por-evidencia (zep-python `entity_edge.py`; record.rs:74-81) |
| 6 | Metadata de origen | (A) claves fijas de procedencia (`source`, `source_id`, scope) + campos fuente bajo prefijo (`mem0.*`, `zep.*`, `letta.*`); objetos anidados → JSON string; arrays de strings → `ListString` / (B) flatten sin prefijo (colisiones) | **A** — sin colisiones, filtrable, y `validate_metadata` prohíbe solo `__vanta_` (serialization/mod.rs:12/:155-168) | ✅ decidido-por-evidencia |
| 7 | Tolerancia de parser | (A) aceptar envelopes documentados (array, `{results}`, `{data}`; `{episodes,facts\|edges}`; `.af` raíz) + campos alternos (`memory\|text\|content`) + fixture mínimo / (B) solo shape estricto | **A** — export rivales versionan; pre-mortem F2 exige parser tolerante | ✅ decidido-por-evidencia (mem0 CLI README: import acepta `memory\|text\|content`; get_all `{results}`) |
| 8 | Formato de intercambio | (A) `MemoryExportLine` v2 existente (no inventar) / (B) formato nuevo | **A** — ley del plan ("NO se inventa"); `EXPORT_SCHEMA_VERSION` → `pub(crate)` para que los importadores no dupliquen el literal `2` | ✅ decidido-por-evidencia (plan Notas Task 36; mod.rs:54) |
| 9 | Alcance Letta | (A) blocks + message history + metadata de agente; passages/data-sources declarados NO soportados por `.af` (roadmap upstream) / (B) inventar import de passages | **A** — `.af` no los contiene: no hay nada que mapear | ✅ decidido-por-evidencia (agent-file README: "We currently do not support Passages"; Roadmap "Support data sources") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) no tocar wire/serialización on-disk ni `MemoryRecord`/`MemoryExportLine` shape; (2) importar vía `record_from_export_line` + `import_records` (validación de confianza/ventana una sola vez, sin reimplementar); (3) keys determinísticas → re-import idempotente (update, nunca duplicate); (4) sin credenciales ni llamadas de red — solo archivos de export locales; (5) `src/cli.rs`, `src/wal*.rs`, `vanta-memory/src/core/dream/**`, `vantadb-mcp/src/dreams.rs` intactos (co-batch F4.1).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --test importers --build-jobs 2` (verde) · `cargo nextest run --profile audit -p vantadb --build-jobs 2` (sin regresiones) · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo fmt --check` · `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` · `scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente:** flag CLI de import (`vanta-cli import --source mem0|zep|letta`) diferido → ICP-03 (F5) o VER-06, cuando `src/cli.rs` no esté en co-batch; conversión expuesta solo vía SDK Rust.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (aditivo: 0 `unsafe`, 0 dependencias nuevas, 0 hot paths; el único `#![allow]` es el blanket de test del patrón de la casa — `tests/importers.rs:2` `clippy::expect_used, clippy::unwrap_used`, igual que el resto de la suite). **Deuda declarada (diferida, no introducida):** flag CLI de import (`vanta-cli import --source mem0|zep|letta`) queda para ICP-03 (F5) / VER-06 — esta wave no toca `src/cli.rs` (co-batch VER-01); la conversión se expone vía SDK Rust (`vantadb::sdk::importers`).

## Verificación (review-fix batch 2026-09-29T17:40)

| Ítem | Fix aplicado | Evidencia (comando → resultado) |
|------|--------------|---------------------------------|
| **R1** `content: []` importado como `"[]"` | `letta.rs`: arrays de content parts → text parts unidas (`\n`); arrays vacíos/sin texto → skip declarado; parts no-texto contadas (`message.content_parts`) | `cargo nextest run --profile audit -p vantadb --test importers --build-jobs 2` → **19/19 ✅** (nuevo test `letta_empty_and_text_part_content_arrays_are_handled` con shapes reales de `loop.af`) |
| **O1** idempotencia/determinismo solo Mem0 | `tests/importers.rs`: tests parametrizados `every_source_reimport_updates_instead_of_duplicating` + `every_source_conversion_is_deterministic_across_runs` (los 3 sources, counts por namespace) | misma corrida → 19/19 ✅ |
| **O2** `derived_from` colgado si episodio skipped | `zep.rs`: parents filtrados a `converted_episodes` (BTreeSet); leftover contado `fact.episodes_unresolved`; downgrade a Asserted si no quedan | test `zep_fact_parents_are_filtered_to_converted_episodes` ✅ + assert de no-unresolved en fixture ✅ |
| **O3** descartes de envelope sin contador | `zep.rs`: `edges` ignorado contado cuando `facts` presente; `mod.rs`/`MEMORY_INTERCHANGE_FORMAT.md`: wording "counters are record-level" + lista explícita de envelope drops por fuente | test `zep_edges_are_ignored_and_counted_when_facts_are_present` ✅ |
| **N1** "0 suppressions" inexacto | re-wording en §Deuda (arriba) | — |
| **N2** `write_jsonl` sin sandbox | documentado en `MEMORY_INTERCHANGE_FORMAT.md`: helper plano sin sandbox vs `resolve_export_path` (WIRE-09) en el lado engine | — |
| **N3** untracked en commit | LEAD (sin acción del worker) | — |

**Gates re-corridos:** `nextest -p vantadb --test importers` ✅ 19/19 · clippy `-p vantadb --all-targets --all-features -- -D warnings` ✅ exit 0 · fmt scoped ✅ · markdownlint scoped docs tocados ✅ 0 issues · `check-links`/`check-docs`/`gen-index --check` ✅ (sin gating propio) · suite completa `cargo nextest run --profile audit -p vantadb --build-jobs 2` ✅ **2513/2513** (1 slow, 0 timeout).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del plan ✅ cláusula por cláusula (importadores ×3 + fixtures + roundtrip + doc `docs/api/` + guías ×3) + fmt/clippy/nextest scoped verdes |
| **Commit** | LEAD (no commitea el worker); diff limpio, conventional commit `feat: VER-05 — ...` |
| **Release** | N/A local (release-plz post-push, lane owner) |

## Herramientas necesarias
- Terminal cargo (nextest scoped `-p vantadb`, clippy, fmt) con `CARGO_BUILD_JOBS=2`
- codegraph_codegraph_explore (blast radius — hecho) · codebase-memory-mcp_check_index_coverage (hecho)
- `node scripts/docs/*.mjs` (links/docs/index) · `scripts/validate-docs-coverage.ps1`

**Skills cargadas (SDP):** `source-driven-development` (formatos rivales = docs oficiales), `test-driven-development` + `rust-write-tests` (RED→GREEN, fixtures, determinismo), `documentation-skill` (docs/api + docs/user/tutorials + gen-index), `doubt-driven-development` (review adversarial pendiente, tier `src/sdk/**`), `incremental-implementation` (slices por importador). SDP v2 delegado al prompt del lead (lista fija).

## Investigation Notes

### Formatos de origen (fuentes oficiales — verificadas 2026-09-29)

**Mem0** (memories JSON):
- `get_all()` (OSS) devuelve `{"results": [{"id", "memory", ...}]}` — docstring en `mem0/memory/main.py` (GitHub mem0ai/mem0, main). Cookbook oficial "Export Stored Memories": `get_all(filters={"user_id": ...})` → `results[i]["memory"]`; export estructurado por schema (Pydantic) es otra vía (solo Platform).
- CLI oficial (`GitHub mem0ai/mem0/cli/README.md`): modo agente emite `{"status","data":[{"id","memory","score","created_at","categories"}]}`; `mem0 import` acepta array con `memory` (o `text` o `content`) + `user_id`/`agent_id`/`metadata`.
- Campos del objeto: `id`, `memory`, `hash`, `metadata`, `created_at`, `updated_at`, `user_id`, `agent_id`, `run_id`, `categories`, `score` (search). → Importer tolera array crudo, `{results}`, `{data}`.

**Zep/Graphiti** (episodios + facts):
- "Reading Data from the Graph" (help.getzep.com): enumeración bulk `graph.node/edge/episode.get_by_user_id|get_by_graph_id` con cursor — la exportación del usuario es un dump de listas.
- Episode (zep-python `types/episode.py`): `uuid`, `content`, `created_at`, `role`, `role_type`, `source`, `source_description`, `thread_id`, `metadata`, `processed`, `document_id`, `task_id`.
- EntityEdge/fact (`types/entity_edge.py`): `uuid`, `name` (SCREAMING_SNAKE), `fact`, `created_at`, `valid_at`, `invalid_at`, `expired_at`, `episodes[]`, `attributes{}`, `source_node_uuid`, `target_node_uuid`, `source_node_name`, `target_node_name`, `score`.
- Semántica temporal (help.getzep.com/facts): valid_at/invalid_at = valid time; created_at/expired_at = transaction time.

**Letta** (.af):
- docs.letta.com/v1-sdk/concepts/agent-file + GitHub letta-ai/agent-file: top-level `{agents, groups, blocks, files, sources, tools, mcp_servers, metadata, created_at}`; agente con `messages[]` (`type`, `role`, `content`, `created_at`, `id`, `model`, ...), `block_ids`, `system`; block con `{id, label, value, limit, description, read_only, metadata}`.
- **Límites del formato (upstream):** "We currently do not support Passages (Archival Memory)"; Roadmap: "Support data sources (i.e. files)" → passages y contenido de archivos NO están en `.af` (se declara en la guía); secrets exportados como `null` (sin credenciales).

### Import existente (pipeline consumido, no reimplementado)
- `record_from_export_line` (serialization/mod.rs:695) valida `schema_version ∈ 1..=2`, confianza, ventana; `import_records` (impl_export.rs:306) → `put_record_exact` (api/memory.rs:814) re-valida; `import_file` (impl_export.rs:388, CLI `vanta-cli import`) consume JSONL directo.
- `validate_metadata` solo prohíbe prefijo `__vanta_` → prefijos `mem0.*`/`zep.*`/`letta.*` legales.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — formatos verificados; decisiones de Spec resueltas |
| Pendientes de ejecución (downhill) | 0 (10/10 steps ✅); cierre LEAD: review + commit |
| % completado | 100% (implementación+verificación); ✅ tarea supeditado a review P2-01 |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — aplica parcial: input de usuario (archivos de export de terceros) → parser tolerante con límites (sin panic, sin unwrap, errores tipados; JSON sin recursión ilimitada vía serde_json defaults) y sin red/credenciales. No agrega dependencias. Justificación: no hay trust boundary de red/auth; el riesgo es de datos (fidelidad) y se mitiga con tests + contadores de descarte. `security-and-hardening` checklist aplicado a lectura de archivos (path handling con `Result`).
- [ ] **PERFORMANCE** — N/A justificado: conversión offline O(n) one-shot, sin hot path (no toca `engine.rs`, `vector/`, loops de search; Regla 9 no aplica sin claim de performance).

## Steps

### Step 1: Task file + Spec (DISCOVERY)
- **Archivos:** `docs/dev/tasks/VER-05.md`
- **Acción:** crear task file con contrato verbatim, blast radius, Regla 0, Spec, invariantes, steps, sources
- **Verify:** archivo existe con contrato verbatim + Spec LLENA
- **Estado:** ✅ DONE

### Step 2: RED — fixtures + tests de importadores
- **Archivos:** `tests/fixtures/importers/mem0/memories.json`, `tests/fixtures/importers/zep/graph.json`, `tests/fixtures/importers/letta/agent.af`, `tests/importers.rs`
- **Acción:** fixtures realistas sanitizados (shape documentado) + suite completa (mapeo, tolerancia, roundtrip, idempotencia, determinismo) que falla a compilar (API inexistente)
- **Verify:** `cargo nextest run --profile audit -p vantadb --test importers` → compile error (RED) ✅ (evidencia: build falló por API inexistente)
- **Estado:** ✅ DONE

### Step 3: GREEN parcial — `src/sdk/importers/mod.rs` (tipos + helpers) + wiring
- **Archivos:** `src/sdk/importers/mod.rs`, `src/sdk/mod.rs`, `src/sdk/serialization/mod.rs` (1 palabra: `pub(crate)`)
- **Acción:** `Conversion`, `ConversionStats`, helpers (timestamp ISO/unix, json→Value, namespace sanitize, content-key xxh3), `pub mod importers` + re-exports
- **Verify:** `cargo check -p vantadb` ✅ + unit tests helpers 4/4 ✅
- **Estado:** ✅ DONE

### Step 4: `mem0.rs` (importador 1/3)
- **Archivos:** `src/sdk/importers/mem0.rs`
- **Acción:** array/`{results}`/`{data}` → líneas v2; `memory|text|content`; procedencia + scope; descartes contados
- **Verify:** `cargo nextest run --profile audit -p vantadb --test importers -- mem0` ✅
- **Estado:** ✅ DONE

### Step 5: `zep.rs` (importador 2/3)
- **Archivos:** `src/sdk/importers/zep.rs`
- **Acción:** episodios (asserted + timestamps) y facts (derived + `derived_from` + ventana validez); `edges` alias; `expired_at` descartado
- **Verify:** `cargo nextest run --profile audit -p vantadb --test importers -- zep` ✅ (1 fallo inicial por null-counting → fix; luego verde)
- **Estado:** ✅ DONE

### Step 6: `letta.rs` (importador 3/3)
- **Archivos:** `src/sdk/importers/letta.rs`
- **Acción:** `.af` → blocks por agente + message history; docs de límites (passages/files)
- **Verify:** `cargo nextest run --profile audit -p vantadb --test importers -- letta` ✅
- **Estado:** ✅ DONE

### Step 7: Suite completa de importadores + gates locales
- **Archivos:** (tests)
- **Acción:** correr todo `--test importers` + `cargo check -p vantadb` + clippy scoped
- **Verify:** `cargo nextest run --profile audit -p vantadb --test importers --build-jobs 2` ✅ 16/16 + clippy `-p vantadb --all-targets --all-features -- -D warnings` ✅
- **Estado:** ✅ DONE

### Step 8: Doc del formato de intercambio (`docs/api/`)
- **Archivos:** `docs/api/MEMORY_INTERCHANGE_FORMAT.md` (nuevo), `docs/api/EMBEDDED_SDK.md` (pointer)
- **Acción:** JSONL v2 + semántica temporal/dedup/conflicto + límites de lo preservable + tabla de mapeo por sistema + API de importadores
- **Verify:** `check-docs.mjs` ✅ / `gen-index.mjs --check` exit 0 ✅ (sin gating propio)
- **Estado:** ✅ DONE

### Step 9: Guías de migración ×3 (`docs/user/tutorials/`)
- **Archivos:** `docs/user/tutorials/migrating-from-mem0.md`, `migrating-from-zep.md`, `migrating-from-letta.md`
- **Acción:** patrón `03-migrating-from-chromadb.md`; export del usuario → conversión → `vanta-cli import`/SDK; sin credenciales
- **Verify:** `check-links.mjs` ✅ (0 rotos propios) + `gen-index --write` ✅ + markdownlint scoped 0 issues ✅
- **Estado:** ✅ DONE

### Step 10: Verify full + sync + recitation
- **Archivos:** task file + plan recitation
- **Acción:** fmt + clippy workspace + nextest `-p vantadb` full + validate-docs-coverage; sync task file; RESULTADO
- **Verify:** `nextest -p vantadb` 2510/2510 ✅ (corrida 1) · `clippy -p vantadb --all-targets` ✅ · fmt mis archivos ✅ · coverage: 0 gaps propios · snapshot public-api regenerado ✅
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (aditivo). Coordina ICP-03 (F5) — consume estos importadores. No toca regiones de co-batch F4.1 (VER-01 `cli.rs`/`wal*`, VER-07 dream).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** PENDIENTE — LEAD decide (`vanta-review` adversarial obligatorio: el diff toca `src/sdk/**`).
- **Enfoque:** fidelidad de mapeo (tabla campo↔campo sin pérdidas silenciosas), correctitud del parser tolerante, idempotencia re-import, que el snapshot `public-api.txt` regenerado refleje solo cambios intencionales.
- **Cómo se probó (evidencia para el revisor):** `tests/importers.rs` 16/16 (fixtures → import → export v2 → diff, re-import idempotente, determinismo) · unit tests helpers 4/4 · `cargo nextest run --profile audit -p vantadb --build-jobs 2` 2510/2510 (corrida 1) · `clippy -p vantadb --all-targets --all-features -- -D warnings` exit 0 · fmt de archivos propios exit 0 · docs: check-links/check-docs/gen-index/markdownlint verdes para archivos propios.
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (evidencia en `%TEMP%\ver05_*.txt`); fallos reportados (timeout flake + fallos co-batch) sin ocultar; research multi-fuente con URLs citadas; sin reintentos en bucle; sin scope creep (no CLI, no bindings); error handling intacto (errores tipados, sin unwrap); presupuesto acotado.
- **Veredicto:** ⬜ pendiente (LEAD)

## Notas
- Contrato verbatim del plan (Task 36) — no reinterpretar.
- `src/sdk/serialization/mod.rs`: único cambio quirúrgico `const EXPORT_SCHEMA_VERSION` → `pub(crate)` (evita duplicar el literal `2`; no cambia API pública).
- CLI flag diferido (deuda declarada) por co-batch VER-01 sobre `src/cli.rs`.
- Fixtures sintéticos modelados sobre shapes documentados (sanitizados; no hay export real a mano — permitido por plan: "sintéticos si no hay export real a mano").
- **Snapshot `tests/api/public-api.txt` regenerado** (`VANTADB_PUBLIC_API_UPDATE=1`): incluye el módulo `sdk::importers` de VER-05 **y** `cli::Commands::Verify` de VER-01 (árbol compartido de la wave). +51 líneas. Si otro co-batch agrega API pública después, regenerar una vez más al cierre de la wave.
- **Environment findings (para routing del LEAD):**
  - Disco C: llegó a **0 bytes libres** durante la wave → linkers fallaron (os error 112 / LNK1180). Mitigado: se eliminaron cachés obsoletas `target/session-wire06` (2.5 GB, 28/9), `target/session-wire07` (8.3 GB, 28/9) y `target/debug/incremental` (26.2 GB) → ~34.7 GB libres. Candidato FIND: higiene de disco/`target` en co-batch (los `session-wire*` quedan huérfanos).
  - Procesos huérfanos `vanta-cli.exe` (28/9 y 15:04) sostenían lock del binario → `cargo` no podía relinkear (Access denied). Terminados; candidato FIND: limpieza de procesos de test al cierre de wave.
  - Flake de carga: `index::core::tests::concurrent_insert_preserves_hnsw_invariants` (HNSW concurrency, ajeno a este diff) TIMEOUT (180s) bajo carga co-batch en la corrida 2; pasa aislado en 48.7s y pasó en la corrida 1 (2510/2510). Candidato FIND (Regla 2): marcar `flaky` si reaparece con la wave en paralelo.
  - Fallos co-batch ajenos en gates globales: `clippy --workspace` falla en `vanta-memory` test `dreaming` (unused variable `reloaded`, VER-07 in-flight); `validate-docs-coverage.ps1` reporta 1 gap: comando CLI `verify` sin doc en `CONFIGURATION.md` (VER-01); `cargo fmt --check` reprueba solo archivos de VER-01/VER-07; `check-docs.mjs` gating restante: VER-07 sin frontmatter + descripción larga de VER-01. Ninguno es de VER-05.
- **Cobertura de acceptance (cláusula por cláusula):** 3 importadores ✅ (`mem0::convert_*`, `zep::convert_*`, `letta::convert_*`) · mapean a `MemoryExportLine` v2 preservando metadata/procedencia ✅ (incluye Derived+`derived_from` para facts Zep y ventana valid/invalid; parents filtrados a episodios convertidos) · test sobre fixture por fuente ✅ (19 tests, 3 fixtures) · roundtrip estable ✅ (3 tests convert→import→export v2→diff por fuente + re-import idempotente + determinismo para los 3 sources) · formato documentado en `docs/api/` ✅ (`MEMORY_INTERCHANGE_FORMAT.md` + pointer EMBEDDED_SDK) · guía por sistema en `docs/user/tutorials/` ✅ (3, solo archivos de export, sin credenciales).
- **Review-fix batch (2026-09-29T17:40):** R1+O1+O2+O3+N1 aplicados; N2 documentado; N3 = LEAD. Evidencia en §Verificación.

## Recitation (canónico — MCP §12.3)

- **activeGoal:** VER-05 — Importadores Mem0/Zep/Letta→VantaDB + formato de intercambio documentado
- **lastAction:** review-fix batch R1 (content `[]`), O1 (idempotencia/determinismo ×3), O2 (parents Zep filtrados), O3 (`edges` contado + wording envelope), N1 (deuda) + re-verify: importers 19/19, clippy pkg ✅, fmt/markdownlint/docs gates ✅
- **result:** OK (implementación + verificación; commit y review adversarial = LEAD)
- **nextAction:** LEAD — review adversarial P2-01 (`src/sdk/**`) + commit `feat: VER-05 …` incluyendo untracked (N3)
- **contract:**
  - `verificacion`: `cargo nextest run --profile audit -p vantadb --test importers --build-jobs 2` → **19/19 ✅**; `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → **exit 0**; fmt scoped → **exit 0**; markdownlint docs tocados → **0 issues**; `check-links`/`check-docs`/`gen-index --check` → **✅ sin gating propio**
  - `evidencia`: tests R1/O1/O2/O3 verdes (`tests/importers.rs`); shapes reales verificados contra `agent-file` upstream (`loop.af`: `content: []` + `tool_calls`, parts `[{type:"text",text:…}]`); `tests/api/public-api.txt` regenerado (+51)
  - `artefactos`: `src/sdk/importers/**`, `src/sdk/mod.rs`, `src/sdk/serialization/mod.rs`, `tests/importers.rs`, `tests/fixtures/importers/**`, `docs/api/MEMORY_INTERCHANGE_FORMAT.md`, `docs/api/EMBEDDED_SDK.md`, `docs/user/tutorials/migrating-from-{mem0,zep,letta}.md`, `tests/api/public-api.txt`, índices regenerados
  - `invariantes`: sin cambios de wire; import vía `record_from_export_line`; keys determinísticas (re-import = update); sin red/credenciales; regiones co-batch intactas (`cli.rs`, `wal*`, dream)
  - `deuda`: flag CLI `import --source` diferido (ICP-03/VER-06); regenerar `public-api.txt` si otra task agrega API pública
  - `queda_pendiente`: review adversarial + commit (LEAD)
- **nextTask:** VER-06 (o ICP-03 según lane del plan)
