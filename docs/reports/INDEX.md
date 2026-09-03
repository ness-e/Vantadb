# Report Registry — docs/reports/INDEX.md

> **Registro maestro de reportes de review/audit.** Una fila por reporte.
> Columna **estado**: `vigente` (último del modo) · `superado` (reemplazado por uno más nuevo) · `archivado` (movido a `archive/`) · `—` (pendiente de sync).
>
> **Se actualiza:**
> 1. en cada `/review` y `/audit` (fase L11b de `unified-review`),
> 2. al inicio de sesión (`progreso` **Trigger 4**: sincroniza reportes nuevos/huérfanos).
>
> **Hallazgos ≥ medium** de cada reporte se derivan a `docs/Backlog.md` →
> sección `## Hallazgos pendientes de reportes` (IDs `REVIEW-NN` / `AUD-NN`).

## Index

| Fecha (YYYYMMDD-HHMMSS) | Modo | Archivo | QG | C/H/M/L/I | Estado | Resumen |
|---|---|---|---|---|---|---|
| 20260825-223924 | research | `docs/reviews/research-web-prod-20260825.md` | - | - | vigente | INV-web-01: producto web 7.2/10 vs 5 competidores (extraídos en vivo). 11 hallazgos → 7 APLICAR (plan WEB-03..09) + WEB-01/02 backlog + registro corregido (H-01) + H-09 diferido |

