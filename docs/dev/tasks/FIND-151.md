# FIND-151 — Eval de agentes (Giskard) + Lurkr obligatorio en CI host

> **Plan:** `docs/dev/plans/2026-09-24-harness-gaps.md` §FIND-151 · **Wave:** 0 (paralelo FIND-148/150, archivos disjuntos)
> **Tipo:** devops (detect_task_type) · **Branch:** develop (+ `.opencode` repo separado) · **Commit:** conventional + FIND-151
> **SDP:** ci-cd-and-automation + doubt-driven-development + campaign-executor + ponytail (base)

## Contrato

Eval 10/10 en main + Lurkr exit 0 en `.opencode/` + CI verde 2 semanas.
Riesgo controlado: Giskard LLM-judge cuesta tokens → 10 casos deterministas offline, SOLO en `/audit full`, nunca en quick.

## Spec (decisiones técnicas — gate spec-first N/A: tarea devops/CI sin símbolos públicos nuevos)

| # | Decisión | Alternativas | Default + evidencia |
|---|----------|--------------|---------------------|
| 1 | Eval runner determinista offline, sin LLM-judge | Giskard `giskard.agents` ChatWorkflow con judge (tokens por run) | ✅ determinista (ref: plan riesgo "10 casos solo full"; `giskard.agents` v3.0.0 verificado importable, reservado como judge opcional futuro) |
| 2 | Lurkr vía `pip install lurkr` + GitHub Action `agentveil-protocol/lurkr@v0.4.0` | `npx lurkr scan` (Rule 17) | ✅ pip/Action (ref: https://github.com/agentveil-protocol/lurkr — `lurkr` NO existe en npm, 404 verificado; CLI real: `lurkr scan --path . --output report.json`) |
| 3 | Baseline `.lurkr-baseline.json` con 1 FP grandfathered | `--fail-on high` desde día 1 | ✅ baseline (ref: docs BASELINE.md upstream; triaje: `design_system.py:58` interpola query de buscador, no prompt LLM → FP) |
| 4 | Workflow NUEVO `lurkr-informational.yml`, `continue-on-error: true` con `# CATEGORY: INFORMATIONAL` | Editar `ci-rust.yml` (FIND-150 podría tocarlo) / sin CATEGORY | ✅ nuevo + CATEGORY (ref: AGENTS.md Regla 2; `arch-metrics-informational.yml` como modelo) |
| 5 | Hook L9 en perfil unified-review NO tocado | 1 línea en skill profile | ✅ no tocar (fuente viva de `/audit`; wiring mecánico = deuda Wave 1 con FIND-149; binding documentado en harness.md + Gate H) |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `docs/dev/plans/2026-09-24-harness-gaps.md` §FIND-151, `.opencode/commands/harness.md` (37L), `.opencode/task-system/prompts/question-gates.md` (Gate H §98-108), `.opencode/task-system/RULES.md` Rule 17, `.github/workflows/arch-metrics-informational.yml` (modelo), `.opencode/commands/audit.md` (router, L9 vive en skill unified-review), `evals/memory_bench.py` (convención evals).
- **Referencias hacia dentro:** evals nuevo no es importado por nadie (standalone `python evals/agent/run_eval.py`); workflow nuevo no es referenciado (standalone, `workflow_dispatch` + PR paths); baseline solo leído por el workflow nuevo.
- **Referencias entrantes:** harness.md + question-gates.md serán leídos por `/harness` y gates; audit.md NO se edita.
- **Veredicto:** blast radius = 3 archivos nuevos VantaDB + 1 baseline + 2 docs `.opencode` (repo separado). Sin hot path, sin API pública, sin símbolos nuevos. Gate D no dispara. PROHIBIDOS respetados: `agents/*`, `deny.toml`, DoD.

## Research digest (≤500 palabras, URLs verificadas 2026-09-24)

- **Giskard OSS** (Apache-2.0): plataforma de eval/red-teaming de agentes LLM; OSS en https://github.com/Giskard-AI/giskard, site https://www.giskard.ai, docs https://docs.giskard.ai. `pip install giskard` → v3.0.0 (verificado local); API `giskard.agents` (`Chat`, `ChatWorkflow`, `Generator`, `Tool` — verificado `import`). Hub 3.0 añade custom LLM Judge + 21 checks (https://www.giskard.ai/knowledge/giskard-hub-3-0-red-team-any-ai-agent-whatever-its-api). Uso aquí: solo casos dorados deterministas offline (0 tokens); judge LLM reservado a `/audit full` futuro.
- **Lurkr** (MIT, 6★): scanner estático local-only de capability risk en agentes (https://github.com/agentveil-protocol/lurkr). Instalación real: `pip install lurkr` (v0.4.0 verificado) o Action `agentveil-protocol/lurkr@v0.4.0` con `path/output/fail-on/baseline` (verificado en README upstream). `npx lurkr` NO existe (npm 404). CLI: `lurkr scan --path . --output report.json [--baseline X --fail-on high]`. 19 reglas `high`; modo baseline para adoptar en CI sin bloquear. Scan local de `.opencode/`: 1 hallazgo `agent.dynamic_prompt_from_user_input` en `skills/ui-ux-pro-max/scripts/design_system.py:58` → triaje: FP (construye query de buscador web, no prompt LLM) → grandfathered en baseline.

## Steps

- [x] **Step 1 — Eval scaffold:** `evals/agent/{golden_cases.json (10), run_eval.py, README.md}` (~200L total) — verify: `python evals/agent/run_eval.py` → 10/10 exit 0
- [x] **Step 2 — Lurkr CI:** `.lurkr-baseline.json` + `.github/workflows/lurkr-informational.yml` — verify: `lurkr scan --path .opencode --baseline .lurkr-baseline.json` exit 0 + YAML parse OK
- [x] **Step 3 — Docs harness:** `.opencode`: `commands/harness.md` §Eval+Lurkr + `question-gates.md` Gate H fila eval/Lurkr (vía `git -C .opencode`, commit separado) — verify: `git -C .opencode diff --stat`
- [x] **Step 4 — Cierre:** verify contrato + commits separados + plan/Backlog update + RESULTADO §7

## Context Save Point

COMPLETO 2026-09-24. Commits: VantaDB `7bda5df6`, .opencode `f425d88`.
Verify: eval 10/10 online+offline, lurkr exit 0 con baseline, YAML OK, diff-check OK.
Deuda: (1) hook mecánico L9 en skill unified-review (Wave 1 con FIND-149);
(2) promoción Lurkr a `--fail-on high` el 2026-10-08 si 0 FPs nuevos (ratchet en workflow + Gate H).
