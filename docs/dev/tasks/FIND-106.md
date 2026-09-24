# FIND-106 — ganchos de memoria por cliente

> **Plan:** `docs/dev/plans/2026-09-17-mvp-memoria-agentes.md` (Wave2, con FIND-104 ✅ `a1bea54b` en paralelo)
> **Campaign:** `b2ece025-e9d3-4f8b-835d-1d0143a86b66` · **Branch:** `develop` · **Commit:** `feat: FIND-106 — ...`
> **Ruta:** vanta-worker · **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta
> **Estado:** ⏳ IN PROGRESS (S0 discovery ✅, S1–S4 pendientes)

## 1. TAREA: objetivo + contrato + AC

**Objetivo:** plantillas de ganchos de memoria por cliente (OpenCode + Claude + Cursor + Codex, Q3 del plan)
que implementan la política MOTOR de FIND-103 de forma determinista — el único "siempre" del MVP.
Se consume la política, no se re-deriva: `skills/vantadb-mcp/references/recall-policy.md`
+ `instructions` del `initialize` + traductor temporal (`commit c01baa90`).

**Investigación + plantillas:**
- SessionStart recall (`memory_recall` scope agent, top_k 5 → inyectar `prepend_context` como `additionalContext`).
- Recall por mensaje con umbral top-k (query = texto verbatim del usuario, nunca paráfrasis; modo hybrid con
  degradación keyword anunciada vía `effective_mode`).
