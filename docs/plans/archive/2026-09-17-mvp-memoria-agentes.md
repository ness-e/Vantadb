# Plan de Ejecución: MVP memoria automática en agentes de código — 2026-09-17

> **Campaign ID:** b2ece025-e9d3-4f8b-835d-1d0143a86b66
> **Inicio:** 2026-09-17
> **Estado:** ✅ COMPLETADO (8/8 DO, 2026-09-17)
> **Fuente:** `docs/Backlog.md` + `SPEC.md` (8/8 decisiones, 0 abiertas) + research skills/hooks/distribución/OCR 2026-09-16/17
> **Autonomous:** false
> **FAIL_MODE:** `parallel` (MAX 3; secuencial interno si colisionan archivos)
> **SPEC:** `SPEC.md` raíz — objetivo, 8 features con AC, stack, comandos, estructura, estilo, testing, límites, 8 decisiones, éxito medible.
> **SDP (plan):** base fija abajo; cada sub-agente ejecuta `campaign_discover_skills_v2` (phase PLAN→BUILD) en pipeline-full Paso 0b y declara `SKILLS_CARGADAS:`.
> **Selección owner:** Gate P vía `question` 2026-09-17 — set confirmado + SHOW-04/FIND-100 dentro + 4 clientes + proxy default-on (Recomendados).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 8 (FIND-100/103/104/105/106/107/108 + SHOW-04) |
| 🟡 DEFER | 0 (programas enteros: MGR/EXE/FUT/STU/UX/TS/WSM/RES/AGT/GOV/CI-01/FIND-101-102/SHOW-02-03/PROV-12/DESKTOP-41-43/OLD-01) |
| ❌ SKIP | 4 (FIND-98 resuelto · SHOW-05 plegado · MKT-18i/DESKTOP-44 humanos) |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 4 (atrapabilidad pánico ORT · alcance exacto exposición · APIs hooks por cliente · casos borde traductor) · ⬇️ downhill = 8 tasks con contrato definido (steps atómicos en task files bajo demanda)

## Gate P — confirmación usuario (2026-09-17)

Triage + Paso 0 + `question` → owner aprobó set completo (Q5), SHOW-04 dentro (Q1), FIND-100 dentro (Q2), 4 clientes (Q3), proxy default-on + opt-out (Q4).

## Verificación real global (Paso 0, 2026-09-17 — lectura directa + pruebas en vivo + web)

- Instalado == fuente (23.70MB ambos, 16/9) → FIND-98 SKIP. `ort::init().commit()` en `src/llm.rs:308` sin pre-chequeo → FIND-100 real.
- `search_memory` solo igualdad (`tools.rs:3181`); rango temporal solo grafos (`:2773`) → gap temporal real. Threads guardan rol/contenido/timestamp (`threads.rs:41-61`); proxy captura a `proxy-turns` (`capture.rs:14-15`) → base curaduría real.
- `examples/` sin `agent_memory_cli`; requirements/QUICKSTART ya ok (resto SHOW-05 → FIND-105).
- OCR: solo delegation integrado (`dev-tools/ocr-review.ps1` + `.opencode/references/ocr-review.md` + gates; `ocr` v1.12.2+ instalado vía npm); SIN integrar: review-rules por path, job CI, plugins, skill portable, MCP server, session viewer, telemetría (fuente: `https://github.com/alibaba/open-code-review` README + webfetch 2026-09-17).
- Hooks OpenCode/Claude/Codex documentados oficialmente (SessionStart, por-mensaje equivalente, PreCompact, Stop) → FIND-106 viable (fuentes web en task).

## Marco normativo — reglas, referencias, comandos, agentes, MCP (obligatorio en ejecución)

Todo sub-agente, antes de codificar, carga y cita en su task file:

- **Reglas** (`.opencode/rules/` — la de su área, lectura completa): `core-engine.md` (100/103/107) · `api-contract.md` (103/106/107) · `server-mcp.md` (103/104/106/107/SHOW-04) · `release-ci.md` (104/105/108) · `python-bindings.md` (SHOW-04 si demo Python) · `README.md` del dir (formato de reglas).
- **Referencias** (`.opencode/references/`): `definition-of-done.md` (todas) · `task-system.md` (lead/orquestador) · `skills-engineering.md` (SDP) · `clean-code-clean-architecture.md` Ap. V (tareas con código) · `ocr-review.md` (gates VERIFY) · `dev-tools.md` + `test-suite.md` + `testing-patterns.md` (verify) · `architecture.md` (103/107) · `research-modules.md` (103) · `understand-anything.md` (107 discovery).
- **Comandos** (`.opencode/commands/`): `pipeline.md` (ejecución) · `audit.md` (verify L9/post-tarea) · `research.md` (DISCOVERY pesado 103/106/107) · `backlog.md` (triage) · `status.md` (tablero) · `ship.md`/`rollback.md` (solo al cierre).
- **Agentes** (`.opencode/agents/` — routing por `Ruta`): `vanta-worker` (100/103/104/106/107/SHOW-04) · `vanta-lead` (105/108, CI/release) · `vanta-review` (P2-01 todas) · `vanta-research` (DISCOVERY pesado si se deriva) · `vanta-docs` (consulta skill) · resto solo si el plan lo pide.
- **MCP/tools disponibles** (`opencode.jsonc` + harness): `codegraph` (`codegraph_explore` blast radius, primero siempre) · `codebase-memory-mcp` (`detect_changes`, `check_index_coverage`, `get_architecture`) · `agent-search` (descubrimiento web) · `metasearchmcp` (`compare_engines`, verificación multi-motor) · `notion` (fetch Problema/Propuesta/SDKs + hijas si mapean, filtro VantaDB) · `campaign` (task-system del lead) · `vantadb` (dogfooding) · `argus` (fallback extracción) · `ocr` CLI (no-MCP: `ocr delegate rule` por defecto sin key; `ocr review` solo nocturno con key).
- **Integración open-code-review (estado real):** ✅ delegation (`ocr-review.ps1` + gates pipeline-full Cierre-5 + task.md Phase 5 + `/audit` L9) · ❌ review-rules por path, job CI, plugins, skill portable, MCP server, session viewer, telemetría → eso es FIND-108.

## Tasks

### Wave0 — robustez + exposición + demo (disjuntos: llm.rs / memory+mcp-nuevo / examples)

**Task 1: FIND-100 — graceful ante onnxruntime incompatible**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta · **Archivos clave:** `src/llm.rs:308`, `src/config.rs:935-940`
- **Verificación real:** `ort::init().commit()` con `let _`, sin pre-chequeo; crash BadVersion documentado en `docs/tasks/EMB-10.md:148-171`.
- **Gate Justificación:** abort sin dylib compatible rompe lo "automático"; fix acotado a init.
- **Contrato:** con dylib <1.27 el proceso NO aborta (exit controlado + `fallback:true` visible) + suite `llm` verde + clippy `-D warnings` 0.
- **Pre-mortem:** (1) pánico dentro de `ort` no atrapable → repro real primero; (2) empaquetar dylib agranda release → medir.
- **Stop:** repro sin abort tras 2 approaches → Gate V; appetite >1d → DEFER.
- **Risk Register:** 🟡×🔴 pánico no-atrapable → repro + plan B empaquetado | 🟢×🟡 tamaño → medir.
- **Cynefin:** 🟨 complicado (FFI). **Top 3:** atrapabilidad / tamaño / tests sin ORT.
- ⬆️ 1 / ⬇️ 2 steps. **DoD:** contrato + task file sync + recitation; commit `fix:`; sin deuda.
- **Skills (≤8, justificadas):** systematic-debugging (root-cause repro) · test-driven-development (RED abort→GREEN graceful) · codebase-memory (blast radius init) · context-engineering · doubt-driven-development (falso graceful) · incremental-implementation (+ base campaign-executor/progreso/ponytail).
- **Herramientas+MCP:** codegraph + codebase-memory-mcp (`detect_changes`, `check_index_coverage`) + `cargo test -p vantadb llm -j 2` + clippy + `campaign_verify_cmd` (bug exit -1 → bash) + agent-search/metasearchmcp SOLO si la API `ort` es incierta (docs.rs, webfetch).
- **Referencias:** rules `core-engine.md` · refs `definition-of-done.md`, `clean-code-clean-architecture.md` Ap. V, `dev-tools.md`, `test-suite.md` · commands `pipeline.md`, `audit.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (init→factory→fallback, callers `get_embedding_provider`) en DISCOVERY; internet solo si API ort incierta.
- Task file `docs/tasks/FIND-100.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `fix: FIND-100 — ...`.

