---
title: "FIND-164 - scope every cargo nextest/test mention in the harness with -p"
kind: task
status: completed
description: "Bare cargo nextest/cargo test mentions in the normative harness surfaces contradicted the hard -p rule (incident f6c395ef). Every normative mention is now scoped with -p, a deliberate --workspace closure tier, or carries the closure marker; rg contract returns 0."
tags: [harness, gates, nextest, docs]
---

# FIND-164: Drift de la regla `-p` en superficies normativas del harness

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§5, Fase 1, wave 1.2)
- **Fuente:** `docs/dev/Backlog.md` fila `FIND-164` (origen: Gate H HARD-05, vanta-harness ronda 1, 2026-09-27)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Tipo:** Docs / harness (`.opencode/` — repo Git separado). Sin código Rust.
- **Turns estimados:** 6
- **Creado:** 2026-11-24T20:40
- **last-synced:** 2026-11-24T20:40
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0
- **Carril:** D — Docs (cupo 4). Sin `cargo`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Todo agente que lee `.opencode/agents/*`, `.opencode/task-system/RULES.md` + `prompts/*` + `workflows/*` + `enforcement/*`, y la skill `unified-review` antes de correr tests. No hay callers de código: son instrucciones. |
| Callees | `.opencode/AGENTS.md` §Regla 1 "Regla dura `-p`" (la norma que estas superficies debían reflejar), `docs/dev/references/troubleshooting.md` (síntoma/causa del incidente `f6c395ef`), `dev-tools/verify.ps1` (gate de cierre). |
| Implicaciones | Ningún contrato de API ni comportamiento de runtime cambia. Lo que cambia es **lo que un agente cree que debe ejecutar**: una mención bare reproducida por un agente cuelga el proceso (build implícito de `default-members` unifica features en `target/` → arranca el server HTTP en `:8080`). Afecta todas las sesiones futuras, no un test concreto. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/task-system/RULES.md`, `.opencode/task-system/prompts/task.md` (formato de task file), `.opencode/task-system/prompts/pipeline-full.md`, `.opencode/AGENTS.md` §Regla 1. Leídas las regiones completas alrededor de cada línea tocada en: `agents/vanta-lead.md` (83-96, 160-200, 218-270), `agents/vanta-worker.md` (84-100, 225-270, 360-372), `agents/vanta-engine.md` (300-320), `agents/vanta-docs.md` (60-77, 330-339, 405-413), `task-system/prompts/iter-loop-tools.md` (170-240, 365-375), `task-system/enforcement/pre-call-checks.md` (55-92), `task-system/workflows/{feature-add,refactor}.json`.
- **Archivos referenciados hacia dentro:** ninguno ejecutable — son Markdown/JSON de instrucciones. `workflows/*.json` se consumen por `campaign_get_workflow` (allowed_commands = prefijos de comando permitidos); `enforcement/pre-call-checks.md` es documentación del schema de enforcement.
- **Archivos que referencian a los editados:** `rg -n "unified-review" .opencode/SKILLS-MANIFEST.md` → la skill figura KEEP en el manifiesto; **no existe** copia en `.opencode/skills/` (ver §Notas, desvío 1). `rg -n "RULES.md" .opencode/agents/*.md` → citada como norma de tarea en varios agentes, no como comando.
- **Veredicto impacto:** **bajo en mecánica, alto en consecuencia**. Cero riesgo de romper código; el riesgo es dejar una superficie contradictoria que reproduzca el hang. Se preservan intactos todos los `--workspace` de cierre legítimos (CI, verify full, certificación).

## Contrato

```
rg -n 'cargo (nextest|test)' .opencode/agents .opencode/task-system .opencode/skills "$HOME/.agents/skills/unified-review" \
  --glob '!**/task-system/memory/**' \
  | rg -v -- '-p |--workspace|# tier|cierre'
```
→ **0 líneas**.

Cada mención de `cargo nextest`/`cargo test` queda en exactamente una de tres clases:
1. **scoped** — `-p <crate>` (iteración),
2. **tier de cierre** — `--workspace` deliberado,
3. **marcada** — texto que dice `cierre`/`# tier` y justifica el `--workspace`.

**Exclusión declarada:** `.opencode/task-system/memory/**` (y su `archive/`) es un **log histórico inmutable** — 6 líneas citan `cargo test --doc` / `cargo test` dentro de contratos de tareas de agosto-septiembre ya cerradas. Reescribirlas falsificaría el registro; no son superficies que un agente lea como comando a ejecutar. Se excluyen del contrato con `--glob` explícito.

## Spec

N/A — tarea 100% docs/normativa sin decisiones técnicas de runtime. La única decisión de diseño (scopear vs. marcar como cierre) se resuelve por evidencia línea a línea y queda registrada en §Notas.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Ningún `--workspace` de **cierre** legítimo se degrada a `-p` (CI `vanta-lead.md:295`, `RULES.md:142`, `pipeline-full.md:157`, `task.md:212/231`, `iter-loop-tools.md:325`). El tier de cierre es correcto a propósito.
  2. `.opencode/AGENTS.md` §Regla 1 **no se toca** (es la norma; esta tarea alinea las superficies a ella, no al revés).
  3. `.opencode/` es repo Git separado: los cambios no se trackean desde VantaDB ni se commitean desde acá.
- **Comandos de verificación:** el `rg` del §Contrato → 0 líneas; `git -C .opencode status --short` → los 10 archivos listados, sin commitear.
- **Deuda pendiente:** ver §Deuda.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-164` |
| `lastAction` | §Steps — Step 3 auditado y cerrado |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | ninguna (wave 1.2 cerrada para esta fila) |
| `contract` | §Contrato + §Invariantes |
| `nextTask` | lo decide el orquestador (wave 1.2 sigue con FIND-177/FIND-323/FIND-98/FIND-192) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** **negativo** — la tarea elimina drift normativo y no introduce código.

### §Deuda — gate que detecta el próximo drift (propuesta, NO implementada)

El drift existe porque la Regla dura `-p` se añadió a `.opencode/AGENTS.md` **después** de que los agentes estuvieran escritos: nada verifica que una norma nueva se propague. Dos gates posibles:

| Opción | Dónde | Coste | Cobertura | Techo conocido |
|---|---|---|---|---|
| **A (recomendada)** | `dev-tools/verify.ps1`, gate nuevo "harness" condicional a que el diff toque `.opencode/**` | ~15 líneas PowerShell, ~0.3 s | Bloqueante en cada verify | Sólo corre si `.opencode/` está en el diff; un cambio de norma sin tocar `.opencode/` (p.ej. sólo AGENTS.md) no dispara — **AGENTS.md sí está en `.opencode/`**, así que en la práctica dispara |
| **B** | Checklist de Gate H (`.opencode/commands/harness.md`) | 1 línea | Manual, al correr `/harness` | Nadie corre `/harness` si no se acuerda — fue exactamente el fallo de HARD-05 |
| **C** | Job de CI sobre `.opencode/` | workflow + trigger `paths:` | Bloqueante en PR | El repo `.opencode/` es separado; CI de VantaDB no lo ve |

Comando del gate (idéntico al contrato de este task, con la exclusión de `memory/`):

```powershell
$hits = rg -n 'cargo (nextest|test)' .opencode/agents .opencode/task-system .opencode/skills `
  --glob '!**/task-system/memory/**' | rg -v -- '-p |--workspace|# tier|cierre'
if ($hits) { $hits; exit 1 }
```

**Coste total estimado:** 30 min implementar A + B, 0 min de mantenimiento (el rg es la especificación). **Pago de la deuda:** ninguna fila P2 existente; se propone abrir fila nueva sólo si el owner quiere el gate — hoy es 1h de higiene ya pagada, no un gate implementado.

### Deuda colateral registrada en este cierre (Gate C)

| Hallazgo | Evidencia | Propuesta |
|---|---|---|
| `.opencode/skills/unified-review/` **no existe**; la única copia es global (`C:\Users\Eros\.agents\skills\unified-review\SKILL.md`) | `Get-ChildItem -Recurse -Directory -Filter unified-review` → 0 resultados en el repo | O el manifiesto (`SKILLS-MANIFEST.md:476`, KEEP 9 archivos) está stale, o falta sincronizar el espejo. Fila `FIND-*` para el owner del harness |
| `vanta-docs.md` tenía 4 `cargo test --doc` bare (fuera de la lista de escritura de la fila) | `agents/vanta-docs.md:64,73,336,411` | **Corregido acá** (el contrato cubre `.opencode/agents`) — sin él, el rg no daba 0 |
| `task-system/prompts/*`, `workflows/*.json`, `enforcement/*.md` tenían 12 menciones bare (fuera de la lista de escritura: sólo decía `RULES.md`) | ver §Steps 2 | **Corregido acá**, por la misma razón (el contrato cubre `.opencode/task-system`). Sin colisión: ningún otro agente de la wave toca `.opencode/task-system` |

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato `rg` = 0 + las 3 clases respetadas | ✅ |
| **Commit** | Commit atómico en VantaDB con `docs(harness): … (FIND-164)`; el harness queda sin commitear en su repo (fuera de control de este repo) | ✅ |
| **Release** | No aplica: no toca código, no cambia comportamiento, no requiere verify full | justificado |

## Herramientas necesarias
- `rg` (el contrato es un `rg`), Read, `edit`.

**Skills cargadas (SDP):** `doubt-driven-development` (policy pin: harness `.opencode/`), `campaign-executor` (base), `progreso` (cierre). Descartadas del output SDP por no aplicar a una tarea Markdown-only: `test-driven-development`, `incremental-implementation`, `context-engineering`, `source-driven-development`, `frontend-ui-engineering`.

## Investigation Notes

**Estado real medido 2026-11-24 (no el del backlog 2026-09-27):** la lista del backlog estaba **mayormente vigente** pero incompleta y con dos referencias rotas:
- `vanta-lead.md:84` **sí** era bare (`cargo test --workspace` / `cargo nextest --profile audit`): el primero traía `--workspace`, el segundo no.
- `vanta-lead.md:228,265`, `vanta-worker.md:93,237,367`, `vanta-engine.md:318`, `RULES.md:123,154` → vigentes tal cual decía el backlog.
- `skills/unified-review/SKILL.md:1098` → **ruta inexistente en el repo**; la línea vive en la copia global.
- **No listadas en el backlog y sí bare:** `vanta-lead.md:170`, `vanta-worker.md:257,262`, `vanta-docs.md:64,73,336,411`, `prompts/iter-loop-tools.md:178,205,234,371`, `prompts/task.md:357,395,398`, `workflows/feature-add.json:23`, `workflows/refactor.json:29`, `enforcement/pre-call-checks.md:61,64,88`, `unified-review/SKILL.md:685`.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Steps

### Step 1: Verificar la lista real contra el harness de hoy — ✅
`rg -n 'cargo (nextest|test)' .opencode/agents .opencode/task-system .opencode/skills` → 60+ hits; filtrado → **26 bare** en superficies normativas + 6 en `memory/` (excluido). Backlog re-validado: 9/9 líneas vigentes, 2 rutas stale, 17 líneas no inventariadas.

### Step 2: Aplicar las 3 clases a cada mención — ✅
- **Tier de cierre (`--workspace` explícito):** `lead:84` (pre-flight de release), `lead:265` (diagrama de gate CI), `worker:367` (suite completa antes de done), `RULES.md:154` (cobertura DoD v2, workspace-wide).
- **Scoped (`-p`):** `lead:170` (`-p vantadb`, post-rollback), `lead:228` (`-p vantadb`, pre-commit), `worker:93,237,262` (`-p <crate>`), `worker:257` (`cargo test -p <crate> --doc`), `engine:318` (`-p vantadb`), `docs:64,73,336,411` (`-p <crate>`), `RULES.md:123` (`-p <crate>`), `iter-loop-tools:178,205,234,371`, `task:357,395,398`, `enforcement:61,64,88`, `workflows/*.json` (prefijo `cargo nextest run -p`), `unified-review:685,1098`.
- **Sin tocar:** `lead:295`, `RULES.md:142`, `pipeline-full.md:157,227`, `task.md:212,231`, `iter-loop-tools.md:325`, `sdp-v3.mjs:32`, `unified-review:390` — ya eran `--workspace`.

### Step 3: Verify del contrato + review — ✅
```
rg … | rg -v -- '-p |--workspace|# tier|cierre'   → 0 líneas
```
Review P2-01 tier **Fast** (`docs/dev/**` + `.opencode/**` no es adversarial): verify fast mecánico (el rg del contrato) + review degradado marcado (sub-agente no disponible en esta sesión).

## Dependencias
- Ninguna. Wave 1.2, carril Docs. En paralelo: `FIND-177` (`docs/dev/references/troubleshooting.md`), `FIND-328` (`.github/workflows/`), `F0.2` (build). Rutas disjuntas; `FIND-164` es el único que escribe `.opencode/`.

## Review (GATE — agente distinto, P2-01)

**Tier:** Fast (paths: `.opencode/**`, `docs/dev/tasks/FIND-164.md`).
**Verdict:** approve — **degradado**: no se pudo spawnear `vanta-review`/`vanta-audit` desde este sub-agente. Mitigación: el contrato es mecánico y re-ejecutable por el orquestador (un `rg` de 0 líneas no necesita ojo humano); el riesgo residual es de **criterio** (haber marcado como "cierre" algo que debía ser `-p`), mitigado listando las 4 decisiones de cierre en §Steps 2 para spot-check del reviewer.

**Escalado al owner:** confirmar que `lead:84` y `worker:367` son tier de cierre (pre-flight de release / antes de done) y no iteración.

## Notas

- **Desvío 1 — ruta inexistente:** el plan y el backlog nombran `.opencode/skills/unified-review/SKILL.md`; no existe. Se editó la copia global `C:\Users\Eros\.agents\skills\unified-review\SKILL.md` (la que el agente realmente lee). Reportado en §Deuda colateral.
- **Desvío 2 — scope extendido:** la lista de escritura de la fila nombraba `RULES.md` dentro de `task-system`; el contrato del propio contrato cubre **todo** `.opencode/task-system`. Se corrigieron `prompts/iter-loop-tools.md`, `prompts/task.md`, `workflows/{feature-add,refactor}.json` y `enforcement/pre-call-checks.md` porque sin ellos el rg no devuelve 0. Cero colisión con otras filas de la wave.
- **Desvío 3 — `-p` vs `--workspace` en `RULES.md:154`:** DoD v2 pide "70% de coverage en módulos nuevos" — la medición es necesariamente workspace-wide. Marcado `--workspace` con la razón inline.
- **Regla 5 / ADR:** no aplica (no hay trade-off arquitectónico; sí una decisión de higiene ya normada en `AGENTS.md`).
