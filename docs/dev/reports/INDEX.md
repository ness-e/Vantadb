---
title: Report Registry — docs/dev/reports/INDEX.md
kind: index
status: active
description: Notas
tags: [vantadb, reports, index, registry]
---

# Report Registry — docs/dev/reports/INDEX.md

> **Registro maestro de reportes de review/audit.** Una fila por reporte.
> Columna **estado**: `vigente` (último del modo) · `superado` (reemplazado por uno más nuevo) · `archivado` (movido a `archive/`) · `—` (pendiente de sync).
>
> **Se actualiza:**
> 1. en cada `/review` y `/audit` (fase L11b de `unified-review`),
> 2. al inicio de sesión (`progreso` **Trigger 4**: sincroniza reportes nuevos/huérfanos).
>
> **Hallazgos ≥ medium** de cada reporte se derivan a `docs/dev/Backlog.md` →
> sección `## Hallazgos pendientes de reportes` (IDs `REVIEW-NN` / `AUD-NN`).
>
> **Reviews archivados (45).** Los reportes de `docs/dev/reviews/archive/` ya no
> viven en el repo: se conservan en Notion › «Reviews 2026»
> (https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd), una página por
> review con el análisis de vigencia a 2026-10-22 (qué hallazgos siguen abiertos
> y con qué evidencia). Las filas de la tabla que citan
> `Notion/Reviews 2026/<archivo>.md` apuntan a esas páginas.

## Index

| Fecha (YYYYMMDD-HHMMSS) | Modo | Archivo | QG | C/H/M/L/I | Estado | Resumen |
|---|---|---|---|---|---|---|
| 20260825-223924 | research | `docs/dev/reviews/research-web-prod-20260825.md` | - | - | migrado 2026-09-22 a Vantadb-web `docs/history/research-web-prod-20260825.md` | INV-web-01: producto web 7.2/10 vs 5 competidores (extraídos en vivo). 11 hallazgos → 7 APLICAR (plan WEB-03..09) + WEB-01/02 backlog + registro corregido (H-01) + H-09 diferido |

