# BND-12 — Cobertura tests vantadb-node 8→~20 (search/explain_search/put_batch/capabilities/close-drain)

- **Plan:** `docs/dev/plans/2026-09-07-backlog-triage.md` (Task 4, Wave1)
- **Estado:** ⏳ IN PROGRESS
- **Branch:** develop
- **Contrato:** `npm test` en `vantadb-node/` → ≥18 tests, 0 failed; cobertura de `search`, `explain_search`, `put_batch`, `capabilities`, `close` presente
- **SDP:** test-driven-development, incremental-implementation, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design (+ campaign-executor/progreso base; frontend-ui-engineering descartada por irrelevante — sin UI)

## Descubrimiento (DISCOVERY completo — task file no existía)

- Baseline del plan (8 tests, research H-06) **stale**: commits `a86c7e4e` (25 tests) y `a8e70c40` (BND-10 parity) ya expandieron la suite a **34 tests** (api 26 + graph 5 + persistence 3). `.node` binario presente (5.3MB), toolchain napi OK.
- `npm test` real (2026-09-07): **31 passed / 3 failed** — gaps de cobertura CERRADOS, lo que falta es GREEN de 3 tests con aserciones contra runtime real:
  1. `purgeExpired`/`count` devuelven **BigInt** (`0n`), no `number` — napi-rs mapea `u64` Rust → BigInt; `index.d.ts:359,366` dice `Promise<number>` (type-lie → FIND-BND12-01, no se toca en este task).
  2. `supersede` no back-propaga el marcador a snapshots de `versions()`; el marcador vive en el record vivo (`get()` → `superseded_by: "new"`, verificado con repro node).
- Firma `put_batch` nativa (`lib.rs:117`): `MemoryInput[]` array — tests ya la usan bien. Sin divergencia python.
- Gate D: **no disparado** — solo tests, sin símbolos públicos nuevos, sin hot path, contrato mecánico.
- Gate spec-first: **N/A** — no es feature-add, es GREEN de cobertura existente.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-node/tests/api.test.ts` (491L), `graph.test.ts` (181L), `persistence.test.ts` (103L), `vantadb-node/src/lib.rs:99-132` (put/put_batch), `:93-97` (close/OpGate), `:422-432` (explain_search), `:216-227` (capabilities), `index.d.ts:359,366` (tipos count/purgeExpired), `index.js:706-707` (re-export directo del binding, sin wrapper JS).
- **Referencias hacia dentro (lo que el cambio necesita):** `VantaDb.connect/put/putBatch/get/versions/supersede/count/purgeExpired/close` del binding nativo; `serde_json` records con `superseded_by`/`superseded_at_ms`.
- **Referencias entrantes (quién depende):** `bench/bench-abi.mjs` usa `putBatch`; ningún otro módulo importa los tests. Cambio test-only → blast radius = 1 archivo.
- **Veredicto:** impacto mínimo y reversible; no se toca `src/lib.rs`, `index.d.ts`, ni archivos de MOD-24/TS-09/UX-A11Y-01.

## Steps

### Step 1 — Fix bigint assertions (purgeExpired/count) ✅ DONE (`npm test` focado verde)
- **Qué:** `toBe(0)` → `toBe(0n)` + `Number(...)` donde se compara con literales; renombrar test a "...returns a bigint count".
- **Contrato step:** `npx vitest run tests/api.test.ts -t "purgeExpired"` y `-t "count returns"` verdes.
- **Verificación:** vitest focado.

### Step 2 — Fix supersede assertion (vía get, no versions) ✅ DONE (`npm test` focado verde)
- **Qué:** afirmar `superseded_by === "new"` sobre `get("ns","old")` (runtime truth del repro); mantener `versions().length >= 1` como historia.
- **Contrato step:** `npx vitest run tests/api.test.ts -t "supersede marks"` verde.
- **Verificación:** vitest focado.

### Step 3 — Full verify + sync plan ✅ DONE (`npm test` 34/34; plan sync; NO commit — solo vanta-lead)
- **Qué:** `npm test` 34/34 + actualizar plan file Task 4 → COMPLETED + recitation. **NO commitear** (solo vanta-lead).
- **Contrato step:** `npm test` en `vantadb-node/` → 34 passed, 0 failed.

## Colaterales (routing findings.md)
- **FIND-BND12-01:** `index.d.ts:359,366` declara `Promise<number>` para `purgeExpired()`/`count()` pero runtime devuelve BigInt (u64 napi). Fila Backlog pendiente (type-lie familia MOD-24). En este task solo se ajustan los tests a runtime truth.

## Recitation
- Ver plan file `=== RECITATION BND-12 ===` al cerrar.
