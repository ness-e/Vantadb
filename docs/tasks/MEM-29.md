# Task MEM-29 — Fuentes locales del wiki + chunker 12k/400

**Plan:** docs/plans/2026-08-21-vanta-proxy-knowledge.md · **Task ID:** 3 · **Wave 0**
**Contrato (D19):** (a) scanner descubre .md en path local recursivo; (b) chunker 12000/400
produce chunks esperados; (c) SOURCE_CHAR_BUDGET 28000 respeta; (d) boundaries sin corromper
estructura; (e) path traversal guard (canonicalize + starts_with raíz).
**D36:** paths locales v1, SIN red. Fetcher HTTPS/SSRF FUERA de scope (diferido documentado).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/wiki/mod.rs` (29L) — declaraciones de módulos, re-exports, `SYNC_ERROR_MAX_CHARS`, `mod tests`.
- `src/wiki/tests.rs` (235L) — harness `in_memory_engine()` (BackendKind::InMemory), patrón AAA.
- `src/wiki/store.rs` (467L vía codegraph) — Wiki/WikiPage structs, validación, VantaError usage.
- `src/wiki/state.rs` (57L vía codegraph) — WikiState enum `#[non_exhaustive]`.
- TDAM refs: `MemoryKnowledge/src/engines/wiki/ingest-v2/chunker.ts` (100L completo),
  `ingest-v2/index.ts:65-94` (SOURCE_CHAR_BUDGET=28_000).

**Referencias hacia dentro (nuevo código):** ninguna — archivos nuevos.

**Referencias entrantes:** `src/wiki/mod.rs` agregará `pub mod sources; pub mod chunker;`
+ re-exports. Ningún caller existente se rompe (solo aditivo). Blast radius verificado por
codegraph: WikiStore/WikiState/WikiPage tienen callers solo dentro de src/wiki/*.

**Veredicto:** cambio ADITIVO en módulo wiki existente (MEM-28 commiteada `0c3a9dcf`).
Sin modificaciones a archivos existentes salvo `src/wiki/mod.rs` (2 líneas de declaración +
re-exports). Sin deps nuevas (std::fs + tracing ya presente). Riesgo bajo.

## Steps

### Step 1 — chunker.rs ✅ DONE
Port de TDAM chunker.ts a Rust: `DEFAULT_TARGET_CHARS=12000`, `DEFAULT_OVERLAP_CHARS=400`,
`chunk_text(text, target, overlap) -> Vec<String>`. Split por headings markdown → párrafos
(líneas en blanco) → hard-cut. Agregación con overlap tail. Tests (b) y (d). 6 tests.
Fix aplicado: filler de test con trailing space era comido por trim() del chunker —
hard-cut test ahora usa "x".repeat(25000).

### Step 2 — sources.rs ✅ DONE
Scanner local recursivo: `scan_local_sources(root) -> Result<Vec<SourceFile>>`. Solo `.md`,
canonicalize + starts_with raíz (`ensure_within_root`, symlink skip con tracing::warn),
SOURCE_CHAR_BUDGET=28_000 (corte por presupuesto total, truncando el archivo que cruza),
error claro si raíz no existe (InvalidInput). Tests (a), (c), (e). 7 tests.

### Step 3 — Verify mecánico + SECURITY checklist ✅ DONE
- `cargo check -p vantadb` ✅ exit 0
- `cargo nextest run -p vantadb wiki::` ✅ 24/24 (11 MEM-28 + 13 nuevos)
- `cargo fmt --check` ✅ (tras `cargo fmt`)
- `cargo clippy -p vantadb --all-targets --no-deps -- -D warnings` ✅ (fix: `idx` unused;
  warnings pre-existentes de unused_unsafe en storage/ fuera de scope)
- `cargo check -p vanta-memory` ✅ exit 0
SECURITY checklist: canonicalize+starts_with en cada file (trust boundary filesystem);
symlinks resueltos y escapados skipped con warn; binarios/no-UTF8 skipped vía read_to_string;
errores tipados VantaError::InvalidInput; sin unwrap/expect en prod; sin deps nuevas; sin red (D36).

## Context Save Point
Tarea COMPLETA. Sin commit (ordenado por orquestador). Archivos: src/wiki/{chunker,sources}.rs,
src/wiki/mod.rs (+4 líneas wiring), este task file.
