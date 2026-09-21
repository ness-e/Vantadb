# Plan: 2026-08-27 Backlog Pipeline

> Fuente: `docs/Backlog.md` (triage 2026-08-27)
> Creado: 2026-08-27
> Estado: ⬜ PENDING

---

## Triage Summary

| Gate | Count | Items |
|------|-------|-------|
| ✅ **DO** | 68 | P5(2), P6(5), P25(11), P27(38), GOV(30), P36(6), P38(17), Hallazgos(31), P28(2), P32(5), P26(1) |
| 🟡 **DEFER** | 7 | P6(3: CLD), P8(1), P33(4), P34(8) |
| ❌ **SKIP** | 17 | P23(6), P24(10), P9(1) |
| 🔴 **BLOQUEADO** | 0 | — |

---

## Ordered Tasks (DO only)

### Wave 0 — Foundation (no deps, can run parallel)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 1 | DISC-01 | Configurar Discord (reaction roles, autorole, logging, welcome DM) | `vanta-docs` | Discord server con roles/autorole funcionando + docs/discord/todo.md actualizado | 🟡 2-3d | 🟢 |
| 2 | MKT-04 | Publicar 3 drafts Reddit (r/rust, r/ML, r/LocalLLaMA) | `vanta-docs` | 3 posts publicados con claims verificados (ver `docs/strategy/REDDIT_POSTS.md`) | 🟢 2-4h | 🟠 |
| 3 | AGT-01 | Fixes AGENTS.md pendientes (commits + verificación stats CodeGraph) | `vanta-lead` | Diffs commiteados + `codegraph_status` verde + refs file:line de deuda P2 actualizadas | 🟢 4h | 🟠 |
| 4 | AGT-02 | Limpieza opencode-loop corrupt/tmp | `vanta-lead` | `.opencode/task-system/` limpio + convención checkpoints paralelos documentada | 🟢 2h | 🟠 |
| 5 | AGT-03 | Script anti-drift de refs AGENTS.md | `vanta-lead` | Script ejecutable + test de regresión pasando | 🟢 2h | 🟠 |

### Wave 1 — Core Engine Fixes (depende de Wave 0)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 6 | FIND-34 | Ciclo WAL Writer (4 nodos: open↔open_with_buffer↔recover_valid_records↔quarantine_corrupt_tail) | `vanta-worker` | `codegraph_explore` muestra ciclo roto + tests `recover_valid_records`/`quarantine_corrupt_tail` pasando | 🟠 | 🔴 |
| 7 | FIND-35 | Ciclo StorageEngine get/prefetch (2 nodos) | `vanta-worker` | `codegraph_explore` muestra ciclo roto o justificación documentada en código | 🟠 | 🔴 |
| 8 | FIND-36 | Cross-crate NativeConnection ↔ RocksDbBackend (3 ciclos get/put/delete) | `vanta-arch` | Backend no llama a frontend; dependencia invertida verificada | 🟠 | 🔴 |
| 9 | FIND-37 | `query_sparse.as_ref().unwrap()` ×6 sin validar | `vanta-worker` | 6 sitios en `src/sdk/search/mod.rs` validan `is_some()` o propagan `Option` | 🟠 | 🔴 |
| 10 | FIND-38 | Ciclo Serialization (5 nodos) | `vanta-worker` | Helpers `memory_record_from_node*` + `VantaEmbedded.get` consolidados | 🟡 | 🟡 |
| 11 | FIND-39 | `ScalarIndex.remove` sin test | `vanta-worker` | Test dedicado en `src/storage/engine/tests/` pasando | 🟢 | 🟡 |
| 12 | CORE-01 | Persistencia on-disk de vectores Binary (y no-F32) en vstore | `vanta-arch` | ADR de formato + escritura/lectura en `write_node_to_vstore`/`get()`/`rebuild` + migración | 🟡 | 🟡 |
| 13 | CORE-02 | Integrar PITR al engine (restaurar wal_archiver.rs + wiring) | `vanta-arch` | `wal_archiver.rs` restaurado + hooked a StorageEngine/SDK + tests point-in-time | 🔴 | 🔴 |

