# FIND-83: Unificar skills (copias + MCP-27/29 + api-ref)

## Metadata
- **Plan file:** `docs/plans/2026-09-15-find-correcciones.md` (Task 15, Wave4)
- **Fuente:** Backlog FIND-83 + plan §"Verificación real global" (línea 52)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Docs (skills + manifest + gate script; 0 líneas Rust)
- **Turns estimados:** 12
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas (3 cerradas en DISCOVERY con evidencia de código)
- **Pendientes (downhill):** 5 steps (4 merge + verify/commit; review P2-01 queda al orquestador)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Consumidores de skills: agentes/OpenCode (resolución `.opencode/skills/` → global), `scripts/validate-docs-coverage.ps1` (nueva sección 7), `SKILLS-MANIFEST.md` (referencia) |
| Callees | Código fuente solo como evidencia read-only: `src/error.rs` (nombres `VantaError`), `src/physical_plan/scan.rs` (MCP-29), `src/sdk/serialization/mod.rs::iql_table_name_for_namespace`, `docs/api/MCP.md:129` (tabla `-320xx`) |
| Implicaciones | Ningún contrato público cambia (docs-only); sin cambio de comportamiento, perf, memoria, serialización; sin migración; tests existentes no afectados (gate script sale 0) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `skills/vantadb/SKILL.md` (789L), `skills/vantadb-mcp/SKILL.md` (599L), `.opencode/skills/vantadb/SKILL.md` (791L), `.opencode/skills/vantadb-mcp/SKILL.md` (599L), `skills/vantadb-mcp/references/api-reference.md` (521L), `.opencode/skills/vantadb-mcp/references/api-reference.md` (521L), `SKILLS-MANIFEST.md` (601L), `scripts/validate-docs-coverage.ps1` (197L)
- **Archivos referenciados hacia dentro:** los SKILL.md referencian `docs/api/MCP.md`, `docs/architecture/adr/`, `examples/python/`, `docs/archive/case-studies-unverified/` — solo lectura de existencia (Test-Path), sin editarlos
- **Archivos que referencian a los editados:** `SKILLS-MANIFEST.md` (lista `vantadb`, `vantadb-mcp` KEEP), `AGENTS.md` (regla de resolución skills), plan file Task 15; ningún `.rs`/`.py` importa skills en runtime (son docs para agentes)
- **Veredicto impacto:** BAJO — merges aditivos (0 líneas borradas con contenido único), 8 archivos docs-only en 2 repos (parent + submodule working tree)

## Contrato

"`Get-FileHash` 10/11 pares SAME (excepción `test-mcp.py` owned by FIND-82) + `git diff --check` limpio + `pwsh scripts/validate-docs-coverage.ps1` exit 0 (nueva sección 7 verde) + `SKILLS-MANIFEST.md` con conteo 196 y nota mirror"

## Spec (SDD — Phase 1b)

