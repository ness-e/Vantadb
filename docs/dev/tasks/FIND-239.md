---
title: "FIND-239: Docs DX — encoding Windows + puente de formatos de import + ejemplo get_node + caveats WASM"
kind: task
description: "Cuatro gaps de docs medidos en el flujo usuario-real (validación externa v0.8.0): nota UTF-8/chcp en QUICKSTART, puente export_all (JSONL) ↔ import_file vs bulk_import* (.vdbdump) en PYTHON_SDK, ejemplo explícito node[\"fields\"][\"content\"], y caveat por método en las tablas Maintenance/Export de vantadb-ts."
---

# FIND-239: Docs DX — encoding Windows + puente de formatos de import + ejemplo `get_node` + caveats WASM

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 3, F0)
- **Fuente:** Backlog fila FIND-239 · validación externa v0.8.0 (2026-10-03)
- **Esfuerzo:** 🟢 3-4h
- **Prioridad:** 🟡
- **Tipo:** Docs (docs-only — sin cambios de código; el hint del magic va como FIND-240)
- **Turns estimados:** 5-10
- **Creado:** 2026-10-04T02:17
- **last-synced:** 2026-10-04T02:17
- **Estado:** ⏳ IN PROGRESS — 6/6 steps ✅ + commit local; pendiente review P2-01 (vanta-review, orquestador)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (review P2-01 delegado al orquestador)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Docs sin consumidores de código. Enlaces entrantes a los 3 archivos (grep): `README.md:47`, `README_ES.md:43`, `SUPPORT.md:11`, `docs/user/index.md:100`, `docs/index.md`, `llms.txt`, `examples/README.md:5`, `vantadb-python/README.md:120`, `vantadb-wasm/README.md:279-280`, `docs/user/tutorials/index.md:46`, `docs/user/glosario/{similar_to_key,put_batch}.md` — todos a nivel archivo/sección; ninguno depende del contenido exacto editado |
| Callees | QUICKSTART → `examples/README.md`, `docs/api/EMBEDDINGS.md`, `docs/user/tutorials/05-embedding-integrations.md`; PYTHON_SDK → `EMBEDDED_SDK.md`, `BINDINGS_NAMESPACES.md`, `VERSIONING.md`, ADR-0030; TS README → `../vantadb-wasm/README.md`, `../docs/user/QUICKSTART.md#4-real-embeddings-optional`, ADR-0030. Links NUEVOS: `./EMBEDDED_SDK.md#bulk-import`, `./EMBEDDED_SDK.md#export--import`, `#wasm-bundle--lazy-loading`, `#vantadb-vs-vantadb-node-npm` — verificados por `check-links` |
| Implicaciones | Docs-only: sin cambio de API, comportamiento, performance ni migración. Gates que cubren el diff: `check-links` (44/58 broken budget — no agregar roto), `check-docs` (gating clear), `gen-index --check` (stale pre-existente por FIND-237/238), `check-doc-examples` (los snippets editados usan API real verificada) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/user/QUICKSTART.md` (305L) · `docs/api/PYTHON_SDK.md` (~1323L) · `vantadb-ts/README.md` (376L).
- **Evidencia de código leída (para que los ejemplos sean ejecutables):** `vantadb-python/src/lib.rs` (`insert_node` :624-681, `get_node` :1619-1633, `export_all`/`import_file`/`bulk_import*` :1478-1553) · `vantadb-python/src/convert.rs:287-323` (`node_to_pydict` → shape real del dict) · `vantadb-python/tests/test_sdk.py:75-139` (`node["fields"]["content"]` es el shape testeado) · `src/sdk/api/memory.rs:1457-1510` (`bulk_import_stream`: magic `VDBJSON\n` + error de magic) · `src/sdk/api.rs:718-727` (test del magic: solo asserta `field == "header"`) · `src/sdk/serialization/impl_export.rs:246-272, 388-447` (`export_all`/`import_file` = JSONL) · `vantadb-wasm/src/lib.rs:1522-1667` (export/import + flush H-05) · `vantadb-wasm/tests/wasm_tests.rs:851-872` · `vantadb-ts/src/vantadb.ts:170-254` · `vantadb-ts/src/native.ts:120-249` · `vantadb-ts/src/__tests__/portability.test.ts:1-166` · `docs/api/EMBEDDED_SDK.md:80-125, 500-521`.
- **Archivos referenciados hacia dentro (imports/deps):** n/a (Markdown). Links salientes listados en Blast Radius Callees.
- **Archivos que referencian a los editados (referencias entrantes):** grep de links entrantes (Blast Radius Callers) — ninguno acoplado a las secciones editadas (todos apuntan a archivo o sección estable).
- **Veredicto impacto:** **bajo** — ediciones aditivas en prosa/ejemplos/tablas de 3 docs; sin cambios de contrato; gates de links/schema cubren.

## Contrato

"`node scripts/docs/check-links.mjs` exit 0 **y** `node scripts/docs/check-docs.mjs` exit 0, con los 4 puntos verificados:

- **(a)** `QUICKSTART` §5 con nota de encoding Windows (`chcp 65001` / `$env:PYTHONIOENCODING="utf-8"`) para el fallo medido (PowerShell rompe tildes/flechas al imprimir salida no-ASCII).
- **(b)** `PYTHON_SDK.md` cruza formatos: `export_namespace()`/`export_all()` = JSONL → `import_file()`; `bulk_import()`/`bulk_import_bytes()` = binario `.vdbdump` (magic `VDBJSON\n`) + link a `EMBEDDED_SDK.md` §Bulk Import (L94/102) y §Export / Import (L516-517). Hint del error de magic: **diferido a FIND-240** (docs-only por diseño; el cambio Rust es adversarial tier y va separado).
- **(c)** ejemplo explícito `node["fields"]["content"]` en §`get_node` (shape verificado contra `test_sdk.py:83`).
- **(d)** caveat por método en las 2 tablas TS (Maintenance / Export-Import) de `vantadb-ts/README.md`, enlazando §WASM y §vantadb vs vantadb-node (enlazar, no duplicar)."

## Spec (docs-only — micro-decisiones resueltas por evidencia)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Hint en error de magic | A: cambio Rust en `src/sdk/api/memory.rs:1472-1479` + test (adversarial tier, otro verify/commit) / B: FIND derivado | B | ✅ B — pre-mortem del plan lo separa ("docs ahora; hint opcional") y la stop-condition lo permite; commit esperado `docs:` (plan L122). Registrado como **FIND-240** |
| 2 | Colocación nota encoding | A: §5 junto a "Run it:" / B: §0 Windows install | A | ✅ A — el fallo medido ocurre al ejecutar el ejemplo Python; §0 es solo instalación |
| 3 | Forma de los caveats TS | A: 3ª columna "WASM caveat" por método / B: footnotes al pie de tabla | A | ✅ A — "caveat por método" literal; la sección WASM del README (L82-138) y la tabla `vantadb vs vantadb-node` (L154-163) se enlazan, no se duplican |
| 4 | Regenerar índices | A: `gen-index --write` + commit del diff / B: diferir | A | ✅ A — convención del repo: commit `1bc550b2` agregó plan + regeneró `docs/index.md`/`llms.txt` en el mismo commit |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** docs-only (no tocar código — el hint del magic queda fuera); los links nuevos deben resolver (gate); frontmatter `title`/`kind` intactos; los ejemplos editados deben usar API real (gate `check-doc-examples`); no introducir claims sin evidencia (Regla 11 — no hay números nuevos).
- **Comandos de verificación:** `node scripts/docs/check-links.mjs` → exit 0 · `node scripts/docs/check-docs.mjs` → exit 0 · `node scripts/docs/gen-index.mjs --write` (commit del diff).
- **Deuda pendiente:** FIND-240 (hint del magic error — fila en Backlog); staleness de `gen-index` pre-existente por FIND-237/238 (se resuelve con el `--write` del cierre).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-239: ...` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | FIND-233 (siguiente del plan F0) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda. Docs-only; el único shortcut deliberado (hint del magic no implementado) queda con ID de backlog (FIND-240) — deuda visible, no silenciosa.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato a-d verificado + gates docs `0/0` (`check-links` / `check-docs`) + revisión de cada punto |
| **Commit** | Atómico `docs:`, verificación mecánica vía `campaign_verify_cmd` (nunca auto-reporte) |
| **Release** | n/a — docs-only, sin entrada de changelog (no user-visible a nivel release) |

