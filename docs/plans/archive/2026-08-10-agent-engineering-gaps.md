# Plan: 2026-08-10-agent-engineering-gaps.md

**Activa:** `fix(harness): cerrar gaps accionables de la investigación agent-engineering`

Cierra los 5 gaps de **familia A** (sistema de tareas, todos 🟢/🟡, <1h c/u) que
los 3 sub-agentes de auditoría (gap-01, gap-02, REPORTE-FINAL) confirmaron como
reales y accionables. Fuente: `docs/research/2026-08-10-agent-engineering/REPORTE-FINAL.md`.

Excluidos a propósito de este lote:
- **Dependientes de datos** (verify-log.jsonl = 0 bytes): P3-2 calibración, DORA rework/recovery, SLA. → desbloquear ejecutando la próxima tarea real por el pipeline.
- **Family B / dominio (heavy, deuda de código)**: P3-3 mutation gate ≥70%, P3-9 cargo-mutants, cobertura 60→80%, perf-vs-baseline, flakiness monitor. → ya en Backlog (COV-001..004) o backlog de deuda; agenda de revisión 2 semanas.
- **WONTFIT (dependencia externa)**: enforcement absoluto del MCP server (harness OpenCode), rainbow deploys (release-plz/GHA).

## Cantidad de tareas

5 (todas familia A). Ownership por archivo EXCLUSIVO — cero colisiones entre sub-agentes.

| # | Owner | Archivo(s) | Gap | Esfuerzo |
|---|-------|------------|-----|----------|
| T1 | vanta-worker | `.agents/skills/systematic-debugging/SKILL.md` | Contención (§3.3#15) + git bisect automatizable (§3.3#16) + DMAIC control (§3.3#28) | 🟢 |
| T2 | vanta-docs | `.opencode/references/definition-of-done.md` | Merge a main + CI verde como Release gate (§3.3#25) + rollback plan/feature flags como DoD de feature (§3.3#26) + post-release monitoring (§3.3#27) | 🟡 |
| T3 | vanta-worker | `.opencode/task-system/prompts/task.md` | Regla 6 mecanizada: campo "Deuda registrada/Sin deuda" (§3.5) + RED test explícito en bug gate (P1-5) | 🟢 |
| T4 | vanta-docs | `.opencode/AGENTS.md`, `.opencode/VANTADB-OPERATING-MANUAL.md` | Drift P1-7: docs dicen "hooks NO instalados" cuando core.hooksPath=.githooks los tiene activos | 🟢 |
| T5 | vanta-worker | `.opencode/task-system/config/state-tools.mjs` | P1-4: ACT.denied=["delete"] no prohíbe nada; documentar scope enforcement + verify node --check | 🟢 |

**Orden de ejecución:** T1..T5 en paralelo (ownership disjunto). Verificar contratos de cada una, luego git add/commit.

## Contratos

| Tarea | Contrato (condición verificable) |
|-------|----------------------------------|
| T1 | `rg "Fase 0 — Contención|git bisect|DMAIC|Control" .agents/skills/systematic-debugging/SKILL.md` → 4 hits; skill sin romper estructura (Iron Law intacta) |
| T2 | `rg "merge a main|rollback|post-release|monitoring" .opencode/references/definition-of-done.md` → secciones nuevas presentes; tabla DoD por nivel intacta |
| T3 | `rg "Deuda registrada|RED test|test que falle" .opencode/task-system/prompts/task.md` → 3 hits; formato del task file coherente (Markdown no roto) |
| T4 | `rg "NO están instalados" .opencode/AGENTS.md .opencode/VANTADB-OPERATING-MANUAL.md` → 0 hits; `rg "core.hooksPath"` → ≥2 hits con estado correcto |
| T5 | `node --check .opencode/task-system/config/state-tools.mjs` → exit 0; comentario/guard de scope presente |

## Handoff

- Commit: `fix(harness): cerrar 5 gaps familia A de investigación agent-engineering`
- Registrar en `.opencode/skills/campaign-executor/RULES.md` si T3 lo requiere (Regla 6).
- Próx. tarea recomendada: ejecutar una tarea real del backlog por el pipeline para poblar `verify-log.jsonl` (desbloquea North Star, DORA, P3-2, SLA).