| 2026-08-10-1740 | eval | `docs/reports/dora.md` | — | — | vigente | P3-07 (P3): DORA flow metrics — cycle/lead time, CFR, throughput, flow table desde plan files + task files + `verify-log.jsonl` (fechas derivadas best-effort, fallback mtime) |
| 2026-08-10-2007 | eval | `docs/reports/pipeline-evals.md` | — | — | vigente | EVAL-01 (P0): harness de evals del pipeline — North Star metrics (primer intento, falsos positivos, regresión) desde `verify-log.jsonl` |
| 2026-08-08-0026 | full | `docs/reviews/audit-full-20260808-002617.md` | ? FAIL | 0/5/8/9/0 | superado | INV-024 sin tests + prune duplica select_neighbors; clippy gate rojo (5 pre-existentes). AUD-012..021 derivados. (superado por audit-full-20260812-231204) |
| 2026-08-12-2312 | full | `docs/reviews/audit-full-20260812-231204.md` | ✅ PASS | 0/5/5/10/3 | superado (por audit-full-20260825-031011) | CI-07 pinning vérifié 17/17 + P2-7 VERIFIED ROI (serde_json fuera del write path); VETO falso reversado (`.env.tokens` gitignored); certify nocturno 7/7 (Recall 0.971). AUD-022..031 derivados |
| 2026-08-05-2025 | certify | `docs/reviews/review-certify-2026-08-05-2025.md` | ✅ PASS | 0/0/2/8/7 | vigente | Certify pre-push 9.4/10 (A). Recomendaciones: mitigar npm-audit web (6 vulns), migrar put_batch a keyword |
| 2026-08-22-2008 | full | `docs/reviews/review-full-20260822-200850.md` | ❌ FAIL | 0/2/8/12/9 | vigente | Full 9 fases paralelo, 7.9/10 (B). QG falla: nextest roto (filtro stale) + OOM cargo test workspace + h2 RUSTSEC (deny advisories FAIL). Bug real cache_warmer latch. Backlog sync completo: REVIEW-06..20 derivados |
| 2026-08-05-1545 | full | `docs/reviews/review-full-2026-08-05-t1545.md` | ❌ FAIL | — | vigente | Full ISO 7.1/10 (B). (Sync pendiente) |
| 2026-07-27-0309 | full | `docs/reviews/review-full-2026-07-27-0309.md` | — | — | superado | Full previo; findings consumidos en P14 REVIEW del backlog |
| 2026-08-04-1745 | full | `docs/reviews/archive/audit-full-2026-08-04T174544.md` | ❌ FAIL | 1 C | superado | &#8672; AUDIT-01: UAF memory-safety en Python SDK NumPy path; benchmark nocturno crashea (superado por audit-full-20260808-002617) |
| 2026-07-24-1751 | full | `docs/reviews/archive/audit-full-2026-07-24T1751Z.md` | ❌ FAIL | 1 C | superado | pre-existing test failure bloqueó pipeline |
| 2026-07-24 | full | `docs/reviews/archive/audit-full-2026-07-24.md` | — | — | — | (Sync pendiente) |
| 2026-07-18 | full | `docs/reviews/archive/audit-full-2026-07-18.md` | — | — | — | (Sync pendiente) |
| 2026-07-28 | backlog-validation | `docs/reviews/archive/process-backlog-validation-2026-07-28.md` | — | — | archivado | Validación de backlog — archivo movido a `archive/` (origen AUD-001..011) |
| 2026-07-29 | inv | `docs/reviews/archive/inv-001-rustsec-2026-07-29.md` | ✅ | — | COMPLETED | 3 dependencias RUSTSEC gestionadas o stale |
| 2026-07-30 | inv | `docs/reviews/archive/inv-024-unsafe-audit-2026-07-30.md` | ✅ | — | COMPLETED | 39 bloques unsafe auditados: 1 High + 1 Medium |
| 2026-07-27 | meta-001 | `docs/research/meta-001-root-cause-analysis.md` | ✅ | — | COMPLETED | Root-cause de inconsistencias del backlog — movido a `Investigaciones/` (RCA de proceso, no audit) |
| 2026-08-03 | progreso-part1 | `docs/reviews/archive/process-progreso-readme-part1-2026-08-03.md` | — | — | archivado | Auditoría parcial del progreso README; movido a `archive/` |
| 2026-08-03 | progreso-part2 | `docs/reviews/archive/process-progreso-readme-part2-2026-08-03.md` | — | — | archivado | Auditoría parcial del progreso README; movido a `archive/` |
| 2026-08-03 | progreso-part3 | `docs/reviews/archive/process-progreso-readme-part3-2026-08-03.md` | — | — | archivado | Auditoría parcial del progreso README; movido a `archive/` |
| 2026-08-03 | progreso-sistema | `docs/reviews/archive/process-progreso-sistema-2026-08-03.md` | — | — | archivado | UX doc-hygiene; movido a `archive/` |
| 2025-07-27 | audit-full | `docs/reviews/archive/audit-full-2025-07-27.md` | — | — | archivado | Auditoría estática multi-agente (era `vantadb-audit-report.md`); findings consumidos 100% en P13 AUDREP 2026-08-05..08 |
| 2026-07-27 | research | `docs/research/vectara-competitive-research-2026-07-27.md` | — | — | consumido | Vectara cerró self-service → gap local-first (movido a `Investigaciones` — research, no audit) |

Notas:
- Los rows marcados `(Sync pendiente)` necesitan que se complete su fila con datos reales (fecha estándar, QG, findings) antes de archivarse.
- `docs/reviews/archive/review.md` (2026-07-21): pre-unified-review legacy, ya archivado.

