# WSM-13: Estrategia de bundle documentada

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (W17-3)
- **Creado:** 2026-08-30T19:00
- **last-synced:** 2026-08-30T19:30
- **Estado:** ✅ COMPLETED → ✅ COMPLETED (verify-only, staged para vanta-lead commit)
- **Owner:** vanta-docs (leaf specialist — no commit per role rules)
- **Source:** research-vantadb-wasm-20260825 §H-17, FIND-11 (parcial)
- **Branch:** develop
- **Roles:** docs only — sin cambios de código (ya existe `wasm-opt = ["-Oz"]` en Cargo.toml + feature flags `default=["tracing-wasm"]`, `opfs`)

## Blast Radius

**Files to read:**
- `vantadb-wasm/Cargo.toml` — features: `default=["tracing-wasm"]`, `opfs=[]`; `[package.metadata.wasm-pack.profile.release].wasm-opt = ["-Oz"]`
- `vantadb-wasm/pkg/` — bundle artifacts actuales
- `vantadb-wasm/demo/README.md` — usage docs
- `vantadb-ts/README.md` — npm package (líneas 80-119 ya cubren lazy load + CDN)

**Files to create:**
- `vantadb-wasm/README.md` — **new** (no existía) — bundle strategy completa ✅ CREATED

**Files to update:**
- `vantadb-wasm/demo/README.md` — agregar sección "Bundle size & lazy loading" ✅ UPDATED (+26 líneas)
- `vantadb-ts/README.md` — agregar tabla comparativa bundle (Orama/MiniSearch/Lunr) ✅ UPDATED (+19 líneas)

**Referred by:**
- Ninguno en código (solo docs); pero `vanta-docs` debe NO commitear (regla de rol).

**Referred to (out):**
- `docs/QUICKSTART.md` — cross-link desde estrategia de bundle
- `docs/research/research-vantadb-wasm-20260825.md` H-17 referencia origen

**Veredicto impacto:** ✅ doc-only, blast radius acotado a 4 archivos docs, sin tocar código fuente.

## Contrato

`Select-String -Path "vantadb-wasm/README.md" -Pattern "bundle.*size|lazy.*load|1\.3" | Measure-Object | Select-Object Count` >= 1

**Resultado:** Count = **12** (≥1 requerido) ✅ PASS

(Plan original apuntaba a `vantadb-wasm/README.md` que NO existía. Lo creamos con la estrategia documentada. Bundle size real medido: 1.35 MB raw / 578 KB gzipped — se documenta el 1.3 MB aproximado + comparativa honesta vs Orama/MiniSearch/Lunr con citations de bundlephobia.)

## Herramientas
- codegraph_explore (no necesario — task es doc-only)
- Bash (PowerShell) para mediciones + verify
- webfetch (para citations honestas Regla 11)
- argus_search_web + argus_extract_content (router en cascada per coordinated-web-search skill)

## Steps

### Step 1: Medir bundle sizes reales (no asumir)
- **Archivos:** `vantadb-wasm/pkg/`
- **Acción:** Medir raw bytes + gzipped bytes (PowerShell .NET GzipStream)
- **Verify:** Números reportados con reproducibilidad
- **Estado:** ✅ COMPLETED — 1.35 MB raw / 578 KB gzipped (40% de raw)

### Step 2: Investigar comparativa bundle JS puros
- **Archivos:** bundlephobia, npmjs.com, github.com
- **Acción:** Bundle size + gzipped para Orama, MiniSearch, Lunr (Regla 11 — citations honestas)
- **Verify:** URLs de bundlephobia citadas, números reportados en README
- **Estado:** ✅ COMPLETED — Orama 23.8 KB gzipped (bundlephobia), MiniSearch 5.9 KB gzipped (devpick), Lunr 8.1 KB gzipped (bundlephobia)

### Step 3: Crear `vantadb-wasm/README.md` con estrategia completa
- **Archivos:** `vantadb-wasm/README.md` (new) — 12596 bytes, 12 matches del contrato
- **Acción:** Documentar bundle sizes, lazy loading, code-split, feature flags, comparativa honesta
- **Verify:** Contrato `bundle.*size|lazy.*load|1\.3` Count = 12 >=1 ✅
- **Estado:** ✅ COMPLETED