| 2026-08-10-1740 | eval | `docs/dev/reports/dora.md` | — | — | vigente | P3-07 (P3): DORA flow metrics — cycle/lead time, CFR, throughput, flow table desde plan files + task files + `verify-log.jsonl` (fechas derivadas best-effort, fallback mtime) |
| 2026-08-10-2007 | eval | `docs/reports/pipeline-evals.md` | — | — | vigente | EVAL-01 (P0): harness de evals del pipeline — North Star metrics (primer intento, falsos positivos, regresión) desde `verify-log.jsonl` |
| 2026-08-08-0026 | full | `Notion/Reviews 2026/audit-full-20260808-002617.md` | ? FAIL | 0/5/8/9/0 | superado | INV-024 sin tests + prune duplica select_neighbors; clippy gate rojo (5 pre-existentes). AUD-012..021 derivados. (superado por audit-full-20260812-231204) |
| 2026-08-12-2312 | full | `Notion/Reviews 2026/audit-full-20260812-231204.md` | ✅ PASS | 0/5/5/10/3 | superado (por audit-full-20260825-031011) | CI-07 pinning vérifié 17/17 + P2-7 VERIFIED ROI (serde_json fuera del write path); VETO falso reversado (`.env.tokens` gitignored); certify nocturno 7/7 (Recall 0.971). AUD-022..031 derivados |
| 2026-08-05-2025 | certify | `Notion/Reviews 2026/review-certify-2026-08-05-2025.md` | ✅ PASS | 0/0/2/8/7 | vigente | Certify pre-push 9.4/10 (A). Recomendaciones: mitigar npm-audit web (6 vulns), migrar put_batch a keyword |
| 2026-08-22-2008 | full | `Notion/Reviews 2026/review-full-20260822-200850.md` | ❌ FAIL | 0/2/8/12/9 | vigente | Full 9 fases paralelo, 7.9/10 (B). QG falla: nextest roto (filtro stale) + OOM cargo test workspace + h2 RUSTSEC (deny advisories FAIL). Bug real cache_warmer latch. Backlog sync completo: REVIEW-06..20 derivados |
| 2026-08-05-1545 | full | `Notion/Reviews 2026/review-full-2026-08-05-t1545.md` | ❌ FAIL | — | vigente | Full ISO 7.1/10 (B). (Sync pendiente) |
| 2026-07-27-0309 | full | `Notion/Reviews 2026/review-full-2026-07-27-0309.md` | — | — | superado | Full previo; findings consumidos en P14 REVIEW del backlog |
| 2026-08-04-1745 | full | `Notion/Reviews 2026/audit-full-2026-08-04T174544.md` | ❌ FAIL | 1 C | superado | &#8672; AUDIT-01: UAF memory-safety en Python SDK NumPy path; benchmark nocturno crashea (superado por audit-full-20260808-002617) |
| 2026-07-24-1751 | full | `Notion/Reviews 2026/audit-full-2026-07-24T1751Z.md` | ❌ FAIL | 1 C | superado | pre-existing test failure bloqueó pipeline |
| 2026-07-24 | full | `Notion/Reviews 2026/audit-full-2026-07-24.md` | — | — | — | (Sync pendiente) |
| 2026-07-18 | full | `Notion/Reviews 2026/audit-full-2026-07-18.md` | — | — | — | (Sync pendiente) |
| 2026-07-28 | backlog-validation | `Notion/Reviews 2026/process-backlog-validation-2026-07-28.md` | — | — | archivado | Validación de backlog — archivo movido a `archive/` (origen AUD-001..011) |
| 2026-07-29 | inv | `Notion/Reviews 2026/inv-001-rustsec-2026-07-29.md` | ✅ | — | COMPLETED | 3 dependencias RUSTSEC gestionadas o stale |
| 2026-07-30 | inv | `Notion/Reviews 2026/inv-024-unsafe-audit-2026-07-30.md` | ✅ | — | COMPLETED | 39 bloques unsafe auditados: 1 High + 1 Medium |
| 2026-07-27 | meta-001 | `docs/dev/research/meta-001-root-cause-analysis.md` | ✅ | — | COMPLETED | Root-cause de inconsistencias del backlog — movido a `Investigaciones/` (RCA de proceso, no audit) |
| 2026-08-03 | progreso-part1 | `Notion/Reviews 2026/process-progreso-readme-part1-2026-08-03.md` | — | — | archivado | Auditoría parcial del progreso README; movido a `archive/` |
| 2026-08-03 | progreso-part2 | `Notion/Reviews 2026/process-progreso-readme-part2-2026-08-03.md` | — | — | archivado | Auditoría parcial del progreso README; movido a `archive/` |
| 2026-08-03 | progreso-part3 | `Notion/Reviews 2026/process-progreso-readme-part3-2026-08-03.md` | — | — | archivado | Auditoría parcial del progreso README; movido a `archive/` |
| 2026-08-03 | progreso-sistema | `Notion/Reviews 2026/process-progreso-sistema-2026-08-03.md` | — | — | archivado | UX doc-hygiene; movido a `archive/` |
| 2025-07-27 | audit-full | `Notion/Reviews 2026/audit-full-2025-07-27.md` | — | — | archivado | Auditoría estática multi-agente (era `vantadb-audit-report.md`); findings consumidos 100% en P13 AUDREP 2026-08-05..08 |
| 2026-07-27 | research | `docs/dev/research/vectara-competitive-research-2026-07-27.md` | — | — | consumido | Vectara cerró self-service → gap local-first (movido a `Investigaciones` — research, no audit) |