No es feature-add: 0 símbolos públicos nuevos (sin `pub fn`, tool MCP, endpoint, binding). Decisiones técnicas cerradas por evidencia:

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dirección sync error-channel `vantadb-mcp/SKILL.md` | A: `skills/`→`.opencode/` (párrafo ERR-MCP-01 completo) / B: inverso (borraría contenido) | ✅ A por evidencia: commit `0ce791ce` + tabla `docs/api/MCP.md:129` existe |
| 2 | Dirección sync enum `api-reference.md` | A: `skills/`→`.opencode/` (nombres cortos) / B: inverso (`*Error`, no compila contra código) | ✅ A por evidencia: `src/error.rs:142,157,161,196,216,288` nombres cortos |
| 3 | MCP-27 vs MCP-29 (contradicción real) | A: MCP-29 supersedea (unión) / B: MCP-27 vigente / C: Gate V al dueño | ✅ A por evidencia: `src/physical_plan/scan.rs:81-92,185-194` + tests sanitización/unión + `src/sdk/serialization/mod.rs:66-70` |
| 4 | Dirección sync `vantadb/SKILL.md` (doble) | MCP-29: `.opencode/`→`skills/`; case-studies: `skills/`→`.opencode/` | ✅ merge bidireccional por evidencia: `docs/case_studies/` NO existe, `docs/archive/case-studies-unverified/` SÍ (3 files) |
| 5 | Gate persistente | A: sección 7 en `validate-docs-coverage.ps1` / B: script nuevo / C: solo documentar comando | ✅ A: es el gate citado por FIND-76 (EXIT 0), aditivo, ~30 líneas |
| 6 | Commit submodule `.opencode/` | A: commitear dentro / B: dejar working-tree + lead hace bump | ✅ B: submodule con WIP ajeno (`AGENTS.md`, `rules/`, `task-system/`); precedente parent `chore: bump opencode submodule` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** merge nunca overwrite (contenido único de cada copia sobrevive); `test-mcp.py` NO se toca (owned by FIND-82); prohibidos intactos (`.opencode/` fuera de `skills/`, `completions/`, tauri lock, stash@{0}, FIND-77/82, código Rust); Backlog y plan file NO se editan (race Wave4 paralela — solo task file propio + commit parent)
- **Comandos de verificación:** `pwsh scripts/validate-docs-coverage.ps1` (exit 0) + `git diff --check` + `Get-FileHash` 10 pares
- **Deuda pendiente:** review P2-01 por agente distinto (sin sub-agente disponible en este runner); commit submodule + bump a cargo de vanta-lead; `test-mcp.py` divergente hasta FIND-82

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-83 — unificar skills (copias + MCP-27/29 + api-ref) |
| `lastAction` | DISCOVERY + task file (ver Steps) |
| `result` | PARTIAL (implementación+verify+commit parent; review y bump pendientes) |
| `nextAction` | Review P2-01 (vanta-docs/vanta-review) + bump submodule (vanta-lead) |
| `contract` | ver §Contrato + evidencias abajo |
| `nextTask` | FIND-67 (Wave5, disjunta) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda: 0 líneas de código productivo; gate script aditivo (~30L ps1); saldo neto 0.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable (§Contrato) + `validate-docs-coverage` exit 0 |
| **Commit** | Commit atómico parent `docs:` (4 paths propios), submodule sin commitear (decisión 6), `git diff --check` limpio |
| **Release** | N/A (docs-only, sin semver/changelog; justificar: sin cambio de comportamiento) |

## Herramientas necesarias

- bash pwsh (hash/diff/test), edit/write (docs), grep/read (evidencia)
- codegraph_explore (ruido vercel — sin señal; evidencia real vía grep + lectura directa)
- campaign_* (state/discover/verify; `campaign_verify_cmd` con bug exit -1 conocido → bash directa)

**Skills cargadas (SDP):** `documentation-and-adrs` (ADRs/merge-docs, base del rol vanta-docs) · `codebase-memory` (ritual grafos + coverage) · `api-and-interface-design` (contrato api-reference 79 tools) · SDP-v2 BUILD aportó base/lifecycle (`source-driven-development`, `campaign-executor`, `incremental-implementation`, `test-driven-development`, `context-engineering`, `doubt-driven-development`, `frontend-ui-engineering`) — de ellas solo se consume `source-driven-development` (código como fuente de verdad) y `doubt-driven-development` (autorrevisión adversarial); resto N/A docs-only. Keywords: skills-sync, mcp-27, mcp-29, api-reference, manifest.

## Investigation Notes

- **FIND-83 era uphill con 3 incógnitas → 0:** (1) alcance contradicción MCP-27/29 = doc-stale, no feature: MCP-29 implementada (`scan.rs:81-92`, tests unión/sanitización, `mod.rs:66-70`); MCP-27 `:200` ("`SELECT * FROM <namespace>` returns `[]`") refutado por código → se reescribe como alcance extendido, contradicción cerrada. (2) `api-reference` 33→79 ya completa en ambas copias (`## MCP Tools (79)`); único drift = nombres enum `*Error` en copia `.opencode/` refutados por `error.rs`. (3) copias: 7/11 SAME, 4 DIFF (1 owned FIND-82) → colapsa a merge mecánico bidireccional, 0 Gate V (aditivo, sin dueño bloqueando).
- **Pre-mortem confirmado:** cada copia tenía contenido único (overwrite ciego habría borrado ERR-MCP-01 o MCP-29 o paths correctos) → merge bidireccional aplicado.
- **codegraph_explore** devolvió ruido (scanners vercel-optimize) — sin señal para skills-md; evidencia vía grep + Read directo.
- **Manifest:** header decía 194, §Source Locations 193, disco real 196 → se fijan a 196 medido + nota mirror `skills/`↔`.opencode/skills/` + excepción `test-mcp.py`.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (Steps 1-5 ✅; Review pendiente = gate externo, no step ejecutable aquí) |
| % completado | 100% (implementación+verify+commit) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — N/A: docs-only, sin trust boundaries/input/auth/dependencias/storage/FFI/red. Justificado.
- [x] **PERFORMANCE** — N/A: sin hot paths (vector/engine/search/serialización intactos). Justificado.