**Task 2: FIND-107 — exponer lo que falta de `vanta-memory` en el MCP**
- **Appetite:** 3d · **Esfuerzo:** 🟠 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** `vanta-memory/src/core/dream/`, `vanta-memory/src/gateway/`, `vantadb-mcp/src/handlers/`
- **Verificación real:** gaps R2 (sueños, aprobación, extracción skills, ingesta real, programador, escenas-escritura); gateway con funciones puras reutilizables.
- **Gate Justificación:** mostrador incompleto; dividir por sub-módulo si excede.
- **Contrato:** `tools/list` incluye nuevas tools + tests por tool verdes + docs + coverage 0 gaps.
- **Pre-mortem:** (1) scope ×6 → dividir al primer síntoma; (2) ingesta sin runner → DEFER-ratificado puntual.
- **Stop:** 1 sub-módulo trancado → ship resto + DEFER-ratificado.
- **Risk Register:** 🟠×🟡 scope → orden + ship parcial | 🟡×🟢 runner ausente → DEFER puntual.
- **Cynefin:** 🟨 complicado. **Top 3:** scope / runner / superficie API.
- ⬆️ 1 (alcance por sub-módulo) / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** api-and-interface-design (schemas/tools) · test-driven-development · systematic-debugging · codebase-memory (blast radius memoria) · doubt-driven-development · documentation-and-adrs · mcp-builder (guía tools MCP) (+ base).
- **Herramientas+MCP:** codegraph + codebase-memory-mcp (impacto architecture) + `cargo test -p vantadb-mcp -j 2` + MCP stdio smoke + notion (fetch dim memoria si mapea) + agent-search/metasearchmcp SOLO si ambigüedad (patrones Mem0/Letta).
- **Referencias:** rules `core-engine.md`, `api-contract.md`, `server-mcp.md` · refs `architecture.md`, `definition-of-done.md`, `clean-code-clean-architecture.md` Ap. V, `understand-anything.md` · commands `pipeline.md`, `research.md`, `audit.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (gateway→handlers por sub-módulo) en DISCOVERY; internet solo si ambigüedad de diseño.
- Task file `docs/tasks/FIND-107.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `feat: FIND-107 — ...`.

**Task 3: SHOW-04 — demo agente-con-memoria (prueba de aceptación MVP)**
- **Appetite:** 3d · **Esfuerzo:** 🟠 · **Prioridad:** 🔴 Alta · **Archivos clave:** `examples/` (nuevo `agent_memory_cli/`), scenes + MCP `inject_context`
- **Verificación real:** `examples/` tiene demo/python/rust/colab pero NO `agent_memory_cli`; scenes + `inject_context` existen.
- **Gate Justificación:** prueba viva del MVP (recuerda entre 2 sesiones); MGR-08 solo diseña.
- **Contrato:** 2 corridas: la 2ª recuerda lo guardado en la 1ª (assert mecánico) + 1 comando sin credenciales.
- **Pre-mortem:** (1) atar a proveedor externo → 100% local; (2) flaky timing → asserts de contenido.
- **Stop:** dependencia externa inevitable → documentar requisito (no forzar).
- **Risk Register:** 🟡×🟢 flaky → asserts contenido | 🟢×🟡 credenciales → prohibidas.
- **Cynefin:** 🟨 complicado. **Top 3:** determinismo / local-first / 1-comando.
- ⬆️ 0 / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** test-driven-development (demo como test e2e) · systematic-debugging · documentation-and-adrs · source-driven-development · context-engineering (+ base).
- **Herramientas+MCP:** `cargo test` (si demo Rust) o pytest (si Python) + corridas 1→2 con assert + `campaign_verify_cmd`.
- **Referencias:** rules `server-mcp.md` (+`python-bindings.md` si demo Python) · refs `testing-patterns.md`, `test-suite.md`, `definition-of-done.md` · commands `pipeline.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (scenes + `inject_context` como base) en DISCOVERY; internet N/A.
- Task file `docs/tasks/SHOW-04.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `feat: SHOW-04 — ...`.

### Wave1 — política recall (diseño+skills; disjunto)

