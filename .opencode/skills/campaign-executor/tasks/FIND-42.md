# FIND-42: Boundary src → skills (inversión de dependencia)

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md`
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30T12:00
- **Estado:** ✅ COMPLETED
- **SDP:** base-only + `documentation-and-adrs` (boundary/architecture decision)
  keywords: `["src skills", "boundary", "impeccable", "agents"]`

## Context

codegraph-20260827 Fase 1 reportó 173 (y codebase-memory-mcp 184) aristas
`src → skills`. El plan file interpretó esto como inversión de dependencia
semántica (core llamando a skills de agentes — "impeccable").

## Blast Radius

- **CodeGraph report:** 173 calls `src → skills` (artefacto codegraph-20260827)
- **codebase-memory-mcp:** 184 edges `src → skills` (índice `moderate`,
  47.7K nodos, 157.7K edges, 25 edge types)
- **Implicaciones:** ninguno en código actual — el contrato `== 0` ya pasa
  textualmente. La "inversión" reportada es un **falso positivo** del
  codegraph causado por **path-homonymy** (path-collapse de 4 carpetas con
  nombre "skills" distintas).

## Contrato

```powershell
Select-String -Path "src/**/*.rs" -Pattern "\.agents\.skills" 2>$null |
  Measure-Object | Select-Object Count
```

Debe devolver `0` (ya pasa) **OR** ADR que documente como intencional.

Resultado mecánico (2026-08-30): **0** hits ✅

Además:
- `grep` por `\agents/skills|\opencode/skills` en `src/**/*.rs` → 0 hits ✅
- codegraph `codegraph_explore`: ninguna arista `src → .agents/skills` o
  `src → .opencode/skills` (solo referencias a `/api/v2/skills/*` HTTP
  endpoints del server routing — strings literales, no inversión semántica).
- codebase-memory-mcp Cypher:
  `MATCH (src:Folder {file_path:'src'})-[]->(t) WHERE t.file_path CONTAINS '.agents' OR ...`
  → **0 rows**.

## Tools

- `Select-String` (PowerShell) — verificación mecánica textual
- `grep` — verificación cruzada literal
- `codegraph_codegraph_explore` — blast radius graph-aware (codegraph)
- `codebase-memory-mcp_query_graph` / `get_architecture` — architecture +
  boundary cross-check (índice `moderate`)
- `Write` — ADR-034 + task file

## Root-cause

| Carpeta | Qué es | Tiene aristas desde `src/`? |
|---|---|---|
| `src/skills/` | **Módulo core** (`SkillStore`, persistencia de skills como memoria de agentes — D19 plan vanta-memory) | **Sí** — 184 (es un sub-módulo del core; CONTAINS_FOLDER + calls/references dentro del propio core) |
| `skills/` (raíz) | (vacío / placeholder histórico) | No |
| `.opencode/skills/` | Workflows del agente OpenCode (no parte del grafo Rust) | **No** |
| `.agents/skills/` | Skills del agente (162 dirs de workflows; no parte del grafo Rust) | **No** |

El **codegraph-20260827 Fase 1** y **codebase-memory-mcp** colapsaron
"src/skills" (módulo interno del core) y ".agents/skills" (skills del
agente) en un único nodo "skills" por **path-homonymy** (la última
componente del path coincide). El resultado fue una métrica agregada
que **no representa inversión de dependencia** — representa acoplamiento
interno del core con su propio sub-módulo `SkillStore`.

## Steps

### Step 1: Verificación mecánica textual
- **Acción:** `Select-String -Path "src/**/*.rs" -Pattern "\.agents.skills" | Measure-Object Count`
- **Resultado:** `Count=0` ✅
- **Estado:** ✅ COMPLETED

### Step 2: Grep cruzado literal
- **Acción:** `grep -r --include='*.rs' '\.agents/skills|\.opencode/skills' src/`
- **Resultado:** No files found ✅
- **Estado:** ✅ COMPLETED

### Step 3: Verificación via codegraph
- **Acción:** `codegraph_explore` con query sobre boundary `src → skills`
- **Resultado:** ninguna arista real a `.agents/skills/` o `.opencode/skills/`; solo strings de `/api/v2/skills/*` (endpoints HTTP)
- **Estado:** ✅ COMPLETED