## Steps

### Step 1: DISCOVERY + task file (Regla 0)
- **Archivos:** este file
- **Acción:** hash 11 pares (4 DIFF), diffs por copia, ground truth código (error.rs/scan.rs/mod.rs), Test-Path case-studies/MCP.md, conteo manifest, alcance Gate D
- **Verify:** evidencias file:línea en Investigation Notes ✅
- **Estado:** ✅ COMPLETED

### Step 2: Merge `vantadb/SKILL.md` + scope MCP-27/29 en ambas `vantadb-mcp/SKILL.md`
- **Archivos:** `skills/vantadb/SKILL.md`, `skills/vantadb-mcp/SKILL.md`, `.opencode/skills/vantadb-mcp/SKILL.md`
- **Acción:** insertar párrafo MCP-29 en `skills/vantadb/SKILL.md:326`; reescribir párrafo Scope MCP-27 en ambas copias con semántica unión; propagar párrafo Error Channels ERR-MCP-01 a copia `.opencode/`
- **Verify:** `git diff --no-index` muestra solo hunks esperados + contenido único preservado ✅ (3 hunks: MCP-29 insert, scope rewrite ×2 copias, error-channel ×2 líneas)
- **Estado:** ✅ COMPLETED

### Step 3: Merge `api-reference.md` + `vantadb/SKILL.md` (case-studies) en `.opencode/`
- **Archivos:** `.opencode/skills/vantadb-mcp/references/api-reference.md`, `.opencode/skills/vantadb/SKILL.md`
- **Acción:** bloque enum ← nombres cortos `error.rs`; línea case-studies ← path existente
- **Verify:** hash-SAME en ambos pares ✅ (+ normalización CRLF en `skills/vantadb/SKILL.md` para byte-SAME)
- **Estado:** ✅ COMPLETED

### Step 4: Manifest al día + gate sección 7
- **Archivos:** `SKILLS-MANIFEST.md`, `scripts/validate-docs-coverage.ps1`
- **Acción:** conteos 196 + nota mirror + excepción FIND-82; sección 7 hash-gate 10 pares
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 ✅ (sección 7: 10 pares hash-SAME)
- **Estado:** ✅ COMPLETED

### Step 5: Verify full + commit parent + lesson
- **Archivos:** todos los tocados
- **Acción:** `git diff --check`, hashes 10/11, commit `docs:` solo paths propios parent, lesson, RESULTADO
- **Verify:** COMMIT_HASH real + `git status` sin propios pendientes (plan/Backlog intactos) — ver RESULTADO del turno
- **Estado:** ✅ COMPLETED

## Dependencias
- Wave4 paralela disjunta (FIND-77 mismo dir distinto archivo — orden Wave0 OK según plan; FIND-82 owns `test-mcp.py` — no colisión)
- Previa Wave3 DONE · Next Wave5 (FIND-67)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** pendiente (sin tool sub-agente en este runner; se solicita vanta-docs/vanta-review al orquestador)
- **Enfoque:** ¿merge bidireccional correcto? ¿direcciones validadas por código? ¿gate script no rompe EXIT 0 previo?
- **Cómo se probó:** hashes + `git diff --check` + script exit 0 (evidencia en RESULTADO, no auto-reporte)
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (toda evidencia es output real citado)
  - [x] No saltarse clarificación (Gate D evaluado con motivo; C6 decidió no commitear submodule con WIP ajeno)
  - [x] No declarar done sin acceptance (contrato mecánico §Contrato)
  - [x] No ignorar fallos (codegraph ruido reportado, no ocultado)
  - [x] No un solo intento (hash+diff+grep+read+Test-Path+git-log convergentes)
  - [x] No copiar sin citar (toda dirección merge cita file:línea)
  - [x] No reintentar en bucle (0 retries necesarios)
  - [x] Pasos conectados al objetivo (cada edit ↔ contrato)
  - [x] Sin paths dinero/seguridad (docs-only)
  - [x] Presupuesto explícito (appetite 1d, ejecución parcial de jornada)
- **Veredicto:** ⏳ pendiente revisor distinto

## Notas
- `campaign_verify_cmd` con bug exit -1 conocido (plan §Riesgos) → verificación vía bash directa, documentada.
- Submodule `.opencode/` en `main` con WIP ajeno → decisión C6 (nota en Spec): edits en working tree, commit+bump al lead.
- `__pycache__/test-mcp.cpython-314.pyc` en `skills/` ignorado (fuera de scope, no tocado).
