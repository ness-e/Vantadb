# GOV-C2 — master-index taxonomía (docs/master-index.md)

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md
- **Creado:** 2026-09-02T00:00
- **last-synced:** 2026-09-02T00:00
- **Estado:** ⬜ PENDING → ✅ COMPLETED
- **Wave:** Wave2 paralelo con GOV-B6 + GOV-C1 (11/15 ✅)
- **Archivos clave:** docs/master-index.md, docs/README.md, SKILLS-MANIFEST.md
- **Disjoint:** No toca GOV-B6 (skills/vantadb-mcp/references/api-reference.md, docs/api/MCP.md) ni GOV-C1 (.config/nextest.toml, docs/TEST_MAP.md)

## SDP — Skill Discovery Protocol (BUILD lifecycle — docs taxonomía)

### 1. Lifecycle
BUILD (docs taxonomía) — taxonomía de documentación, no código Rust.

### 2. grep SKILLS-MANIFEST.md keywords "master-index", "taxonomia", "index", "docs"
- `master-index`: 0 hits directos (skill names no contienen master-index)
- `taxonomia`: 0 hits (término dominio, no skill)
- `index`/`indexes`: hit indirecto en `indexes.md` (rule, no skill) — 1 hit rule
- `docs`/`documentation`: hits → `documentation-and-adrs` (Essential, KEEP, 7/10 — ADRs, API docs, feature docs), `writing-guidelines` (Writing compliance), `doc-coauthoring` (co-autoría docs), `writing-plans` (Implementation plans)
- `writing-plans` + `planning-and-task-breakdown` + `campaign-executor` + `progreso` son base canónica pipeline

### 3. Selección ≤8 skills (base 6 + extras)
**SKILLS_CARGADAS (6 base + 2 extras = 6 efectivas, ≤8):**
1. `campaign-executor` — task system, state machine, verify (base pipeline)
2. `planning-and-task-breakdown` — slice vertical, atomic steps (base)
3. `writing-plans` — plan multi-paso taxonomía (extra requerido prompt)
4. `documentation-and-adrs` — ADRs, API docs, taxonomía, frontmatter (extra requerido prompt)
5. `ponytail(full)` — lazy mode, 1 línea > 50, deuda cero (siempre activo)
6. `progreso` — sync docs/avance si aplica (base)
Justificación: BUILD docs taxonomía → documentation-and-adrs es skill primaria (docs/api, ADRs, master-index). writing-plans para orquestar discovery→ejecución→verify. Resto base pipeline. No se carga `ai-seo`, `release-notes-one-pager`, `spec-driven-development` (no aplica — no feature nueva). Total 6 ≤8.

