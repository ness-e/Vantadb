# TASKS-MOVE-01: Mover tasks fuera del submodule a docs/tasks/ (D1)

## Metadata
- **Plan file:** N/A (tarea directa D1, sin plan file)
- **Creado:** 2026-09-05
- **last-synced:** 2026-09-05
- **Estado:** ⏳ IN PROGRESS (copy + docs hecho, sin commit por contrato)
- **Tipo:** docs (refactor localización, sin lógica)
- **SDP:** campaign-executor, progreso, ponytail, writing-guidelines, writing-plans, incremental-implementation, test-driven-development, context-engineering

## Descripción (D1)
Mover tasks fuera del submodule privado configOpencode a `docs/tasks/` en repo
principal VantaDB para eliminar doble-commit y mezcla config/datos.

## Blast Radius
- **Archivos leídos completos:** `.opencode/AGENTS.md` § Path Resolution (80-87),
  `.opencode/skills/campaign-executor/SKILL.md` § Componentes (33-44) + System Integration (283-285),
  `.opencode/commands/pipeline.md` header (9) + MODO PLAN (77) + MODO TAREA (112,145)
- **Referencias hacia dentro:** los 3 archivos referencian `campaign-executor/tasks` como resolución de `tasks/<ID>.md`
- **Referencias entrantes:** ~100 matches de `campaign-executor/tasks` en `.opencode/`
  (prompts, RULES.md, references, skills, agents, memory) — fuera de scope, solo los 3 del contrato se actualizan
- **Veredicto:** impacto bajo, docs-only, sin código Rust, sin hot path, sin trust boundary.
  Sin FASE SECURITY ni PERFORMANCE.

## Impacto mapeado (Regla 0)
- **Contenido:** 635 raíz + 92 complete + 2 closed = 729 `.md` en origen
  (contrato decía 628+92+2=722; drift +7 en raíz por tasks nuevas desde el conteo del plan)
- **Referencias entrantes:** grep `campaign-executor/tasks` → 100+ hits en `.opencode/`; solo AGENTS/SKILL/pipeline son scope
- **Referencias salientes:** origen → ningún código Rust; solo docs/prompts lo citan
- **Veredicto:** copy (NO move) + actualizar 3 archivos + README redirect. Reversible (origen intacto).

## Contrato
"docs/tasks/ contiene 635+92+2=729 .md (copia fiel del origen), .opencode/skills/campaign-executor/tasks/
queda como redirect/compat con README, rg tasks/campaign-executor en AGENTS/SKILL/pipeline
apunta a docs/tasks como canónico. Verify: conteos + rg + git status. Sin commit."

## Herramientas
- bash (PowerShell: Copy-Item, Get-FileHash, Measure-Object)
- grep / Select-String
- edit (3 archivos submodule) + write (README + este file)

## Steps
### Step 1: Crear docs/tasks/ + complete/ + closed/
- **Archivos:** `docs/tasks/`, `docs/tasks/complete/`, `docs/tasks/closed/`
- **Acción:** New-Item -ItemType Directory -Force
- **Verify:** Get-ChildItem docs/tasks → complete + closed presentes
- **Estado:** ✅ COMPLETED

### Step 2: Copiar 635 + 92 + 2 preservando nombres, verificar conteos + sample hash
- **Archivos:** `.opencode/skills/campaign-executor/tasks/**/*.md` → `docs/tasks/**`
- **Acción:** Copy-Item raíz/complete/closed con -Force (NO borrar origen, NO git mv)
- **Verify:** conteos destino 635/92/2 == origen; sample hash SHA256 match (ADMIN-01, FIND-62, GOV-T01, RES-01, 1/2/3/4/10, CLI-01, COMP-006/008, DEVOPS-10/15)
- **Estado:** ✅ COMPLETED (drift documentado: 635 vs 628 del contrato)

### Step 3: Actualizar Path Resolution en AGENTS + SKILL + pipeline
- **Archivos:** `.opencode/AGENTS.md` (tabla + nota), `.opencode/skills/campaign-executor/SKILL.md`
  (tabla Componentes ×2 + Fase 1 + System Integration), `.opencode/commands/pipeline.md` (header + 3 refs)
- **Acción:** `tasks/<ID>.md` → `docs/tasks/<ID>.md` + fallback legacy nota D1
- **Verify:** rg en los 3 archivos apunta a docs/tasks como canónico
- **Estado:** ✅ COMPLETED

### Step 4: README redirect en origen (sin borrar origen, sin commit)
- **Archivos:** `.opencode/skills/campaign-executor/tasks/README.md` (nuevo), `docs/tasks/TASKS-MOVE-01.md` (este file)
- **Acción:** explicar migración D1, canónico vs fallback, motivo doble-commit
- **Verify:** Test-Path ambos True; git status muestra diff sin commit
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (tarea standalone D1)

## Notas
- Contrato decía 628 raíz; medido 635 (7 tasks nuevas desde el conteo). Copia fiel 729 total.
- TASKS-MOVE-01.md no existía en origen (es esta tarea); vive solo en canónico `docs/tasks/`.
- NO commit por orden explícita (pipeline-full pide commit, el contrato D1 lo prohíbe en este slice).
- NOTICED BUT NOT TOUCHING: `pipeline-full.md:3,79`, `pipeline-run.md:64,106`, `RULES.md:93,462`,
  `plan.md:188,271`, `task.md:5`, `iter-loop-tools.md:101`, `vanta-lead.md:444`,
  `planning-and-task-breakdown/SKILL.md`, `incremental-implementation/SKILL.md`,
  `test-driven-development/SKILL.md`, `code-review-and-quality/SKILL.md`,
  `references/skills-engineering.md:260` siguen citando el path legacy → tarea follow-up.
- `campaign_verify_cmd` reportado roto ("autoTransition is not defined", ref lessons 2026-09-03) → verify por bash directo.
- Regla 0, Regla 0.5 scope discipline, ponytail full: mínimo docs, sin abstracciones.

## Context Save Point
- **Fecha:** 2026-09-05
- **Branch:** develop
- **CI pendiente:** no (docs-only, sin código)
- **Decisiones:** copy-no-move + sin commit por contrato D1; drift +7 documentado no bloquea
- **Problemas conocidos:** ninguno; submodule `.opencode` ya estaba dirty antes (M AGENTS/SKILL/pipeline + tasks ??/M ajenos) — este diff se suma sin commit
- **Próxima tarea:** ninguna (D1 cierra con follow-up opcional: migrar refs restantes + git mv/commit vía vanta-lead)

## Referencias
- `.opencode/AGENTS.md` § Path Resolution + § .opencode Submodule
- `.opencode/skills/campaign-executor/SKILL.md` § Componentes + System Integration
- `.opencode/commands/pipeline.md` header Tasks
- `.opencode/task-system/prompts/pipeline-full.md` (prompt ejecutor, §7 RESULTADO)
