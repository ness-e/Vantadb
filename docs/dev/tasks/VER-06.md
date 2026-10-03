---
title: "VER-06: Export file-native Markdown + rebuild_index (git-friendly)"
kind: task
description: "Markdown file-native v2 git-friendly: frontmatter 1->2 (import 1..=2), wikilinks informativos recortables, export byte-estable, E2E export->import-md->rebuild->search"
---

# VER-06: Export file-native Markdown + `rebuild_index` (git-friendly)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 37, Fase F4 — wave F4.2)
- **Fuente:** `docs/dev/Backlog.md:932` (VER-06, P52 2026-09-24) + plan Task 37 + FUT-11 (`Backlog.md:408` → `backlog-futuro.md:25`)
- **Esfuerzo:** 🟢 1-2d · **Prioridad:** 🟡
- **Tipo:** Rust (core CLI handler + vanta-memory seed) + Docs
- **Turns estimados:** 15-25
- **Creado:** 2026-09-29T23:00
- **last-synced:** 2026-09-29T23:00
- **Estado:** ⏳ IN PROGRESS (S1..S6 ✅ verificados scoped; S7 verify full pendiente — co-batch wave F4.2; commits = LEAD)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 1 step (S7: verify full + recitation)

## Contrato (verbatim — plan Task 37)

> "flujo E2E verde con test: `vanta-cli export --format md` → edición manual de un .md → `vanta-seed import-md` → `vanta-cli rebuild-index` → search/get devuelve el valor editado Y frontmatter MD extendido a v2 (valid_at/invalid_at/confidence/quarantine; `MD_EXPORT_SCHEMA_VERSION` 1→2 con import aceptando 1..=2, sin breaking) Y wikilinks a registros relacionados (superseded_by/derived_from → `[[ns/key]]`) Y doc del flujo git-friendly (editar, diff, PR, rebuild) publicada"

**Operacionalización (verificación mecánica):**
1. `cargo nextest run --profile audit -p vanta-memory --build-jobs 2 -E 'test(md_git_e2e) or test(md_roundtrip)'` → E2E: export (handler CLI real) → edit .md → `vanta-seed import-md` (binario real) → `rebuild-index` (handler CLI real) → `db.get`/`db.search` devuelve el payload editado; export doble byte-idéntico (git-friendly).
2. `cargo nextest run --profile audit -p vantadb --build-jobs 2 -E 'test(export_md)'` → frontmatter v2 + wikilinks + determinismo (sin timestamp volátil en `index.json`).
3. `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` → import acepta 1..=2; v1 compat; v2 preservado (valid_at/invalid_at/confidence/derived_from/quarantine ×4/superseded).
4. `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps; doc git-friendly publicada en `docs/api/VANTA_MEMORY.md`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/bin/vanta-cli.rs:82` (`cmd_export_md` = dispatch real) · `vanta-memory/src/bin/vanta-seed.rs:87` (`import_md_dir`) · `vanta-memory/src/seed/mod.rs:22` (re-export) · `vanta-memory/tests/md_roundtrip.rs` |
| Callees | `vantadb::sdk` (`Embedded::{get,import_records}`, `MemoryExportLine`, `record_from_export_line` sdk/mod.rs:20-21) · `serde_json` · `fs` |
| Implicaciones | Aditivo: frontmatter MD v1→v2 (import acepta 1..=2, sin breaking) · `index.json` pierde `generated_at_ms` (0 consumidores — grep) y ordena entradas → export byte-estable · `SeedCounts` gana campo aditivo `links_unresolved` · `test_render_md` (fixture, seed/mod.rs:109) se actualiza a v2 · **NO toca** wire/on-disk/WAL/índices; **NO toca** regiones co-batch (`src/shred/**`, `src/gc.rs`, `src/storage/engine/delete.rs`, `vanta-proxy/**`, `src/crypto.rs`) |