## Herramientas necesarias
- `campaign_verify_cmd` (gates), `campaign_update_task_state` (recitation), `campaign_memory_write` (lecciones)
- `codegraph_codegraph_explore` / `codebase-memory-mcp_check_index_coverage` (blast radius + cobertura)
- Gates docs: `node scripts/docs/check-links.mjs` · `check-docs.mjs` · `gen-index.mjs --write`
- OCR delegation: `pwsh dev-tools/ocr-review.ps1` (cierre)

**Skills cargadas (SDP v3):** `documentation-skill` (obligatoria — docs/), `documentation-and-adrs` (pin SDP: API docs), `api-and-interface-design` (pin SDP: API docs), `writing-guidelines` (sugerida por el plan), `security-and-hardening` (pin SDP: trust boundary — cargada; checklist N/A justificado: docs-only), `source-driven-development` (base SDP: ejemplos validados contra fuente real). Base `campaign-executor`/`progreso`/`ponytail` auto-cargadas vía MCP. Candidatas SDP no cargadas: `incremental-implementation`, `test-driven-development` (no aplican a docs-only — sin lógica nueva).

## Investigation Notes

- **`get_node` shape (verificado):** `node_to_pydict` (`vantadb-python/src/convert.rs:287-323`) arma `id`/`confidence_score`/`importance`/`hits`/`last_accessed`/`epoch`/`tier`/`is_alive`/`vector`/`vector_dims`/`fields`/`edges`; el `content` de `insert_node` cae en `fields["content"]` (test real: `vantadb-python/tests/test_sdk.py:83` y `:138`).
- **Formatos de import (verificado):** `export_all`/`export_namespace` escriben JSONL (`impl_export.rs:246-272`); `import_file` lee JSONL línea a línea (`impl_export.rs:404-447`); `bulk_import_stream`/`bulk_import_file` esperan binario: magic `VDBJSON\n` + version `0x01` + count LE + `Vec<MemoryInput>` JSON (`src/sdk/api/memory.rs:1459-1460`); error de magic en `:1472-1479`.
- **Hint del magic (diferido):** el mensaje actual es `invalid magic bytes: expected VDBJSON\n, got {:?}` sin referencia a `import_file`. El test `test_bulk_import_stream_invalid_magic` (`src/sdk/api.rs:718-727`) solo asserta `field == "header"` → un hint no rompe tests, pero es cambio en `src/sdk/**` (adversarial tier) con verify propio → FIND-240.
- **Caveats WASM (verificados):** `flush()` no es garantía de durabilidad en browser — warning H-05 y requiere `save()`/`save_idb()` (`vantadb-wasm/src/lib.rs:1636-1651`); `exportAll`/`exportNamespace`/`importFile` lanzan `IO error: operation not supported on this platform` en el runtime WASM, incluso en Node (no hay `std::fs`) — `vantadb-ts/src/__tests__/portability.test.ts:130-163` (FIND-79, round-trip real diferido); el wrapper nativo TS (`native.ts:120-249`) NO expone export/import → no recomendar el native backend para file round-trips; usar CLI/Python.
- **Gates baseline (pre-edición):** `check-links` exit 0 (44/58 broken, dentro de budget) · `check-docs` exit 0 (gating clear) · `gen-index --check` **rojo pre-existente** (FIND-237/238 task files sin indexar — no causado por esta tarea; el `--write` del cierre incluirá sus filas, convención `1bc550b2`).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — shape, formatos y caveats verificados contra fuente real |
| Pendientes de ejecución (downhill) | 0 — steps 1-6 ✅ (review P2-01 delegado al orquestador) |
| % completado | 100% mecánico — pendiente veredicto de review P2-01 |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY — no aplica (justificado):** docs-only; sin input de usuario, auth, storage, FFI ni dependencias nuevas. `security-and-hardening` (pin SDP) cargada; su checklist no tiene superficie que aplicar.
- [x] **PERFORMANCE — no aplica (justificado):** sin hot paths ni serialización; solo prosa/ejemplos/tablas.

