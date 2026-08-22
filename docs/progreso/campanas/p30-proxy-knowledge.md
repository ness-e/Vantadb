# Campaña P30 — Vanta Proxy + Knowledge (F6+F7)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### Campaña P30 Vanta Proxy + Knowledge completada 2026-08-21 - 9/9 tareas F6+F7 (plan `2026-08-21-vanta-proxy-knowledge.md`)

Cierre del roadmap TDAM: **F6 vanta-proxy** (binario transparente 3 protocolos wire) + **F7 knowledge** (wiki state machine + ingest + 12 tools MCP query-only). Suites finales: vanta-proxy **52/52**, vantadb-mcp **29/29**, vanta-memory **453/453**, core wiki **24/24** — fmt/clippy `-D warnings` limpios, docs coverage 0 gaps.

- **Wave 0:** MEM-32 tools MCP code_* sobre graphrag propio, D28 sin deps externas (`70048abf`); MEM-28 wiki store state machine pending→ready con CAS optimista en InternalMetadata (`0c3a9dcf`); MEM-29 fuentes locales + chunker 12k/400 con path traversal guard (`e4767c0a`).
- **Wave 1:** MEM-25 crate vanta-proxy + 3 protocolos wire verbatim con SSE passthrough (`eb354c0d`); MEM-30 ingest worker serial + merge por página + fallback P4 (`2542498d`).
- **Wave 2:** MEM-26 ciclo auth→session→inject (D34 fail-closed, D29 system-prompt-only) (`df9f6dc0`); MEM-33 tools wiki_* BM25 propio título×5 + BFS cap 200 (`02c87177`); MEM-31 progreso ingest canal interno + polling run_id throttle 500ms (`efa12bab`).
- **Wave 3:** MEM-27 rate-limit sliding window in-process 60 req/min (D35) + write-back retry persistido + mem-command TDAM (D33) + reporting JSON (`11d443cd`).

**Decisiones cerradas upfront:** D24-D37 (TOML config, auth obligatoria fail-closed, sessionKey 5 aliases headers, canal interno+polling sin HTTP, paths locales sin fetcher, riesgos aceptados D37). La lección MEM-24 aplicada: cero decisiones abiertas al delegar.

**Retrospectiva (D2):** Start: decisiones cerradas antes de delegar + verify mecánico por tarea. Stop: RESUME genérico — el que funciona lleva feedback exacto del fallo. Continue: SARL por escalones; primer-intento sub-agentes mejoró vs P29 (~22% → ~44% con decisiones cerradas), aún bajo objetivo 90%. Acción medida: investigar cutoff de contexto de vanta-worker ANTES de la campaña bindings (MEM-36).

**Plan archivado:** `docs/plans/archive/2026-08-21-vanta-proxy-knowledge.md` — 9/9 completadas. Roadmap TDAM F1-F7 COMPLETO (P27+P29+P30 = 42 tareas).

### Campaña P30 Vanta Proxy + Knowledge en ejecución 2026-08-21 - Wave 0 (plan `2026-08-21-vanta-proxy-knowledge.md`)

- **MEM-32** MCP tools code_* query-only (8 tools sobre graphrag propio + `src/graph.rs`, D28) ✅ commit `70048abf` — nextest 22/22, fmt/clippy `-D warnings` limpios.
- **MEM-28** F7 Wiki store + state machine pending→ready en core (`src/wiki/{mod,state,store}.rs` + wiring `lib.rs`) ✅ código verificado: patrón InternalMetadata (EntityStore/SceneNodeStore D4), CAS por estado con `version` optimista, `run_id` por build, re-ingest → `ExecutionConflict` (409-equivalente), sync_error truncado 500 chars, páginas gestionadas `locked:true` con cascade delete y dedup por path canónico type+title. Tests D19 11/11 (`cargo nextest run -p vantadb wiki::`), fmt/clippy `-D warnings` exit 0, `cargo check` de vanta-memory y vantadb-mcp sin regresión. Commit pendiente del lead.