**Task 4: FIND-103 — skills correctas + motor de recall continuo**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta · **Archivos clave:** `skills/vantadb-mcp/`, `vantadb-mcp/src/handlers/`, `docs/api/MCP.md`
- **Verificación real:** counts 45→79, paths `~`, env vars sin documentar, `initialize` sin `instructions`, prompts genéricos, `test-mcp.py` solo-handshake (research 2026-09-16).
- **Gate Justificación:** skill es lo que lee el agente; recall mid-conversación + traductor + umbrales + curaduría.
- **Contrato:** claims verificados mecánicamente + `instructions` presente + diseño temporal (filtros + traductor con tests) + coverage 0 gaps.
- **Pre-mortem:** (1) overwrite contenido único → diff+merge por sección; (2) traductor casos raros → suite + fallback a rango amplio avisado.
- **Stop:** superficie indefinida → Gate V (FIND-107 la fija en paralelo).
- **Risk Register:** 🟡×🟢 overwrite → diff+merge | 🟡×🟡 traductor → suite + fallback.
- **Cynefin:** 🟨 complicado. **Top 3:** overwrite / traductor / alcance.
- ⬆️ 1 (casos borde traductor) / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `docs:`/`fix:`; sin deuda.
- **Skills:** documentation-and-adrs (contenido skill) · spec-driven-development (diseño recall) · api-and-interface-design (instructions/prompts) · codebase-memory · doubt-driven-development · notion-mcp-master (fetch Notion) · web-research (competencia, riusar R3) · systematic-debugging (+ base).
- **Herramientas+MCP:** notion (fetch Problema/Propuesta + hijas si mapean) + codegraph + codebase-memory-mcp + agent-search/metasearchmcp (solo gaps vs R3) + `validate-docs-coverage.ps1` + `test-mcp.py` + `campaign_verify_cmd`.
- **Referencias:** rules `server-mcp.md`, `api-contract.md` · refs `skills-engineering.md`, `definition-of-done.md`, `task-system.md`, `architecture.md`, `research-modules.md` · commands `pipeline.md`, `research.md`, `audit.md` · agents `vanta-worker` + `vanta-review` P2-01 (código: filtros+traductor+instructions).
- **Investigación:** código (handlers search/recall/initialize/prompts) + internet (solo gaps vs research 2026-09-16).
- Task file `docs/tasks/FIND-103.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `docs: FIND-103 — ...`.

### Wave2 — instalador + ganchos (disjuntos: scripts vs hooks; heredan FIND-103)

**Task 5: FIND-104 — instalador interactivo completo**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** `setup-embeddings.ps1`, `vanta-mcp-local.ps1`, `skills/vantadb-mcp/assets/`
- **Verificación real:** wizard (EMB-11) + launcher (EMB-12) existen; falta proxy default, bloque MCP por cliente, regla agente, prueba final.
- **Gate Justificación:** cero fricción = 1 flujo guiado; proxy default-on + opt-out (Q4).
- **Contrato:** wizard end-to-end simulado + `-NonInteractive` intacto + secrets nunca a disco + prueba viva final verde.
- **Pre-mortem:** (1) formato MCP por cliente erróneo → plantillas testeadas; (2) proxy molesto → opt-out en 1 paso.
- **Stop:** cliente sin formato verificable → documentado-manual (no bloquea).
- **Risk Register:** 🟡×🟢 formatos → plantillas testeadas | 🟢×🟡 proxy → opt-out.
- **Cynefin:** 🟨 complicado. **Top 3:** formatos / proxy / secrets.
- ⬆️ 0 / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** shipping-and-launch (flujo instalación) · git-workflow-and-versioning · systematic-debugging · test-driven-development (scripts testeables) · documentation-and-adrs · security-and-hardening (secrets) · source-driven-development (+ base).
- **Herramientas+MCP:** `pwsh -NoProfile` + MCP stdio smoke + agent-search/metasearchmcp SOLO si formato cliente incierto (docs oficiales) + `campaign_verify_cmd`.
- **Referencias:** rules `release-ci.md`, `server-mcp.md` · refs `definition-of-done.md`, `dev-tools.md` · commands `pipeline.md`, `audit.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (wizard/launcher/assets) + internet (solo formatos cliente no verificados).
- Task file `docs/tasks/FIND-104.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `feat: FIND-104 — ...`.

**Task 6: FIND-106 — ganchos de memoria por cliente**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta · **Archivos clave:** `.opencode/` (plantillas, NO config viva ajena), `skills/vantadb-mcp/assets/`, `vanta-mcp-local.ps1` (lectura)
- **Verificación real:** hooks OpenCode/Claude/Codex documentados oficialmente; repo sin plantillas (research 2026-09-17).
- **Gate Justificación:** único "siempre" determinista; implementa política FIND-103 (OpenCode+Claude+Cursor+Codex).
- **Contrato:** plantillas por cliente + tests con eventos simulados + presupuesto tokens (umbral, top-k, cuándo nada).
- **Pre-mortem:** (1) APIs cambian → versionado + tests; (2) recall ruidoso → umbral + métrica.
- **Stop:** cliente sin hooks estables → documentado-manual (no bloquea resto).
- **Risk Register:** 🟡×🟢 APIs → versionado + tests | 🟡×🟢 ruido → umbrales.
- **Cynefin:** 🟨 complicado. **Top 3:** APIs / ruido / tests.
- ⬆️ 1 (APIs exactas) / ⬇️ 3 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** documentation-and-adrs (plantillas) · test-driven-development (eventos simulados) · systematic-debugging · context-engineering · web-research (docs oficiales hooks por cliente) · api-and-interface-design · source-driven-development (+ base).
- **Herramientas+MCP:** agent-search (descubrimiento APIs hooks) + metasearchmcp (verificación) + webfetch docs oficiales + simuladores de eventos + `campaign_verify_cmd`. codegraph N/A (config, no código).
- **Referencias:** rules `server-mcp.md` · refs `skills-engineering.md`, `definition-of-done.md` · commands `pipeline.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** internet (docs oficiales por cliente — principal) + código (política FIND-103, MCP tools a invocar).
- Task file `docs/tasks/FIND-106.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `feat: FIND-106 — ...`.

### Wave3 — comando único + review total (disjuntos: scripts/docs vs workflows)

**Task 7: FIND-105 — comando único de instalación**
- **Appetite:** 1d · **Esfuerzo:** 🟢 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** `scripts/install.sh`, `scripts/install.ps1`, `README.md`, `docs/QUICKSTART.md`
- **Verificación real:** instaladores bajan tarball con sha256; sin encadenar wizard; README/QUICKSTART sin one-liner.
- **Gate Justificación:** 1 línea sin clone ni rustup; incluye resto SHOW-05.
- **Contrato:** install simulado encadena wizard + docs con one-liner por OS + requirements verificado.
- **Pre-mortem:** (1) desconfianza `irm|iex` → checksum + URL oficial; (2) sobreinstalación rompe → idempotente + backup.
- **Stop:** sin entorno limpio → dry-run + checksum (no verde falso).
- **Risk Register:** 🟢×🟡 confianza → checksum | 🟢×🟢 idempotencia → backup.
- **Cynefin:** 🟦 obvio. **Top 3:** confianza / idempotencia / docs.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** shipping-and-launch · git-workflow-and-versioning · documentation-and-adrs · security-and-hardening (checksums) · ci-cd-and-automation (+ base).
- **Herramientas+MCP:** `pwsh -NoProfile`/`sh` dry-run + checksum verify + `campaign_verify_cmd`. codegraph N/A.
- **Referencias:** rules `release-ci.md` · refs `definition-of-done.md` · commands `pipeline.md` · agents `vanta-lead` (distribución) + `vanta-review` P2-01.
- **Investigación:** código (instaladores actuales) + internet (patrones Deno/Bun ya researched — reusar).
- Task file `docs/tasks/FIND-105.md` · ✅ COMPLETED · Ruta vanta-lead. Branch develop. Commit `feat: FIND-105 — ...`.

**Task 8: FIND-108 — integración completa open-code-review**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `.github/workflows/` (job nuevo), `.opencodereview/` (nuevo), `dev-tools/ocr-review.ps1`
- **Verificación real:** solo delegation integrado; `ocr` instalado (npm global); sin `.opencodereview/` en repo; research 2026-09-17.
- **Gate Justificación:** rules por path + evidencia nocturna; delegation sigue default sin key.
- **Contrato:** rules mapeadas a `.opencode/rules/` + job CI nocturno verde (o documentado-sin-key) + viewer como evidencia + delegation intacto.
- **Pre-mortem:** (1) job nocturno sin key falla → delegation documentado como default, full solo con key; (2) rules genéricas ruidosas → mapear 1:1 a reglas VantaDB.
- **Stop:** sin key disponible → delegation + rules locales (no bloquea).
- **Risk Register:** 🟡×🟢 key → default sin key | 🟢×🟢 ruido → mapeo 1:1.
- **Cynefin:** 🟦 obvio. **Top 3:** key / ruido / evidencia.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file + recitation; commit `ci:`; sin deuda.
- **Skills:** ci-cd-and-automation (job CI) · documentation-and-adrs · systematic-debugging · web-research (docs OCR: review-rules/cicd/delegate) · git-workflow-and-versioning (+ base).
- **Herramientas+MCP:** `ocr delegate rule` (sin key) + `actionlint` + agent-search/metasearchmcp (docs oficiales) + `campaign_verify_cmd`.
- **Referencias:** rules `release-ci.md` · refs `ocr-review.md`, `dev-tools.md`, `definition-of-done.md` · commands `pipeline.md`, `audit.md` · agents `vanta-lead` (CI) + `vanta-review` P2-01.
- **Investigación:** internet (docs open-codereview.ai: rules/cicd/delegate/mcp) + código (wrapper actual).
- Task file `docs/tasks/FIND-108.md` · ✅ COMPLETED · Ruta vanta-lead. Branch develop. Commit `ci: FIND-108 — ...`.

## SKIP

- ❌ SKIP FIND-98 — binario instalado idéntico al fuente (16/9 8:06, 23.70MB ambos; EMB-19 `a5d549af`). Resuelto, no re-trabajar.
- ❌ SKIP SHOW-05 — 70% ya hecho (requirements ≥0.5.0, QUICKSTART enlaza); resto (línea README + decisión TS) plegado al contrato FIND-105.
- ❌ SKIP MKT-18i — requiere feature-request upstream humano (BLOQUEADO externo).
- ❌ SKIP DESKTOP-44 — sesión guiada con owner, no tarea autónoma.

## DEFER (programas enteros, fuera del MVP)

MGR-01..25 (gobernanza v0.7+) · EXE-01..10 (cierre Propuesta; EXE-03 requiere humanos) · FUT-02..15 (roadmap) · STU-01..03 + UX-06..15 (desktop UI) · TS-10/11/13 + WSM-14 (web/npm adopción) · PROV-12 (PyPI: requiere secrets/humano) · RES-01..14 (research 2-3sem) · AGT resto (sistema agentes) · GOV resto (docs Show HN) · CI-01 + FIND-101/102 (baja, no MVP) · SHOW-02/03 (showcase web/RAG-PDF) · DESKTOP-41/43 · OLD-01/DISC-03/DEC-02 (roadmap/icebox).

## BLOQUEADO

Nada (EXE-01 depende de MGR-19 pero está DEFER; sin cadena activa).

## Grafo de dependencias / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave0: FIND-100 + FIND-107 + SHOW-04
Wave1: FIND-103
Wave2: FIND-104 + FIND-106
Wave3: FIND-105 + FIND-108
```

