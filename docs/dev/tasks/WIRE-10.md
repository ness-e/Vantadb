---
title: "WIRE-10: Distribución P0 — `install.sh` macOS, Colab a `Client`, hooks sin `pwsh`"
kind: task
description: "Install macOS verde en CI/docs + notebook ejecuta (Client/search) + hooks inyectan contexto real sin pwsh (vanta-cli mcp-call one-shot).\""
---

# WIRE-10: Distribución P0 — `install.sh` macOS, Colab a `Client`, hooks sin `pwsh`

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (F1; ex W2/A2 C5/C9/C12 del plan archivado)
- **Fuente:** Backlog `P56` (fila WIRE-10) + plan `:108,120,156`
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** Mixto (`fix:`, shell + notebook + Rust CLI + skills)
- **Turns estimados:** 15-25
- **Creado:** 2026-09-25 · **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED (2026-09-27 — cierre formal F1a; commit canónico `8e55e853`, hash pre-rebase `aac7001f`; P2-01 fresco APPROVE — ver §Review)
- **SDP (Paso 0b):** source-driven-development + systematic-debugging + test-driven-development + incremental-implementation + api-and-interface-design (task) + security-and-hardening + doubt-driven-development (SDP v2 BUILD; frontend-ui-engineering N/A — sin web/; context-engineering N/A — contexto ya empaquetado)
- **Incógnitas (uphill):** 0 (Spec P/D resuelta 2026-09-25) · **Pendientes (downhill):** 0 steps (5/5 ✅ 2026-09-25; cierre 2026-09-27)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | hooks (claude/codex/cursor/opencode templates) ← `vanta-mcp-local.ps1`; Colab ← docs/Quickstart; install.sh ← README/QUICKSTART |
| Callees | `scripts/install.sh:62-118` (uname/checksum); `examples/colab/vantadb_quickstart.ipynb` (API removida); `src/cli*.rs` (nuevo subcomando); `skills/vantadb-mcp/assets/hooks/` (4 clientes + tests) |
| Implicaciones | `mcp-call` = superficie CLI **nueva** (documentar en ayuda + docs); notebook = JSON editable + ejecutable; install.sh = shell POSIX/macOS (sin GNU-isms); EST-10 ya barrió repo (excluyó ipynb — sin duplicación) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `scripts/install.sh`, `examples/colab/vantadb_quickstart.ipynb`, `skills/vantadb-mcp/assets/hooks/README.md`, templates `claude/settings.example.json` + `codex/hooks.json` (+ cursor/opencode equivalentes), `vanta-mcp-local.ps1`
- **Referencias hacia dentro:** `FIND-106` (templates), `FIND-104` (install), `FIND-114/116` (ejemplos en `Client`), `FIND-119` (mirrors), EST-10 (exclusiones)
- **Referencias entrantes:** README/QUICKSTART → install.sh; docs → Colab; skills → hooks
- **Veredicto impacto:** medio — 3 slices disjuntos; único símbolo público nuevo: `vanta-cli mcp-call`

## Contrato
"Install macOS verde en CI/docs + notebook ejecuta (`Client`/`search`) + hooks inyectan contexto real sin `pwsh` (`vanta-cli mcp-call` one-shot)."

## Spec (SDD — Phase 1b: feature-add detectado)

> `vanta-cli mcp-call` = comando CLI nuevo → Spec obligatoria. Ronda P/D resuelta 2026-09-25 con el owner.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Diseño del bridge sin pwsh | A) `vanta-cli mcp-call` one-shot (nueva superficie CLI, documentada; reutiliza binario existente) / B) hooks POSIX puros por stdio (sin binario, más frágil) / C) bridge Node (runtime extra) | A | ✅ pregunta owner 2026-09-25 → A |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** install.sh sigue funcionando en Linux (no romper GNU path); notebook mantiene narrativa didáctica (migrar llamadas, no reescribir); launcher `vanta-mcp-local.ps1` intacto (los templates nuevos lo sustituyen, no lo modifican); secretos solo vía env de sesión.
- **Comandos de verificación:** `shellcheck scripts/install.sh` (si disponible) + `cargo test -p vantadb --lib` (tras mcp-call) + ejecución notebook + `skills/.../tests/test-hooks.*` equivalente sin pwsh
- **Deuda pendiente:** ninguna al abrir