## Steps

### Step 1: Task file + FIND-240 (artefactos de discovery)
- **Archivos:** `docs/dev/tasks/FIND-239.md`, `docs/dev/Backlog.md`
- **Acción:** crear el task file canónico; registrar el hallazgo derivado (hint del magic) como fila `FIND-240` en Backlog (findings.md: al discovery).
- **Verify:** task file existe; fila FIND-240 en el esquema 10-col.
- **Estado:** ✅ COMPLETED

### Step 2: QUICKSTART — nota de encoding Windows
- **Archivos:** `docs/user/QUICKSTART.md` (§5, tras el script Python / antes de "Run it:")
- **Acción:** callout con el fallo medido + `chcp 65001` + `$env:PYTHONIOENCODING="utf-8"` + one-liner `python -X utf8`.
- **Verify:** `node scripts/docs/check-links.mjs` + `check-docs.mjs` (exit 0).
- **Estado:** ✅ COMPLETED

### Step 3: PYTHON_SDK — puente de formatos de import
- **Archivos:** `docs/api/PYTHON_SDK.md` (§`bulk_import`, §`bulk_import_bytes`, §`export_namespace`, §`export_all`, §`import_file`)
- **Acción:** notas de formato JSONL vs `.vdbdump` (magic `VDBJSON\n`) + links a `EMBEDDED_SDK.md#bulk-import` / `#export--import`.
- **Verify:** gates + anclas resuelven.
- **Estado:** ✅ COMPLETED