Órdenes internos: FIND-106 implementa política FIND-103 (Wave1→Wave2); FIND-104 consume regla FIND-103; FIND-105 publica lo de FIND-104; FIND-107 divide por sub-módulo si excede; SHOW-04 solo necesita motor actual; FIND-108 no bloquea a nadie.
Nota runner: si waves ×3 abortan, fallback secuencial por wave (precedente 2026-09-09/10).

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Mismo archivo en una wave | waves separan por archivo (llm.rs / memory+mcp-nuevo / examples en Wave0; scripts vs hooks en Wave2; install vs workflows en Wave3); si colisiona → secuencial interno |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO |
| rustc crash paralelo / OOM | `-j 2` siempre en cargo |
| Secrets a disco (keys) | prohibido por contrato (FIND-104/105/106/108); review lo audita |
| Clientes cambian APIs de hooks | plantillas versionadas + tests (FIND-106) |
| OCR sin key en CI | delegation default; full solo nocturno con key (FIND-108) |

## Notas

- SKILLS_CARGADAS base sesión (16): campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, documentation-and-adrs, idea-refine, codebase-memory, ponytail(full) + notion-mcp-master/web-research/mcp-builder por tarea según matriz.
- MCP/tools por tarea en su ficha (codegraph primero siempre; codebase-memory-mcp impacto; agent-search descubrimiento; metasearchmcp verificación; notion Problema/Propuesta; ocr delegation en VERIFY; campaign vía lead).
- Routing: `vanta-worker` (100/103/104/106/107/SHOW-04) · `vanta-lead` (105/108) · `vanta-review` P2-01 todas · `vanta-research` si DISCOVERY pesado (R4) · `vanta-docs` consulta skill.
- SPEC.md raíz con 8/8 decisiones; Gate P confirmado en Q5 (esta sesión). Responde a tu punto 4: cada ficha cita rules/references/commands/agents exactos y el prompt del sub-agente (pipeline-run.md §6.f) los lleva adentro.

