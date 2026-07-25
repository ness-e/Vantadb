# Decisiones Arquitectónicas

> ADR-light: decisiones importantes tomadas durante ejecución del pipeline.
> Cada entrada: `YYYY-MM-DD | Contexto | Decisión | Alternativas | Razón`

---

- 2026-07-15 | Campaign Executor test: all 15+ MCP tools validated working end-to-end
- 2026-07-19 | 2026-07-18: Feature Freeze sealed at v0.3.0-stable. P0 + P1-1/P1-3/P1-4 completed. P1-2 deferred (theoretical UB, no trigger). P1-5..9 deferred (post-RC optimizations). Develop branch created. Certify gate passes.
- 2026-07-19 | 2026-07-18: Batch 2 complete — fixed MCP unwrap (proper error), python.rs Default (removed panic), serialize.rs (file size check before map_mut), archive.rs (flush before remap), worker.rs (is_retryable robust, eval setTimeout → Reflect::apply). Docs P2-1/2/3 completed via vanta-docs. P1-10 machete clean. 9/10 Critical + 10/15 Important audit findings resolved.
- 2026-07-21 | progreso: migradas DOC-API-01..06 de plan 2026-07-21-docs-api-audit-fixes a progreso/README.md. Score docs/api audit: 6.4/10 → ~9/10. Archivos: EMBEDDED_SDK.md, openapi.yaml, MCP.md, PYTHON_SDK.md, TS_SDK.md, IQL.md (+nuevo), HTTP_API.md.
- 2026-07-23 | progreso: migrada REV-003 de Backlog a progreso. Coverage 53.85% -> 80.55% (+728 tests en 23 módulos). CI threshold 76% -> 80%. CII Silver met.
- 2026-07-23 | DRV-001: Refactor search.rs (1162L→845L, 5 sub-modules). Extraído phrase.rs, snippet.rs, debug.rs, text_index.rs. Sin breaking changes en API pública. 22 unit tests nuevos. 1598/1599 tests pass.
- 2026-07-24 | pipeline.md: added skill progreso Trigger 1 + auto-commit to both MODO TAREA (steps 6-7) and MODO RUN. Fixes gap where completed tasks weren't migrated to progreso/README.md or committed.
- 2026-07-24 | progreso: migrada DRV-015 de Backlog a progreso. Refactor: extraído recover_valid_records() de open_with_buffer() en src/wal.rs. open_with_buffer() reducido de ~100L a ~55L.
- 2026-07-24 | progreso: 2026-07-24 Pipeline Run completada — 12 tasks migradas de Backlog a progreso. Commits en develop. Patrón: CI/CD 5, SDK 4, web 2, docs 1.
