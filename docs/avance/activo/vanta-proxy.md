---
title: "Avance — Vanta Proxy"
type: domain-log
status: active
tags: [vantadb, avance, vanta-proxy, proxy, gateway, knowledge, wiki, rate-limit]
last_reviewed: 2026-08-22
aliases: []
---

# Avance — Vanta Proxy

> Registro consolidado del trabajo completado sobre el crate `vanta-proxy/` y la superficie Knowledge (F7): protocolos wire verbatim (OpenAI/Anthropic/Responses), ciclo auth→session→inject, rate-limit, y el store/wiki de conocimiento consultable desde MCP. **IDs originales conservados.** Catch-up por campaña (no commit-por-commit).

## Cobertura rápida

- **F6 (proxy):** crate `vanta-proxy` con 3 protocolos wire verbatim + SSE passthrough; auth fail-closed → sesión → inyección de contexto system-prompt-only; rate-limit sliding window + write-back retry + mem-command.
- **F7 (knowledge):** wiki store con state machine pending→ready, fuentes locales con chunker 12k/400, ingest worker serial + polling de progreso, tools MCP query-only code_\*/wiki_\*.

---

## Campaña P30 — Vanta Proxy + Knowledge (F6+F7)

### F6: MEM-25..27 — Proxy LLM
- **Fecha:** 2026-08-21
- **MEM-25** (`eb354c0d`): crate nuevo `vanta-proxy` con 3 protocolos wire verbatim (OpenAI / Anthropic / Responses, subset), passthrough SSE sin re-buffering.
- **MEM-26** (`df9f6dc0`): ciclo completo auth→session→inject: D34 fail-closed (sin auth no hay request), 5 aliases de sessionKey, D29 inyección solo vía system prompt.
- **MEM-27** (`11d443cd`): rate-limit sliding window + write-back retry + mem-command (comando TDAM embebido) + reporting de uso. Cierra F6.
- **Resultado:** ✅ F6 completa.

### F7: MEM-28..33 — Knowledge (wiki store + ingest + tools)
- **Fecha:** 2026-08-21
- **MEM-28** (`0c3a9dcf`): wiki store en core (`vantadb`) con state machine pending→ready sobre InternalMetadata, CAS optimista.
- **MEM-29** (`e4767c0a`): fuentes locales wiki + chunker 12k tokens / overlap 400, guard path traversal, decisión D36.
- **MEM-30** (`2542498d`): ingest worker serial + merge por página + fallback P4 (STRUCTURAL_FILES, ensureSources).
- **MEM-31** (`efa12bab`): progreso de ingest por canal interno + polling run_id (throttle 500ms, P4 best-effort).
- **MEM-32** (`70048abf`): tools MCP `code_*` query-only sobre graphrag propio (D28).
- **MEM-33** (`02c87177`): tools MCP `wiki_*` query-only con BM25 propio título×5, BFS cap 200, guard require_ready.
- **Resultado:** ✅ F7 completa. Campaña 9/9 cerrada (`b316e3eb`, plan archivado). Roadmap TDAM F1-F7 al 100%.

### PRX-01 (worker, cierre lead): wiring advance+classifier+mem-commands+degraded - Resultado: (1) POST /session/advance + header x-vanta-session disparan SessionStore::advance; (2) classify_cc_request consume routing Main/Fork/Sidequery; (3) mem:sync/create-skill ejecutan pipeline real (create-skill persiste record en SKILLS_NAMESPACE); (4) UpstreamHealth::observe: DEGRADED_ENTER_FAILURES=3 (429/5xx) -> set_degraded(true), DEGRADED_EXIT_SUCCESSES=5 -> sale; suite 111/111 (86 lib + 5 pipeline + 10 proxy_wire + 5 prx01_wiring + 5 tool_loop) + clippy -D warnings + fmt limpios. Commit a4f63290 (2026-09-07).

### PRX-05: Model discovery + endpoints auxiliares
- **Fecha:** 2026-09-09
- **Objetivo:** `GET /v1/models` (picker /model Claude Code), `count_tokens`, beta headers sin 404s (litellm#13252)
- **Resultado:** ✅
- **Commit:** a1855dd6 (11 files: `handlers/auxiliary.rs` nuevo + `UpstreamConfig.models` + 4 rutas + `tests/prx05_aux.rs` + fix 1-línea `models: Vec::new()` en 4 suites existentes)
- **Contrato:** `cargo test -p vanta-proxy` 134 passed 0 failed + clippy `-D warnings` 0 + fmt 0. Modelos derivados de `[upstream] models` (nunca hardcodeados); `count_tokens` estimación local `ceil(chars/4)`; beta headers passthrough verbatim + idempotencia PRX-04 byte-estable; auth D34 en las 4 rutas.