- PreCompact save-state (`thread_send` turnos abiertos y/o `scene_write` — lo no guardado se pierde).
- Stop auto-capture (`thread_send` role+content; proxy-turns se cura vía inbox, nunca auto-promote).
- Presupuesto de tokens con umbrales, qué se inyecta y cuándo NADA se inyecta.
- Tests por cliente con eventos simulados (sin servidor vivo, sin red, deterministas).
- Versionado de plantillas ante APIs cambiantes (pre-mortem #1).

**Contrato:**
1. Plantillas por cliente (OpenCode + Claude + Cursor + Codex, Q3).
2. Tests con eventos simulados verdes.
3. Presupuesto de tokens documentado (umbral, top-k, cuándo NADA se inyecta).

**AC:**
- (a) `skills/vantadb-mcp/assets/hooks/` existe con 1 subdir/archivo por cliente y cada plantilla
  mapea los 4 ganchos de la política (SessionStart / por-mensaje / PreCompact / Stop).
- (b) 1 suite de eventos simulados por cliente, verde sin red ni servidor (`pwsh -NoProfile` exit 0).
- (c) `TOKEN-BUDGET.md` documenta umbral (~10% ventana por mensaje), top-k default 5, y las 4 reglas
  de "cuándo NADA se inyecta" (§3 recall-policy: recalled vacío / non-empty verbatim / keyword-débil /
  fallback temporal 30d avisado).

## 2. ARCHIVOS

**Clave (nuevos, míos):**
- `skills/vantadb-mcp/assets/hooks/` (subdir NUEVO — todo lo mío vive acá)
- `skills/vantadb-mcp/assets/hooks/README.md` — mapa + instalación por cliente + versionado
- `skills/vantadb-mcp/assets/hooks/TOKEN-BUDGET.md` — presupuesto tokens (AC-c)
- `skills/vantadb-mcp/assets/hooks/VERSION.md` — versión plantilla + matriz compatibilidad
- `skills/vantadb-mcp/assets/hooks/opencode/vantadb-memory.js` — plugin (`session.created`,
  `tool.execute.before`, `experimental.session.compacting`, `session.idle`)
- `skills/vantadb-mcp/assets/hooks/claude/settings.example.json` — SessionStart/UserPromptSubmit/PreCompact/Stop
- `skills/vantadb-mcp/assets/hooks/cursor/hooks.json` — sessionStart/beforeSubmitPrompt/preCompact/stop
- `skills/vantadb-mcp/assets/hooks/codex/hooks.json` — SessionStart/UserPromptSubmit/PreCompact/Stop
- `skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1` — simuladores de eventos (AC-b)

**Relacionados (SOLO lectura, jamás edits):**
- `skills/vantadb-mcp/references/recall-policy.md` (FIND-103 ✅ `c01baa90` — política MOTOR, se consume)
- `skills/vantadb-mcp/SKILL.md` § Continuous Recall (ya apunta a FIND-106, no editar)
- `skills/vantadb-mcp/assets/install/` (FIND-104 ✅ `a1bea54b` — NO tocar; mis plantillas reutilizan su
  comando `pwsh -NoProfile -File <REPO>/vanta-mcp-local.ps1 -DbPath <DB_PATH>` y su `agent-rule.md`)
- `vanta-mcp-local.ps1` (solo lectura — lo editó FIND-104 en paralelo, NO tocar)
- `.opencode/` config VIVA (ajena — SOLO lectura para no colisionar con plugin dir `.opencode/plugins/`;
  mis plantillas van a `assets/hooks/`, nunca a `.opencode/` vivo)
- `.opencode/rules/server-mcp.md` (leída completa 2026-09-17 — R-1 versiones coherentes aplica a plantillas)

**Prohibidos (no tocar ni leer como spec):**
`reparacion.bat`, `.opencode` vivo (edits), `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`,
`completions/*`, `desktop/src-tauri/Cargo.lock`, `stash@{0}` GOV-C4, `docs/dev/Backlog.md`, plan file (solo append
recitation lo hace el orquestador), `src/llm.rs`, `examples/`, `vanta-memory/`, `vantadb-mcp/src/`,
`setup-embeddings.ps1`.

## 3. DEPENDENCIAS

- **Wave2 paralelo:** FIND-104 ✅ `a1bea54b` — split respetado por directorio: él scripts + `assets/install/`,
  yo `assets/hooks/`. Cero archivos compartidos en escritura. `git status` verificado 2026-09-17: WIP ajeno
  (`M .opencode`, `M completions/*`, `M docs/dev/Backlog.md`, `M plan`, `?? reparacion.bat`) intocable.
- **Hereda:** FIND-103 `c01baa90` (recall-policy.md + instructions + traductor temporal + tests 12/12).
  Mis plantillas citan la política, no la duplican.
- **NextTask:** Wave3 (FIND-105 publica wizard+hooks, FIND-108). Orquestador decide.
- **Stop rule:** cliente sin hooks estables → `codex-manual.md` style: documentado-manual, no bloquea resto.
  No aplica (los 4 tienen hooks estables verificados web 2026-09-17).

## 4. REFERENCIAS

- **Rules:** `.opencode/rules/server-mcp.md` (lectura completa ✅ — R-1 coherencia doc/código extiendo a
  plantillas: versiones y nombres de tools siempre reales `memory_recall`/`thread_send`/`scene_write`).
- **Refs:** `skills-engineering.md` (SDP v2 § abajo), `definition-of-done.md` (DoD v1: contrato + determinista
  + blast-radius + deuda + docs).
- **Commands:** `pipeline.md` (este task via pipeline-full.md exacto).
- **SPEC.md (Q3):** 4 clientes OpenCode+Claude+Cursor+Codex.
- **Tabla Spec:** N/A (config/plantillas, sin símbolos públicos Rust — Gate D no dispara por pub-symbols).
- **Docs oficiales (fuente primaria, webfetch 2026-09-17, digest ≤500 palabras en §9).**

## 5. SKILLS (SDP Paso 0b real, ≤8)

`campaign_discover_skills_v2(phase=BUILD, keywords=[hooks,memory,recall,templates,opencode,claude,cursor,codex,token-budget])`
→ 8 candidatas (score 1.0). Selección justificada (7/8, descarto `frontend-ui-engineering` — sin `web/`):

| Skill | Fase | Justificación | Score |
|-------|------|---------------|-------|
| documentation-and-adrs | BUILD | plantillas = docs versionadas + ADR implícita (por qué hook→tool) | 1.0 |
| test-driven-development | BUILD | eventos simulados RED→GREEN por cliente | 1.0 |
| context-engineering | BUILD | context pack por slice (rules→spec→source→error) | 1.0 |
| source-driven-development | BUILD | docs oficiales hooks como fuente primaria, citar URLs | 1.0 |
| api-and-interface-design | BUILD | mapa hook→MCP-tool es diseño de interfaz (contrato estable) | 1.0 |
| systematic-debugging | VERIFY | si un simulador falla, root-cause antes de fix | 0.9 |
| incremental-implementation | BUILD | slices verticales por cliente, ~100 líneas, siempre verde | 1.0 |

`SDP: documentation-and-adrs, test-driven-development, context-engineering, source-driven-development, api-and-interface-design, systematic-debugging, incremental-implementation`
`SKILLS_CARGADAS:` idem (las 7 cargadas con `skill <nombre>` esta sesión; campaign-executor/progreso/ponytail base sesión).

## 6. HERRAMIENTAS + MCP

- `agent-search` (descubrimiento APIs hooks — intentado 2026-09-17: `duckduckgo:bot_challenge, sogou:bot_challenge`, 0 resultados → escalado a webfetch directo ✅).
- `webfetch` docs oficiales (4/4 verificadas, §9).
- metasearchmcp: no disponible en este harness → verificación cruzada = 4 webfetch independientes + consistencia con recall-policy + FIND-104 `codex-manual.md`.
- Simuladores propios: `tests/test-hooks.ps1` (`pwsh -NoProfile`, sin red, exit 0 = verde).
- `campaign_verify_cmd`: bug exit -1 conocido (plan Riesgos) → bash directa documentada en RESULTADO.
- codegraph: N/A (config/plantillas, cero código Rust — declarado en plan y verificado: ningún `*.rs` tocado).

## 7. INVESTIGACIÓN CÓDIGO

N/A código Rust (cero `*.rs`). Base consumida como política (lectura completa):
- `recall-policy.md` (61L): hook map §1, defaults §2 (query verbatim, scope agent, top_k 5, hybrid),
  thresholds §3 (4 reglas NADA), temporal §4 (`parse_temporal_expression`, fallback 30d avisado), budget §5
  (~10% ventana), curaduría §6 (proxy-turns inbox, tools FIND-107 S4 DEFER), non-goals §7.
- MCP tools a invocar (nombres reales, de SKILL.md): `memory_recall` (alias `memory_search`), `thread_send`,
  `scene_write`/`scene_edit`, `memory_list` (temporal v1 client-side), `context_assemble`.
- Patrón in-repo citado por la política: `docs/dev/research/archive/COGNEE_EVALUATION.md:390` (`additionalContext`
  en SessionStart) + `docs/dev/tasks/complete/ECO-001.md:10-17` (OpenCode `session.created` vía JS plugin).
- `assets/install/*` (6 files leídos): comando canónico launcher + `agent-rule.md` (1 línea) reutilizados.

## 8. INVESTIGACIÓN PROBLEMA

- **Ruido vs señal:** scores de recall son rangos RRF fusionados, no probabilidades calibradas → v1 SIN cutoff
  absoluto; regla estructural (§3): vacío→NADA (decir "no recuerdo nada sobre X", nunca rellenar); no-vacío→
  `prepend_context` verbatim (nunca re-resumir — resúmenes derivan, Notion Problema); keyword-only→inyectar
  pero ranking débil; rango temporal nunca inventado → fallback 30d + avisar.
- **Presupuesto tokens:** ~10% ventana por mensaje para recall inyectado; PreCompact acotado por budget de
  compactación de la sesión (no por este archivo); preferir menos+recientes (recencia = desempate, nunca filtro;
  decisiones viejas vía §4). Números exactos por cliente en TOKEN-BUDGET.md (ventanas 2026: Claude 200k,
  Cursor ~128-200k según modelo, Codex/GPT-5.x 272k-400k, OpenCode según modelo).
- **Versionado ante APIs cambiantes:** `VERSION.md` por plantilla (versión plantilla + doc oficial + fecha
  verificación) + tests simulados que fallan si el schema cambia (campo ausente = rojo) + matriz compat.

## 9. INVESTIGACIÓN INTERNET (principal — digest ≤500 palabras + URLs verificadas webfetch 2026-09-17)

1. **Claude Code hooks** — `https://docs.anthropic.com/en/docs/claude-code/hooks` ✅ (ref completa: eventos,
   schema, stdin JSON, exit codes; lifecycle SessionStart/UserPromptSubmit/PreCompact/Stop). Eventos que mapeo:
   `SessionStart` (matcher startup|resume|clear|compact) → recall top-5; `UserPromptSubmit` (sin matcher) →
   recall por mensaje; `PreCompact` (manual|auto) → save-state; `Stop` → auto-capture. Config en
   `~/.claude/settings.json` / `.claude/settings.json` / plugin `hooks/hooks.json`. Confianza: alta.
2. **OpenCode plugins** — `https://opencode.ai/docs/plugins/` ✅ (eventos `session.created`, `session.compacted`,
   `session.idle`, `experimental.session.compacting`, `tool.execute.before/after`). Sin `SessionStart`: se usa
   `session.created` vía JS plugin (coincide con ECO-001 citado por la política). Confianza: alta.
3. **Cursor hooks** — `https://cursor.com/docs/agent/hooks` ✅ (`sessionStart/sessionEnd`, `beforeSubmitPrompt`,
   `preCompact`, `stop`, `beforeShellExecution`...; `hooks.json` proyecto `<root>/.cursor/hooks.json` o usuario
   `~/.cursor/hooks.json`; `beforeSubmitPrompt` = recall por mensaje; `preCompact` = save; `stop` = capture;
   exit 2 = bloquear, otro ≠0 = fail-open). Confianza: alta.
4. **Codex hooks** — `https://developers.openai.com/codex/hooks` ✅ (`SessionStart` matcher startup|resume|clear|
   compact, `UserPromptSubmit` sin matcher, `PreCompact`/`PostCompact` manual|auto, `Stop`, `SessionEnd`;
   `~/.codex/hooks.json` / `<repo>/.codex/hooks.json` / `config.toml [hooks]`; trust-review `/hooks`;
   `additionalContextLimit` para salidas grandes). Confianza: alta.
5. Token windows (contexto presupuesto, de docs vistas + modelos conocidos 2026): Claude 200k default,
   Cursor 200k default (modelos hasta 1M max), Codex GPT-5.x 272k. Cifras exactas por modelo en TOKEN-BUDGET
   como tabla con fecha; si el cliente cambia ventana, el ~10% se recalcula solo (fórmula, no número duro).

Total: 4/4 clientes con hooks estables → Stop rule no dispara. Sin citas NO VERIFICADAS (todo webfetch ✅).

## 10. VALIDACIÓN + CIERRE

- Verify contrato: `pwsh -NoProfile -File skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1` (4 clientes ×
  4 eventos simulados + budget assertions) + `cargo fmt --check` (N/A Rust — sin `*.rs`, se documenta) +
  OCR delegation advisory + DoD 3 niveles (task: contrato+tests; feature: integración con FIND-103/104 sin edits
  cruzados; release: no aplica) + P2-01 orquestador (agente distinto) + Gates D/V/C vía `question`
  (D: no dispara — blast <10 files, sin pub-symbols; V: si 2 fallas mismo-error; C: staging selectivo).
- RESULTADO §7 pipeline-full siempre (con `campaign_verify_cmd` o bash directa por bug exit -1).

## Impacto mapeado (Regla 0) — MUST antes del primer edit

- **Archivos leídos completos:** recall-policy.md (61L) · SKILL.md §§ Continuous Recall/Embeddings/Threat-Model ·
  install/{opencode.json, cursor.json, claude-desktop.json, agent-rule.md, codex-manual.md} · server-mcp.md ·
  skills-engineering.md · definition-of-done.md · pipeline-full.md · plan Wave2 ficha FIND-106.
- **Referencias hacia dentro (qué citan mis plantillas):** recall-policy.md (política) · `vanta-mcp-local.ps1`
  (comando launcher, solo citado) · `agent-rule.md` (1 línea, citada) · docs oficiales (4 URLs §9).
- **Referencias entrantes (quién me cita):** SKILL.md § Continuous Recall ("per-client hook templates are
  FIND-106 (not here)") — NO edito SKILL.md (evito colisión); Wave3 FIND-105 me consumirá (wizard+hooks).
- **Veredicto:** ADITIVO PURO — 0 archivos existentes modificados, subdir nuevo `assets/hooks/` (~9 files).
  Blast radius: solo mis files + task file. Rollback: `git rm` del subdir. Sin hot paths, sin FFI, sin secrets.

## Spec (feature-add Gate mecánico — tabla de decisiones)

| # | Decisión | Opción elegida | Evidencia / por qué |
|---|----------|----------------|---------------------|
| 1 | Dónde viven plantillas | `assets/hooks/` nuevo, nunca `.opencode/` vivo | split Wave2 plan + DETALLE p2 (FIND-104 owns install/) |
| 2 | Formato OpenCode | JS plugin (`session.created` etc.), no JSON | docs oficiales plugins + ECO-001 (no hay SessionStart) |
| 3 | Recall por mensaje | hook por-mensaje con query verbatim (Claude UserPromptSubmit, Cursor beforeSubmitPrompt, Codex UserPromptSubmit, OpenCode tool.execute.before) | recall-policy §1-2 |
| 4 | Umbral inyección | estructural, sin cutoff absoluto (4 reglas §3) | scores RRF no calibrados (política §3) |
| 5 | Presupuesto | ~10% ventana/mensaje + top_k 5 default | política §5 + §2 |
| 6 | Tests | PS1 simulado offline por cliente (JSON parse + schema + eventos) | contrato AC-b, sin servidor en CI fast-gate |
| 7 | Codex sin formato verificado | hooks.json oficial (docs 2026) + nota manual heredada | FIND-104 codex-manual Stop rule superada por docs oficiales |
| 8 | Secrets | nunca en plantillas (solo `<DB_PATH>` placeholder + env sesión) | threat model SKILL.md + contrato FIND-104 |

## Steps atómicos

- [x] **S0 — Discovery + task file** (este file; research web 4/4 + Regla 0 + Spec) — PLAN
- [x] **S1 — Docs base** (`README.md` + `TOKEN-BUDGET.md` + `VERSION.md`) — ACT→VERIFY
- [x] **S2 — OpenCode + Claude** (`opencode/vantadb-memory.js` + `claude/settings.example.json`) — ACT→VERIFY
- [x] **S3 — Cursor + Codex** (`cursor/hooks.json` + `codex/hooks.json`) — ACT→VERIFY
- [x] **S4 — Tests + commit** (`tests/test-hooks.ps1` 50/50 verde + JS/JSON OK + secrets 0 hits + OCR advisory) — VERIFY→CLOSE

## Context Save Point

- Hecho: S0 ✅ (research + task file). Next: S1 docs base.
- Reanudar: leer este file + `recall-policy.md` + `assets/hooks/` existente; continuar primer step ⬜.
- Invariantes: no tocar prohibidos §2; staging selectivo; NO PUSH (solo lead); secrets nunca a disco.

## Log

- 2026-09-17 S0: discovery (SDP 7 skills, webfetch 4/4, Regla 0, Spec 8 filas) + task file creado.
- 2026-09-17 S1–S4: 9 files en `assets/hooks/` (README+TOKEN-BUDGET+VERSION, opencode JS, claude/cursor/codex JSON, test PS1 50/50 ✅) + JS --check OK + JSON OK + secrets 0 hits + OCR advisory sin Critical/High + commit selectivo sin push.
