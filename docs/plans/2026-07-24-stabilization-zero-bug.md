# Plan de Ejecución: Stabilization — Zero-Bug Policy

> **Inicio:** 2026-07-24
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/Backlog.md` (TIER 0-5)
> **Directiva:** `docs/plans/PROMPT-MAESTRO-FREEZE.md` — Feature freeze. Solo bugs y estabilización.
> **Contexto:** DRV-027 completado (lib.rs God module → 4 archivos). 129 items abiertos, plan apunta a los 15 que bloquean o rompen algo.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 15 |
| 🟡 DEFER | 18 |
| ❌ SKIP | 65 |
| 🔴 BLOQUEADO | 0 |

## Tasks

### Phase 0: Release Blockers (2 quick tag pushes)

#### Task 1: INT-01 — LangChain adapter → PyPI
- **Esfuerzo:** 🟢 5min
- **Prioridad:** 🔴
- **Archivos clave:** GitHub → tag `adapters-v0.3.0` en `integrations/langchain/`
- **Gate Justificación:** Release blocker. Código listo v0.3.0, CI configurado, 5/5 tests pasan. Solo falta pushear tag.
- **Gate Result:** ✅ DO
- **Contrato:** Tag pusheado → `release-adapters-62.yml` dispara publicación automática
- **Task file:** `skills/campaign-executor/tasks/INT-01.md`
- **Estado:** ⬜ PENDING

#### Task 2: INT-02 — LlamaIndex adapter → PyPI
- **Esfuerzo:** 🟢 5min
- **Prioridad:** 🔴
- **Archivos clave:** GitHub → tag `adapters-v0.3.0` en `integrations/llamaindex/`
- **Gate Justificación:** Release blocker. Mismo que INT-01, mismo tag.
- **Gate Result:** ✅ DO
- **Contrato:** Tag pusheado → CI publica automágicamente
- **Task file:** `skills/campaign-executor/tasks/INT-02.md`
- **Estado:** ⬜ PENDING

### Phase 1: Critical Bugs (2-3d)

#### Task 3: WEB-02 — Fix false claims on landing page
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴
- **Archivos clave:** `web/src/` — hero, feature sections, benchmark claims
- **Gate Justificación:** Legal/product risk. Claims falsos (benchmark 50x vs 40x real, "SQL support", "auto-embeddings", "cloud tiers"). Bloquea Show HN.
- **Gate Result:** ✅ DO
- **Contrato:** Landing page claims verificados contra código real. Benchmarks corregidos.
- **Task file:** `skills/campaign-executor/tasks/WEB-02.md`
- **Estado:** ⬜ PENDING

#### Task 4: VFY-001 — TS SDK catch {} silences errors
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/vantadb.ts:176,215,249`
- **Gate Justificación:** Bug real. 4+ bloques catch vacíos tragan errores WASM. El usuario no sabe si una operación falló.
- **Gate Result:** ✅ DO
- **Contrato:** `catch {}` reemplazados por error logging o re-throw en `vantadb.ts`. `npx tsc --noEmit` pasa.
- **Task file:** `skills/campaign-executor/tasks/VFY-001.md`
- **Estado:** ⬜ PENDING

#### Task 5: VFY-012 — musllinux target gap
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟢
- **Archivos clave:** CI config, maturin build matrix
- **Gate Justificación:** Bug real. Algunos targets Linux sin wheel → usuarios Alpine/musl no pueden instalar.
- **Gate Result:** ✅ DO
- **Contrato:** Alpine Linux wheels generados en CI. `pip install vantadb-py` funciona en Alpine 3.19+.
- **Task file:** `skills/campaign-executor/tasks/VFY-012.md`
- **Estado:** ⬜ PENDING