### Step 4: Verificación via codebase-memory-mcp
- **Acción:** `query_graph` Cypher — `MATCH (src:Folder {file_path:'src'})-[]->(t) WHERE t.file_path CONTAINS '.agents' OR t.file_path STARTS WITH 'agents/skills' ...`
- **Resultado:** 0 rows ✅
- **Acción 2:** `get_architecture aspects=['boundaries','dependencies']` — boundary `src skills 184` confirmado como **sub-módulo core** (`CONTAINS_FOLDER`), no dependencia externa
- **Estado:** ✅ COMPLETED

### Step 5: ADR-034 documenting como NO-inversión (resolución del falso positivo)
- **Acción:** escribir `docs/architecture/adr/ADR-034-no-src-to-agents-skills-boundary.md`
- **Status:** accepted-pending-owner-review (Regla 5 — IA redacta evidencia; owner humano articula trade-off)
- **Estado:** ✅ COMPLETED

### Step 6: Actualizar plan file
- **Acción:** marcar `Estado: ✅ COMPLETED` con nota del resultado
- **Estado:** ✅ COMPLETED

### Step 7: Commit + skill progreso
- **Acción:** `git add` (solo task file + ADR + plan file) + commit `docs: FIND-42 — ADR boundary src->skills (inversion de dependencia)`
- **Estado:** ⬜ PENDING

## Dependencias

- Ninguna (la tarea es netamente documental — no toca código core)

## Notas

- **Inversión de dependencia**: NO EXISTE. El plan file (W25-3) y
  codegraph-20260827 Fase 1 midieron mal por **path-homonymy**.
- El **contrato pasa con `Count == 0`** sin necesidad de fix mecánico.
- El **ADR-034** documenta (a) por qué el codegraph reportó la métrica,
  (b) por qué NO indica inversión real, (c) la métrica correcta que
  distinguiría `src/skills` (sub-módulo interno) de `.agents/skills`
  (skills de agente) en futuros reportes.
- **Deuda abierta** (dejar registrada): el codegraph necesita un
  pre-procesador de paths que distinga `src/...` de `.agents/...` y
  `.opencode/...` por prefijo de raíz, no por nombre de última
  componente. ADR-034 lo deja documentado como mejora futura (P3).

## Context Save Point

- **Fecha:** 2026-08-30T12:00
- **Branch:** develop
- **CI pendiente:** no (cambios solo en `docs/` + `.opencode/skills/`)
- **Decisiones:**
  1. **No fix mecánico** — el contrato ya pasa con `Count=0`; el "bug"
     no está en el código sino en el reporte del codegraph.
  2. **ADR en lugar de fix** — explica path-homonymy y deja la métrica
     correcta documentada para futuros reportes (deuda P3 registrada).
- **Problemas conocidos:** ninguno en el scope de FIND-42. La métrica
  agregada `src skills 184` del codegraph se reporta en otros lugares
  (FIND-41 clusters, FIND-45) y podría generar nuevos falsos positivos
  — owners deben leer ADR-034 antes de actuar sobre esas métricas.
- **Próxima tarea:** W26-SOLO FIND-33 (snapshot filesystem backend KV)
  o, si FIND-41/FIND-45 dependen de esta aclaración, ejecutar esas
  primero con ADR-034 como referencia.

## References

- `docs/plans/2026-08-29-full-backlog-parallel.md` §W25-3 (este task)
- `docs/plans/2026-08-29-full-backlog-parallel.md` §W25-1 (FIND-24),
  §W25-2 (FIND-41 — clusters; puede tener métrica afectada por el mismo
  path-homonymy)
- `codegraph-20260827 Fase 1` report (artefacto histórico; métrica
  agregada que origina el falso positivo)
- `codebase-memory-mcp` arquitectura — boundary `src skills 184` en
  output de `get_architecture aspects=['boundaries']` (índice
  `moderate`, 47.7K nodos)
- ADR-034 (a crear): "No existe boundary `src → .agents/skills` —
  resolución de falso positivo codegraph por path-homonymy"
- Skill `documentation-and-adrs` (Regla 5 — ADR como memoria de
  decisiones arquitectónicas)
- Regla 0 (AGENTS.md) — análisis de impacto antes de modificar