**Riesgo:** bajo (aditivo, cero hot path).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/cli_handlers/export_md.rs` (1-328) · `vanta-memory/src/seed/md_import.rs` (1-315) · `vanta-memory/src/seed/mod.rs` (1-258) · `vanta-memory/src/bin/vanta-seed.rs` (1-124) · `vanta-memory/tests/md_roundtrip.rs` (1-145) · `vanta-memory/tests/seed.rs` (1-90+) · `src/cli_handlers/index.rs` (1-283) · `src/sdk/types/record.rs` (MemoryInput :124-193, MemoryRecord :198-307, MemoryExportLine :447-539) · `src/sdk/serialization/mod.rs` (:40-149, :692-795) · `src/sdk/api/memory.rs` (:448-577, :811-879 put_record_exact) · `src/sdk/serialization/impl_export.rs` (:280-429) · `src/sdk/api/namespaces.rs` (:20-45) · `docs/api/VANTA_MEMORY.md` (:1-100)
- **Referencias hacia dentro:** `vantadb::cli_handlers::{cmd_export_md}` ← `src/bin/vanta-cli.rs:82`; `vanta_memory::seed::import_md_dir` ← `vanta-seed.rs:87` + `seed/mod.rs:22` + `tests/md_roundtrip.rs:112`; `MD_EXPORT_SCHEMA_VERSION`/`MD_IMPORT_SCHEMA_VERSION` ← solo internos (+ `test_render_md`).
- **Referencias entrantes (grep workspace):** `parse_md_file` → solo `md_import.rs:220` (hacerla privada no rompe nada); `test_render_md` → solo `tests/md_roundtrip.rs:85`; `index.json` → solo lo escribe `export_md.rs` (0 lectores); `SeedCounts {` literal → solo `vanta-memory/tests/seed.rs:76`.
- **Veredicto impacto:** bajo. Cambios aditivos; único consumidor interno de `parse_md_file` (se vuelve privada); `SeedCounts` gana 1 campo (actualizar literal de test); `test_render_md` es fixture de test. Sin migración de datos ni re-indexación obligatoria (MD es proyección; el store es la fuente).

## Spec (SDD — decisiones)

| # | Decisión | Opciones (+tradeoff) | Elegido | Resuelto |
|---|----------|----------------------|---------|----------|
| 1 | Ubicación de wikilinks | A) bloque `## Related` al final del body con marcador `<!-- vanta:links -->` (clickeable en editor; el import lo recorta → informativo, no fuente de joins) / B) solo array en frontmatter (robusto; no navegable como link) | A | ✅ decidido-por-evidencia (Backlog:928 "Markdown+frontmatter con wikilinks (ReMe/Claude-filesystem)"; plan F2 "informativos, no fuente de joins") |
| 2 | Vía de escritura del import para v2 | A) `MemoryInput` + `db.put` — NO puede expresar `superseded_by`/`invalid_at_ms`/`quarantine_*` (`record.rs:124-168`; put resetea superseded a `None` `memory.rs:574-576`) → pierde semántica F3 / B) frontmatter → `MemoryExportLine` → `record_from_export_line` (normalización v1/v2 ADR-046 §D7 reusada) → `db.import_records` (validado + rebuild de índices derivados/texto) | B | ✅ decidido-por-evidencia (`serialization/mod.rs:695`, `impl_export.rs:306-385`) |
| 3 | Determinismo byte a byte (git-friendly) | A) mantener `generated_at_ms` en `index.json` (timestamp volátil → diff en cada export) / B) eliminarlo + ordenar `records` por `file` (0 consumidores del campo; grep) | B | ✅ decidido-por-evidencia (contexto §7 "sin timestamps volátiles"; `index.json` sin lectores) |
| 4 | Reporte de links sin destino | A) `tracing::warn` (invisible en CLI) / B) campo aditivo `SeedCounts.links_unresolved` (visible en stdout de `vanta-seed`; requiere actualizar literal de `tests/seed.rs:76`) | B | ✅ decidido-por-evidencia (plan F2 "reporte de links sin destino"); vanta-memory = crate interna `publish=false` (`VANTA_MEMORY.md:21-30`) |
| 5 | Ubicación/cómo del E2E | A) `vanta-memory/tests/md_git_e2e.rs`: `cmd_export_md`/`cmd_rebuild_index` (handlers = dispatch real de `vanta-cli`, `src/bin/vanta-cli.rs:64/:82`) + spawn del binario real `vanta-seed` (`CARGO_BIN_EXE_vanta-seed`) / B) root tests: spawn `vanta-cli` pero sin acceso al bin `vanta-seed` (`CARGO_BIN_EXE_*` es same-package) | A | ✅ decidido-por-evidencia (`cargo tree` confirma dev-dep unification: vantadb default+cli+fjall en tests de vanta-memory; patrón `tests/cli_tests.rs:1777`) |
| 6 | Fixture `test_render_md` | A) actualizarlo a v2+links (mantiene el patrón fixture) / B) exponer `render_record_md` del core como `pub` (símbolo público nuevo → Gate D) | A | ✅ decidido-por-evidencia (evita símbolo público nuevo; `api-contract.md` R-1) |
| 7 | Quarantine en import | `import_records(records, quarantine=false)` — los registros llevan su propio estado verbatim (roundtrip exacto), mismo contrato que `import_file` (SCH-05/T1c) | — | ✅ decidido-por-evidencia (`impl_export.rs:346-350`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Import **acepta 1..=2** y **rechaza 0 y ≥3** (sin breaking para consumidores v1).
  2. Roundtrip sin pérdida de campos v2: `valid_at_ms`, `invalid_at_ms`, `confidence_class`, `confidence`, `last_validated_at_ms`, `derived_from`, `quarantined_at_ms`, `quarantine_reason`, `quarantined_by`, `quarantine_review_due_ms`, `superseded_by`, `superseded_at_ms`.
  3. El bloque `## Related` es **informativo**: JAMÁS entra al payload (se recorta al import); el payload editado a mano se preserva.
  4. `MD` es proyección: el store es la fuente de verdad (doc explícita).
  5. `index.json` byte-estable para datos sin cambios (sin reloj).
  6. No tocar regiones co-batch F4.2 (`src/shred/**`, `src/gc.rs`, `src/storage/engine/delete.rs`, `vanta-proxy/**`, `src/crypto.rs`) ni `Backlog.md`/`perf-bench.yml`/`CONSTRAINTS.md`/`desktop/**`/`opencode.jsonc`/plan file.
- **Comandos de verificación:** `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vanta-memory --build-jobs 2` + `... -p vantadb --build-jobs 2` + `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `pwsh scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente:** ninguna prevista (Regla 6: saldo 0). Si wikilinks exigen resolver relaciones del grafo en export → stop condition ya recortada a `superseded_by`/`derived_from` planos (contrato cumplido así).

## Recitation

```
=== RECITATION ===
Objetivo activo: VER-06 — Export file-native Markdown (v2 + wikilinks) + rebuild_index E2E (git-friendly)
Estado: COMPLETO (worker) — S1..S7 ✅ + batch review P2-01 aplicado (O1/O2/O3/N1/F1; veredicto ✅ APPROVE); commit = LEAD
Última acción: Batch P2-01: O1 normalización CRLF en `split_frontmatter` + test `import_md_dir_normalizes_crlf_files`; O2/O3 caveats doc (sin deletes/prune; vectores no viajan); N1 cita export 6/6; F1 no-dedupe de `links_unresolved` documentado. Re-verify: md_git_e2e+md_roundtrip+seed 8/8 · md_import 12/12 · full -p vanta-memory 567/567 · export_md 6/6 · rustfmt scoped 0 · markdownlint 0 issues.
Resultado: OK (worker completo y verificado; commit/workspace-gate = LEAD por instrucción de wave)
Próxima acción: LEAD — commit local `feat(cli): md export v2 + wikilinks + git-friendly determinism (VER-06)` + re-run `cargo nextest --workspace --build-jobs 2` en árbol convergido
Contrato: "E2E export→edit→import-md→rebuild-index→search/get + frontmatter v2 (1→2, import 1..=2) + wikilinks [[ns/key]] + doc git-friendly" → ✅ verificado por tests (md_git_e2e 2/2 con binario real vanta-seed; md_roundtrip 2/2; md_import 12/12; export_md 6/6)
Invariantes: import acepta 1..=2 / roundtrip sin pérdida v2 / related block nunca al payload / MD = proyección / index.json byte-estable
Deuda: ninguna (Regla 6 saldo 0)
Próxima tarea si completa: VER-04
last-synced: 2026-09-29T20:15
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva. Se paga deuda existente: el import MD ahora propaga ediciones de frontmatter/metadata (antes el hash payload-only las ignoraba silenciosamente — comportamiento corregido) y el export deja de tener timestamp volátil.

## Definition of Done (contrato multi-nivel)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verbatim ✅ (E2E + v2 1→2 + wikilinks + doc) + fmt/clippy/nextest scoped y full verdes + docs-coverage 0 gaps |
| **Commit** | **LO EJECUTA EL LEAD** (sub-agente sin commit — instrucción de wave). Mensaje sugerido: `feat(cli): md export v2 + wikilinks + git-friendly determinism (VER-06)` |
| **Release** | N/A para este agente (corte 0.8.0 = lane release-plz; verificar que el schema MD no rompe el marcador R1) — no aplica más allá de verify full local |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) · `codebase-memory-mcp_check_index_coverage` ✅ (12 paths, sin gaps) · `cargo nextest run --profile audit -p vantadb|vanta-memory --build-jobs 2` scoped (`-E 'test(export_md) or test(md_import) or test(md_roundtrip) or test(md_git_e2e)'`) · `CARGO_BUILD_JOBS=2` · regla dura `-p` · `campaign_verify_cmd` · `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `pwsh scripts/validate-docs-coverage.ps1` · `pwsh dev-tools/ocr-review.ps1 -Format json`
- **Skills cargadas (SDP v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · `test-driven-development` · `rust-write-tests` · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `systematic-debugging` (pin) · `deprecation-and-migration` (pin) · `documentation-skill` (obligatoria al tocar `docs/`)

## Investigation Notes

- **Re-baseline (del plan, verificado):** export MD + import-md + rebuild-index YA existen (MEM-62). El slice real = v2 + E2E + wikilinks + doc. `md_roundtrip.rs` usa fixture `test_render_md`, NO el handler real → gap E2E.
- **`MemoryInput` NO puede expresar v2 completo** (`record.rs:124-168`): sin `superseded_by`/`invalid_at_ms`/`quarantine_reason/by/due`; `put_one` resetea `superseded_by=None`/`invalid_at_ms=None` (`memory.rs:574-576`). Por eso el import usa `record_from_export_line` + `import_records`.
- **`import_records` (pub, `impl_export.rs:306`)**: valida vía `put_record_exact` (D4b, ventana, node_id), count `inserted/updated/errors`, y **rebuild de índices derivados+texto** al final (`:380-381`) → search post-import funciona.
- **`superseded_by` = key (mismo namespace)** (`memory.rs:944`); `derived_from` = keys same-namespace → wikilink `[[{record.namespace}/{target}]]`.
- **Determinismo:** `list_namespaces` devuelve BTreeSet ordenado (`namespaces.rs:25`) y `metadata` es BTreeMap → único volátil era `generated_at_ms` (export_md.rs:240-243). `DefaultHasher::new()` es determinístico.
- **Cargo feature unification:** `cargo tree -p vanta-memory -e features -i vantadb` → dev build tiene vantadb `default` (cli+fjall+arrow) + `remote-inference` ⇒ el E2E en vanta-memory puede usar `cli_handlers` y fjall, y el bin `vanta-seed` persiste.
- **SCH-02.md:81** dejó explícito "`export_md.rs` usa su propio `MD_EXPORT_SCHEMA_VERSION=1` — NO tocar" (en ese entonces) → este task es la coordinación diferida del bump.
- **Web research:** no requerida (sin APIs externas; formatos propios del repo).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — decisiones resueltas por evidencia (spec §7); validación empírica fjall del bin `vanta-seed` ✅ (E2E verde) |
| Pendientes de ejecución (downhill) | **0 del worker** — S1..S7 ✅ + batch review ✅ (O1/O2/O3/N1/F1, §Verificación). Residuales delegados: commit local + re-run workspace gate (LEAD, por instrucción de wave) |
| % completado | 100% (worker) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — trust boundary: import de archivos MD editables a mano (contenido no confiable: payload, frontmatter, wikilinks). Mitigación: toda escritura pasa por `record_from_export_line` + `put_record_exact` (validación de clase/confianza/ventana/node_id); `sanitize_component` ya cubre traversal de paths en export; el import solo lee dentro de `dir` (walk recursivo sin symlink-follow especial — mismo comportamiento existente). Sin FFI nueva, sin red, sin secretos. `security-and-hardening` no dispara features nuevas de ataque; checklist revisada en el review del LEAD.
- [x] **PERFORMANCE** — no hay hot path (CLI export/import offline). Cambio de complejidad: import ahora hace UN `import_records` batch (antes N `put`) + un rebuild de índices al final → igual o mejor. Sin claims de performance (Regla 11 N/A).

## Steps

### Step 1: Export — frontmatter v2 (RED→GREEN)
- **Archivos:** `src/cli_handlers/export_md.rs`
- **Acción:** actualizar test `render_record_md_has_frontmatter` a `schema_version==2` + asserts de campos v2 (`valid_at_ms`, `invalid_at_ms`, `confidence_class`, `confidence`, `last_validated_at_ms`, `derived_from`, `quarantined_at_ms`, `quarantine_reason`, `quarantined_by`, `quarantine_review_due_ms`) → RED; bump `MD_EXPORT_SCHEMA_VERSION=2`; extender `Frontmatter` + `render_record_md`.
- **Verify:** `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vantadb --lib --build-jobs 2 -E 'test(export_md)'` → **6/6 ✅** (N1: cita canónica; corrida combinada inicial `test(export_md) or test(render_record_md) or test(sanitize_)` → 7/7)
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 2: Export — determinismo git-friendly
- **Archivos:** `src/cli_handlers/export_md.rs`
- **Acción:** quitar `generated_at_ms` de `IndexFile`; ordenar `records` por `file` antes de serializar; doc del módulo actualizada (layout `index.json` byte-estable).
- **Verify:** cubierto por `render_record_md` unit + E2E byte-identidad (`md_export_is_byte_identical_across_runs` ✅)
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 3: Import — v2 (1..=2) + mapeo `record_from_export_line` + `import_records`
- **Archivos:** `vanta-memory/src/seed/md_import.rs` · `vanta-memory/src/seed/mod.rs` (`test_render_md`) · `vanta-memory/tests/md_roundtrip.rs`
- **Acción:** RED: tests in-crate (fixture v2 en tempdir → `import_md_dir`) + v1 compat + rechazo v3; `MD_IMPORT_SCHEMA_VERSION=2` + aceptar `1..=2`; `FrontmatterRead` con campos v2 (`#[serde(default)]`); construir `MemoryExportLine` → `record_from_export_line`; pre-filtro idempotente comparando `MemoryRecord` normalizado (vector/sparse ignorados) contra `db.get`; escritura batch con `db.import_records(changed, false)`; `parse_md_file` privada; `test_render_md` a v2.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --build-jobs 2 -E 'test(md_import) or test(md_roundtrip)'` → **13/13 ✅** (incluye v2/v1/rechazo v3/idempotencia/edición frontmatter)
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 4: Wikilinks (export + strip/import + reporte)
- **Archivos:** `src/cli_handlers/export_md.rs` · `vanta-memory/src/seed/md_import.rs` · `vanta-memory/src/seed/mod.rs` (`SeedCounts.links_unresolved` + Display) · `vanta-memory/tests/seed.rs` (literal) · `vanta-memory/tests/md_roundtrip.rs`
- **Acción:** RED: export render emite bloque `<!-- vanta:links -->\n\n## Related\n\n- [[ns/key]]…` para `superseded_by`/`derived_from`; import lo recorta (payload exacto) y cuenta links sin destino. GREEN: `related_links()` + `split_related_block()` + resolución best-effort por splits de `/` contra `db.get` post-import.
- **Verify:** `-E 'binary(md_git_e2e) or binary(md_roundtrip) or binary(seed)'` → **8/8 ✅** (dedupe de links, strip, unresolved, seed.rs literal)
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 5: E2E CLI (export→edit→vanta-seed→rebuild→search/get) + determinismo
- **Archivos:** `vanta-memory/tests/md_git_e2e.rs` (nuevo)
- **Acción:** flujo completo: seed fjall (2 records + `supersede` para wikilink) → `cmd_export_md` → assert v2+links → editar payload a mano → spawn `env!("CARGO_BIN_EXE_vanta-seed") import-md <dir> --db <db>` → `cmd_rebuild_index` → `db.get` + `db.search` devuelven el editado → export doble byte-idéntico.
- **Verify:** `-p vanta-memory -E 'binary(md_git_e2e)'` → **2/2 ✅** (incluye `updated=1`/`unchanged=2`/`links_unresolved=0` del binario real + search post-rebuild)
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 6: Doc git-friendly publicada
- **Archivos:** `docs/api/VANTA_MEMORY.md` (+ doc-comments de `export_md.rs`/`md_import.rs` ya en S1-S4)
- **Acción:** `documentation-skill` cargada; sección "Git-friendly Markdown export & rebuild (VER-06)" (comandos exactos editar/diff/import-md/rebuild-index/get/search; v2 fields; wikilinks informativos; store = fuente de verdad; determinismo).
- **Verify:** `scripts/validate-docs-coverage.ps1` → **0 gaps ✅**; `check-links`/`check-docs` dentro de budget (broken/wikilinks pre-existentes); `gen-index --check` stale **pre-existente en HEAD** (verificado con `git archive` — docs-session lane, no VER-06)
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 7: Verify full + recitation + cierre (sin commit)
- **Archivos:** `docs/dev/tasks/VER-06.md` (+ plan file: LEAD)
- **Acción:** `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; nextest scoped ambos crates + full por crate; docs coverage; OCR advisory; recitation + RESULTADO (commit/self-review = LEAD).
- **Verify (resultados reales):**
  - `cargo fmt --check` → **exit 0** (workspace completo) ✅
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → **exit 0** ✅
  - `cargo nextest -p vantadb` full → **2533/2534** (1 sola falla: `public_api_snapshot_matches_committed_file` por items nuevos de `attestation` de VER-02 — snapshot sin valores de const, `MD_EXPORT_SCHEMA_VERSION` no afecta; **no es de VER-06**) ⚠️ externa
  - `cargo nextest -p vanta-memory` full → **566/566** ✅
  - `pwsh scripts/validate-docs-coverage.ps1` → **0 gaps** ✅
  - `pwsh dev-tools/ocr-review.ps1 -Format json` → Rule Group 1 (`**/*.rs`) asignado a mis archivos; self-check sin Critical/High (sin unwrap/unsafe/concurrencia) ✅
  - `cargo nextest --workspace --build-jobs 2` → **bloqueado por entorno**: `link.exe LNK1180 no hay espacio en disco` (C: llegó a 0 free durante el link de `--workspace` con sesiones co-batch construyendo en paralelo; recuperado a ~8.5 GB; el run workspace re-linkea TODOS los binarios por unificación de features → inviable seguro con el headroom actual). Diferido al cierre de wave del LEAD sobre árbol convergido (el diff no toca proxy/server/mcp/python/wasm).
- **Estado:** ✅ COMPLETED (con residuales delegados: commit/review = LEAD; re-run workspace gate = cierre de wave)

## Dependencias

- MEM-62 (export/import/rebuild existentes) — ✅ en develop.
- SCH-02 (v2 fields en record/export JSONL) — ✅ (`AA111979`?) y nota explícita `SCH-02.md:81` difería este bump a VER-06.
- nextTask: **VER-04** (F4.3, tras VER-06 ‖ VER-02 ‖ VER-03).

## Verificación (review batch P2-01 — evidencia por ítem)

> Batch aplicado 2026-09-29 tras veredicto **✅ APPROVE** (0 Critical/Required; 3 Optional + 1 Nit). Comandos → resultado:

| Ítem | Fix aplicado | Verificación (comando → resultado) |
|------|--------------|-------------------------------------|
| **O1** | `split_frontmatter` normaliza `\r\n`→`\n` antes de parsear (el orden `\n`/`\r` no colapsaba el par → payload con `\n` líder en archivos CRLF); test nuevo `import_md_dir_normalizes_crlf_files` | `cargo nextest run --profile audit -p vanta-memory --lib --build-jobs 2 -E 'test(md_import)'` → **12/12 ✅** (CRLF: payload `windows payload` sin newline líder + re-import `unchanged=1`) · `... --test md_git_e2e --test md_roundtrip --test seed` → **8/8 ✅** |
| **O2** | Doc `VANTA_MEMORY.md`: el flujo git "never deletes records or prunes stale `.md` files — deletes go through the store API" | `node_modules/.bin/markdownlint-cli2 docs/api/VANTA_MEMORY.md` → **0 issues ✅** |
| **O3** | Doc `VANTA_MEMORY.md`: vectores no viajan en MD; re-import de un registro editado deja `vector`/`sparse_vector` vacíos (pre-existente MEM-62) | idem O2 |
| **N1** | Citas corregidas: `export_md` real **6/6** con `-E 'test(export_md)'` (S1 y §Notas) | `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2 -E 'test(export_md)'` → **6/6 ✅** |
| **F1** | Documentado que `links_unresolved` cuenta **ocurrencias**, no targets distintos (doc-comment `SeedCounts` + comentario del loop en `import_md_dir` + frase en la doc MD) | revisión de código + runs O2/O3 |
| — | fmt scoped de los 6 archivos tocados | `rustfmt --edition 2021 --check <files>` → **exit 0 ✅** |
| — | Regresión del crate completo tras el batch | `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` → **567/567 ✅** (566 previos + CRLF test) |

## Review (GATE — agente distinto, P2-01)

> Tier mecánico: diff toca `docs/api/**` (adversarial) + consumo de `src/sdk/serialization` → **adversarial** por glob. Reviewer: `vanta-review`.

- **Revisor:** `vanta-review` (agente distinto — LEAD)
- **Veredicto:** ✅ **APPROVE** — 0 Critical/Required; 3 Optional (O1-O3) + 1 Nit (N1) + 1 follow-up (F1)
- **Disposición:** batch O1/O2/O3/N1/F1 **aplicado** (evidencia en §Verificación); sin hallazgos abiertos
- **Pendiente LEAD:** commit local + re-run workspace gate en árbol convergido

## Notas

- **NO commit / NO self-review** (wave F4.2). `git add`/commit = LEAD.
- **Evidencia de verificación (resumen):**
  - E2E real: `vanta-memory/tests/md_git_e2e.rs` → `md_git_flow_export_edit_import_rebuild_search` (2/2 con `CARGO_BIN_EXE_vanta-seed`) + `md_export_is_byte_identical_across_runs`.
  - Unit import: **12/12** (`-p vanta-memory --lib -E 'test(md_import)'`, incluye CRLF); roundtrip: 2/2; export: **6/6** (`-p vantadb --lib -E 'test(export_md)'`); seed literal: 4/4.
  - Review batch P2-01 (O1/O2/O3/N1/F1): evidencia por ítem en §Verificación; full `-p vanta-memory` post-batch **567/567**.
  - Full por crate: `-p vanta-memory` 566/566; `-p vantadb` 2533/2534 (**única falla: `public_api_snapshot_matches_committed_file` por items `attestation` de VER-02** — el snapshot lista consts sin valor, `MD_EXPORT_SCHEMA_VERSION` 1→2 no lo afecta; VER-02 debe regenerar su snapshot).
  - `cargo fmt --check` 0 · `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 · `validate-docs-coverage` 0 gaps · OCR advisory (Rule Group 1) sin Critical/High.
  - **Workspace-wide nextest (`--workspace --build-jobs 2`) NO corrido completo**: `link.exe LNK1180` sin espacio en disco (C: 0 free transitorio por builds co-batch paralelos; recuperado ~8.5 GB). El run workspace re-linkea todos los binarios (unificación de features) → diferido al cierre de wave del LEAD. El diff no toca proxy/server/mcp/python/wasm.
- **RED no ejecutable en ventana limpia:** el árbol compartido estuvo rojo por VER-02 (attestation.rs / cli.rs) durante la fase de tests; el RED lógico está documentado en el task file (v1 no tenía los campos v2; el import rechazaba `schema_version != 1` — evidencia: md_import.rs original :88-93 y Frontmatter v1 :61-75 en el diff).
- **`parse_md_file` pasa a privada** (0 consumidores fuera del módulo — grep). Evita nueva superficie y permite retorno rico (`ParsedMd { record, links }`).
- **Determinismo de import:** el hash payload-only anterior ignoraba ediciones de metadata/frontmatter (skip silencioso); la comparación nueva es proyección completa (payload+metadata+v2+timestamps) → las ediciones de frontmatter se propagan. Comportamiento corregido, alineado al contrato.
- **`ponytail:`** el strip del bloque related acepta líneas extra no-bullet (best-effort) — techo: si un payload de usuario contiene el marcador exacto + `## Related`, se recortaría; documentado en el código.
- **Fuera de scope anotado:** `sparse_vector`/vector denso no viajan en MD (documentado desde MEM-62; `vector_dim` es informativo) — re-importar un export MD sigue dejando `vector=None` en ese registro (comportamiento pre-existente, no regresión).
- **Docs session del owner (co-conflicto, no tocado):** `node scripts/docs/gen-index.mjs --check` reporta `docs/index.md`/`llms.txt` stale — **verificado pre-existente en HEAD** con experimento `git archive HEAD` (stale también sin mis cambios; su edit de `scripts/docs/lib.mjs`+archivos de research lo causa). NO regenerado (fuera de scope; lane docs).
- **Co-batch (no tocado):** `src/shred/**`, `src/gc.rs`, `src/storage/engine/delete.rs`, `src/attestation.rs`, `src/lib.rs` (VER-02); `vanta-proxy/**`, `src/crypto.rs` (VER-03). Edits quirúrgicos releyendo cada archivo antes de editar.
