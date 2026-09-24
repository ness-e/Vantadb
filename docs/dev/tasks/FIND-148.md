# FIND-148 — Frontmatter v2 + descriptions trigger (11 agents)

> **Plan:** `docs/dev/plans/2026-09-24-harness-gaps.md` §FIND-148 · **Wave:** 0 (paralelo FIND-150/151, sin solapamiento) · **Repo:** `configOpencode` (`.opencode/`, `git -C .opencode`)
> **Estado:** ✅ COMPLETO (`a56dde9` en repo `configOpencode`; task file en worktree host sin commitear — lo commitea el lead) · **SDP:** base-only + webfetch docs oficiales (keywords: frontmatter, agents, permissions)

## Contrato

- `rg -n "bash:|task:" .opencode/agents/` → vacío (cero matches)
- `opencode debug agents` lista los 11 `vanta-*` sin warnings de permisos
- descriptions ≤1024 chars, formato trigger ("Use me when X… Never use me for Y…")
- NO tocar `mode:` ni `model:` en frontmatters · NO tocar nada fuera de `.opencode/`

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.opencode/agents/vanta-review.md` (1-228), `.opencode/agents/vanta-lead.md` (1-36 + §8), `.opencode/AGENTS.md` (tabla límites:239-258 + enforcement:253-258 + excepción asimétrica:211-214); frontmatters (1-40) de arch/audit/chaos/docs/engine/research/tuner/worker/harness.
- **Referencias hacia dentro:** frontmatters referencian `question-gates.md` §Routing, `SKILLS-MANIFEST.md`, `task-system/prompts/task.md`, `references/task-system.md`; cuerpos citan `` `task()` `` / `task file` / `/pipeline task` (conceptos task-system, NO permission keys — no se tocan).
- **Referencias entrantes:** `.opencode/AGENTS.md` tabla de límites (contrato TSYS-11) + comentarios `# TSYS11:` por dominio; `vanta-lead.md` §8 delega vía `task(description, prompt, subagent_type)` a `vanta-*`.
- **Veredicto:** blast radius = 11 frontmatters + tabla AGENTS.md + 3 líneas cuerpo vanta-research (56/62/66, usan `bash:`/`task:` con colon). Riesgo bajo (solo metadata permisos). Rollback = `git -C .opencode revert` del commit piloto/global.

## Investigación internet (digest ≤500 palabras, URLs verificadas vía webfetch 2026-09-24)

- https://v2.opencode.ai/docs/agents — vocab v2: acciones `shell` (comandos), `edit` (edit/write/patch), `subagent` (hijos); `permissions` = lista ordenada `{action, resource, effect}`, **last-match-wins** ("broad wildcard first, exceptions after"); `description` explica propósito y OpenCode **la muestra al modelo que elige qué agente lanzar**; legacy `permission`/`tools`/`temperature`/`maxSteps` → "Do not use in new V2 config". Sin mención de límite 1024 chars en esta página [límite 1024 NO VERIFICADO en fuente — viene del plan].
- https://code.claude.com/docs/en/subagents.md — Claude delega por `description` ("write a clear description so Claude knows when to use it"); formato trigger `description: ... Use proactively after code changes`; warning si descriptions combinadas >15000 tokens (cortas + detalle al system prompt). Sin límite 1024 por agente en esta página [límite 1024 NO VERIFICADO en fuente — viene del plan].
- **Hallazgo vivo (verificación propia):** `opencode debug agents` (v2.0.16) parsea el legacy actual mapeando `bash:`→`shell` y `task:`→`subagent` (evidencia: vanta-review con `bash: allow`+`task: deny` en disco aparece con `{"action":"shell",allow}` + `{"action":"subagent",deny}`). Gate piloto: VERDE — el rename es higiénico (future-proof), no fix de ruptura.

## Steps

- [x] **Step 1 — verificar base:** `git -C .opencode log --oneline -3` = `ceca4d7` ✅; `rg -c` confirma 11 archivos con `bash:|task:` (26 matches) ✅
- [x] **Step 2 — piloto vanta-review:** `bash:`→`shell:`, `task: deny`→`subagent: deny`, `vía bash`→`vía shell` (líneas 28-29), description → trigger ✅
- [x] **Step 3 — verificación piloto:** `opencode debug agents` → vanta-review con `shell`+`subagent`, presente en catálogo ✅ (verificación viva: mapeo legacy ya existía → sin rollback)
- [x] **Step 4 — masificar 10 restantes:** mismo rename + `vía bash`→`vía shell` en comentarios; cuerpo vanta-research:56/62/66 (`bash:`/`task:`→`shell:`/`subagent:`) ✅
- [x] **Step 5 — descriptions trigger:** reescribir 10 descriptions (harness: 1 palabra `Use me when`, se conserva resto) ≤1024 chars ✅
- [x] **Step 6 — tabla AGENTS.md:** header `Bash (build/test/bench)`→`Shell (…)`, `task (delegar…)`→`subagent (delegar…)`; `task: * deny`→`subagent: * deny` (líneas 213, 256); verify contrato + commit `git -C .opencode` ✅ (`a56dde9`)

## Context Save Point

Steps 1-3 ✅ (piloto + gate verde). Próximo: Step 4 (masificar). Deuda explícita: reinicio interactivo de OpenCode + sesión de prueba "revisa este plan con vanta-review" no ejecutable por subagente → pendiente usuario.
