# FND-13-F2: BENCHMARKS.md inconsistencias + PERFORMANCE_TUNING sin fuente (Regla 11)

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W3)
- **Fuente:** deuda anotada en FND-13 (inventario benchmarks honestos)
- **Estado:** ⏳ IN PROGRESS · **Sub-agente:** vanta-docs
- **Prioridad:** 🟡

## Objetivo
Aplicar la Regla 11 (claims con benchmark reproducible) a la documentación pendiente:
1. **BENCHMARKS.md §2 vs JSON:** §2 dice 95 ops/s, vector p50 62ms, hybrid p50 180ms; el JSON citado (gitignored) dice insert 74.0 rec/s, p50 13.2/2.0/3.1 ms; README alineado dice 74.0/2.0/3.1. Resolver: corregir §2 o marcarlo como medición antigua con fecha+comando (NO inventar números)
2. **Comandos faltantes en §4/§6/§7:** cada sección con números DEBE citar comando exacto (o marcar "sin comando documentado — no reproducible" y quitar los números)
3. **PERFORMANCE_TUNING T2-T12 sin fuente:** claims de HNSW/sync/SIMD/storage sin bench citado → o citar el bench existente o marcar "no verificado — Regla 11" o quitar el número (Regla 11: el adjetivo/número sin fuente no es evidencia)

## Archivos clave
- `docs/operations/BENCHMARKS.md` (212L), `docs/operations/PERFORMANCE_TUNING.md` (494L), `docs/Investigaciones/FND-13-benchmarks-honestos.md` (inventario con la deuda), `benchmarks/vanta_benchmark_report.json` (gitignored — fuente del README), `docs/operations/BENCHMARKS.md` §8 (baseline canónico FND-10)

## Steps
1. DISCOVERY: leer BENCHMARKS.md completo + PERFORMANCE_TUNING.md + inventario FND-13; localizar §2/§4/§6/§7 y T2-T12
2. Aplicar Regla 11: cada número → comando + entorno + fecha, O marcado "sin fuente documentada", O eliminado. §2: alinear con fuente citable o datar la medición antigua
3. Verificar: grep de números sin fuente restantes → 0 (o todos marcados); secciones con comando citado
4. Task file + RESULTADO

## Contrato (verify mecánico)
- BENCHMARKS.md sin contradicciones internas (mismos números ↔ misma fuente)
- Cada sección con números tiene comando exacto (o marcado explícito "no reproducible")
- PERFORMANCE_TUNING: claims sin fuente marcados o removidos (Regla 11)
- Inglés (docs técnicas — Doc Language Split)

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*, código, JSON gitignored
- NO git add/commit; NO campaign_update_task_state
- NO inventar números ni comandos — marcar "sin fuente" si no existe evidencia
- No tocar §8 (baseline canónico FND-10 — es fuente)

## Fases
- SECURITY: n/a
- PERFORMANCE: n/a

## Resultado
```
RESULTADO: ✅ COMPLETO | 🟡 INCOMPLETO | ❌ FALLIDO
STEPS_OK: <n>/<M>
PROXIMO_STEP: <...>
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: <paths tocados>
VERIFY_CONTRATO: <pasa | no-corrido | falla>
BLOQUEO: <ninguno | ...>
```