### Step 4: Update `vantadb-wasm/demo/README.md` con sección bundle
- **Archivos:** `vantadb-wasm/demo/README.md`
- **Acción:** Agregar nota breve sobre bundle size (1.35 MB raw / 578 KB gzipped) + lazy loading + transformers.js
- **Verify:** Diff mínimo, +26 líneas, sin sobreescribir contenido existente
- **Estado:** ✅ COMPLETED — 9 matches de "bundle|1.35|gzipped|lazy" (vs 0 antes)

### Step 5: Update `vantadb-ts/README.md` con tabla comparativa bundle
- **Archivos:** `vantadb-ts/README.md`
- **Acción:** Agregar tabla comparativa Orama/MiniSearch/Lunr con citations bundlephobia
- **Verify:** Diff mínimo, +19 líneas, 6 matches de competitors
- **Estado:** ✅ COMPLETED

### Step 6: Verify mecánico del contrato
- **Archivos:** ninguno (verify)
- **Acción:** `Select-String -Path "vantadb-wasm/README.md" -Pattern "bundle.*size|lazy.*load|1\.3" | Measure-Object | Select-Object Count` >=1
- **Verify:** Count = 12 ✅ PASS
- **Estado:** ✅ COMPLETED

## Notas

- **vanta-docs no commitea** (regla del rol en AGENTS.md §"Límites de herramientas por rol"). Esta tarea es docs-only → consistente con la regla. **3 archivos modificados staged en working tree para vanta-lead integrar en próximo PR.**
- **Pre-mortem Fallo 1** (bundle size dinámico): medido con .NET GzipStream (sistema, no asumir) ✅
- **Pre-mortem Fallo 2** (comparativa honesta): números verificados contra bundlephobia + GitHub de Orama/MiniSearch/Lunr con URLs en README ✅
  - **Corrección honesta**: el plan original estimaba "Orama ~50KB" — bundlephobia dice **23.8 KB gzipped** (más pequeño aún, lo cual hace el ratio VantaDB/Orama más honesto en 25×, no 27×).
- **Pre-mortem Fallo 3** (lazy-load requiere infra): documentamos `vite-plugin-wasm`, native loader (Node), code-split (Vanta Studio `vite --mode wasm`), y self-host con `wasm-pack build --target web` ✅
- **Regla 11 satisfied**: cada claim de tamaño tiene URL reproducible (bundlephobia.com, devpick.co, GitHub) + comando de medición embebido en README §1 (`Get-Item ... | Length` + .NET GzipStream).
- **Feature flags evaluados sin tocar**: `wasm-opt = ["-Oz"]` (ya activo, ahorra bytes vs `-Os` default), features `default=["tracing-wasm"]`, `opfs=[]`, `wasm` core flag (drop rocksdb/fjall/arrow IPC — dominante size win). Tabla §3 documenta cada uno.
- **Future levers documentados honestamente** (§6): LTO, mimalloc-rs, split engine → todos diferidos per Regla 9 ("no optimize without measuring") hasta tener `wasm_size` bench reproducible.

## Context Save Point

- **Fecha:** 2026-08-30T19:30
- **Branch:** develop
- **CI pendiente:** N/A (cambios solo en archivos docs; vanta-docs no commitea per rol)
- **Decisiones:**
  1. Crear `vantadb-wasm/README.md` (no existía) con estrategia completa (better que asumir que ya estaba y re-escribirlo)
  2. Usar gzipped sizes reales (.NET GzipStream) para honesty (Regla 11)
  3. Tabla comparativa incluye FEATURE GAP (no solo números) para que no parezca cherry-pick — VantaDB OPFS+HNSW+RRF+TTL+grafos vs Orama in-mem + plugin disk
  4. **Honest correction**: el plan estimaba "Orama ~50 KB"; el real (bundlephobia 2026-08-30) es **23.8 KB gzipped** — README cita el dato real
  5. Feature flags NO modificados (todos ya canónicos, scope de esta tarea es docs)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** vanta-lead integra staged files en próximo PR Wave17 (PROV-09/PROV-10/WSM-13 batch) — mensaje sugerido: `docs: WSM-13 — Estrategia de bundle documentada (lazy-load + comparativa honesta vs Orama/MiniSearch/Lunr)`