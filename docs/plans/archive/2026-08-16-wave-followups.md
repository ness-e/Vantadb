# Wave Follow-Ups P20 — Aplicar y arreglar los 15 follow-ups post-campaña

> **Estado:** in-progress · **Campaign ID:** fup-2026-08-16
> **Fuente:** revisión de los 25 task files de la campaña `2026-08-16-wave-p20-tsys.md` (ejecutada esta sesión, 25/25 ✅)
> **FAIL_MODE:** parallel · **MAX_CONCURRENT:** 3 · **DO:** 15 (11 delegadas + 4 lead)

## Waves

| Wave | Tareas | Sub-agente | Tipo |
|---|---|---|---|
| W1 | FND-01-F1, FND-02-M3, FND-06-F1 | worker / arch / worker | Core fixes |
| W2 | FND-05-F1, TSYS-06-F1, FND-02-M2 | worker / arch / chaos | Bindings + infra + stress |
| W3 | FND-23-F1, FND-13-F2, FND-13-F1 | tuner / docs / worker | Obs + docs + web |
| W4 | FND-04-F1, P2R-01 | docs / review | ADR + gates P2-01 |
| W5 | Lead close (metadata, git rm, FND-16, FND-24, commits) | vanta-lead | Housekeeping |

## Routing

| Task | Archivos | Sub-agente |
|---|---|---|
| FND-01-F1 | src/storage/engine/stats.rs, .opencode/rules/memory-budget.md | vanta-worker |
| FND-02-M3 | src/storage/engine/maintenance.rs, delete.rs, tests | vanta-arch |
| FND-06-F1 | vantadb-ts/src/vantadb.ts (fix en binding; core ya rechaza zero-norm) | vanta-worker |
| FND-05-F1 | vantadb-python/*.pyi, __init__.py, pyproject.toml | vanta-worker |
| TSYS-06-F1 | .opencode/task-system/mcp/campaign-server.mjs, parsers.mjs (nuevo) | vanta-arch |
| FND-02-M2 | src/storage/engine/tests/ops.rs (stress test) | vanta-chaos |
| FND-23-F1 | src/metrics/core/registry.rs, ADR-024 (nota) | vanta-tuner |
| FND-13-F2 | docs/operations/BENCHMARKS.md, PERFORMANCE_TUNING.md | vanta-docs |
| FND-13-F1 | web/src/ (claims fantasma) | vanta-worker |
| FND-04-F1 | docs/architecture/adr/ADR-025-*.md (nuevo) | vanta-docs |
| P2R-01 | revisión read-only de todos los fixes + retros TSYS-06/FND-07 | vanta-review |
| W5 lead | task files metadata, git rm typescript-expert, FND-16 actionlint, FND-24 watch, commits | vanta-lead |

## Archivos protegidos (TODOS los sub-agentes)

`docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/AUD-024.md`, `.opencode/task-system/enforcement/verify-log.jsonl`, `completions/_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-followups.md`, `.opencode/AGENTS.md`, `.opencode/agents/*`.

**Excepción única:** TSYS-06-F1 toca `.opencode/task-system/mcp/campaign-server.mjs` (+ crea parsers.mjs) — solo esa tarea.

## Reglas

- Sub-agentes NO hacen git add/commit; NO usan campaign_update_task_state; crean su task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`; devuelven bloque RESULTADO (pipeline-full.md §7).
- El lead commitea por wave tras revisar cada RESULTADO (escalera SARL).
- Pre-commit hook: fmt/clippy/actionlint corren por wave.
- Cierre: plan → `docs/plans/archive/`, reporte final con veredictos P2-01.

## Resultado esperado

- Guard RSS real (FND-01-F1) — cierra riesgo OOM
- Stubs Python alineados (FND-05-F1) — type-checking del usuario OK
- Web sin claims fantasma (FND-13-F1)
- Race delete-vs-consolidate resuelto + stress test evicción (FND-02-M3/M2)
- Zero-norm sin fallback silencioso (FND-06-F1)
- Task-system con 3 behavior changes + parsers extraídos (TSYS-06-F1)
- Métrica vanta_graph_ops_total instrumentada (FND-23-F1)
- BENCHMARKS/PERFORMANCE_TUNING sin inconsistencias ni claims sin fuente (FND-13-F2)
- ADR-025 formal (FND-04-F1)
- Gates P2-01 retro + post-fixes (P2R-01)
- Metadata task files sync + git rm typescript-expert + FND-16 validado (W5)