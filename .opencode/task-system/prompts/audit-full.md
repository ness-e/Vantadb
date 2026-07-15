# /audit — Unified Audit Pipeline Orchestrator

Cargá las skills `vanta-design-orchestrator`, `progreso`, `ponytail` (full).

## Modo de operación

Ejecutás UN modo a la vez. El modo determina qué fases corren.
Usás sub-agentes vía `task` tool para skills de review — cada skill es un sub-agente con contexto fresco que devuelve findings estructurados.

## Modos

| Comando | Fases | Uso |
|---------|-------|-----|
| `/audit` | 0→1→2→3→4→5→6→7→8 | Full pipeline |
| `/audit quick` | 1 | Solo CLI checks |
| `/audit certify` | 0→1→2→3→4→8 | Pre-push gate |
| `/audit review` | 0→4→5→6→7 | Deep review sin CLI |

## Flujo por fase

### Phase 0: Pre-check (30s, directo)

```bash
git diff --name-only HEAD    # cambios sin commit?
git status --short           # estado del working tree
```

- Sin cambios → sugerir `/audit full`
- Cambios sin stage → continuar normal
- Cambios staged → sugerir `/audit certify`

Output: `{hasChanges, stagedCount, unstagedCount, branch}`

### Phase 1: CLI Mechanical (directo)

Si mode=quick: `just verify` (fmt + clippy + test + deny)
Si mode=ci/certify: `just ci` (fmt + clippy + test + deny + audit)
Si mode=full: `just verify && just audit-cargo && just machete && just size`

```bash
just verify                    # fmt + clippy + test + deny
just audit-cargo               # cargo audit
just machete                   # unused deps
just size                      # binary bloat
```

Capturar outputs, contar failures.
**Si fmt o test fallan → ABORTAR.** No seguir a Phase 2.

Output: `{passed, failed, results: {fmt, clippy, test, deny, audit, machete, bloat}}`

### Phase 2: Security (sub-agente paralelo + skill)

Spawn UN sub-agente:

```
Task: Security audit of VantaDB
Skills: skill security-and-hardening
Data: cargo audit output (Phase 1), deny.toml
Deliverable: findings list [{file, line, severity, description, recommendation}]
Rules:
- Load skill security-and-hardening
- Review threat model: input validation, unsafe blocks, unwrap(), panics in public API
- Review deny.toml for allowed advisories
- Return findings as structured markdown with file:line references
- Can VETO with reason
```

### Phase 3: Performance (sub-agente paralelo + skill)

Spawn UN sub-agente:

```
Task: Performance audit of VantaDB
Skills: skill performance-optimization
Data: cargo bloat --crates output (Phase 1)
Deliverable: findings list [{file, line, severity, description, recommendation}]
Rules:
- Load skill performance-optimization
- Review binary size (cargo bloat output)
- Review hot paths: vector search, serialization, WAL
- Check for unnecessary allocations, clones, large enum variants
- Return findings as structured markdown with file:line references
- Can VETO with reason
```

### Phase 4: Code Review (sub-agente paralelo + 2 skills)

Spawn UN sub-agente:

```
Task: Code quality review of VantaDB
Skills: skill code-review-and-quality, skill ponytail-review
Data: git diff HEAD~1 (or staged), recent files changed
Deliverable: findings list [{file, line, severity, description, recommendation}]
Rules:
- Load BOTH skills: code-review-and-quality (5-axis review), ponytail-review (over-engineering)
- code-review-and-quality: correctness, security, performance, maintainability, style
- ponytail-review: YAGNI violations, unnecessary abstractions, stdlib replacements
- Review recent changes (last 10 commits or staged changes)
- Return findings as structured markdown
- Each skill can VETO independently
```

### Phase 5: Root-Cause Debugging (condicional, sub-agente)

SOLO si hay failures en phases 1-4. Si todas pasaron → SKIP.

Spawn UN sub-agente:

```
Task: Root cause analysis of Phase 1-4 failures
Skills: skill systematic-debugging
Data: All failure outputs from phases 1-4
Deliverable: rootCause analysis [{failure, rootCause, fix, estimatedEffort}]
Rules:
- Load skill systematic-debugging
- Analyze each failure: logs, test output, compiler errors
- Identify root cause (not symptom)
- Propose specific fix with file:line
- Estimate effort (minutes/hours)
```

### Phase 6: Deep Module Review (sub-agente + skill)

SOLO en mode `review` o `full`.

Spawn UN sub-agente:

```
Task: Deep module review of each VantaDB module
Skills: skill review-deep
Deliverable: perModule report [{module, score, issues, recommendations}]
Rules:
- Load skill review-deep
- Iterate per module: vantadb core, vantadb-python, vantadb-server, vantadb-mcp
- For each: codegraph_explore → web research → competitor compare
- Score each module 0-10
- Return findings as structured markdown
```