### Step 4: PYTHON_SDK — ejemplo explícito `fields["content"]`
- **Archivos:** `docs/api/PYTHON_SDK.md` (§`get_node`)
- **Acción:** prosa + ejemplo con `node["fields"]["content"]` (shape testeado en `test_sdk.py:83`).
- **Verify:** gates; coherencia con `convert.rs:287-323`.
- **Estado:** ✅ COMPLETED

### Step 5: TS README — caveats por método en Maintenance / Export-Import
- **Archivos:** `vantadb-ts/README.md`
- **Acción:** 3ª columna "WASM caveat" en ambas tablas (flush durabilidad H-05; export/import `operation not supported`; `importRecords` OK) + nota de round-trips con links internos (§WASM, §vantadb vs vantadb-node).
- **Verify:** gates + anclas internas resuelven.
- **Estado:** ✅ COMPLETED

### Step 6: CIERRE — regen de índices + verify full + OCR + commit local
- **Archivos:** `docs/index.md`, `llms.txt` (generados), `docs/dev/tasks/FIND-239.md`
- **Acción:** `gen-index --write` (commit del diff); `campaign_verify_cmd` de los 2 gates del contrato; OCR delegation (`pwsh dev-tools/ocr-review.ps1`); commit **LOCAL** `docs: FIND-239 — ...` (solo archivos propios; nunca push); recitation.
- **Verify:** gates `0/0` + OCR sin Critical/High + `git show --stat` limitado a los archivos propios.
- **Estado:** ✅ COMPLETED

## Dependencias
- F0 wave 0 — sin dependencias. nextTask: FIND-233.

## Review (GATE — agente distinto, P2-01)

> Leaf (`vanta-docs`, sin capacidad de spawn): la evidencia completa queda abajo para que el **orquestador** corra `vanta-review` (tier **adversarial** — el diff toca `docs/api/**`). Sin veredicto registrado la tarea NO se marca COMPLETED (HARD-07).

- **Revisor:** ⬜ pendiente — orquestador (`vanta-review`)
- **Enfoque:** (1) ¿los 4 puntos a-d cierran el gap medido sin scope creep a código? (2) ¿los caveats TS son fieles al runtime real (no recomiendan métodos inexistentes)? (3) ¿el puente de formatos no induce a error (JSONL vs binario)?
- **Cómo se probó:** gates mecánicos `check-links`/`check-docs` (exit 0) + evidencia de código citada por claim en Investigation Notes (rutas:línea verificables) + OCR delegation como input.
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (todo comando ejecutado y registrado); sin "done" sin gates; sin scope creep (magic hint separado a FIND-240); sin huérfanos (cada step conectado al contrato).
- **Veredicto:** ⬜ pendiente del orquestador

## Notas