## Recitation
```
=== RECITATION ===
Objetivo activo: WIRE-10 — distribución P0 (install/Colab/hooks)
Estado: completed (desde: in-progress)
Última acción: Cierre 2026-09-27 — reconciliación (5/5 steps ✅ desde 2026-09-25, commit 8e55e853) + verify scoped (fmt/clippy/nextest 11/11/hooks 65/65/notebook e2e 8/8/harness 8/8) + verify.ps1 ALL 10 PASS + P2-01 fresco APPROVE (vanta-review ses_f1c06efc4ffeLk9FF3gTqyO9Hw) + FIND-169
Resultado: ✅
Próxima acción: ninguna — LEAD: commit local del cierre (task file/plan/Backlog) sin push
Contrato: ver ## Contrato — verificado (harness + notebook e2e + hooks sin pwsh + mcp-call one-shot)
Invariantes: Linux intacto; narrativa notebook intacta; launcher ps1 intacto; secretos por env
Deuda: FIND-169 (Media pre-existente, no bloqueante)
Próxima tarea si completa: DEF-03
last-synced: 2026-09-27
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (elimina 3 fricciones de distribución; `mcp-call` documentado desde el día 1).

## Definition of Done (3 niveles)
- **Task:** contrato + clippy/fmt donde aplique + notebook ejecutado de punta a punta
- **Commit:** atómico(s) por slice si excede ~100 líneas, `fix:` + WIRE-10, verify mecánico antes
- **Release:** N/A (sin versionado; install.sh/Colab/hooks no publican paquetes)

## Herramientas necesarias
- shellcheck/sh, jupyter/nbconvert (o extracción de celdas), cargo check/test/clippy/fmt, `campaign_verify_cmd`
- **Skills cargadas (SDP):** source-driven-development (base) + systematic-debugging (3 fixes con causa raíz: sha256sum ausente, API removida, pwsh+JSON-RPC) + test-driven-development (RED: notebook roto, hook que inyecta nothing) + incremental-implementation (1 slice por vez) + api-and-interface-design (superficie `mcp-call` nueva: ayuda + errores tipados)

## Investigation Notes
- install.sh:62-118: `uname` + darwin OK; checksum en :117-118 con `sha256sum` (inexistente macOS) → `command -v sha256sum || shasum -a 256`.
- Colab (`vantadb_quickstart.ipynb:50,106,126,164,197,202`): `vanta.VantaDB(...)`, `get_memory`, `search_memory`, `list_memory` (removidas 0.5.0) → migrar a `Client`/`search` (patrón FIND-114/116).
- Hooks: templates invocan `pwsh -NoProfile -File vanta-mcp-local.ps1` (claude:11,23,36,48; codex:10,23,36; +cursor/opencode) → sustituir por `vanta-cli mcp-call` one-shot (0 hits en src: no existe, se crea).
- EST-10 (b461c9e8) barrió ~75 archivos pero excluyó ipynb → Colab pendiente real, sin duplicación.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes | 0 steps |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE Bug)
- **Repro:** (a) `scripts/install.sh` en macOS sin coreutils → `sha256sum: command not found`; (b) notebook Colab → `AttributeError` en `VantaDB(`/`search_memory`; (c) hook sin pwsh → inyecta nothing en silencio.
- **Hipótesis:** (a) GNU-ismo; (b) rename 0.5.0 no alcanzó ipynb; (c) dependencia pwsh + protocolo JSON-RPC equivocado.
- **1 variable:** un slice por step, en orden A→B→C.
- **Test RED:** (a) shellcheck/simulate; (b) ejecución notebook que falla; (c) hook que inyecta vacío.

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — install.sh descarga+ejecuta (checksum fail-closed, sin `|| true`; validar URL https); notebook sin secretos; hooks con secretos solo por env. Sin auth nueva.
- [x] **PERFORMANCE** — N/A: instalador/notebook/hooks fuera de hot path.

## Steps
### Step 1: install.sh macOS
- **Archivos:** `scripts/install.sh:105-125`
- **Acción:** `command -v sha256sum || shasum -a 256`; checksum fail-closed; verificar en CI/docs (macOS runner o doc de verificación)
- **Verify:** shellcheck (si disponible) + dry-run path + job CI macOS verde o evidencia doc
- **Estado:** ✅ DONE (2026-09-25 — harness `/tmp/wire10/step1-red.sh`: FORMAT old="Checksum mismatch"/new="verified"; ABSENT old="sha256sum: command not found"/new="verified vía shasum"; fail-closed sin ambas=exit 1; `sh -n`+`--dry-run` OK; shellcheck N/A — no instalado local)

### Step 2: Colab a `Client`
- **Archivos:** `examples/colab/vantadb_quickstart.ipynb`
- **Acción:** migrar `VantaDB(`→`Client`, `search_memory`/`get_memory`→`search`/gets canónicos (patrón FIND-114/116); ejecutar notebook punta a punta
- **Verify:** `jupyter nbconvert --execute` (o equivalente) exit 0
- **Estado:** ✅ DONE (2026-09-25 — RED: `AttributeError: no attribute 'VantaDB'`; GREEN: 7+2 reemplazos a `from vantadb import Client` + `db.search` plano + `db.memory.get/list/delete`; ejecutado e2e vía `run_nb.py` (8 celdas, exit 0, output completo put/get/search/list/update/delete/flush/close); jupyter N/A — no instalado, extractor stdlib equivalente)

### Step 3: `vanta-cli mcp-call` one-shot
- **Archivos:** `src/cli*.rs` (nuevo subcomando), ayuda CLI
- **Acción:** implementar comando one-shot (lee request, llama MCP, imprime response, exit codes); documentar en `--help`
- **Verify:** `cargo test -p vantadb --lib` + invocación manual e2e
- **Estado:** ✅ DONE (2026-09-25 — RED: `unrecognized subcommand 'mcp-call'`; GREEN: `Commands::McpCall` + `cli_handlers/mcp_call.rs` (spawn `vantadb-server --mcp` stdio pipeado, JSON-RPC por líneas, exit 0/1/2) + dispatch en bin; 7/7 unit tests (atraparon 2 bugs: código numérico + prefijo id); e2e real: memory_put→OK, memory_recall→prepend_context verificado, unknown-tool→2, bad-args→2, sin huérfanos; clippy+fmt verdes)

### Step 4: Hooks sin pwsh
- **Archivos:** `skills/vantadb-mcp/assets/hooks/{claude,codex,cursor,opencode}/`, tests
- **Acción:** templates usan `vanta-cli mcp-call`; test que prueba inyección real de contexto (no vacío)
- **Verify:** hooks inyectan contexto real sin pwsh en PATH
- **Estado:** ✅ DONE (2026-09-25 — RED: comandos con `pwsh -NoProfile -File vanta-mcp-local.ps1`; GREEN: templates 1.1.0 con `mcp-call memory_recall` (SessionStart estático, per-message `{{prompt}}` verbatim) + reminders estáticos PreCompact/Stop + plugin opencode por argv con parse `structuredContent→prepend_context`; requiere placeholder `{{}}` en mcp-call (10/10 tests); test-hooks.ps1 65/65 con grupo (d) WIRE-10; `node --check` OK; prueba real bajo sh sin pwsh: UserPromptSubmit inyecta `<relevant-memories>` verificado, SessionStart exit 0 envelope válido; `vanta-mcp-local.ps1` INTACTO)

### Step 5: Cierre
- **Archivos:** —
- **Acción:** fmt+clippy+tests scope, `verify_changed.ps1`, commit(s) `fix:`, progreso
- **Verify:** `campaign_verify_cmd` con el contrato
- **Estado:** ✅ DONE (2026-09-25 — `verify_changed.ps1` ALL 4 PASS (docs-coverage exigió fila `mcp-call` en CONFIGURATION.md §4); lib 10/10 mcp_call + cli_tests 85/85 + hooks 65/65 + notebook e2e; commit canónico `8e55e853` (hash pre-rebase `aac7001f`) local, sin push — Regla 7)

## Dependencias
- Ninguna (serializar con WIRE-01/WIRE-09 si tocan proxy/server el mismo día — no esperado). Siguiente: WIRE-11.

## Review (GATE P2-01)
- **Revisor:** vanta-review — distinto del implementador
- **Enfoque:** ¿instalador macOS real? ¿notebook ejecuta? ¿hooks inyectan de verdad?
- **Cómo se probó:** ejecución real en cada slice, no auto-reporte
- **Checklist anti-hábitos:** según plantilla
- **Veredicto:** changes-required → aplicado (Medium: `truncate` UTF-8 panic → `floor_char_boundary` + test regresión; Low: reap con `child.wait()` tras kill/disconnect, taxonomía exit-2 documentada en cli.rs, nota `{{prompt}}` por cliente en README; trade-off aceptado: save-state hooks = reminder estático, sin `trim_matches` churn). Re-verify: lib 11/11, hooks 65/65, smoke recall OK sin huérfanos.
- **Cierre (2026-09-27) — ronda fresca para el ACCEPT mecanizado (HARD-07):** revisor `vanta-review` (contexto distinto; sesión `ses_f1c06efc4ffeLk9FF3gTqyO9Hw`; autor de cierre sesión `ses_f1c16a800ffewd03Ct0GuglWpT`). **Veredicto: ✅ APPROVE** (evidencia re-ejecutada por el revisor, no auto-reporte):
  - install.sh: harness `step1-verify.ps1` → `pass=8 fail=0` (GNU, fallback shasum macOS, fail-closed sin ambas, mismatch, dry-run, `sh -n`).
  - Notebook: `run-nb.ps1` → `NOTEBOOK OK executed=8 skipped=1`; 0 APIs stale.
  - Hooks: `test-hooks.ps1` → `PASS=65 FAIL=0`; recall sin `pwsh`; `vanta-mcp-local.ps1` intacto (`git diff` vacío); smoke live `mcp-call memory_recall/memory_put` OK sin huérfanos nuevos.
  - `mcp-call`: nextest → `11 tests run: 11 passed`; fixes de la ronda previa verificados en código (`floor_char_boundary` L200, `kill()+wait()` L333-368, taxonomía exit-2).
  - Gate fast-tier completo: `verify.ps1` **ALL 10 PASS** (vía `CARGO_TARGET_DIR=target/session-api01` por lock de binario en `target/debug` — 8 sesiones MCP live del owner; no es fallo de código) + `validate-docs-coverage` exit 0.
- **Hallazgos de la ronda de cierre:** Medium pre-existente (no bloqueante) → `FIND-169` (save-state hook opencode: `thread_send` sin `thread_id` = no-op determinista; no es regresión de WIRE-10). Low informativo: sin job CI que ejecute install.sh (evidencia = harness mock + docs; consistente con la Stop condition del plan "smoke documentado + CI matrix").

## Notas
- WIP ajeno PROHIBIDO: `src/sdk/types/`, `QueryResult` (API-01 en curso) — `mcp-call` no debe tocar tipos del SDK.
- `vanta-mcp-local.ps1` intacto (sustitución en templates, no modificación).
- **Cierre 2026-09-27 (F1a):** reconciliación — los 5 steps ya estaban ✅ DONE desde 2026-09-25 (commits `8e55e853` + progreso `6f1ca39d`/`c3ccaed0`); el "⏳ IN PROGRESS / 5 steps pendientes" del plan era metadata stale (los Steps del task file mandan). Cierre = verify scoped completo + P2-01 fresco (HARD-07) + sync task file/plan. Commit del cierre → LEAD (sin push).