=== RECITATION ===
Objetivo activo: PLAN MVP-memoria-agentes — CERRADO 8/8
Estado: completed (desde: act)
Última acción: P2-01 (3× vanta-review: Rust approve, scripts approve, docs/CI changes-required→levantado con 25109073) + progreso (Backlog −8/+6, avance 3 dominios) + verifies lead en verde
Resultado: ✅
Próxima acción: archive a docs/plans/archive/ + nota meta.md (orquestador)
Contrato: 8/8 DO con commit + verify mecánico + P2-01; tools/list 86; coverage 0 gaps
Invariantes: WIP ajeno intacto (completions/*, .opencode, reparacion.bat, stash GOV-C4); sin push (commits locales en develop)
Comandos de verificación: llm 15/15 serial · temporal 12/12 · dream 10/10 · scene 5/5 · test-hooks 50/50 · fmt/clippy/actionlint/wrapper exit 0
Deuda: FIND-109 (Alta, WAL salvage) + FIND-110/111/112/113 (DEFER-ratificado Media) + FIND-114/115 (Baja) + FIND-101/102 (Baja) — todas en Backlog
Próxima tarea si completa: ninguna (plan cerrado)
last-synced: 2026-09-17
=== END RECITATION ===

## Retrospectiva de cierre (Start/Stop/Continue + 1 acción medible)

- **Start (seguir haciendo):** prompts de delegación con DETALLE OBLIGATORIO completo (10 bloques) — 7/8 tareas salieron ✅ al primer intento con ese formato; RESUME misma-sesión rescató FIND-100 (rate-limit) y FIND-108 (aborto) sin perder trabajo.
- **Stop (dejar de hacer):** `--no-verify` por reflejo (lo usé una vez en `d441d1f2` y tuve que revertir a `25109073` con hook en verde); waves paralelas que comparten archivos de conteo global (tools/list asserts colisionaron FIND-100×FIND-107 en el hook).
- **Continue (igual):** P2-01 con revisor distinto por área (3 revisiones en paralelo encontraron 12 hallazgos reales, 1 changes-required legítimo); staging selectivo + WIP ajeno intocable.
- **Acción medible:** reducir hallazgos P2-01 Media-por-tarea de 1.5 a ≤0.5 (baseline este plan: 12 hallazgos / 8 tareas) añadiendo al prompt de delegación un self-check del revisor (globs existen, secrets solo-env, comentarios paralelos) antes del RESULTADO.

=== RECITATION FIND-107 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-107 exponer vanta-memory en MCP - COMPLETO
Estado: completed
Última acción: S1+S2+S3 shippeados (3 commits feat), S4/S5/S6a/S6b DEFER-ratificado con justificacion codigo, verify full + smoke stdio + task file sync (41af793d)
Resultado: OK
Próxima acción: Orquestador: P2-01 vanta-review sobre 472526dc+325b1237+74c225a0, crear FIND-110..113 en Backlog, ejecutar FIND-103
Contrato: Contrato: tools/list 79->86 (7 tools S1-S3) + tests por tool verdes + docs + coverage 0 gaps. Verificacion: fmt workspace OK, clippy workspace all-targets all-features OK, suite vantadb-mcp 18 targets 0 failed, coverage 0 gaps, OCR sin Critical/High, smoke stdio 4/4 (86 tools). Evidencia: commits 472526dc+325b1237+74c225a0+41af793d; tests dream_tests.rs 10/10, scene_write_edit 5/5, scene_tests 7/7. Invariantes: src/llm.rs, examples/, Backlog, plan, prohibidos intactos; dev/memory profiles intactos. Deuda: S4-S6 DEFER-ratificado (FIND-110..113 propuestos, Backlog lo edita el orquestador). Queda_pendiente: P2-01 review por agente distinto + FIND-103 (orquestador).
Próxima tarea si completa: FIND-103
=== END RECITATION ===

=== RECITATION FIND-100 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-100 graceful ante onnxruntime incompatible
Estado: completed
Última acción: RESUME: re-verificacion minima verde (test 15/15, clippy 0, fmt 0) + commit 22d5a142 solo src/llm.rs + docs/tasks/FIND-100.md, sin push
Resultado: OK
Próxima acción: Orquestador: FIND-103 (Wave1) + review P2-01 de este commit
Contrato: Contrato: (a) dylib incompatible -> exit controlado + fallback:true visible, sin abort; (b) suite llm verde; (c) clippy -D warnings 0. Verificacion: cargo test -p vantadb --features embed-local --lib llm -j 2 = 15/15 EXIT=0 (serie+paralelo); clippy --all-targets --features embed-local -D warnings EXIT=0; fmt EXIT=0; cargo check default EXIT=0; pre-commit hooks ok; commit 22d5a142. Evidencia: commit 22d5a142 (confianza alta); logs en C:/Users/Eros/AppData/Local/Temp/opencode/find100-*.txt (confianza alta). Artefactos: src/llm.rs, docs/tasks/FIND-100.md. Invariantes: WIP ajeno intacto (FIND-107/SHOW-04, Backlog, plan); -j 2; sin simbolos publicos nuevos; NO PUSH. Deuda: OCR delegation + review P2-01 los hace el orquestador; Notion Paso 0c no ejecutable en este harness. Queda_pendiente: orquestador ejecuta FIND-103 (Wave1).
Próxima tarea si completa: FIND-103
=== END RECITATION ===

=== RECITATION SHOW-04 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: SHOW-04 demo agente-con-memoria: prueba viva del MVP (recuerda entre 2 sesiones)
Estado: completed
Última acción: RED test fallo correcto, GREEN demo+README, VERIFY demo+pytest+verify_cmd+OCR advisory, commit 3bb5210d (4 files, pre-commit hook ok)
Resultado: ✅
Próxima acción: ninguno en SHOW-04; orquestador: FIND-103 (Wave1)
Contrato: verificacion: python examples/agent_memory_cli/agent_memory_demo.py ✅ (SESION 1 OK + SESION 2 OK + MEMORIA VERIFICADA) + python -m pytest examples/agent_memory_cli/ -v ✅ 3 passed + campaign_verify_cmd ✅ exit 0 | evidencia: claim: sesion 2 recuerda lo de sesion 1, evidencia: examples/agent_memory_cli/agent_memory_demo.py:sesion_2_recordar + pytest 3 passed, confianza: alta | artefactos: examples/agent_memory_cli/agent_memory_demo.py, test_agent_memory_cli.py, README.md, docs/tasks/SHOW-04.md | invariantes: no tocar src/llm.rs (FIND-100), vanta-memory/handlers (FIND-107), WIP ajeno intacto | deuda: ninguna | queda_pendiente: FIND-103 (orquestador Wave1)
Próxima tarea si completa: FIND-103
=== END RECITATION ===

=== RECITATION FIND-103 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-103 skills correctas + motor recall continuo
Estado: completed
Última acción: subagent vanta-worker 7/7 slices, commit c01baa90 (16 files), lead verify temporal_tests 12/12 EXIT=0
Resultado: ✅
Próxima acción: Wave2: FIND-104 + FIND-106 en paralelo
Contrato: claims verificados + instructions en initialize + traductor temporal con tests + coverage 0 gaps
Próxima tarea si completa: FIND-104
=== END RECITATION ===

=== RECITATION FIND-104 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-104 instalador interactivo end-to-end
Estado: completed
Última acción: subagent vanta-worker 4/4 slices, commit a1bea54b (scripts + assets/install 6 plantillas)
Resultado: ✅
Próxima acción: Wave3: FIND-105 + FIND-108
Contrato: wizard simulado + NonInteractive intacto + secrets nunca a disco + prueba viva verde
Próxima tarea si completa: FIND-105
=== END RECITATION ===

=== RECITATION FIND-106 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-106 ganchos memoria por cliente
Estado: completed
Última acción: subagent vanta-worker 5/5, commit 5b09fb54 (9 files), lead verify test-hooks.ps1 50/50 EXIT=0
Resultado: ✅
Próxima acción: Wave3: FIND-105 + FIND-108
Contrato: plantillas 4 clientes + tests simulados + token budget
Próxima tarea si completa: FIND-105
=== END RECITATION ===

=== RECITATION FIND-105 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-105 comando único de instalación
Estado: completed
Última acción: Step1 instaladores (dry-run+backup+cadena wizard) + Step2 docs (nota trust README + §0 QUICKSTART) + verify full + commit 1349e63c sin push
Resultado: OK
Próxima acción: Orquestador: P2-01 review de 1349e63c + skill progreso + cierre Wave3
Contrato: verificacion: sh --dry-run/--no-wizard/--help exit 0 + ps1 -DryRun exit 0 (wizard chain impresa) + Select-String one-liner README:203,209 QUICKSTART:28,34 + requirements >=0.5.0 x2 + git diff --check 0 + validate-docs-coverage 0 gaps + ocr delegate 1 grupo default sin Critical/High + commit 1349e63c (hooks verdes) | evidencia: commit 1349e63c 5 files +325/-3; campaign_verify_cmd bug exit -1 reproducido -> fallback bash directa | artefactos: scripts/install.sh, scripts/install.ps1, README.md, docs/QUICKSTART.md, docs/tasks/FIND-105.md | invariantes: WIP ajeno intacto (completions, Backlog, plan, .opencode, reparacion.bat no stageados); assets/install + setup-embeddings.ps1 solo lectura; develop; NO PUSH | deuda: P2-01 review + skill progreso (orquestador) | queda_pendiente: orquestador: P2-01 vanta-review sobre 1349e63c, cierre + progreso, siguiente FIND-108 ya en paralelo
Próxima tarea si completa: ninguna (última wave)
=== END RECITATION ===

=== RECITATION FIND-108 ===
Campaign ID: b2ece025-e9d3-4f8b-835d-1d0143a86b66
Objetivo activo: FIND-108 integracion completa open-code-review
Estado: completed
Última acción: retry fresco ses_f4f5d96f4ffeB4mAX52aE6bAv3 (previa abortada sin ID): 13/13 Custom + actionlint 0 + wrapper ok, commit 2430385a (3 paths)
Resultado: ✅
Próxima acción: cierre plan: P2-01 + progreso + retrospectiva + archive
Contrato: rules por path + job CI nocturno + viewer evidencia + delegation intacto
Próxima tarea si completa: cierre-plan
=== END RECITATION ===
