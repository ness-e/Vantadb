# Plan — Quick wins INV-vantadb-ts (2026-08-25)

> **Origen:** `/research vantadb-ts` → `docs/reviews/research-vantadb-ts-20260825.md` (score 7.2/10).
> Decisiones HITL: 5 hallazgos 🟢 esfuerzo aprobados como quick wins. El resto quedó en
> Backlog P41 (`TS-01..TS-13`) para ejecución posterior.
> **Verificación global:** `cd vantadb-ts && npm run build && npx vitest run` (261 tests) + `npx eslint .`

## Waves

MAX_CONCURRENT = 3. Sub-agentes NO commitean; lead verifica mecánico y commitea por tarea.

### Wave 0

| Task | Backlog | Descripción | Sub-agente |
|---|---|---|---|
| 1 | TS-02 | Fix `_native` async en `vantadb-ts/src/native.ts:89-95`: convertir a `async`, `try { return await fn(); } catch` + test que afirme que un rechazo async del inner se envuelve en `VantaError` con `code`. | vanta-worker |
| 2 | TS-05 | Preservar `engines:{node:">=22.12"}` en tarball publicado. Diagnosticar dónde se pierde (release-plz config / script npm publish / workflow `release-npm-61.yml`) y corregir. Verificación: comparar `package.json` local vs campo `engines` tras `npm pack --dry-run`. | vanta-lead |

### Wave 1

| Task | Backlog | Descripción | Sub-agente |
|---|---|---|---|
| 3 | TS-06 | Gate CI para tests TS: job en workflow existente (Fast Gate si <5min medido, sino Heavy) que corre `cd vantadb-ts && npm ci && npm run build && npx vitest run`. Medir duración antes de elegir tier. Respetar CI_POLICY (sin continue-on-error). | vanta-lead |
| 4 | TS-08 | CDN ESM: verificar empíricamente si el glue wasm-bindgen funciona vía `cdn.jsdelivr.net/npm/vantadb@latest/+esm` (probablemente NO por import del `.wasm` binario). Documentar el resultado real en README §WASM bundle: si no funciona, mostrar la alternativa mínima (importmap + fetch del wasm o demo playground). | vanta-worker |

### Wave 2 (depende de Wave 1 para reusar el pipeline npm tocado en task 2)

| Task | Backlog | Descripción | Sub-agente |
|---|---|---|---|
| 5 | TS-07 | Smoke-test del tarball: script `vantadb-ts/scripts/smoke-pack.mjs` que hace `npm pack`, instala el tgz en un dir temporal limpio, corre un quickstart mínimo (create+put+get+close) y limpia. Wire al release npm después del build, antes de publish. | vanta-lead |

## Notas

- MOD-22/MOD-23/MOD-24 ya restauradas en P32 (aplicación inline de H-05 durante materialización).
- No paralelizar tasks que tocan el mismo workflow de npm (2→3→5 secuenciales entre sí).
- Al completar: registrar en `docs/avance/activo/bindings.md` + mover plan a archive con budget.