### Wave 2 — Memory Engine (depende de Wave 1 partially)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 14 | MEM-01..38 | Vanta Memory Engine TDAM (orden F1-F7) | `vanta-worker` | Plan `docs/plans/2026-08-18-vanta-memory.md` ejecutado; 38 tareas → progreso | 🔴 8-12s | 🔴 |
| 15 | FIND-44 | Sin ADRs registrados — crear ADR inicial | `vanta-arch` | ADR creado en `docs/architecture/adr/` con PURPOSE, STACK, ARCHITECTURE, PATTERNS, TRADEOFFS, PHILOSOPHY | 🟢 | 🔴 |

### Wave 3 — MCP/HTTP Exposure (puede iniciar en paralelo con Wave 1-2)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 16 | MCP-16..26 | Exposición MCP/HTTP gaps del SDK | `vanta-worker` | 11 tools MCP wrapper sobre métodos públicos existentes (put_batch, versions, supersede, purge_expired, compact_wal, delete_by_filter, search_with_method, search_multi, rebuild/recovery) | 🟡 2-3s | 🟡 |
| 17 | FIND-40 | Drift docs/api vs firmas reales | `vanta-docs` | `cargo semver-checks` PASS + review manual `EMBEDDED_SDK.md`, `PYTHON_SDK.md`, `HTTP_API.md` actualizadas | 🟡 | 🟡 |
| 18 | FIND-46 | Doc drift gate semver-checks | `vanta-docs` | `cargo semver-checks` integrado en CI + doc actualizada | 🟢 | 🟡 |

### Wave 4 — Governance & Research (paralelo con Wave 3)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 19 | GOV (30 tareas) | Gobernanza Documental Wave B (Show HN blocker) | `vanta-docs` | Plan `docs/plans/2026-08-22-doc-governance-plan.md` ejecutado; 29✅ 1⬛ | ~6d | 🔴 |
| 20 | P38 (17 items) | Research huérfanas → tarea (RES-01..15, DEC-01/02) | `vanta-research` + `vanta-worker` | Cada RES/DEC con task file + evidencia contra código | ~2-3s | 🟡 |
| 21 | RES-01 | ACID Phase 4a: WAL v2 con `WalRecord::Prepare` | `vanta-arch` | WAL_FORMAT_VERSION=2 + Prepare record + acceptance criteria por fase | 🟡 | 🔴 |
| 22 | RES-02 | Separar binario `chaos_failpoints` + `crash_kill_recovery.rs` | `vanta-chaos` | Binario separado + test kill real a mitad de escritura | 🟡 | 🔴 |

### Wave 5 — Module Reviews & Architecture (depende de Wave 1-2)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 23 | FIND-41 | 6 clusters `src` fragmentados (cohesión 0.59-0.71) | `vanta-arch` | Leiden clusters consolidados o fronteras documentadas + cohesión ≥0.8 | 🟡 | 🟡 |
| 24 | FIND-42 / FIND-45 | Boundary `src → skills` (173 calls / impeccable leakage) | `vanta-arch` | Dependencia core→skills invertida o aislada; decisión documentada | 🟡 | 🟡 |
| 25 | FIND-43 | Ciclo CacheWarmer (3 nodos) | `vanta-worker` | Builder pattern aplanado (new → with_config → with_config_and_cap sin recursión) | 🟢 | 🟡 |
| 26 | MOD-05 | Deprecar `InMemoryEngine` hacia StorageEngine in-memory | `vanta-worker` | `InMemoryEngine` removido ~850 líneas; tests pasando | 🟢 | 🟢 |
| 27 | MOD-15 | Nits server: middleware.rs re-export, feature sysinfo vacía, main.rs, ServerState ctor | `vanta-worker` | 4 fixes aplicados + tests pasando | 🟢 | 🟢 |
| 28 | MOD-22 | Tipos grafo ficticios TS (`GraphBfsResult`) | `vanta-worker` | Tipos TS = wire format real; test afirme shape | 🟡 | 🔴 |
| 29 | MOD-23 | NativeVantaDB._native captura solo throws síncronos | `vanta-worker` | Rechazos async envueltos en `VantaError` | 🟡 | 🟠 |
| 30 | MOD-24 | Nits TS agrupados (distance/score JSDoc, type-lie guard, duplicación, ejemplos) | `vanta-worker` | 6 nits resueltos + `cargo clippy`/`npm run check` clean | 🟢 | 🟡 |

