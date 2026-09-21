# TIR-06: Post-release / monitoring en el loop

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 452)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — doc `docs/Investigaciones/TIR-06-post-release-monitoring.md` (DEFERIR; cierre opcional 1 línea en pipeline-run.md)
- **Incógnitas (uphill):** 1 — ¿paso de "verificación post-release" opcional en el loop vs delegación a progreso/registro?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | cierre de campaña (CLOSE → commit sin verificación post-merge) |
| Callees | `REPORTE-FINAL §3.3-27`, `definition-of-done.md:104` (DoD monitoring), `progreso` skill |
| Implicaciones | Decisión de diseño del pipeline — sin cambios de código del producto |

## Contrato
"`docs/Investigaciones/TIR-06-post-release-monitoring.md` existe con: (1) análisis del gap (el pipeline cierra en CLOSE sin verificación post-merge; DoD (d) monitoring); (2) comparación paso-verificación-post-release-opcional vs delegación a progreso/registro; (3) recomendación EXPLÍCITA (implementar / WONTFIT / deferir)."

## Herramientas
- Read, grep

## Steps
### Step 1: Leer fuentes
- **Archivos:** `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` (§3.3-27), `.opencode/references/definition-of-done.md:104`, `progreso` skill (Trigger 1)
- **Acción:** leer el gap actual (cierre en CLOSE, sin post-merge) y qué cubre hoy el DoD monitoring
- **Verify:** gap documentado
- **Estado:** ⬜ PENDING

### Step 2: Comparar opciones
- **Acción:** (a) paso de verificación post-release opcional en el pipeline (qué verificar, cuándo, quién), (b) delegación a progreso/registro (ya existe Trigger 4 sync reportes — ¿cubre?), (c) no hacer nada
- **Verify:** comparación documentada
- **Estado:** ⬜ PENDING

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-06-post-release-monitoring.md`
- **Acción:** doc con análisis + recomendación explícita
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna

## Notas
- Tarea read-only de investigación. NO commitear (el lead commitea).
- SECURITY/PERFORMANCE: no aplica — justificado.
- Review: lead (orquestador).