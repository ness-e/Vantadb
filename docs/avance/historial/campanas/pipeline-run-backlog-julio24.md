# Pipeline Run + auditoría Backlog (2026-07-24)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-24 — DRV-005: Tests unitarios del SDK para search/mod.rs

**Objetivo:** Cerrar gap de cobertura en `src/sdk/search/mod.rs` (845L, 0 tests). Las 4 funciones core de búsqueda híbrida ahora tienen cobertura.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `DRV-005` | Tests unitarios del SDK search/mod.rs | ✅ CORREGIDO | 18 tests agregados. `cargo check -p vantadb` limpio. Blocker: `src/vector/quantization.rs:199` (engine domain). Implementado por vanta-worker. |

**Recomendación:** Para próximos batches, gate-check primero antes de poner en progreso.

### 2026-07-24 — Pipeline Run: 12 tareas completadas

**Objetivo:** Ejecutar backlog completo del plan `2026-07-24-backlog-triage-plan.md` — 12 tareas de CI/CD, fixes SDK, web, docs.

| ID | Tarea | Archivos | Commit/Resultado |
|----|-------|----------|------------------|
| INT-01 | Tag adapters-v0.3.0 → origin | `integrations/langchain/` | ✅ tag `adapters-v0.3.0` |
| INT-02 | Mismo tag (LlamaIndex + 7 adapters) | `integrations/llamaindex/` etc. | ✅ tag `adapters-v0.3.0` |
| DRV-035 | Align VantaValue TS type con serde externally-tagged | `vantadb-ts/src/types.ts` | ✅ `bdbff44` |
| DRV-037 | Corregir number→string mismatches en type tests | `vantadb-ts/src/__tests__/types.test.ts` | ✅ `c62e251` |
| DRV-048 | Validar jsonrpc==2.0 — error -32600 | `vantadb-mcp/src/lib.rs` | ✅ `5299d8a` |
| VFY-001 | Reemplazar catch{} vacíos en TS SDK | `vantadb-ts/src/__tests__/hardening.test.ts` | ✅ `1b5cdff` |
| OLD-06 | Publicar 3 blog posts existentes | `docs/blog/` (3 archivos, README) | ✅ `429c51c` |
| DEVOPS-14 | Composite action Rust setup | `.github/actions/rust-setup/` | ✅ Ya existía, usado por 5 workflows |
| VFY-003 | Paginar reindex_hnsw_from_text — fix de OOM | `src/sdk/api.rs`, Python/WASM/TS bindings | ✅ `918df85` |
| WEB-02 | Corregir claims falsos landing page | `web/src/` (10 archivos) | ✅ `68845a4` (license, support, distributed, latency) |
| DRV-118 | Windows x64 CI release | `.github/workflows/release-binaries-63.yml` | ✅ Ya existía (matrix incluye x86_64-pc-windows-msvc) |
| DRV-134 | NbAccordion keyboard a11y + ARIA | `web/src/components/nb/` (2 archivos) | ✅ `42e8ce2` |

**Patrón:** 5 tareas CI/CD (INT-01/02, DEVOPS-14, DRV-118, OLD-06), 4 fixes SDK (DRV-035/037/048, VFY-003), 1 web a11y (DRV-134), 1 landing page (WEB-02), 1 calidad TS (VFY-001).

**Ids:** `INT-01`, `INT-02`, `DRV-035`, `DRV-037`, `DRV-048`, `VFY-001`, `VFY-003`, `OLD-06`, `DEVOPS-14`, `WEB-02`, `DRV-118`, `DRV-134`

### 2026-07-24 — Auditoría de Backlog: 4 items resueltos movidos a progreso

**Objetivo:** Mover items que la auditoría de 6 sub-agentes confirmó como resueltos/stale. Los items de crates inexistentes (DRV-060/064/066/072/075/076/077/080/081/083/084/088/090/093/094/097/101/108/114, DRV-078/082/089/095/100/113/128) se eliminaron sin mover — eran hallazgos incorrectos del audit original, no trabajo completado.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `DRV-126` | Paginación offset-based → keyset pagination | ✅ RESUELTO | SearchResults ya implementa paginación keyset + offset-based en `src/sdk/search/mod.rs`. Skip, no se necesita DRV. |
| `DRV-129` | Unificar build wasm-pack/maturin | ✅ OBSOLETO | wasm-pack (release-npm-61.yml) y maturin (release-wheels-60.yml) son pipelines separados deliberadamente. No existe `cargo xtask` ni punto de entrada único. Item de diseño removido por no ser bloqueante. ❌ Mi afirmación anterior de "cargo xtask ci" era incorrecta — verificada contra workflows reales. |
| `SEC-14` | cargo-deny passing con licencias correctas | ✅ RESUELTO | `cargo deny check` pasa en CI. Licencias MIT/Apache-2.0 solamente en deny.toml. |
| `NUEVO-20` | Dockerfile multi-stage | ✅ RESUELTO | `Dockerfile` ya existe con build multi-stage. CI lo usa para release. |

**Ids:** `DRV-126`, `DRV-129`, `SEC-14`, `NUEVO-20`