#### Task 6: DRV-035 — TS SDK metadata type mismatch
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/__tests__/*.test.ts`, `vantadb-ts/src/types.ts:1-10`
- **Gate Justificación:** Bug dormido. Tests usan metadata formato shorthand `{ source: { String: "test" } }` vs tipo real `{ type: "String", value: "test" }`. Metadata no se serializa correctamente via WASM bridge.
- **Gate Result:** ✅ DO
- **Contrato:** Tests actualizados a formato correcto. `tsc --noEmit` pasa en todos los archivos.
- **Task file:** `skills/campaign-executor/tasks/DRV-035.md`
- **Estado:** ⬜ PENDING

#### Task 7: DRV-050 — Fix LISP injection vector in inject_context
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1154-1187`
- **Gate Justificación:** Security issue. String interpolation en query LISP sin escapar paréntesis/newlines/metacaracteres. Potencial injection si content no es confiable.
- **Gate Result:** ✅ DO
- **Contrato:** `inject_context` usa escaping completo para metacaracteres LISP o query parameterized. Test de injection pasa. `cargo check` ✅.
- **Task file:** `skills/campaign-executor/tasks/DRV-050.md`
- **Estado:** ⬜ PENDING

#### Task 8: DRV-097 — Fix count_documents truncates at 100
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `integrations/haystack/vantadb_haystack/*` (post-migration Python code)
- **Gate Justificación:** Bug real. `count_documents()` usa `Default::default()` como `VantaMemoryListOptions` con `limit: Some(100)`. Namespace con >100 docs → count reporta 100.
- **Gate Result:** ✅ DO
- **Contrato:** `count_documents()` pagina todos los resultados o usa API count nativa. Tests pasan para namespaces de 200+ docs.
- **Task file:** `skills/campaign-executor/tasks/DRV-097.md`
- **Estado:** ⬜ PENDING

#### Task 9: DRV-134 — Fix NbAccordion keyboard navigation
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `web/src/components/nb/NbAccordion.tsx`
- **Gate Justificación:** WCAG violation. Sin keyboard navigation (Enter/Space/ArrowKeys), sin focus management, sin aria-expandido dinámico. Bloquea accesibilidad.
- **Gate Result:** ✅ DO
- **Contrato:** NbAccordion navegable por teclado. `role="button"`, `aria-expanded`, focus management implementados.
- **Task file:** `skills/campaign-executor/tasks/DRV-134.md`
- **Estado:** ⬜ PENDING

### Phase 2: CI & Dead Code Cleanup (1-2d)

#### Task 10: DRV-118 — Add Windows builds to CI release
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `.github/workflows/release.yml`
- **Gate Justificación:** Release blocker. Solo Linux+macOS en release matrix. Sin Windows binaries → no adopción Windows.
- **Gate Result:** ✅ DO
- **Contrato:** Windows MSVC build + test en release.yml. Binarios .exe publicados en release assets.
- **Task file:** `skills/campaign-executor/tasks/DRV-118.md`
- **Estado:** ⬜ PENDING

#### Task 11: DRV-022 — Cleanup governance/ dead code
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `src/governance/` (4 módulos: admission, conflict, consistency, worker)
- **Gate Justificación:** 1235L de código gated tras feature `governance` sin consumidores. Feature nunca activada. Depende de `sync_ext` que hace compilación inviable incluso si se activara.
- **Gate Result:** ✅ DO
- **Contrato:** Feature `governance` removida de Cargo.toml, `src/governance/` eliminado. `cargo check --workspace` ✅.
- **Task file:** `skills/campaign-executor/tasks/DRV-022.md`
- **Estado:** ⬜ PENDING

#### Task 12: DRV-024 — Remove dead memory_governor.rs
- **Esfuerzo:** 🟢 15min
- **Prioridad:** ℹ️
- **Archivos clave:** `src/memory_governor.rs`
- **Gate Justificación:** `#![allow(dead_code)]` en todo el archivo. `pub(crate)` pero nada lo invoca. Cleanup rápido.
- **Gate Result:** ✅ DO
- **Contrato:** `memory_governor.rs` removido. `cargo check --workspace` ✅.
- **Task file:** `skills/campaign-executor/tasks/DRV-024.md`
- **Estado:** ⬜ PENDING

#### Task 13: DRV-129 — Cleanup disconnected enterprise crate
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-enterprise/` (267L, 96% placeholder)
- **Gate Justificación:** Crate disconnected: no importado por ningún otro crate, 96% código placeholder. O integrar o eliminar.
- **Gate Result:** ✅ DO
- **Contrato:** `vantadb-enterprise` integrado con el crate principal o archivado. `cargo check --workspace` ✅.
- **Task file:** `skills/campaign-executor/tasks/DRV-129.md`
- **Estado:** ⬜ PENDING

#### Task 14: REV-014 — Cleanup stale dependabot branches
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `origin/dependabot/*` (24 branches stale)
- **Gate Justificación:** Ruido en git branch list. Auto-delete no configurado. Limpieza única.
- **Gate Result:** ✅ DO
- **Contrato:** 24 stale dependabot branches eliminadas. Configurar auto-delete en `dependabot.yml`.
- **Task file:** `skills/campaign-executor/tasks/REV-014.md`
- **Estado:** ⬜ PENDING

### Phase 3: Content (1-2d)

#### Task 15: OLD-06 — Publish 3 blog posts
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** Old docs batch 13/20 — posts ya escritos
- **Gate Justificación:** Contenido completo sin publicar: `how_hybrid_search_works`, `sqlite_for_ai_agents`, `why_i_built_vantadb`. Tráfico orgánico + credibilidad técnica + contenido para HN launch.
- **Gate Result:** ✅ DO
- **Contrato:** 3 posts publicados en blog/Medium/Dev.to. Linkeados desde README.
- **Task file:** `skills/campaign-executor/tasks/OLD-06.md`
- **Estado:** ⬜ PENDING

---

## 🟡 DEFER (18 items)

| ID | Razón |
|----|-------|
| SEC-14 | Evaluar migrar bincode→postcard/rkyv — No urgente, crate funcional |
| WEB-03 | Async WAL batching fsyncs — Optimización, no bug |
| WEB-04 | Storage format versioning — Feature futura, no urgente |
| DEVOPS-14 | Composite action Rust setup — Nice to have |
| DEVOPS-15 | Mover features heavies — Optimización compilación |
| TEST-11 | Frontend tests — No bloquea release |
| TEST-12 | Fuzzing regression suite — Postergable |
| DOC-20 | mdBook adoption — Nice to have |
| VFY-003 | reindex_hnsw OOM risk — Potencial, no reportado |
| VFY-004 | flat.rs O(n²) filter — Optimización |
| VFY-006 | add_node lock contention — Optimización |
| VFY-008 | WAL fsync por escritura — Optimización |
| DRV-013 | ShardedWal sin unit tests — Test gap |
| DRV-017 | search.rs sin tests — Test gap |
| DRV-048 | JSON-RPC version check — Spec compliance |
| DRV-080 | retrieve_memory distance raw — DX menor |
| DRV-121 | Planner AST — Feature futura |
| DRV-125 | Miri tests — Postergable |

## ❌ SKIP (65 items)

Incluye: todos los DRV ya ✅, todos los REV ya ✅, features futuras (COMP-* 30 items, NUEVO-11→21, WAL encryption, index types, auto-embedding, macOS signing, etc.), debt documentada (DRV-014, DRV-021, DRV-028, DRV-029, DRV-030, DRV-034, DRV-036, DRV-037, DRV-038, DRV-039, DRV-041, DRV-042, DRV-045, DRV-051, DRV-054, DRV-055, DRV-060, DRV-061, DRV-064, DRV-066, DRV-067, DRV-072, DRV-073, DRV-075, DRV-076, DRV-077, DRV-078, DRV-081, DRV-082, DRV-083, DRV-084, DRV-088, DRV-089, DRV-090, DRV-093, DRV-094, DRV-095, DRV-100, DRV-101, DRV-108, DRV-113, DRV-114, DRV-124, DRV-127, DRV-130, DRV-131, DRV-136), REV-012/013 dokumentados para monitoreo.

---

## Dependencias

| Task | Depende de |
|------|-----------|
| Todas Phase 0 | Ninguna |
| Phase 1 | Ninguna entre sí (independientes) |
| Phase 2 | Ninguna entre sí (independientes) |
| Phase 3 | Ninguna |

Todas las tasks son independientes — se pueden ejecutar en **paralelo** (FAIL_MODE=parallel).

---

## Próximo paso

```
/pipeline run docs/plans/2026-07-24-stabilization-zero-bug.md   → ejecutar las 15 tasks
/pipeline task <ID>                                              → ejecutar una específica
```
