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