| 2026-08-22 | integración final | `Notion/Reviews 2026/2026-08-22-auditoria-integracion-final.md` | — | 3🔴/6🟠/2🟢 | vigente | H1-H9 + descartes re-evaluated; hallazgos → Backlog MEM-50..58/BND-05 |
| 2026-08-23 | reviews por módulo (14) | `docs/dev/reviews/modulos/` | — | hallazgos por crate | vigente | sesión GOV/paralela; referenciar en próxima campaña |
| 2026-08-25-0106 | full | `Notion/Reviews 2026/audit-full-20260825-010607.md` | ❌ FAIL | 1/0/0/0/0 | superado | Phase 1 clippy gate rojo (`unused variable ns` en cli_server.rs:1302, commit abc4ec10) - pipeline abortado en Wave 1; fases 2-8 pendientes de re-run. AUD-043 derivado |
| 2026-08-25-0310 | full | `Notion/Reviews 2026/audit-full-20260825-031011.md` | ✅ PASS | 0/4/4/6/0 | superado (por audit-full-20261017-multiagent) | 9/9 fases. ISO 7.3/10 B, certify nocturno 7/7 (Recall 0.971), Security/Perf/Review/Ponytail PASS sin veto. Fix AUD-043 aplicado in-session (_ns). AUD-044..047 derivados (shim mmap write-back, IVF clones, fan-out truncation, layer.rs dedup) |
| 2026-08-25-0400 | task-files | `Notion/Reviews 2026/task-files-verification-20260825.md` | - | 0/1/4/-/- | vigente | Verificacion de 30 task files mas recientes contra codigo: 29/30 completos con commit, 1 parcial legitimo (DESKTOP-24 step manual). Hallazgo principal: bookkeeping stale en 6 headers (UX-16/MCP-33/MCP-24/FIND-04/MOD-19) |
| 2026-08-25-1422 | command-system | `Notion/Reviews 2026/command-system-audit-20260825-142232.md` | - | - | vigente | Auditoria del sistema de comandos/flujos: 8 caminos para ejecutar tareas, 2 implementaciones duplicadas de auditoria (/audit vs unified-review), 3 esquemas de ID de hallazgos, /build con puente muerto (tasks/plan.md no existe). Plan de consolidacion en 4 fases: -20% lineas, -2 comandos, -3 modos |
| 2026-08-25 | inv | `Notion/Reviews 2026/research-vantadb-server-20260825.md` | ✅ | 0/2/7/5/0 | COMPLETED | INV-vantadb-server-01: score 8.0/10; matriz qdrant/weaviate/milvus/marqo con fuentes oficiales; 14 hallazgos → P40 SRV-01..08 + wontfix H-12/H-13 |
| 2026-08-25 | inv | `Notion/Reviews 2026/research-vantadb-wasm-20260825.md` | ✅ | 0/5/9/2/2 | COMPLETED | INV-vantadb-wasm-01: score 6.8/10; matriz Orama/DuckDB-WASM/sql.js-httpvfs/vectra con fuentes oficiales; 23 hallazgos → P42 WSM-01..14 + quick wins plan + wontfix H-23 |
| 2026-08-25 | research | `Notion/Reviews 2026/research-desktop-prod-20260825.md` | ? | 0/1/3/8/3 | vigente | INV-desktop-prod: score 7.4/10; matriz Compass/RedisInsight/TablePlus + docs oficiales Tauri; 15 hallazgos -> quick wins plan (10 items) + P44 DESKTOP-40..44 + firma diferida (DEVOPS-10) |
| 2026-08-25 | inv | `Notion/Reviews 2026/research-providers-20260825.md` | ? | 0/6/1/1/2 | COMPLETED | INV-providers-01: score 4.0/10 (regresion vs 5.0 del 2026-08-23: openai no compila, E0063). Matriz LiteLLM/fastembed/chroma con fuentes oficiales; 14 hallazgos -> P45 PROV-01..12 + quick wins plan + MOD-41..45 archivadas superadas |
| 2026-08-26 | inv-synthesis | `Notion/Reviews 2026/research-bindings-synthesis-20260825.md` | 6.6 prom | - | vigente | INV-DECIDE síntesis global: 121 hallazgos/9 módulos ya materializados por módulo; sala HITL (Q1-Q9): apuesta paridad bindings c/excepciones, inaceptables providers/npm404/trazabilidad/CSP, 7 planes quickwins en ejecución (waves dirs disjuntos), PY-03 nueva, regla derivación atómica (meta.md) |
| 20260903-171641 | certify+L9 | `Notion/Reviews 2026/review-certify-20260903-171641.md` | ✅ cond | 0/0/1/5/3 | vigente | Certify sesion deps/security/e2e: L1-L6+L9 8.7/10 B; 6 findings cerrados en sesion; FIND-60 creado |
| 20260910 | full-modulos | `Notion/Reviews 2026/review-full-20260910-modulos.md` | ⚠️ cond | 1/14/51/42 | vigente | Revisión 19 módulos (vanta-review read-only, waves ×3): score ~7.4; 0 critical; 6 High + 13 medium → FIND-63..81 |
| 20260916-061609 | quick | `docs/dev/reviews/audit-quick-20260916-061609.md` | ❌ FAIL | 0/0/2/1/0 | vigente | Post-cierre campaña FIND 31/31: fmt/check/clippy/deny/machete verdes; cargo audit rojo (rustls RUSTSEC-2026-0285 + rsa RUSTSEC-2023-0071, transitivas) → FIND-96/97; lru unsound low sin fila |
| 20261017-145700 | full | `docs/dev/reviews/audit-full-20261017-multiagent.md` | ✅ PASS (post-fix) | 6/11/11/6/0 | vigente | Auditoría integral 5 dominios (Core/Storage, Services/Memory, FFI/Bindings, Packages/Manifests, Docs/API) + gates mecánicos. 34 hallazgos registrados como `FIND-345..378` en `Backlog.md` y remediados in-session |
| 20261019-033000 | remediation-plan | `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md` | ✅ PASS | 0/0/0/0/0 | vigente | Remediación integral pre-merge de 7 pasos (FIND-213, FIND-214, FIND-242, FIND-381..385, FIND-335) resolviendo memory leak en NL_POOL, mutexes en Scann, OPFS async, strict embeddings, auto-recuperación de locks huérfanos con PID check, unificación server/mcp in-process, setup wizard persistence + self-healing daemon, y config unificada de 5 capas |
| 20261019-040000 | systems-ux-analysis | `docs/dev/reports/2026-10-19-systems-and-ux-analysis.md` | ✅ PASS | 0/0/0/0/0 | vigente | Análisis integral de flujo de usuarios, instalación, experiencia de usuario y mantenimiento automatizado |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w0-harness-20261020-033416.md` | PASS CON OBS | 0/0/2/1/3 | vigente | W0 harness `.opencode/`: 0 referencias rotas entre commands/agents/skills/rules/references; deriva de conteos de skills (FIND-397) y contradiccion "git submodule" vs "repo separado" (FIND-398) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w1-core-rust-20261020-033416.md` | PASS CON OBS | 0/0/1/0/5 | vigente | W1 core Rust (271 rs): ~14 `unwrap()` reales en produccion, 0 `todo!()`, 17 enums `#[non_exhaustive]`, capa `src -> sdk` limpia; 1 bloque `unsafe` sin invariante SAFETY (FIND-396) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w2-bindings-20261020-033416.md` | PASS | 0/0/0/1/6 | vigente | W2 bindings: 0 `unsafe` en los 4 bindings de usuario, `vantadb-ffi-core` std-only sin deps, stubs `.pyi` con gate anti-drift real, ERR-028 paralelo sin fallbacks silenciosos |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w3-server-mcp-20261020-033416.md` | PASS CON OBS | 0/1/0/1/5 | vigente | W3 server + MCP: paridad OpenAPI exacta por doble via, auth fail-closed con guardas de key rotation y loopback; 2 de las 49 tools MCP inalcanzables desde la skill (FIND-408) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w4-embeddings-providers-20261020-033416.md` | FAIL | 0/2/2/1/5 | vigente | W4 embeddings/providers/integrations: licencia real de embeddinggemma-2 = Apache-2.0 (API HF) pero el manifest dice "Gemma" (FIND-382) y la descarga arrastra 5 888 MB de los que se usan 1 034 MB (FIND-383) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w5-docs-20261020-033416.md` | FAIL | 0/3/5/1/3 | vigente | W5 docs + READMEs + skills: 4 badges a workflows inexistentes (FIND-385), politica de soporte en 0.5.x con el crate en 0.9.0 (FIND-386), scores.md con 7 citas rotas (FIND-387), 100 refs stale (FIND-399) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w6-tests-20261020-033416.md` | PASS CON OBS | 0/0/2/2/5 | vigente | W6 tests: 677 `#[test]` + 186 Python, 0 `should_panic`, 0 skips sin justificar, 5 `#[ignore]` todos con causa documentada; 4 modulos de `src/` sin test (FIND-403). Suite NO ejecutada: `cargo` ausente del PATH |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w7-benchmarks-20261020-033416.md` | FAIL | 0/0/3/0/4 | vigente | W7 benchmarks + research: `canonical_p99` solo se compila en el gate de PR (FIND-394), 13 de 23 benches sin ejecutar (FIND-395), BEIR y north-star sin artefacto versionado (FIND-393); research de mercado con URL y verificacion de EmbeddingGemma 2 |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w8-tooling-20261020-033416.md` | PASS CON OBS | 0/1/3/1/6 | vigente | W8 tooling: higiene de artefactos impecable y 20 workflows con `uses:` pinneados por SHA; 11 `continue-on-error` sin `# CATEGORY:` (FIND-390), Formula de Homebrew en 0.8.0 (FIND-388), `fuzz/Cargo.lock` sin versionar (FIND-404) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w9-crosscutting-20261020-033416.md` | PASS CON OBS | 1/0/2/1/6 | vigente | W9 cross-cutting: 1 Critical - `SECURITY.md` declara cifrado en reposo que el crate publicado no tiene y que la WAL no cubre (FIND-407); ADR Gate con filtro por arbol (FIND-405) y politica de publish de AGENTS.md falsa (FIND-406) |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-w10-verificacion-20261020-033416.md` | PASS | 0/0/0/0/8 | vigente | W10 verificacion adversarial: 7 de los 8 Critical/High confirmados (licencia de embeddinggemma-2 reforzada por la API de Hugging Face); 1 refutado parcialmente y ajustado en su reporte de origen; 0 re-tickets duplicados |
| 20261020-033416 | deep | `docs/dev/reviews/audit-deep-sintesis-20261020-033416.md` | FAIL | 1/7/20/8/48 | vigente | W11 sintesis global: FAIL por compliance de licencia y exactitud de la superficie visible, no por defecto del runtime. 28 filas `FIND-382..409` en `docs/dev/Backlog.md`; ruta rapida de 1 dia neutraliza el Critical + 3 High |


Notas:
- Los rows marcados `(Sync pendiente)` necesitan que se complete su fila con datos reales (fecha estándar, QG, findings) antes de archivarse.
- `Notion/Reviews 2026/review.md` (2026-07-21): pre-unified-review legacy, ya archivado.

---
_Generated by vanta-lead / `docs/dev/reports/INDEX.md`. Mantener con las skills `unified-review` (L11b) y `progreso` (Trigger 4)._