- **Pre-mortem del plan cubierto:** (1) tocar el error de magic = cambio de código → separado a FIND-240; (2) ejemplo que no corre → validado contra SDK real (`test_sdk.py:83`, `convert.rs:287-323`); (3) links relativos rotos → gates.
- **Scope (a):** nota en QUICKSTART §5 únicamente (donde corre el ejemplo Python). El README raíz no tiene ejemplos ejecutables con salida no-ASCII → sin nota allí (evita duplicación).
- **`gen-index`:** staleness pre-existente por FIND-237/238; el `--write` del cierre regenera (incluirá sus filas + la de FIND-239). Convención: commit `1bc550b2` (plan + índices en el mismo commit).
- **NOTICED BUT NOT TOUCHING:** `PYTHON_SDK.md` frontmatter tiene keys retiradas (`type`, `last_reviewed`, `related`) — `check-docs` las reporta como `unknownKeys` (353 archivos; reporting-only, no gated). Fuera de scope.
- **NOTICED BUT NOT TOUCHING:** `compactWal()`/`compactLayout()` en WASM no lanzan ni advierten (tests verdes); solo `flush` y el trío export/import tienen comportamiento divergente medido → caveats ahí.
- **validate_scope (advisory):** 3/3 archivos del contrato ✅; `FIND-239.md`/`Backlog.md` OUT_OF_SCOPE (artefactos del task-system no listados en "Archivos clave" del plan — mandato de pipeline-full §Discovery y findings.md).
- **OCR delegation:** `pwsh dev-tools/ocr-review.ps1 -Format json` → 0 archivos propios reviewables (docs excluidos por `unsupported_ext`); los 5 reviewables del preview (`src/cli.rs`, `tests/cli_tests.rs`, `src/bin/vanta-cli.rs`, `vantadb-wasm/Cargo.toml`, `vantadb-wasm/src/lib.rs`) pertenecen a FIND-237/238 (paralelos, fuera de scope) → sin Critical/High atribuibles a FIND-239.
- **gen-index:** el diff regenerado de `docs/index.md` (2091 líneas) es churn mecánico de tabla (3 filas nuevas + reflow de columnas por el ancho máximo); incluye las filas de FIND-237/238 (task files en disco, en curso por agentes paralelos). `llms.txt` solo cambia el conteo (1039→1042).

## Context Save Point

- **Última acción:** Steps 1-6 ✅ — 3 docs editados (QUICKSTART §5 encoding; PYTHON_SDK bridge JSONL↔`.vdbdump` + ejemplo `fields["content"]`; TS README caveats por método), FIND-240 en Backlog, índices regenerados (`gen-index --write` → `--check` exit 0), gates `check-links`/`check-docs` exit 0 vía `campaign_verify_cmd`, smoke real del shape (`vantadb` 0.7.0: `get_node(42)["fields"]["content"]` ✅), `check-doc-examples` exit 0 (budget 1/1 pre-existente), OCR sin archivos propios reviewables (docs excluidos por `unsupported_ext`). Commit local `docs: FIND-239 — ...`.
- **Próximo step:** review P2-01 adversarial (tier `docs/api/**`) → `vanta-review` por el orquestador → ACCEPT (HARD-07).
- **Archivos en vuelo:** ninguno.

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 6/6
PROXIMO_STEP: review P2-01 adversarial (vanta-review) → ACCEPT por orquestador
COMMIT_HASH: docs: FIND-239 — ver git log (commit local; nunca push)
ARCHIVOS: docs/user/QUICKSTART.md, docs/api/PYTHON_SDK.md, vantadb-ts/README.md, docs/dev/tasks/FIND-239.md, docs/dev/Backlog.md (FIND-240), docs/index.md + llms.txt (generados)
VERIFY_CONTRATO: pasa (check-links exit 0 · check-docs exit 0)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no (docs-only sin símbolos públicos) D:no (plan F0 full-detail aprobado) V:no (sin fallas de verify) C:no (sin hallazgos colaterales nuevos)
SKILLS_CARGADAS: documentation-skill · documentation-and-adrs · api-and-interface-design · writing-guidelines · security-and-hardening · source-driven-development (+ base campaign-executor/progreso/ponytail auto vía MCP)
```
