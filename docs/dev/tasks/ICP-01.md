---
title: "ICP-01: Track AI-IDEs (MCP) — one-pager + repo-map/watcher + viewer + hooks + demo CI"
kind: task
description: "One-pager del track AI-IDEs + demo CI MCP (sesión 1 guarda → sesión 2 recupera por sinónimo no-verbatim) + métrica North Star search-side (WriteBack::track → proxy-memory-events) + entrada COMPARISON.md + disposición repo-map/watcher (FIND — MGR-22 sigue ⬜)"
---

# ICP-01: Track AI-IDEs (MCP) — one-pager + repo-map/watcher + viewer + hooks + demo CI

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 42, Fase F5 — wave F5.1, co-batch ICP-02 ‖ VER-08)
- **Fuente:** Backlog `ICP-01` (P54, L966) · master Task 42 · `docs/dev/strategy/GO_TO_MARKET.md` §Vertical 3 (:180-195)
- **Esfuerzo:** 🟡 1sem (re-baseline: hooks/launcher/viewer ya existen → slice real = one-pager + demo CI + métrica + COMPARISON + disposición repo-map)
- **Prioridad:** 🟠
- **Tipo:** CI/CD + Docs + instrumentación mínima (sin símbolos públicos nuevos)
- **Turns estimados:** 8
- **Creado:** 2026-09-29T00:00
- **last-synced:** 2026-09-29
- **Estado:** ⏳ IN PROGRESS (implementación 6/6 ✅ + ronda 1 de review ❌ CHANGES REQUIRED → **batch de fixes aplicado y re-verificado**; **LEAD: re-review ronda 2 + commit local** — HARD-07 bloquea ACCEPT sin reviewer fresco)
- **Incógnitas (uphill):** 0 — MGR-22 resuelta por stop condition declarada → demo SIN repo-map + FIND (no implementar)
- **Pendientes (downhill):** 0 de código — 6/6 steps ✅ + batch ronda 1 cerrado; pendiente solo el cierre de proceso (re-review ronda 2 + commit, LEAD)
- **Iteraciones:** 2 (ejecución completa en wave F5.1 + batch de fixes de la ronda 1 de review; 0 retries de verify)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-proxy` search path (`memory_tools::search` ← `execute` ← loop de tools del proxy) · CI (`ci-ai-ides-demo.yml` nuevo) · ICP-03/ICP-02 (COMPARISON.md coopera — capa memory-as-a-service) · F6 anuncio (vertical 3) · DEF-05 (métrica North Star: FIND search-side se cierra acá) |
| Callees | `WriteBack::track` (`vanta-proxy/src/writeback.rs:54`) · `capture.rs` (patrón `{ms}-{seq}` + job L0 `:74-179`) · `vantadb-mcp` (`handle_tools_call`, `memory_put`/`search_memory` EMB-14/15) · `vantadb::llm` (OllamaProvider `/api/embed`) · `vanta-memory` (`perform_auto_recall_governed`) · `vanta-cli mcp-call` (`src/cli_handlers/mcp_call.rs:234`) |
| Implicaciones | (1) El evento de métrica es **metadata-only** (`{session, kind, hits}` — nunca query ni contenido) y fire-and-forget: no toca el wire ni añade latencia (precedente D47 `memory_tools.rs:108`). (2) El demo usa un **doble de embedding determinista declarado** (fake Ollama in-process): prueba el pipeline (embed-on-put → vector → embed-on-query → similitud), no la calidad del modelo. (3) `desktop/**` NO se toca (viewer ✅ existente, solo documentado) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-proxy/src/memory_tools.rs` (535L) · `vanta-proxy/src/capture.rs` (329L) · `vanta-proxy/src/writeback.rs` (tramo `track`/`enqueue`/`flush` + tests) · `docs/dev/tasks/DEF-05.md` (191L) · `docs/user/COMPARISON.md` (153L) · `docs/user/operations/EDITOR_INTEGRATIONS.md` (tramo 1-130 + headings) · `skills/vantadb-mcp/assets/hooks/README.md` (53L) · `.github/workflows/ci-examples.yml` (146L) · `.github/actions/rust-setup/action.yml` (100L) · `docs/dev/workflow/RULES.md` (218L) · `vantadb-mcp/tests/test_query_embed.rs` (271L) · `src/cli_handlers/mcp_call.rs` (tramo 234-441) · `src/cli.rs` (tramo Commands/McpCall/global `--db`) · `src/llm.rs` (tramos factory `:57-111`, dummy `:205-226,468-505`, OllamaProvider `:670-759`) · `src/config.rs` (tramo `from_env` `:1000-1059`) · `SPEC.md` §North Star (`:106-135`) · `docs/api/MCP.md` (tramos 36-96, 228-272) · `docs/api/PROXY.md` (tramos 55-104) · `vanta-memory/src/core/hooks/auto_recall.rs` (tramos `RecallResult` `:156-208` + firma `perform_auto_recall_governed` `:322`) · `vanta-proxy/tests/tool_loop.rs` (tramo 340-519) · `vantadb-mcp/src/handlers/tools.rs` (tramos 1360-1370 put, 3480-3540 search, 3964-4210 EMB-13/14/15)
- **Archivos referenciados hacia dentro (imports/dependencias):** `memory_tools.rs` importa `crate::capture` · `crate::writeback::{WriteBack, L0Job}` · `vantadb::sdk::{Embedded, MemoryInput, ...}` · `vantadb-mcp/tests/*` importan `vantadb_mcp::*` + `vantadb::storage` · workflow nuevo dependerá de `.github/actions/rust-setup` (regex SHA pins) · `scripts/north_star_metric.py` dependerá de `vanta-cli mcp-call` (contrato de salida: `result` JSON con `content[0].text`)
- **Archivos que referencian a los editados (referencias entrantes):** `vanta-proxy/src/server.rs` (llama `memory_tools::execute` en el loop) · `vanta-proxy/src/capture.rs` (namespace `proxy-turns` citado por SPEC.md:119) · `docs/api/PROXY.md` ← `docs/user/operations/MCP_REGISTRY.md:125` · `EDITOR_INTEGRATIONS.md` ← `MCP_REGISTRY.md`, `docs/user/index.md`, `docs/index.md`, `master-index.md`, `desktop/README.md` · `COMPARISON.md` ← README, `BENCHMARKS.md` (single canonical source) · `SPEC.md` ← VISION/plan F5 · `docs/user/AI_IDES.md` (nuevo) será referenciado por COMPARISON + index generado
- **Veredicto impacto:** **medio-bajo y acotado** — (a) 1 archivo de código editado (`memory_tools.rs`, ~40L: const + seq + job + track en `search` + 2 tests); (b) resto = docs + tests + workflow + script. Sin símbolos públicos nuevos (todo `pub(crate)`/privado/test). Sin cambios de wire, sin deps nuevas, sin tocar regiones de co-batch (`src/sdk/importers/**`, `vanta-proxy/src/{governance,envelope,redact}.rs`, `src/attestation.rs`)

## Contrato

> **Verbatim del plan (Task 42):** "one-pager del track (vertical 3: problema → valor → instalación → límites honestos) publicado Y demo CI MCP verde: sesión 1 guarda (auto-captura/put) → sesión 2 recupera por sinónimo no-verbatim (test/workflow que falla si el recall no llega) Y métrica North Star instrumentada y consultable (sesiones put+search en ventana 7d con comando documentado; search-side vía `WriteBack::track`) Y entrada del track en `COMPARISON.md` Y repo-map/watcher: consumido desde MGR-22 si está; si no, disposición explícita (FIND) sin bloquear demo/métrica"

Desglose verificable:
1. **One-pager** publicado en `docs/user/AI_IDES.md` (vertical 3: problema → valor → instalación → límites honestos) + link desde `COMPARISON.md`.
2. **Demo CI** verde: test `vantadb-mcp/tests/demo_ai_ides.rs` (sesión 1 `memory_put` con auto-embed → sesión 2 `search_memory` con query sinónima no-verbatim, 0 tokens literales en común) + workflow `.github/workflows/ci-ai-ides-demo.yml` PR-blocking que falla si el recall no llega.
3. **Métrica North Star** instrumentada search-side: evento `{session, kind:"search", hits}` en `proxy-memory-events` vía `WriteBack::track` (`memory_tools::search`) + comando documentado (`scripts/north_star_metric.py` + `docs/api/PROXY.md` §North Star).
4. **Entrada del track en `COMPARISON.md`** (capa AI-IDEs/MCP).
5. **repo-map/watcher:** disposición explícita FIND (MGR-22 sigue ⬜ — no se implementa) sin bloquear demo/métrica.
6. **Cursor Claude Code** en `EDITOR_INTEGRATIONS.md` (gap declarado del track; DoD Notas Task 42).

## Spec (tabla de decisiones — gate spec-first)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Forma del evento de métrica search-side | A) `WriteBack::track` fire-and-forget → namespace `proxy-memory-events`, key `{ms}-{seq}`, payload `{session, kind:"search", hits}` (sin tocar el wire) / B) extender `TurnReport`/`/snapshot` (ring 100, vista viva → no persiste) / C) span OTLP (opt-in, default off) | A | ✅ decidido-por-evidencia: especificado en DEF-05 (`docs/dev/tasks/DEF-05.md:98`, `SPEC.md:125`) + precedente `memory_tools.rs:108`; B/C no persisten (`report.rs:18-41`, `langfuse.rs:97-108`) |
| 2 | ¿Dónde vive el comando de la métrica? | A) script stdlib `scripts/north_star_metric.py` que consulta vía `vanta-cli mcp-call` + sección en `docs/api/PROXY.md` / B) solo query manual documentada (jq/lista) / C) endpoint nuevo `/metrics` (código servidor) | A | ✅ decidido-por-evidencia: DEF-05 exige "comando/consulta documentada" (`SPEC.md:121`); C = scope creep; A reusa launcher WIRE-10 (`mcp_call.rs:234`) |
| 3 | Estrategia del demo semántico en CI (sin tokens, sin modelo) | A) fake Ollama in-process (`/api/embed`) determinista + `--features remote-inference` → ejercita el path real EMB-14/15 (embed-on-put + embed-on-query) / B) vectores provistos por el llamante (no ejercita embed; "sinónimo" queda fabricado por el test) / C) modelo real pineado en CI (download ~691MB, `llm.rs:72`) / D) estilo EMB-15 (skip sin modelo → CI no prueba) | A | ✅ decidido-por-evidencia: C descartado por costo (`llm.rs:72` "keeps CI green without 691MB download"); D no cumple "falla si el recall no llega"; A usa el contrato HTTP documentado de `OllamaProvider` (`llm.rs:729-758`, request `{model,input}` → `{embeddings}`) y `auto_embed_one` (`tools.rs:1364`) + `try_embed_query` (`tools.rs:3522`) |
| 4 | ¿Repo-map/watcher? | A) implementar (MGR-22 ⬜, sin research/spec → rabbit hole) / B) demo SIN repo-map + FIND (stop condition del plan) | B | ✅ decidido-por-evidencia: stop condition Task 42 (`master:1115`) + dep declarada MGR-22 ⬜ (`Backlog:890`); `rg 'repo_map\|repo-map'` = 0 hits en código |
| 5 | Entrada COMPARISON.md | A) sección corta "agent memory for AI-IDEs (MCP)" al final (qualitative + links, sin números — Regla 11) / B) fila en tabla §1 (competidores ≠ vector DBs) | A | ✅ decidido-por-evidencia: §1 compara vector DBs (`COMPARISON.md:21-30`); la capa memory-as-a-service está declarada pendiente (`:17`) — ICP-01 aporta su entrada de track |
| 6 | Viewer (desktop) | A) documentar `MemoryLens` como viewer + limitación (requiere build desktop) + FIND si el demo lo exigiera / B) editar `desktop/**` (PROHIBIDO en esta wave) | A | ✅ decidido-por-evidencia: `desktop/**` prohibido (wave F5.1) + viewer ya existe (`desktop/src/components/memory/MemoryLens.tsx`); stop condition "viewer exige release → FIND" (`master:1115`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) el evento de métrica es **metadata-only** — jamás query/texto recordado (patrón governance VER-04); (2) fire-and-forget: el wire nunca espera el write del evento (D47); (3) el demo no usa números/tokens/red externa — determinista y offline; el doble de embedding va **declarado** (nunca presentar su semántica como calidad de modelo real); (4) `proxy-turns` no cambia (DEF-05 lo consumió así); (5) sin dependencias nuevas (axum/tokio/reqwest ya presentes en ambos crates); (6) no tocar regiones de co-batch F5.1.
- **Comandos de verificación:** `cargo nextest run -p vantadb-mcp --features remote-inference --test demo_ai_ides --build-jobs 2` · `cargo nextest run -p vanta-proxy --build-jobs 2` (o `-p vanta-proxy --lib` para el test unit) · `cargo clippy -p vanta-proxy -p vantadb-mcp --all-targets -- -D warnings` · `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write` · `pwsh scripts/validate-docs-coverage.ps1` · `actionlint .github/workflows/ci-ai-ides-demo.yml` (si disponible).
- **Deuda pendiente:** hooks `test-hooks.ps1` bajo pwsh + smoke contra clientes reales (FIND-106 residual, no bloquea) · viewer desktop sin release (FIND) · repo-map/watcher (FIND — MGR-22) · snippets con `~/.vantadb` en EDITOR_INTEGRATIONS (MCP.md:37 lo prohíbe — hallazgo pre-existente, FIND) · `server.json` 0.6.1 stale (FIND candidato del plan).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# ICP-01: Track AI-IDEs (MCP)` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ (S2…S6) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | ICP-02 / ICP-03 (wave F5.1; según deps del grafo) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≈0 → positivo. Introduce instrumentación mínima reutilizando el path existente (`WriteBack::track`, sin deps nuevas); no introduce clonados ni unsafe; el demo añade cobertura CI donde no había (vantadb-mcp no corre en fast gate). Paga: cierra el FIND de DEF-05 (search-side) y da cobertura CI al path de embeddings MCP (EMB-14/15 tenían solo tests skip-capaces).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 5/5 verificable: one-pager + demo CI verde + métrica instrumentada/consultable + COMPARISON + FIND repo-map + Claude Code en EDITOR_INTEGRATIONS · task file sync · gates docs verdes |
| **Commit** | 1 commit atómico `feat:`/`docs:` según diff final (lo ejecuta el LEAD — esta wave NO commitea) · verificación mecánica registrada |
| **Release** | N/A justificado: docs + test CI + instrumentación interna; sin superficie pública nueva (no participa del contrato de release) |

## Herramientas necesarias
- `cargo nextest` (scoped, `--build-jobs 2`; NUNCA workspace sin deliberación) · `cargo clippy -p …` · `cargo fmt --check`
- `node scripts/docs/{check-links,check-docs,gen-index}.mjs` · `pwsh scripts/validate-docs-coverage.ps1`
- `actionlint` (lint del workflow nuevo) · `npx markdownlint-cli2` (docs tocados)
- CodeGraph/CBM para blast radius (Paso 0 ya ejecutado)

**Skills cargadas (SDP v3, 2026-09-29):** `campaign-executor` · `progreso` · `documentation-skill` · `source-driven-development` · `doubt-driven-development` · `ci-cd-and-automation` (pinned policy) · `git-workflow-and-versioning` (pinned policy) · `incremental-implementation` · `test-driven-development` · `context-engineering`
- `documentation-skill` — docs GitHub-first (frontmatter, links, kinds) para one-pager/COMPARISON/PROXY.
- `source-driven-development` — verificar todo contra código real (flags CLI, shapes HTTP, nombres de tools).
- `doubt-driven-development` — gate de duda en las decisiones no triviales (evento metadata-only, fake embedder declarado, workflow PR-blocking).
- `ci-cd-and-automation` + `git-workflow-and-versioning` (pinned) — workflow nuevo bajo `RULES.md` (pins SHA, timeout, permissions, paths).
- `incremental-implementation` / `test-driven-development` — slices verticales; tests primero donde aplique.

## Investigation Notes

- **Paso 0 (re-baseline verificado 2026-09-29):**
  - **hooks ✅** 4 clientes (`skills/vantadb-mcp/assets/hooks/{claude,codex,cursor,opencode}` + `tests/test-hooks.ps1` + `VERSION.md`) y launcher `vanta-cli mcp-call` sin pwsh (README:31-39; WIRE-10 ✅ `8e55e853`). Residual: test bajo pwsh + sin smoke contra clientes reales (FIND-106 residual — no bloquea).
  - **viewer ✅ parcial** `desktop/src/components/memory/MemoryLens.tsx` + lenses graph/search/consolidate + `e2e/visual`; `desktop/**` PROHIBIDO en la wave → solo documentar.
  - **repo-map/watcher = 0 hits en código** (`rg 'repo_map|repo-map'` fuera de docs = 0; `vantadb-mcp/src/code.rs` tiene las 8 tools `code_*` como insumo) → MGR-22 ⬜ (Backlog:890; sin task file) → **stop condition activa**.
  - **métrica PUT ✅** (`server.rs:256-258` → `capture_turn` → `capture.rs:53-139`, namespace `proxy-turns`, key `{ms}-{seq}`); **search ✗** — `rg proxy-memory-events` = 0; precedente `WriteBack::track` en `memory_tools.rs:108` (capture tool).
- **Gap del track:** `EDITOR_INTEGRATIONS.md` sin Claude Code (6 editores: Cursor/VS Code/OpenCode/OpenClaw/Devin/Antigravity). Nota: `docs/api/MCP.md:65-80` SÍ tiene la config Claude Code canónica → reusar verbatim (fuente verificada) y añadir sección en el runbook.
- **Demo — restricciones reales:** CI sin modelo de embeddings (`embeddings/models/**` gitignored `.gitignore:239`; sin setup en workflows) y sin tokens; `Embedded` dummy (`LocalOnnxProvider::new_dummy`) es hash sin señal semántica; EMB-15 skipea sin modelo (`test_query_embed.rs:95-99`). `vanta-proxy` search pasa hook `None` → keyword (`memory_tools.rs:152`). Decisión S3: fake Ollama in-process + `remote-inference` (Spec #3).
- **MCP path verificado:** `memory_put` → `auto_embed_one` (`tools.rs:1364`) rellena vector si proveedor responde; `search_memory` text-only → `try_embed_query` (`tools.rs:3522`) embebe query; `OllamaProvider` request `{model,input}` → response `{embeddings:[[f32]]}` (`llm.rs:674-683,729-758`); env `VANTADB_EMBEDDING_PROVIDER=ollama` + `VANTADB_LLM_URL` (`config.rs:1019-1058`); `agent` profile (default, 37) incluye memory CRUD + search (MCP.md:237).
- **Métrica — comando:** `vanta-cli mcp-call --db <DB> --tool memory_list --args '{"namespace":"proxy-turns","limit":1000}'` imprime el `result` MCP (JSON con `content[0].text` — `mcp_call.rs:415-421`); `--db` es global (`cli.rs:20-22`). El script S2 hará las 2 consultas + intersección 7d.
- **Workflow pattern:** `ci-examples.yml` (PR-blocking, sin `continue-on-error`, matrix OS); `RULES.md` (timeout por job, pins SHA, `permissions: contents: read`, concurrency); `rust-setup` tiene `install-nextest` (action.yml:24-27).
- **Naming/co-batch:** NO tocar `src/sdk/importers/**` (ICP-03), `vanta-proxy/src/{governance,envelope,redact}.rs` + `src/attestation.rs` (ICP-02 — solo LEER). `COMPARISON.md` cooperado (entrada al final; ICP-02/03 añadirán las suyas).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — MGR-22 resuelta por stop condition (demo SIN repo-map + FIND) |
| Pendientes de ejecución (downhill) | 0 — 6/6 steps ✅ + batch ronda 1 ✅ (re-review + commit = LEAD) |
| % completado | 100% implementación/verificación (⬜ pendiente de proceso: re-review ronda 2 + commit local) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: sin input de usuario nuevo (evento interno metadata-only: sesión + contador; sin payload). Red: el test abre un listener **loopback efímero** in-process (no expone red externa; sin credenciales; server solo dentro del test; muere con el proceso). Dependencias nuevas: ninguna. Superficie pública nueva: ninguna. Resultado: checklist `security-and-hardening` aplicable satisfecho (validation en boundaries ya existente; no logging de secretos; sin shell).
- [x] **PERFORMANCE** — evaluado: `memory_tools::search` suma 1 `WriteBack::track` (spawn tokio + 1 put L0 async, fire-and-forget) — no bloquea el wire (D47). El fake embedder del test: 1 request HTTP loopback por texto. Sin cambio de algoritmo ni hot path de engine. Gate: sin regresión esperada; no bench (Regla 9 no dispara; no hay claim de performance).

## Steps

### Step 1: Métrica search-side — instrumentación en `memory_tools::search` + tests
- **Archivos:** `vanta-proxy/src/memory_tools.rs` (edición ~45L + tests)
- **Acción:** const `EVENTS_NAMESPACE = "proxy-memory-events"` + `static EVENT_SEQ` + `search_event_job(memory, session, hits) -> L0Job`; `execute` pasa `writeback` a `search`; `search` cuenta `recalled_memories.len()` y encola el evento para paths `Ok` (fire-and-forget, metadata-only). Tests: nuevo `search_tracks_metric_event_with_hits` (seed turno L1 → search con hit → evento `hits≥1`) y conversión del test existente a `#[tokio::test]` + assert evento `hits==0` (DB vacía).
- **Verify:** `cargo nextest run -p vanta-proxy --lib --build-jobs 2` → **184/184 PASS** ✅ · `cargo clippy -p vanta-proxy --all-targets -- -D warnings` → exit 0 ✅ · suite completa `cargo nextest run -p vanta-proxy --build-jobs 2` → **316/316 PASS** (2 skipped pre-existentes) ✅.
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 2: Comando de la métrica — `scripts/north_star_metric.py` + docs (`PROXY.md`, `SPEC.md`)
- **Archivos:** `scripts/north_star_metric.py` (nuevo, stdlib) · `docs/api/PROXY.md` (§North Star) · `SPEC.md` (§Medición — SEARCH deja de estar pendiente; pointer a comando + ICP-01)
- **Acción:** script que consulta `proxy-turns` + `proxy-memory-events` vía `vanta-cli mcp-call`, filtra keys `{ms}-{seq}` por ventana (default 7d) e imprime sesiones con put ∧ search(hits≥1) + conteo; doc del comando + semántica del evento.
- **Verify:** `--self-test` → OK ✅ · **E2E real** con `vanta-cli mcp-call` (repo `target/debug` 0.7.0) contra store temporal: 1 turno + 1 evento hits=2 + 1 evento hits=0 → `north_star_sessions: 1` (`sess-demo`; `sess-other` excluida por `hits=0`) ✅ (human + `--json`).
- **Re-verify (ronda 1 review):** fix `truncated` shrink-and-retry — repro 61 records **antes 22/61 → después 61/61** (`north_star` 1→2) ✅ + caso truncado en `--self-test` ✅ + `py_compile` ✅.
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 3: Demo test — `vantadb-mcp/tests/demo_ai_ides.rs` (cross-session, sinónimo no-verbatim)
- **Archivos:** `vantadb-mcp/tests/demo_ai_ides.rs` (nuevo)
- **Acción:** fake Ollama in-process (axum, `/api/embed`, embedder determinista declarado con mapa de sinónimos) → env provider; sesión 1 `memory_put` (auto-embed EMB-14) → assert vector persistido; distractor; sesión 2 `search_memory` con query sin 0 tokens literales en común → assert hit top-1 + control inverso + assert mecánico "0 tokens compartidos".
- **Verify:** `cargo nextest run -p vantadb-mcp --features remote-inference --test demo_ai_ides --build-jobs 2` → **1/1 PASS** ✅ · **control RED** (sin `remote-inference`): **FAIL esperado** (`vector: null` — el demo no puede pasar por keyword) ✅ · `cargo clippy -p vantadb-mcp --features remote-inference --all-targets -- -D warnings` → exit 0 ✅.
- **Re-verify (ronda 1 review, C1):** gate `[[test]] required-features = ["remote-inference"]` en `vantadb-mcp/Cargo.toml` → sin features el binario se **saltea** (no falla): `nextest list` ausente ✅ + suite audit sin features **146/146** ✅; con features demo **1/1** ✅ (propiedad PR-blocking conservada).
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 4: Demo CI — `.github/workflows/ci-ai-ides-demo.yml`
- **Archivos:** `.github/workflows/ci-ai-ides-demo.yml` (nuevo)
- **Acción:** workflow PR-blocking (push [main, develop] + PR [main], paths acotados), ubuntu-latest, `rust-setup` + nextest, step del demo test (sin `continue-on-error`), timeout, concurrency, permissions `contents: read`, pins SHA.
- **Verify:** `actionlint .github/workflows/ci-ai-ides-demo.yml` → exit 0 ✅ · reglas `RULES.md` chequeadas (timeout/pins/permissions/concurrency/sin silencios) ✅ · comandos del workflow = los verdes locales de S1/S2/S3 ✅ (workflow real corre en el primer PR/push — no ejecutable en local).
- **Re-verify (ronda 1 review, O1/O2):** paths ampliados a `src/**` (core: storage/executor/sdk) + `vanta-proxy/**` (shape del evento de métrica) → `actionlint` exit 0 ✅.
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 5: One-pager + EDITOR_INTEGRATIONS + COMPARISON
- **Archivos:** `docs/user/AI_IDES.md` (nuevo) · `docs/user/operations/EDITOR_INTEGRATIONS.md` (sección Claude Code) · `docs/user/COMPARISON.md` (§7 track note)
- **Acción:** one-pager (problema → valor → instalación → límites honestos: hooks/launcher/viewer ✅; repo-map ⏳ MGR-22/FIND; viewer requiere desktop; demo/metric comandos; nota de versión `mcp-call` → tren 0.8.0). Claude Code: config canónica de `MCP.md:65-80` + hooks `.claude/settings.example.json` + nota path absoluto. COMPARISON: sección cualitativa con links (sin números — Regla 11).
- **Verify:** `check-links` → 45 rotos (budget 58; 0 nuevos de esta task) ✅ · `check-docs` → GATING all clear ✅ · `gen-index --write` + `--check` → exit 0 ✅ · `markdownlint` → 0 issues en los 5 docs ✅.
- **Estado:** ✅ COMPLETED (2026-09-29)

### Step 6: Gates + cierre
- **Archivos:** `docs/dev/tasks/ICP-01.md` (estado/recitation) + registro FINDs propuestos
- **Acción:** correr gates completos (fmt/clippy/tests scoped/docs/markdownlint/actionlint), `validate-docs-coverage`, OCR advisory, registrar evidencia por cláusula y FINDs (repo-map → MGR-22; viewer-release; server.json; `~` en EDITOR_INTEGRATIONS pre-existente). NO commit (LEAD). NO self-review (LEAD).
- **Verify:** `cargo fmt --check` → exit 0 ✅ · `validate-docs-coverage.ps1` → **0 gaps** ✅ · suite proxy 316/316 + demo 1/1 + self-test/E2E script ✅ · OCR advisory (rule groups por archivo revisados; 2 fixes preventivos aplicados: comentario de ceiling async en el job + `IndexError` en el script) ✅.
- **Estado:** ✅ COMPLETED (2026-09-29) — proceso pendiente: review fresco P2-01 + commit local = LEAD

## Dependencias
- **MGR-22 ⬜** (research repo-map/watcher — P49, cross-track, sin task file): dep declarada; activa la stop condition → demo SIN repo-map + FIND. NO implementar.
- **FIND-106 ✅** (hooks 4 clientes) · **WIRE-10 ✅** (`8e55e853`, launcher sin pwsh) · **DEF-05 ✅** (`58c9903c`, North Star + FIND search-side que esta task cierra) · **WIRE-01 ✅** (`679c75a9`, captura PUT).
- Downstream: ICP-02/ICP-03 (misma wave; COMPARISON coopera) · F6 anuncio (vertical 3) · DEF-07 (criterio).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED. **Esta wave: el LEAD coordina review fresco (no self-review).**

- **Revisor:** `vanta-review` (leaf) — pendiente de asignación por el LEAD.
- **Enfoque:** ¿el demo PRUEBA recuperación no-verbatim (no keyword disfrazado)? ¿el doble de embedding está declarado y no se presenta como modelo real? ¿el evento de métrica es metadata-only y fire-and-forget? ¿workflow PR-blocking sin silencios? ¿one-pager honesto (repo-map pendiente declarado)? ¿COMPARISON sin números ni claims (Regla 11)?
- **Input del implementador (advisory, no sustituye review):** OCR rule groups revisados (ver §Notas) + control RED del demo (sin `remote-inference` falla) + E2E real del script de métrica.
- **Ronda 1 (review fresco, 2026-09-30):** ❌ **CHANGES REQUIRED** (1 Critical + 2 Required + 2 Optional + 1 Nit). Batch de fixes aplicado (detalle y evidencia en §Re-verificación del batch, §Notas):
  - **C1 (Critical):** `demo_ai_ides` rompía las lanes que corren `vantadb-mcp` sin `remote-inference` → gate `[[test]] required-features` en `vantadb-mcp/Cargo.toml`. Re-verify: `cargo nextest run -p vantadb-mcp --profile audit` (sin features) → **146/146**; demo con `--features remote-inference` → **1/1** (la propiedad PR-blocking se conserva).
  - **R1 (Required):** `north_star_metric.py` ignoraba `truncated` → subconteo silencioso con páginas cortadas por el byte budget (40 KB). Fix: shrink-and-retry del mismo cursor hasta que la página entre completa. Repro 61 records: **antes `put_sessions: 22` → después 61** (`north_star_sessions` 1 → 2). Caso nuevo en `--self-test`.
  - **R2:** `COMPARISON.md` NO tocado en este batch (renumeración §7 = co-batch ICP-02).
  - **O1/O2:** paths del workflow ampliados a `src/**` + `vanta-proxy/**`.
  - **N1:** labels duplicados de `WriteBack::track` documentados como ceiling (ver §Notas).
- **Veredicto:** ❌ CHANGES REQUIRED (ronda 1) → fixes aplicados y re-verificados; ⬜ **re-review ronda 2 pendiente (LEAD — reviewer fresco, sesión distinta)**

## Notas
- Re-baseline Task 42: "hooks sin pwsh" ya lo cerró WIRE-10; lo nuevo es one-pager + demo CI + métrica (FIND de DEF-05) + viewer/repo-map consumidos + Claude Code en EDITOR_INTEGRATIONS.
- STOP CONDITION declarada y aplicada: MGR-22 no cerró antes del inicio → demo SIN repo-map + FIND explícito (documentado, no silencioso).
- **Evidencia del contrato (cláusula → verificación):**
  1. One-pager → `docs/user/AI_IDES.md` (indexado en `docs/user/index.md` + `llms.txt` regenerados; `gen-index --check` exit 0).
  2. Demo CI → `vantadb-mcp/tests/demo_ai_ides.rs` (1/1 con `remote-inference`; **gated sin features** (`required-features`; el workflow lo corre CON la feature — PR-blocking)) + `.github/workflows/ci-ai-ides-demo.yml` (actionlint exit 0).
  3. Métrica → `memory_tools.rs` (evento `{session, kind:"search", hits}` en `proxy-memory-events` vía `WriteBack::track`; tests 2/2) + `scripts/north_star_metric.py` (self-test + E2E real con `mcp-call`) + comando en `PROXY.md` §North Star + `SPEC.md` actualizado.
  4. COMPARISON → §7 (qualitative, Regla 11; links a one-pager/hooks/MCP.md).
  5. repo-map/watcher → FIND propuesto abajo; ni demo ni métrica lo requieren (stop condition del plan aplicada).
  6. Claude Code → `EDITOR_INTEGRATIONS.md` §Claude Code (config canónica de `MCP.md:65-80` + hooks) — gap declarado en el DoD del plan cerrado.
- **Hallazgo de distribución (nuevo, honestidad del one-pager):** `vanta-cli mcp-call` (WIRE-10) **no está en v0.7.0** (`git merge-base --is-ancestor 8e55e853 v0.7.0` = 1; tag del 2026-09-25 previo al commit) → viaja en el tren 0.8.0. El one-pager lo declara; hooks/metric quedan funcionales al cortar 0.8.0 (ya preparado en `develop`).
- FINDs propuestos (los registra el LEAD/orquestador en Backlog — esta task no edita `Backlog.md`):
  1. **repo-map/watcher (ICP-01):** no implementado (MGR-22 ⬜ research) — el one-pager lo declara "not yet available"; path = MGR-22 → spec → slice. Demo/métrica no bloqueadas.
  2. **Viewer:** `MemoryLens` documentado como viewer; sin release/paquete desktop no hay captura reproducible en CI → FIND si se quiere demo visual.
  3. **`server.json` 0.6.1 stale** vs workspace 0.7.0 (candidato del plan) — el fix real es automatizar "update on every release", no un bump manual.
  4. **`EDITOR_INTEGRATIONS.md` pre-existente:** snippets con `~/.vantadb` contradicen `MCP.md:37` ("nunca `~`") — no se tocó (scope); corregir en barrido docs.
- **OCR advisory (cierre):** `ocr delegate` emitió rule groups para `memory_tools.rs` (Rust), `ci-ai-ides-demo.yml` (CI project rules), `north_star_metric.py` (Python), `demo_ai_ides.rs` (server/MCP). Revisados por el implementador: 2 ajustes preventivos aplicados (comentario `ponytail` del ceiling sync-en-async en el job L0 — espeja `capture.rs`; `IndexError` sumado al manejo del script). Sin Critical/High identificados. Input para el reviewer fresco.
- **N1 (ronda 1 review — documentado, no fix):** los labels de `WriteBack::track` son **audit-only** y pueden repetirse bajo retry+flush (dos searches de la misma sesión comparten `search:{session}`; un job re-ejecutado en el flush re-escribe su label en el persist file). Sin pérdida de datos: el evento va keyed `{ms}-{seq}` en `proxy-memory-events` y el label solo alimenta `pending_labels`/logs — ceiling heredado de PRX-08, aceptado explícitamente (fuera de scope de ICP-01).
- **R1 — repro reproducible del fix:** store temporal con **61 turnos** (~1.4 KB/record) + 2 eventos search → script pre-fix `put_sessions: 22` (`north_star_sessions: 1` — `sess-59` perdida porque el cursor de la página truncada salta la cola caída) → post-fix **`put_sessions: 61`** (`north_star_sessions: 2`: `sess-00` + `sess-59`). El caso del `--self-test` simula `truncated=true` con cursor "trampa" (apunta al final de la ventana completa) y falla si el cliente vuelve a confiar en la página cortada.
- **Re-verificación del batch (2026-09-30):** `cargo fmt --check` exit 0 · clippy `-p vanta-proxy` + `-p vantadb-mcp` (sin features) exit 0 · `cargo nextest run -p vanta-proxy --build-jobs 2` → **321/321** (2 skipped) · `cargo nextest run -p vantadb-mcp --profile audit --build-jobs 2` (sin features) → **146/146** · demo `--features remote-inference` → **1/1** · `nextest list` sin features → demo ausente · `python scripts/north_star_metric.py --self-test` + `py_compile` ✅ · E2E 61 records ✅ · `actionlint` exit 0.
- Creado por vanta-docs (subagente) desde el master roadmap — wave F5.1.
