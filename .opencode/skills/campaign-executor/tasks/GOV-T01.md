# GOV-T01: TIR-02a — métrica DORA recovery time en evals/dora.mjs

## Metadata
- **Plan file:** docs/plans/2026-08-22-doc-governance-plan.md
- **Fuente:** plan file, Task 1 (línea 42)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Docs/Evals (Node script)
- **Turns estimados:** 5-8
- **Creado:** 2026-08-22T09:00
- **last-synced:** 2026-08-22T09:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 (commit delegado al lead)
- **last-synced:** 2026-08-22T09:40

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno (script standalone; corre manual/CI ad-hoc) |
| Callees | `.opencode/task-system/enforcement/verify-log.jsonl` (read-only), escribe `docs/reports/dora.md` |
| Implicaciones | agrega sección "Recovery Time" al reporte; no cambia secciones existentes salvo renumeración de headers |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `evals/dora.mjs` (321 líneas completas), muestra de `.opencode/task-system/enforcement/verify-log.jsonl` (121 líneas, formato JSONL con ts/taskId/command/passed/exitCode)
- **Archivos referenciados hacia dentro:** dora.mjs lee PLANS_DIR, TASKS_ROOT, LOG (verify-log.jsonl), escribe OUT (docs/reports/dora.md). Sin imports externos (solo node:fs/path/url).
- **Archivos que referencian a los editados:** `evals/dora.mjs` no tiene callers en código; referencias documentales posibles en docs/plans y .opencode (grep "dora.mjs" — solo menciones doc).
- **Veredicto impacto:** bajo — script standalone, salida markdown aditiva, verify-log.jsonl read-only.

## Contrato
"`node evals/dora.mjs` exit 0 y el reporte incluye sección \"Recovery Time\" con los pares fail→pass detectados (se esperan ~3 pares históricos: ≈12.6h [T1-residuo-consolidado], ≈28.6h espurio [CI-05, exitCode:-1], ≈17s [AUD-033])."

### Hallazgo de DISCOVERY (semántica del emparejamiento)
Los 3 valores citados se reproducen SOLO si la clave de emparejamiento es **taskId**
(sin exigir igualdad de `command`): los reintentos cambian flags entre intentos
(p.ej. `cargo test -p vantadb` → `cargo fmt --check`), así que taskId+command estricto
da solo 4 pares recientes y pierde los 3 históricos citados. El par con
`exitCode:-1` SÍ se empareja pero se clasifica "no-ejecutado (espurio)" — así lo
describe el propio contrato ("≈28.6h espurio"). Entradas `taskId:null`: no
pareables (caveat documentado en Limitaciones). Archivo ausente: warning, sin crash
(`readVerifyLog()` ya devuelve `[]`).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** verify-log.jsonl NUNCA se escribe desde este script (read-only); secciones existentes del reporte (CFR, Throughput, Flow) no cambian de semántica; sin crash ante archivo ausente/log vacío/malformado.
- **Comandos de verificación:** `node evals/dora.mjs` → exit 0 + sección "Recovery Time" en docs/reports/dora.md con ≥3 pares incluyendo 12.6h, 28.6h (espurio), 17s.
- **Deuda pendiente:** ninguna

## Recitation (canónico)

Sincronizada vía `campaign_update_task_state`; campos §12 embebidos en `contract`/`result`.

## Deuda técnica (Regla 6)

Sin deuda nueva (≈35 líneas aditivas en script standalone, sin dependencias).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ |
| Commit | Lo ejecuta el LEAD (prohibido git para worker en esta tarea — regla del usuario) | delegado |
| Release | No aplica (script evals/, no crate/npm) | justificado |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries, input de usuario, auth ni dependencias nuevas. Lee JSONL local confiable.
- [x] **PERFORMANCE** — no aplica: script offline O(n²) sobre ≤cientos de entradas de log; no es hot path.

## Steps

### Step 1: Implementar recoveryPairs + sección "Recovery Time"
- **Archivos:** `evals/dora.mjs`
- **Acción:** función `recoveryPairs(entries)` (~15 líneas): empareja cada `passed:false` con taskId no-null con la siguiente `passed:true` del mismo taskId; Δt en horas; clasifica `exitCode:-1` como "no-ejecutado (espurio)". Insertar sección `## 3. Recovery Time` tras CFR (renumerar 3→4, 4→5, 5→6): tabla por-par + promedio de fallos reales + caveat taskId:null. Añadir bullet en Limitaciones.
- **Verify:** `node evals/dora.mjs` exit 0
- **Estado:** ✅ COMPLETED

### Step 2: Verificar contrato mecánico
- **Archivos:** `docs/reports/dora.md` (output generado, no editado a mano)
- **Acción:** correr `node evals/dora.mjs`; confirmar exit 0 y que la sección incluye ≈12.6h, ≈28.6h espurio, ≈17s.
- **Verify:** grep de "Recovery Time", "12.6", "28.6", "17s" sobre docs/reports/dora.md
- **Estado:** ✅ COMPLETED (evidencia: dora.md §3 Recovery Time desde línea 303 — tabla con 12.56h real, 28.59h espurio, 16.8s ≈17s; `campaign_verify_cmd node evals/dora.mjs` passed:true exit 0)

### Step 3: Review (GATE P2-01) + cierre
- **Archivos:** task file
- **Acción:** review por agente distinto (vanta-review) del approach y evidencia; marcar COMPLETED con recitation.
- **Verify:** verdict approve registrado abajo
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (primera tarea del plan GOV).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sesión ses_fd7464e71ffe2wpSiy9B6lzxXD, contexto fresco)
- **Enfoque:** emparejamiento por taskId defendible y justificado; loop O(n²) correcto (múltiples fallos antes de un pass se aparean todos al primer pass — semántica válida); edge cases verificados en código (taskId null, ts inválida, archivo ausente→[], exit -1 fuera del promedio).
- **Cómo se probó:** el revisor ejecutó `node evals/dora.mjs` por su cuenta (exit 0) y confirmó los 3 pares en docs/reports/dora.md:307-309.
- **Hallazgos no bloqueantes:** (🟡) orden de emparejamiento es orden-de-archivo, no ts — OK mientras el log sea append-only; (🟢) si supera 10k entradas, indexar con Map O(n).
- **Veredicto:** ✅ approve

## Notas
- Sin commit: regla explícita del usuario — el lead commitea. Worker solo edita `evals/dora.mjs` y este task file.
- Verify full cargo (fmt/clippy/nextest) no aplica: no se toca código Rust; el contrato mecánico es `node evals/dora.mjs`.
- El log creció desde que se redactó el plan (snapshot ≈2026-08-14 tenía exactamente los 3 pares citados; hoy hay más pares recientes) — "~3 pares históricos" se interpreta como "los 3 citados presentes".