### Wave 6 — Launch Campaign & Polish (paralelo final)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 31 | MKT-18f | Publicar 5 adapters PyPI (langchain, llama-index, mem0, crewai, dspy) | `vanta-lead` | 5 paquetes en PyPI + PRs upstream + `release-wheels-60.yml` ARM64 | 🟡 1-2d | 🔴 |
| 32 | MKT-18h | Wheels ARM64 Linux + SHA reales Homebrew | `vanta-lead` | `release-wheels-60.yml` x86_64+aarch64 + `Formula/vantadb.rb` SHA reales | 🟡 1d | 🔴 |
| 33 | MKT-18i | Docker Compose multi-servicio (Ollama + VantaDB + AnythingLLM) | `vanta-worker` | `docker-compose.yml` con 3 servicios funcionando | 🟢 2-4h | 🟡 |
| 34 | CLD-01 | VantaDB Cloud beta on Fly.io | `vanta-lead` | Infra Fly.io desplegada + checkbox `GO_TO_MARKET.md:420` | 🟠 1-2s | 🔵 |
| 35 | CLD-02 | Pitch deck + one-pager | `vanta-docs` | Archivos `*pitch*`/`*deck*` + checkbox `GO_TO_MARKET.md:408` | 🟡 3-5d | 🔵 |
| 36 | CLD-04 | Case study #1 (enterprise pilot) | `vanta-lead` | Pilot real + case study publicado + checkbox `GO_TO_MARKET.md:409` | 🟠 1s | 🔵 |
| 37 | BLOG-CTA | CTAs + metadata blogs + posts 6-7 (Ollama+VantaDB, Claude Code MCP) | `vanta-docs` | 7 posts con CTAs fuertes + metadata correcta + posts 6-7 redactados | 🟡 3-5d | 🟠 |
| 38 | DISC-02 | Discord: AutoMod, stickers/emojis, forums seed | `vanta-docs` | AutoMod + stickers/emojis + 9 forums activos | 🟢 4-6h | 🟢 |

### Wave 7 — Remaining Hallazgos (Reportes ≥Medium)

| # | Task ID | Título | Ruta | Contrato verificable | Esfuerzo | Prio |
|---|---------|--------|------|---------------------|----------|------|
| 39 | AUD-042 | Upgrade tantivy ≥0.18 (allowlist RUSTSEC-2026-0253) | `vanta-worker` | `tantivy ≥0.27.0` publicado + `lru 0.18.2` + allowlist removida | 🟡 | 🟡 |
| 40 | AUD-043 | Fix clippy `unused variable: ns` → `_ns` | `vanta-worker` | `just verify` PASS + `src/cli_server.rs:1302` fix | 🟢 | 🔴 |
| 41 | AUD-045 | Clones vector per-candidate IVF hot path | `vanta-tuner` | Baseline `canonical_p99` medido + A/B borrowed/slice si mejora p99 | 🟡 | 🟡 |
| 42 | REVIEW-07 | `.config/nextest.toml` profile audit: filtro stale | `vanta-worker` | `cargo nextest list` sin parse failure + nextest PASS | 🟢 | 🟡 |
| 43 | REVIEW-10 | God-file `cli_server.rs` ~3800-4141 líneas | `vanta-arch` | Split bajo `src/server/` + features congeladas | 🟠 | 🔴 |
| 44 | REVIEW-12 | `api.rs` ~2300-2500 líneas → refactor por dominio | `vanta-arch` | Refactor aditivo (memory/search/namespaces/admin) + re-export | 🟡 | 🟡 |
| 45 | FIND-22 | Formalizar 3 exclusiones fast gate en CI_POLICY.md | `vanta-docs` | `CI_POLICY.md` con 3 entradas + tags flaky | 🟢 | 🟡 |
| 46 | FIND-24 | `VentaEmbedded::list` ventana grande lento (cursor cross-namespace) | `vanta-worker` | Cursor server-side + perf `indexed_ids_by_namespace`/`get_many` | 🟠 | 🔴 |
| 47 | FIND-33 | Snapshot filesystem no captura backend KV | `vanta-arch` | Rediseño layout snapshot (copiar backend o mover bajo data_dir) | 🟠 | 🔴 |
| 48 | FIND-47 | `handle_tools_call` complejidad 295 | `vanta-tuner` | Match refactorizado si crece; baseline p99 documentado | 🟢 | 🟢 |

