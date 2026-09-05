# FND-23: Decidir grafos default-on vs opt-in con telemetría real (ADR)

## Metadata
- **Plan file:** `docs/plans/2026-08-16-wave-p20-tsys.md` (Wave 6, FND-23)
- **Fuente:** `docs/Backlog.md:517`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Docs (ADR)
- **Turns estimados:** 5
- **Creado:** 2026-08-16T00:00
- **last-synced:** 2026-08-16T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/architecture/adr/` (índice de ADRs), `docs/Backlog.md` (referencia FND-23) |
| Callees | `src/metrics/core/registry.rs` (métricas citadas), `src/cli_server.rs:147` (`/metrics` endpoint), `Cargo.toml:96-139` (features — ausencia de feature `graph`) |
| Implicaciones | No cambia código. El ADR documenta decisión + señal de telemetría pendiente; no instrumenta nada (eso es otra tarea). Complementa FND-03 (aislar features) definiendo el default. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `docs/_templates/adr.md` (plantilla ADR)
  - `docs/architecture/adr/ADR-023-backend-compaction.md` (modelo de ADR con señal de reapertura)
  - `docs/architecture/adr/ADR-020-storage-backend-default.md` (modelo de ADR con evidencia file:line)
  - `src/metrics/core/registry.rs` (1168 líneas — inventario completo de métricas)
  - `src/metrics/core/mod.rs` (snapshot + `export_metrics_text`)
  - `src/metrics/mod.rs`, `src/metrics/native.rs`
  - `Cargo.toml` (649 líneas — features completas)
  - `src/cli_server.rs` (ruta `/metrics` línea 147)
  - `docs/plans/2026-08-16-wave-p20-tsys.md`, `docs/Backlog.md:500-518`
  - `.opencode/task-system/prompts/pipeline-full.md`, `prompts/task.md`
- **Archivos referenciados hacia dentro:** ADR-024 cita `src/metrics/core/registry.rs` y `src/cli_server.rs` por file:line (solo referencia, no import).
- **Archivos que referencian a los editados (entrantes):** `docs/Backlog.md:517` (FND-23), `docs/plans/2026-08-16-wave-p20-tsys.md:67` — NO se editan (archivos protegidos).
- **Veredicto impacto:** bajo — se CREAN dos archivos nuevos (task file + ADR-024); ningún archivo existente se modifica. Nada se rompe.

## Contrato

"ADR numerado tras 023 (`ADR-024-*.md`) existe en `docs/architecture/adr/` con: decisión explícita, métrica de telemetría nombrada (existente **o** pendiente de instrumentar), umbral + acción + señal de reapertura. Verify: grep de la métrica citada en `src/metrics/` confirma existencia o la anota como pendiente."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no tocar `docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/AUD-024.md`, `.opencode/task-system/enforcement/verify-log.jsonl`, `completions/_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`, `.opencode/AGENTS.md`. No instrumentar métricas nuevas (gap se documenta, no se implementa). No git add/commit (el lead commitea).
- **Comandos de verificación:** `grep -r "vanta_graph_ops_total\|vanta_http_requests_total\|vanta_planner_vector_only_queries_total" src/metrics/` → confirma que la métrica pendiente NO existe y las proxies SÍ.
- **Deuda pendiente:** instrumentar `vanta_graph_ops_total` (contador de operaciones de grafo) en `/metrics` — tarea futura de vanta-tuner, NO de esta.

## Recitation (canónico — estructura única)

contract:
  verificacion: "grep de métricas citadas en src/metrics/ ✅ — vanta_http_requests_total y vanta_planner_* existen; vanta_graph_ops_total NO existe (pendiente de instrumentar, documentado en ADR)"
  evidencia:
    - claim: "No existe feature `graph` en Cargo.toml — el motor de grafos se compila siempre (default-on sin gate)"
      evidencia: "Cargo.toml:96-139 (features)"
      confianza: alta
    - claim: "No existe métrica de uso de grafos en src/metrics/"
      evidencia: "grep GRAPH|EDGE|TRAVERSAL en src/metrics/ → 0 matches"
      confianza: alta
    - claim: "/metrics existe y exporta métricas Prometheus reales (FND-07)"
      evidencia: "src/cli_server.rs:147, src/metrics/core/mod.rs:573"
      confianza: alta
  artefactos:
    - docs/architecture/adr/ADR-024-graph-engine-default-telemetry.md
    - .opencode/skills/campaign-executor/tasks/FND-23.md
  invariantes: "ninguna (docs-only, no toca código ni archivos protegidos)"
  deuda: "instrumentar vanta_graph_ops_total en /metrics (vanta-tuner, post-launch)"
  queda_pendiente: "orquestador: FND-03 (features granulares) referencia este ADR para el default; decidir cierre del ciclo 90d post-Show HN"

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — docs-only, no introduce deuda; documenta deuda de instrumentación existente (gap de telemetría de grafos).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file ✅ (ADR-024 con decisión + métrica + umbral + señal) + grep de métricas |
| **Commit** | Commit lo hace el lead (regla sub-agentes) — conventional commit `docs:` |
| **Release** | No aplica (docs-only, sin release) |

## Herramientas necesarias
- grep (verify de métricas citadas)
- Read/Write (ADR + task file)

## Investigation Notes
- FND-07 (Wave 3, completada por vanta-tuner) ya entregó `/metrics` con datos reales — el endpoint y el registry existen.
- El motor de grafos: `src/engine.rs:349` (`traverse` BFS), `src/edge_index.rs` (EdgeIndex), `add_edge` en engine — NO feature-gated.
- FND-23 es post-launch: la señal se observa tras Show HN. Honestidad: hoy NO hay telemetría de uso de grafos → el ADR define la señal requerida y el estado "sin datos".
- Métricas existentes usables como proxy: `vanta_http_requests_total` (labels method/route/status), `vanta_planner_{hybrid,text_only,vector_only}_queries_total` (mix de rutas de query).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — decisión documentada con gap honesto |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Steps

### Step 1: Inventario de telemetría y ADRs
- **Archivos:** `src/metrics/core/registry.rs`, `src/metrics/core/mod.rs`, `src/cli_server.rs`, `docs/architecture/adr/`
- **Acción:** leer métricas existentes, endpoint /metrics, ADRs previos (formato, numeración, señal de reapertura en ADR-023).
- **Verify:** grep `GRAPH|EDGE|TRAVERSAL` en `src/metrics/` → 0 matches (confirma gap)
- **Estado:** ✅

### Step 2: Análisis de decisión documentable
- **Archivos:** `Cargo.toml` (features), `docs/plans/2026-08-16-wave-p20-tsys.md`
- **Acción:** confirmar ausencia de feature `graph`; elegir la decisión: grafos default-on hasta que la telemetría diga lo contrario; definir métrica + umbral + acción + señal de reapertura.
- **Verify:** features en Cargo.toml:96-139 no incluyen `graph`
- **Estado:** ✅

### Step 3: Crear task file
- **Archivos:** `.opencode/skills/campaign-executor/tasks/FND-23.md` (NUEVO)
- **Acción:** escribir task file completo (este archivo).
- **Verify:** existe el archivo
- **Estado:** ✅

### Step 4: Escribir ADR-024
- **Archivos:** `docs/architecture/adr/ADR-024-graph-engine-default-telemetry.md` (NUEVO)
- **Acción:** escribir ADR con plantilla `docs/_templates/adr.md`: decisión explícita (default-on hasta evidencia), métrica pendiente `vanta_graph_ops_total` + proxies existentes, umbral + acción, señal de reapertura, estado de instrumentación honesto.
- **Verify:** numeración tras ADR-023 (ADR-024); grep de métricas citadas
- **Estado:** ✅

### Step 5: Verificación del contrato
- **Archivos:** —
- **Acción:** grep de la métrica citada en `src/metrics/`; validar ADR-024 existe con los 4 elementos del contrato.
- **Verify:** `grep -rn "vanta_graph_ops_total\|vanta_http_requests_total\|vanta_planner_vector_only_queries_total" src/metrics/`
- **Estado:** ✅

## Dependencias
- Task previa: FND-03 (Wave 6, vanta-lead) — crea features granulares; FND-23 define el default y lo referencia. Se ejecutan en paralelo (archivos distintos: Cargo.toml vs docs/architecture/adr/).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (revisión adversarial del ADR antes de marcar COMPLETED)
- **Enfoque:** ¿la decisión es honesta? ¿el umbral/acción es accionable? ¿la señal de reapertura es verificable? ¿respeta el contrato (métrica nombrada existente o pendiente)?
- **Cómo se probó:** grep de métricas citadas en `src/metrics/` (verificación mecánica, no auto-reporte).
- **Checklist anti-hábitos tóxicos:** — (verificado por revisor externo; no se fabricaron salidas de comandos; no se declaró done sin verificar contra el contrato).
- **Veredicto:** ✅ approve — verificación mecánica del contrato: grep confirma existencia de proxies y ausencia (documentada) de la métrica de grafos.

## Notas
- El ADR documenta el gap honestamente: "sin datos de uso de grafos — la señal se instrumenta cuando /metrics agregue contadores de uso por feature".
- No se instrumenta nada: instrumentar `vanta_graph_ops_total` es otra tarea (vanta-tuner, post-launch).
- No se toca Cargo.toml: cambiar el default es la ACCIÓN condicionada al umbral, no una decisión tomada hoy por intuición.