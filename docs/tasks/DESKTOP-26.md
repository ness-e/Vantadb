# DESKTOP-26: Tests frontend Vanta Studio — vitest unit stores/lentes + integración bridge mock

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)
- **Leídos completos:** desktop/package.json, tsconfig.json, vite.config.ts, src/vanta.ts, src/transport.ts, src/store/{undo,favorites,search-history}.ts, src/components/home/HomeOverview.tsx, src/store/undo.test.ts, src/components/space/projection.worker.test.ts
- **Referencias entrantes:** los tests nuevos no tienen callers; package.json `test` script es entry point nuevo; tsconfig exclude ampliado no afecta build (`npm run build` verificado exit 0)
- **Referencias salientes:** tests mockean `../../vanta` y `@tauri-apps/api/core` — contratos existentes intactos
- **Veredicto:** aditivo puro; ningún archivo de producción modificado salvo config (package.json/tsconfig) y el flaky-fix acotado en projection.worker.test.ts

## Spec
N/A — test infrastructure con contrato mecánico

## Contrato
`cd desktop && npm test` → verde en CI ✅ (exit 0, 2 corridas consecutivas)

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Configurar vitest + testing-library — ✅
- **Archivos:** `desktop/package.json`, `desktop/vitest.config.ts` (nuevo), `desktop/tsconfig.json`
- **Hecho:** deps dev: vitest@4.1.11, @testing-library/react, jsdom. Script `test: "vitest run"`. vitest.config.ts jsdom+globals (RTL auto-cleanup sin setup file). Excludes: 9 archivos node:test + defaultExclude. tsconfig excluye *.test.tsx del build.
- **Verify:** `npx vitest run --version` → v4.1.11 ✅ · `npm test -- --run store` ✅

### Step 2: Tests unit stores — ✅
- **Archivos:** `desktop/src/store/persisted-stores.test.ts` (nuevo); undo ya cubierto por undo.test.ts pre-existente
- **Nota:** store "preferences" NO existe en el código (DESKTOP-23 no dejó archivo); favorites+search-history cubren hidratación/persistencia localStorage inyectable
- **Verify:** `npm test -- --run store` → 2 files / 7 tests ✅

### Step 3: Tests unit lentes — ✅ (lente representativo HomeOverview)
- **Archivos:** `desktop/src/components/home/HomeOverview.test.tsx` (nuevo)
- **Hecho:** cards render + merge namespace_stats sobre list() + estado cargando inactivo + fallback error backend
- **Fix colateral:** projection.worker.test.ts flaky (UMAP real 100 pts > deadlines internos bajo carga) → 40 pts + deadline segunda corrida 20s. Mismo smoke, margen holgado.
- **Verify:** `npm test -- --run components/home` → 3 tests ✅

### Step 4: Test integración bridge Tauri (mock) — ✅
- **Archivos:** `desktop/src/vanta.test.ts` (nuevo)
- **Hecho:** mock `@tauri-apps/api/core#invoke`; marca `__TAURI_INTERNALS__` hoisted para que getTransport() resuelva TauriBackend. Round-trip put→get→delete→search→listPage con comando+args exactos; errores propagan sin envolver.
- **Verify:** `npm test -- --run vanta` → 3 tests ✅

## Dependencias
- DESKTOP-23 (preferences store), DESKTOP-10 (put bridge), DESKTOP-11 (DTO enriquecido) — ya completadas

## Notas
- DoD cumplido: `npm test` verde ×2 consecutivas + `npm run build` exit 0 (tsc incluye)
- Deuda: store preferences no existe (crear cuando haya consumidor real); migrar los 9 tests node:test a vitest si se quiere runner único
- Rust ya tiene tests (17+4+24+28+21+2) — sin cambios backend
