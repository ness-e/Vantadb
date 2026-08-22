# Validación de `docs/research/2026-08-10-agent-engineering` — 2026-08-11

## Summary
- **Método:** 10 sub-agentes `explore` en paralelo (1 por archivo; REPORTE-FINAL dividido en 2 mitades), cada uno contrastando cada claim del documento contra el código/CI/prompts reales del repo con evidencia `ruta:línea`.
- **Resultado:** 390 ✅ (81%), 92 ⚠️ (parcial), 48 ❌ (no aplicado), 14 📌 (informativo). **Los P0-P3 y TSYS-01..16 están aplicados y verificados con commits.**
- **Discrepancias resueltas:** 3 falsos negativos (ADR gate mecánico, DMAIC, refactoring 2 sombreros) descartados a favor de la evidencia directa.

## Conteo por archivo

| Archivo | ✅ | ⚠️ | ❌ | 📌 |
|---------|----|----|----|----|
| agent-01-fundaments.md | 48 | 8 | 7 | 0 |
| agent-02-task-execution.md | 31 | 11 | 3 | 1 |
| agent-03-orchestration.md | 34 | 11 | 3 | 0 |
| eng-01-software.md | 32 | 6 | 4 | 4 |
| eng-02-systems.md | 23 | 4 | 6 | 4 |
| eng-03-project.md | 26 | 10 | 2 | 0 |
| gap-01-agents.md | 45 | 4 | 7 | 0 |
| gap-02-engineering.md | 44 | 8 | 2 | 2 |
| REPORTE-FINAL (2 partes) | 107 | 30 | 14 | 3 |
| **Total** | **390** | **92** | **48** | **14** |

## Gaps reales (deduplicados, sin DEFER ya registrados)

### Alta prioridad
1. **Compaction de contexto** — sin mecanismo runtime; harness depende de note-taking manual (agent-01 #21/#40/#59).
2. **Merge estructural del lead / TSYS-12 runtime** — solo design doc "Proposed"; fork/join síncrono sin merge de duplicados/huecos en runtime (agent-03 #23).
3. **Métricas DORA recovery time + rework rate** — dora.md mide lead/CFR/throughput, no recovery ni reabiertos; bloqueado por verify-log joven (eng-03 §8.3).
4. **Merge a main como DoD de tarea** — pipeline cierra en CLOSE/commit; integración a trunk queda humana (REPORTE §3.3-25).
5. **Mitigación primero en incidentes (contención)** — sin fase de contención previa al fix (FALTA#15, §3.3-15).

### Media
6. **Post-release/monitoring en el loop** — pipeline termina en CLOSE sin verificación post-merge (§3.3-27).
7. **Feature flags / rollback feature-level / canary** — `/rollback` manual, sin flags runtime (eng-01 #2/#27, gap-02 §3.9).
8. **Chaos del task-system (TSYS-06 impl.)** — solo diseño; runner DEFER.
9. **LLM-as-judge (0.0-1.0)** — evals mecánicos, sin judge de fabricación (agent-03 #35).
10. **Dead-letter queue** — tareas que agotan retries solo escalan a humano (agent-02 §8.2).
11. **MTTD/MTTR operativos** — ADR-017 define SLIs sin ventana de medición real (eng-02 §4.4).
12. **Saturación <20% + Broadening/Narrowing + jitter** — criterios de investigación y retry sin implementar (agent-02 §7.2/§7.6/§8.1).

### Baja/cosmética
13. Kepner-Tregoe, Ishikawa, DAG eventos multi-causa, SCQA, Gherkin, Working Backwards, git bisect script, monitor flakiness, TSYS-17 README de evals, data provenance, context_budget_bytes ([SPEC]), múltiples trials por task.

### DEFER legítimos (con fecha, no perdidos)
- P3-2 calibración de estimación (requiere histórico ≥5 tasks con effort real).
- P3-3/P3-9 mutation testing ≥70% (agenda 2 semanas post-baseline).

## Discrepancias resueltas (falsos negativos descartados)
- **ADR gate mecánico (TSYS-03)** — SÍ existe: job `adr-gate` en `ci-rust-10.yml:120-181`.
- **DMAIC** — SÍ está: `systematic-debugging/SKILL.md:229-233` Phase 4.6.
- **Refactoring 2-sombreros** — SÍ está: `code-simplification/SKILL.md:173-185` + `RULES.md:234-247`.

## Desfase de docs corregido
- `docs/Backlog.md` §P17: columna de estado actualizada TSYS-02/03/04/06/07/08/10/11/12/13/16 (Pendiente → Implementado con commit ref) + header sincronizado. Commit a registro.