---

## DEFER (no en plan — se revisan tras waves 0-6)

| Task ID | Título | Motivo |
|---------|--------|--------|
| CLD-01/02/04 | Cloud/Deck/Case study | Infra/venta — requieren decisiones de negocio |
| BIZ-01b | Enterprise features | Post-launch real |
| FIND-11,17,20,21 | DX SDKs / Desktop UX | No bloqueante, smoke E2E PASS |
| UX-02..19 | Studio UX | Fase 4 completada, smoke PASS |

---

## SKIP (no en plan)

| Phase | Items | Motivo |
|-------|-------|--------|
| P23 VantaDB Pro | PRO-01..06 | Futuro, repo separado |
| P24 I+D Futura | FUT-02..11 | Roadmap v3.0+ |
| P9 Old Docs | OLD-01 PGWire | Roadmap |

---

## BLOQUEADO (0 — ninguno)

---

## Checkpoints

| Checkpoint | Criterio |
|------------|----------|
| **CP0** (Wave 0 done) | DISC-01, MKT-04, AGT-01..03 ✅ |
| **CP1** (Waves 1-2 done) | FIND-34..39, CORE-01/02, MEM-01..38, FIND-44 ✅ |
| **CP2** (Waves 3-4 done) | MCP-16..26, FIND-40/46, GOV, P38, RES-01/02 ✅ |
| **CP3** (Waves 5-6 done) | FIND-41..43, MOD-*, MKT-18f/h/i, CLD-*, BLOG-CTA, DISC-02 ✅ |
| **CP4** (Wave 7 done) | Hallazgos reportes restantes ✅ |

---

## Próximo paso

```
/pipeline run
```

O para ejecutar la primera tarea:
```
/pipeline task DISC-01
```

=== RECITATION MCP-36 ===
Campaign ID: b1e58305-e721-49b9-bf77-1482eedc3742
Objetivo activo: MCP-36 Protocolo moderno: negociar protocolVersion 2025-06-18 + structured output
Estado: completed
Última acción: S1-S3 ✅ + verify full (fmt, clippy mcp, nextest audit 62, docs coverage 0 gaps) + commit ca4eef6d
Resultado: OK
Próxima acción: ninguno — tarea cerrada
Contrato: verificacion: cargo test -p vantadb-mcp --lib 11/11 ✅ + grep -n "2025-06-18" initialize.rs 3 hits ✅ + cargo test -p vantadb-mcp --test mcp_tests 75/75 ✅ + cargo nextest audit 62 passed ✅
 evidencia: claim: negotiate 2025-06-18 + structured output, evidencia: vantadb-mcp/src/handlers/initialize.rs:7 + vantadb-mcp/src/validation.rs:structured_text_content, confianza: alta
 artefactos: vantadb-mcp/src/handlers/initialize.rs, vantadb-mcp/src/server.rs, vantadb-mcp/src/validation.rs, vantadb-mcp/src/handlers/tools.rs (outputSchema), vantadb-mcp/tests/mcp_tests.rs (negotiation + structured tests), test-mcp.py (2025-06-18)
 invariantes: No tocar wal/vector/storage; no Roots/Sampling/Logging (deprecados 2026-07-28)
 deuda: ninguna
 queda_pendiente: none
Próxima tarea si completa: MCP-37
=== END RECITATION ===

=== RECITATION FIND-39 ===
Campaign ID: 82f1d232-8914-4aab-b2f6-21492c545a76
Objetivo activo: FIND-39 — ScalarIndex.remove sin test
Estado: completed
Última acción: S1-S3 ✅ + verify full + commits 59f5cdcb/d492deec + progreso
Resultado: OK
Próxima acción: ninguno — tarea cerrada
Contrato: verificacion: cargo nextest run -p vantadb scalar_index --profile audit 15 passed + rg 2 fns + list +1 + fmt + clippy + docs 0 gaps ✅; evidencia: claim: test_scalar_remove verde, evidencia: cargo nextest run -p vantadb scalar_index --profile audit, confianza: alta; artefactos: src/storage/engine/tests/scalar_index.rs, src/storage/engine/tests/mod.rs; invariantes: no tocar wal/vector/storage; deuda: ninguna; queda_pendiente: none
Próxima tarea si completa: MCP-37
=== END RECITATION ===
