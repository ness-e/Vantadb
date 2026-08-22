# Core/WAL hardening — DRV races, SEC-13, WEB visuales

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### DRV-007: Data race en filter_field() (scalar_index sin lock)
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Add `let _nodes = self.nodes.read()` before `self.scalar_index.lookup()` so `filter_field` establishes a happens-before relationship with concurrent writers holding the write lock on `nodes`
- **Resultado:** ✅ `cargo check -p vantadb` clean, clippy clean (zero warnings with `-D warnings`). 1-line fix.
- **Ids:** `DRV-007`

### DRV-006: Race condition en delete()
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Remove `drop(nodes)` in `InMemoryEngine::delete` so `RwLockWriteGuard` covers index cleanup — eliminates unprotected window between node removal and edge_index/scalar_index update
- **Resultado:** ✅ `cargo check` clean, 210/211 tests pass, clippy clean. Commit `de6ecac`.
- **Ids:** `DRV-006`

### DRV-008: Duplicate scoring pipeline en vector_search() y hybrid_search()
- **Fuente:** Backlog
- **Fecha:** 2026-07-23
- **Objetivo:** Extraer ~14 líneas duplicadas de filter_map + scoring chain en `vector_search()` y `hybrid_search()` a helper `collect_scores()`. Remueve DRY violation entre engine.rs:288-305 y :399-413
- **Resultado:** ✅ `cargo check -p vantadb` clean.
- **Ids:** `DRV-008`

### DRV-011: Scan-forward recovery duplicado en WalWriter y WalReader
- **Fuente:** Backlog
- **Fecha:** 2026-07-23
- **Objetivo:** Extraer ~12 líneas duplicadas del patrón de llamada a `scan_forward_valid` en `WalWriter::open()` y `WalReader::next_record()` a helper `try_scan_forward()`. Remueve DRY violation en src/wal.rs
- **Resultado:** ✅ `cargo check -p vantadb` clean. 50 WAL tests pasan (incluye test_wal_auto_healing_and_recovery). Commit `e354c250`.
- **Ids:** `DRV-011`

### DRV-015: Refactor WalWriter::open_with_buffer() función monolítica de 100L
- **Fuente:** Backlog
- **Fecha:** 2026-07-24
- **Objetivo:** Extraer el loop de recovery scanning de `open_with_buffer()` a `recover_valid_records()` + limpiar la función orquestadora. Reduce de ~100L a ~55L. Separa 3 responsabilidades: file opening, header validation, recovery scanning.
- **Resultado:** ✅ `cargo check -p vantadb` clean, `cargo clippy -D warnings` clean, 1616/1617 tests pasan (1 pre-existing fail en metrics).
- **Ids:** `DRV-015`

### DRV-109: LlamaIndex missing GIL release
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Release GIL in `add`, `query`, `delete` using same `py.detach()` pattern as DRV-102
- **Resultado:** ✅ `cargo check -p vantadb-llamaindex` passes. Commit `74fdc23`.
- **Ids:** `DRV-109`

### SEC-13: CSP nonce + HSTS headers
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Add nonce to `style-src-elem` CSP directive; HSTS already configured in vercel.json
- **Resultado:** ✅ `npx tsc --noEmit` clean. Commit `d6282a5`.
- **Ids:** `SEC-13`

### WEB-15/WEB-16: Refinamientos visuales de la home (text-align, font-weight, fondo del Nav)
- **Fecha:** 2026-07-02
- **Objetivo:** Fix text-align from center to left on 9 elements, set H1 font-weight to 700, update Nav background to warm paper (`--surface-glass`).
- **Checklist:**
  - [x] `text-align: left` applied across homepage sections
  - [x] H1 font-weight changed from 800 to 700
  - [x] Nav background: `rgba(10,10,10,0.85)` → `rgba(249,248,246,0.85)`
- **Ids:** `WEB-15`, `WEB-16`

### WEB-09: Consolidar librerías de animación (AnimeJS eliminado)
- **Fecha:** 2026-07-02
- **Objetivo:** Remove AnimeJS (4.5KB) and Motion (12.42KB) — GSAP handles 95% of animations. Reduce bundle by ~155KB+.
- **Checklist:**
  - [x] AnimeJS dependency removed from `package.json`
  - [x] Motion dependency removed from `package.json`
  - [x] All AnimeJS imports refactored to GSAP equivalents
- **Ids:** `WEB-09`
