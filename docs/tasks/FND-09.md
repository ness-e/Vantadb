# FND-09: Regla 8 — Concurrencia paranoica en PRs

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-r2-r7-fnd.md
- **Fuente:** docs/Backlog.md P20b / plan Task 4
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Tipo:** Docs (regla de proceso en AGENTS.md)
- **Turns estimados:** 5
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ✅ COMPLETED (2026-08-16, vanta-worker — pendiente review P2-01 del orquestador)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/AGENTS.md` (reglas 1-7 existentes), `.opencode/agents/vanta-worker.md` (Output Template) |
| Callees | ninguno (docs puro, sin código) |
| Implicaciones | Regla 8 es nueva sección — no rompe contratos; agrega gate de proceso para PRs de concurrencia. `vanta-worker.md` recibe 1 línea de referencia en Verification — no altera frontmatter ni comandos |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0)**

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (540L, incl. sección "VantaDB Development Protocol & AI Guardian Rules" con Reglas 1-7), `.opencode/agents/vanta-worker.md` (167L, estado commiteado tras R7 `5bda5662` — sin diff pendiente)
- **Archivos referenciados hacia dentro:** ninguno (edición de markdown aditivo)
- **Archivos que referencian a los editados (referencias entrantes):** `vanta-worker` referenciado en AGENTS.md L146/L177, VANTADB-OPERATING-MANUAL.md, agents/vanta-{arch,chaos,audit,docs,engine,research,lead,review}.md, commands/{pipeline,build}.md, skills/unified-review/*, 30+ task files (grep 87 matches) — ninguna referencia apunta a una sección específica que se renombre; edición aditiva no las rompe. `Regla 8` no existe en ningún archivo (grep 0 matches) — sin colisión con FND-11/12/13/14 (tareas separadas, mismo archivo: riesgo de merge, mitigado por wave paralela + lead commitea)
- **Veredicto impacto:** bajo — se agrega sección nueva al final de AGENTS.md y 1 línea en vanta-worker.md; no se modifica ni elimina contenido existente

## Contrato
"`grep .opencode/AGENTS.md` contiene 'Regla 8' con las 3 señales (multi-índice o dashmap/parking_lot o Tokio + '10k w/s' + '1k r/s' + delegación vanta-chaos/vanta-review) Y vanta-worker.md referencia 'Regla 8'"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no romper el estado commiteado de vanta-worker.md (R7 ya aplicado); no tocar otras reglas (FND-11/12/13/14 son tareas separadas); no tocar archivos protegidos (docs/Backlog.md, AUD-024.md, vantadb-wasm/src/lib.rs, plan file)
- **Comandos de verificación:** `grep -n "Regla 8" .opencode/AGENTS.md` y `grep -n "Regla 8" .opencode/agents/vanta-worker.md` + `grep -n "10k w/s" .opencode/AGENTS.md` + `grep -n "1k r/s" .opencode/AGENTS.md` + `grep -n "vanta-chaos" .opencode/AGENTS.md`
- **Deuda pendiente:** ninguna

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (cambio docs aditivo, no introduce deuda técnica)

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file se cumple (grep mecánico) |
| **Commit** | NO aplica en esta sesión — lead commitea al cerrar la wave (plan file L7) |
| **Release** | NO aplica (docs de proceso, sin release) |

## Herramientas necesarias
- grep (verificación mecánica del contrato)

## Investigation Notes
- Backlog P20b: "Investigar si hoy existe gate que obligue a auditar deadlocks/data races al tocar paths multi-índice o dashmap/parking_lot/Tokio" → resultado: NO existe gate (grep "Regla 8|concurrencia paranoica|10k w/s" solo matchea Regla 7 pre-existente). Implementación = redactar Regla 8.
- Estilo consistente: Reglas 1/2 usan tabla "Si el usuario hace... → Debes responder...". Regla 8 usa la misma forma de tabla + bullets de carga/delegación.
- Contenido mínimo definido por el orquestador: NO inventar más (3 señales + carga + delegación).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 2 — Step 1 (AGENTS.md), Step 2 (vanta-worker.md) |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — NO aplica: cambio de documentación de proceso, no toca trust boundaries, input, auth, dependencias ni FFI
- [x] **PERFORMANCE** — NO aplica: no toca hot paths, solo docs de proceso

## Steps

### Step 1: Agregar Regla 8 a `.opencode/AGENTS.md`
- **Archivos:** `.opencode/AGENTS.md`
- **Acción:** agregar sección "### Regla 8: Concurrencia Paranoica en PRs" después de Regla 7 (fin del archivo, tras la línea "La verificación pre-push manual corre..."), con tabla estilo Reglas 1/2 + carga objetivo 10k w/s + 1k r/s + delegación vanta-chaos/vanta-review
- **Verify:** `grep -n "Regla 8" .opencode/AGENTS.md` → L541 ✅; `grep -n "multi-índice"` → L543/547 ✅; `grep -n "dashmap|parking_lot|Tokio"` → L543/548 ✅; `grep -n "10k w/s|1k r/s"` → L551 ✅; `grep -n "vanta-chaos|vanta-review"` → L553 ✅
- **Estado:** ✅ COMPLETED

### Step 2: Referenciar Regla 8 en `.opencode/agents/vanta-worker.md`
- **Archivos:** `.opencode/agents/vanta-worker.md`
- **Acción:** agregar 1 línea de referencia en la sección "### Verification" del Output Template (Regla 8 + delegación vanta-chaos/vanta-review para PRs de concurrencia)
- **Verify:** `grep -n "Regla 8" .opencode/agents/vanta-worker.md` → L104 (1 match) ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- R7 (commit `5bda5662`) — completado; vanta-worker.md ya corregido (base sobre la que edito)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** pendiente — el orquestador delega a vanta-review antes de marcar COMPLETED
- **Enfoque:** ¿la redacción cubre las 3 señales del contrato (multi-índice/dashmap/parking_lot/Tokio + 10k w/s + 1k r/s + delegación)? ¿estilo consistente con Reglas 1-7?
- **Cómo se probó:** grep mecánico del contrato (L541/543/547/548/551/553 en AGENTS.md, L104 en vanta-worker.md) — verificado por el implementador; review independiente pendiente
- **Veredicto:** pendiente

## Notas
- NO commitear — lead commitea al cerrar wave (regla explícita del plan y de la tarea)
- NO ejecutar skill progreso (toca docs/Backlog.md — migración la hace el lead)