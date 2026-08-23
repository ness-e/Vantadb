# Wave P20-TSYS — endurecimiento del task-system

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### Wave P20-TSYS completada 2026-08-16 — 25/25 tareas (plan `2026-08-16-wave-p20-tsys.md`)

Cierre de campaña: **25 tareas, 21 commits en `develop`** (desde `ec7f947a` hasta `a159211b`). Migradas a este registry el mismo día. Destacados:

- **TSYS-06**: decisión chaos runner → **DEFERIDO** con tests puntuales (doc `docs/research/TSYS-06-chaos-runner.md`).
- **P19 (R1/R3/R5/R6/R8/R9/R10)**: sistema de agentes endurecido — skills obligatorias §6, DISCOVERY delegado a vanta-research, permission blocks alineados con tablas MCP, §7 consolidado.
- **FND-01..24**: 3 reglas nuevas en AGENTS.md (Reglas 9/10/11), regla memory-budget (🔴 OOM confirmado, guard subestima 6.5×), deadlocks multi-índice fixeados, /metrics con latencia real, ADR-023/024, CONTRIBUTING.md, ICP/JTBD con hipótesis honestas.
- **Follow-ups delegados**: F1/F4 (memory) y 2/3 (deadlocks) → core-engine; FND-04 reapertura condicional → bindings/investigaciones.