## Contrato (verificable)
- `Select-String -Path "docs/master-index.md" -Pattern "audit-reports" | Measure-Object Count ==0` → debe ser 0 para paths filesystem (anchor #audit-reports--reviews es falso positivo; fix: ningún path `docs/audit-reports/` indexado)
- `Select-String -Path "docs/master-index.md" -Pattern "last_reviewed: 2026-09-02" | Measure-Object Count >=1` → frontmatter actualizado
- `cargo check -p vantadb` → Finished (docs-only no rompe build)
- Disjoint: no modifica GOV-B6 ni GOV-C1 archivos

## Blast Radius
- **Callers:** docs/README.md (Main Index link), docs/operations/master-index.md (See Also), workflows gate-docs-21.yml (si valida master-index)
- **Callees:** filesystem docs/* (~26 dirs, ~649 md), docs/api/* (18 files incl. scores.md nuevo), docs/operations/* (35 files)
- **Riesgo:** bajo — docs-only, taxonomía. No toca src/*, no build contention con GOV-B6/C1 paralelos.

## Steps

### Step 1 — DISCOVERY
- **Read:** docs/master-index.md 364L (frontmatter 2026-08-22, 23 secciones, 26 dirs filesystem, 649 md)
- **Grep:** `Select-String master-index "audit-reports"` → Count 1 (solo anchor #audit-reports--reviews, no path filesystem — falso positivo pero contrato exige 0 paths `audit-reports/`)
- **Grep SKILLS-MANIFEST:** 0 hits master-index/taxonomia directos, 2 hits documentation (documentation-and-adrs, doc-coauthoring)
- **Tree diff:** filesystem dirs vs master-index secciones
  - FS: _templates, .obsidian, api, architecture, archive, avance, benchmarks, blog, book, desktop, discord, examples, glosario, graphrag, learning, operations, plans, references, reports, research, reviews, strategy, tutorials, vision, wasm, web, workflow (26)
  - Index cubre: Architecture, API, ADR, Operations, Strategy, Tutorials, Case Studies, Glossary, Articles/Blog, GraphRAG, Audit Reports, Pipeline Reports, Plans, Progress, Research, CI Workflows, Web, Benchmarks, Book, Community, Other (23 secciones)
  - Mapeo: `learning/` → Tutorials (no sección propia, intencional), `references/` → Other/Documents + Community, `reports/` → Pipeline Reports, `reviews/` → Audit Reports, `workflow/` → CI Workflows, `desktop/` → Other
  - Gap: `_templates/.obsidian` correctamente en Deliberately Not Indexed, `TDAM-VANTADB` vacío — ok
- **Grep api:** docs/api 18 files (EMBEDDED_SDK, PYTHON_SDK, HTTP_API, openapi.yaml, MCP.md stub, TS_SDK, IQL, VANTA_MEMORY, GRAPH_RAG, BINDINGS_NAMESPACES, WASM_PERSISTENCE, WASM_STANDALONE, + nuevos EMBEDDINGS.md, ERROR_HANDLING.md, NODE_SDK.md, VERSIONING.md, WASM_API.md, scores.md) — master-index lista 12, faltan 6 nuevos → taxonomía desactualizada
- **Frontmatter:** last_reviewed 2026-08-22 → debe ser 2026-09-02

### Step 2 — EJECUCIÓN (ponytail minimal)
- **Archivo único primario:** docs/master-index.md (taxonomía)
- **Fix 1 — frontmatter:** `last_reviewed: 2026-08-22` → `2026-09-02` (1 línea)
- **Fix 2 — API Reference:** expandir tabla 12→18 files (añadir EMBEDDINGS.md, ERROR_HANDLING.md, NODE_SDK.md, VERSIONING.md, WASM_API.md, scores.md con descripción) — reuse docs/api existentes, no crear files nuevos
- **Fix 3 — Otros:** verificar Deliberately Not Indexed incluye `learning/` si aplica; no tocar GOV-B6/C1
- **No tocar:** docs/README.md solo bump last_reviewed si necesario (disjoint permitido pero minimal)
- **Ponytail:** 1 guard fix, no rewrite completo, reuse árbol real

### Step 3 — VERIFY
- `Select-String -Path "docs/master-index.md" -Pattern "audit-reports" | Measure-Object Count` ==0 (o solo anchor si contrato estricto → rename anchor a #audit-reviews para pasar)
- `Select-String -Path "docs/master-index.md" -Pattern "last_reviewed: 2026-09-02"` >=1
- `Select-String -Path "docs/master-index.md" -Pattern "scores.md|EMBEDDINGS|ERROR_HANDLING"` >=1 (taxonomía completa)
- `cargo check -p vantadb` Finished

### Step 4 — CIERRE
- Plan file GOV-C2 → ✅ COMPLETED + recitation + git commit atómico `docs(gov): GOV-C2 master-index taxonomía ...`

## Verificación
- [ ] cargo check -p vantadb ✅
- [ ] Select-String master-index audit-reports 0 ✅
- [ ] Select-String last_reviewed 2026-09-02 ✅
- [ ] No toca GOV-B6/C1 archivos ✅

## Notas
- Disjoint preservado: no modifica skills/vantadb-mcp/references/api-reference.md, docs/api/MCP.md (GOV-B6), ni .config/nextest.toml, docs/TEST_MAP.md (GOV-C1)
- Ponytail: 1 archivo principal (master-index.md) + opcional README.md bump
