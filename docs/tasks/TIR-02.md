# TIR-02: DORA recovery time + rework rate

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 449)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — doc `docs/Investigaciones/TIR-02-dora-recovery-rework.md` (IMPLEMENTAR recovery time; DEFERIR rework rate)
- **Incógnitas (uphill):** 1 — ¿la telemetría actual (verify-log.jsonl) alcanza para recovery time y rework rate?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | observabilidad del pipeline (reportes DORA) |
| Callees | `docs/reports/dora.md:183-197`, telemetría `verify-log.jsonl` (Task 2 histórico) |
| Implicaciones | Decisión de métricas del pipeline — sin cambios de código del producto |

## Contrato
"`docs/Investigaciones/TIR-02-dora-recovery-rework.md` existe con: (1) viabilidad de recovery time (tiempo en volver a DE tras fallo) y rework rate (tareas reabiertas/total) con la telemetría ACTUAL — estado real de `verify-log.jsonl` verificado; (2) recomendación EXPLÍCITA (implementar / WONTFIT / deferir)."

## Herramientas
- Read, grep/glob (verificar verify-log.jsonl), bash read-only

## Steps
### Step 1: Leer fuentes + verificar telemetría
- **Archivos:** `docs/reports/dora.md:183-197`, `eng-03-project.md §8.3` (en `docs/Investigaciones/2026-08-10-agent-engineering/`)
- **Acción:** leer las métricas DORA actuales; verificar si `verify-log.jsonl` existe y qué registra (grep/glob). ¿Permite calcular recovery time y rework rate?
- **Verify:** estado real de verify-log.jsonl documentado (existe/no-existe, campos)
- **Estado:** ⬜ PENDING

### Step 2: Analizar viabilidad
- **Acción:** mapear qué datos faltan para cada métrica; opciones (registro en campaign-server.mjs vs manual vs inviable)
- **Verify:** análisis documentado
- **Estado:** ⬜ PENDING

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-02-dora-recovery-rework.md`
- **Acción:** doc con viabilidad + recomendación explícita
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna

## Notas
- Tarea read-only de investigación: NO editar código ni telemetría. NO commitear (el lead commitea).
- SECURITY/PERFORMANCE: no aplica (sin cambios de código) — justificado.
- Review: lead (orquestador).