---
_Generated by vanta-lead / `docs/reports/INDEX.md`. Mantener con las skills `unified-review` (L11b) y `progreso` (Trigger 4)._| 2026-08-22 | integración final | docs/reviews/2026-08-22-auditoria-integracion-final.md | 3🔴/6🟠/2🟢 | H1-H9 + descartes re-evaluated | hallazgos → Backlog MEM-50..58/BND-05 |
| 2026-08-23 | reviews por módulo (14) | docs/reviews/modulos/ | hallazgos por crate | sesión GOV/paralela | referenciar en próxima campaña |
| 2026-08-25-0106 | full | `docs/reviews/audit-full-20260825-010607.md` | ❌ FAIL | 1/0/0/0/0 | superado | Phase 1 clippy gate rojo (`unused variable ns` en cli_server.rs:1302, commit abc4ec10) - pipeline abortado en Wave 1; fases 2-8 pendientes de re-run. AUD-043 derivado |
| 2026-08-25-0310 | full | `docs/reviews/audit-full-20260825-031011.md` | ✅ PASS | 0/4/4/6/0 | vigente | 9/9 fases. ISO 7.3/10 B, certify nocturno 7/7 (Recall 0.971), Security/Perf/Review/Ponytail PASS sin veto. Fix AUD-043 aplicado in-session (_ns). AUD-044..047 derivados (shim mmap write-back, IVF clones, fan-out truncation, layer.rs dedup) |
| 2026-08-25-0400 | task-files | `docs/reviews/task-files-verification-20260825.md` | - | 0/1/4/-/- | vigente | Verificacion de 30 task files mas recientes contra codigo: 29/30 completos con commit, 1 parcial legitimo (DESKTOP-24 step manual). Hallazgo principal: bookkeeping stale en 6 headers (UX-16/MCP-33/MCP-24/FIND-04/MOD-19) |
| 2026-08-25-1422 | command-system | `docs/reviews/command-system-audit-20260825-142232.md` | - | - | vigente | Auditoria del sistema de comandos/flujos: 8 caminos para ejecutar tareas, 2 implementaciones duplicadas de auditoria (/audit vs unified-review), 3 esquemas de ID de hallazgos, /build con puente muerto (tasks/plan.md no existe). Plan de consolidacion en 4 fases: -20% lineas, -2 comandos, -3 modos |
| 2026-08-25 | inv | `docs/reviews/research-vantadb-server-20260825.md` | ✅ | 0/2/7/5/0 | COMPLETED | INV-vantadb-server-01: score 8.0/10; matriz qdrant/weaviate/milvus/marqo con fuentes oficiales; 14 hallazgos → P40 SRV-01..08 + wontfix H-12/H-13 |
| 2026-08-25 | inv | `docs/reviews/research-vantadb-wasm-20260825.md` | ✅ | 0/5/9/2/2 | COMPLETED | INV-vantadb-wasm-01: score 6.8/10; matriz Orama/DuckDB-WASM/sql.js-httpvfs/vectra con fuentes oficiales; 23 hallazgos → P42 WSM-01..14 + quick wins plan + wontfix H-23 |
| 2026-08-25 | research | `docs/reviews/research-desktop-prod-20260825.md` | ? | 0/1/3/8/3 | vigente | INV-desktop-prod: score 7.4/10; matriz Compass/RedisInsight/TablePlus + docs oficiales Tauri; 15 hallazgos -> quick wins plan (10 items) + P44 DESKTOP-40..44 + firma diferida (DEVOPS-10) |
| 2026-08-25 | inv | `docs/reviews/research-providers-20260825.md` | ? | 0/6/1/1/2 | COMPLETED | INV-providers-01: score 4.0/10 (regresion vs 5.0 del 2026-08-23: openai no compila, E0063). Matriz LiteLLM/fastembed/chroma con fuentes oficiales; 14 hallazgos -> P45 PROV-01..12 + quick wins plan + MOD-41..45 archivadas superadas |
| 2026-08-26 | inv-synthesis | `docs/reviews/research-bindings-synthesis-20260825.md` | 6.6 prom | - | vigente | INV-DECIDE síntesis global: 121 hallazgos/9 módulos ya materializados por módulo; sala HITL (Q1-Q9): apuesta paridad bindings c/excepciones, inaceptables providers/npm404/trazabilidad/CSP, 7 planes quickwins en ejecución (waves dirs disjuntos), PY-03 nueva, regla derivación atómica (meta.md) |
| 20260903-171641 | certify+L9 | `docs/reviews/review-certify-20260903-171641.md` | ✅ cond | 0/0/1/5/3 | vigente | Certify sesion deps/security/e2e: L1-L6+L9 8.7/10 B; 6 findings cerrados en sesion; FIND-60 creado |