### Phase 7: Full ISO 25010 Review (sub-agente + skill)

SOLO en mode `full`.

Spawn UN sub-agente:

```
Task: Full ISO 25010 review of VantaDB
Skills: skill vantadb-full-review
Data: All findings from phases 1-6
Deliverable: isoReport {scores: {quality, security, performance, architecture, tests, docs}, findings, recommendations}
Rules:
- Load skill vantadb-full-review
- Quality model: functional suitability, reliability, usability, efficiency, maintainability, portability
- Cross-reference findings from all prior phases
- Score each dimension 0-10
- Return structured markdown report
```

### Phase 8: Certification Gate (directo + sub-agente)

SOLO en mode `certify` o `full`.

Primero: `just certify` (nocturnal_suite.ps1 — heavy tests)
Luego: spawn sub-agente con `skill vantadb-certify`

```
Task: Certification gate check
Skills: skill vantadb-certify
Data: nocturnal_suite.ps1 output, all prior findings
Deliverable: certification verdict (PASS/FAIL) + evidence
Rules:
- Load skill vantadb-certify
- Check all contracts from prior phases are resolved
- Verify no blocking issues remain
- Return PASS or FAIL with evidence
```

## Parallel Execution Plan

Fases que pueden correr en PARALELO (misma oleada):
- Wave 1: Phase 0 + Phase 1 (dependencias: CLI debe pasar primero)
- Wave 2: Phase 2 + Phase 3 + Phase 4 (independientes entre sí, dependen de Phase 1)
- Wave 3: Phase 5 (depende de failures en Wave 2)
- Wave 4: Phase 6 + Phase 7 (dependen de Waves 1-3, independientes entre sí)
- Wave 5: Phase 8 (depende de todas las anteriores)

## Report Generation

Al finalizar TODAS las fases del modo actual:

1. Recopilar findings de todos los sub-agentes
2. Escribir reporte en `docs/audit-reports/audit-<mode>-<timestamp>.md`:

```markdown
# Audit Report: <mode>
**Date:** <timestamp>
**Duration:** <elapsed>
**Mode:** <mode>

## Summary
- Phases completed: X/Y
- Blocking issues: N
- Recommendations: N
- Veredict: ✅ PASS / ❌ FAIL

## Per-Phase Results

| Phase | Status | Wave | Details |
|-------|--------|------|---------|
| 0. Pre-check | ✅ | direct | 3 modified files |
| 1. CLI | ✅ | direct | fmt ✅ clippy ✅ test ✅ deny ✅ |
| 2. Security | ✅/❌/⏭ | sub-agent | 2 findings |
| 3. Performance | ✅/❌/⏭ | sub-agent | 1 finding |
| 4. Code Review | ✅/❌/⏭ | sub-agent | 5 findings |
| 5. Root Cause | ✅/❌/⏭ | sub-agent | — |
| 6. Deep Module | ✅/❌/⏭ | sub-agent | 8 findings |
| 7. Full ISO | ✅/❌/⏭ | sub-agent | scores |
| 8. Certify | ✅/❌/⏭ | sub-agent | PASS/FAIL |

## Findings by Phase

### Phase 2: Security
- `src/engine.rs:142` — unsafe without SAFETY comment (high)
- `vantadb-server/src/routes.rs:88` — unwrap() on user input (high)

### Phase 3: Performance
...

## Scoreboard

| Category | Score (0-10) | Notes |
|----------|-------------|-------|
| Code Quality | 8 | Minor clippy warnings |
| Security | 9 | No vulnerabilities |
| Performance | 7 | Binary could be smaller |
| Architecture | 8 | Clean module boundaries |
| Tests | 9 | All passing |
| Docs | 6 | Missing API docs |

## Recommendations (prioritized)
1. (high) `src/engine.rs:142` — add SAFETY comment to unsafe block
2. (medium) `vantadb-server` binary size: 2.3MB → target <1.5MB
```

## Post-Report

- Si mode=full o review: agregar findings como nuevas tareas en `docs/Backlog.md`
- Si mode=certify: si todas las fases pasan → "✅ CERTIFY PASSED — safe to push". Si alguna falla → "❌ CERTIFY FAILED — fix errors above before pushing"
- Ejecutar `skill progreso` al final

## Rules

- **Phase 1 failure = STOP.** No seguir a Phase 2 si fmt o tests fallan.
- **Sub-agentes:** máximo 15 tool calls internas, ~3 min timeout. Si no responden, matar y registrar como timeout.
- **Skills pueden VETO:** registrar el veto en el reporte pero no abortar el pipeline completo (excepto Phase 1).
- **Reporte cubre TODAS las fases** independientemente de pass/fail.
- **Timestamp ISO8601** en todos los reportes para audit trail.
- **Si un sub-agente falla** (timeout/error): registrar como `❌ SUBAGENT_ERROR` y continuar con las siguientes fases.
