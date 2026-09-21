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

=== RECITATION TS-02 ===
Campaign ID: 39ce752f-eff8-41e9-897f-cf604d5123c9
Objetivo activo: TS-02 — Fix _native async en vantadb-ts/src/native.ts
Estado: completed
Última acción: Step 4 ✅ — verify full mecánico (build+vitest+eslint) + fix lint imports (normalizeMetadata unused) + creación task file TS-02.md con blast radius y evidencia debugging
Resultado: OK
Próxima acción: Lead: git add vantadb-ts/src/native.ts vantadb-ts/src/__tests__/native-error.test.ts .opencode/skills/campaign-executor/tasks/TS-02.md && git commit -m 'fix(vantadb-ts): TS-02 wrap async rejections in _native' && campaign_update_task_state completed en plan file
Contrato: verificacion: cd vantadb-ts && npm run build ✅ (1.7s) + npx vitest run ✅ (264 passed) + npx vitest run native-error.test.ts ✅ (3 passed) + npx eslint src/native.ts src/__tests__/native-error.test.ts ✅
evidencia:
- claim: _native es private async con try { return await fn(); } catch y envuelve async rejections en VantaError NATIVE_ERROR
  evidencia: vantadb-ts/src/native.ts:149-154 (private async _native<T> ...) + vantadb-ts/src/__tests__/native-error.test.ts:22-33 (async rejection test)
  confianza: alta
- claim: npm run build pasa sin errores TS
  evidencia: campaign_verify_cmd cd vantadb-ts && npm run build passed 1.7s
  confianza: alta
- claim: npx vitest run pasa 264 tests incluyendo 3 native-error
  evidencia: campaign_verify_cmd cd vantadb-ts && npx vitest run passed 264/264
  confianza: alta
artefactos:
- vantadb-ts/src/native.ts
- vantadb-ts/src/__tests__/native-error.test.ts
- .opencode/skills/campaign-executor/tasks/TS-02.md
invariantes: toda rejection async del binding debe surfacing como VantaError NATIVE_ERROR; VantaError existente pasa untouched
deuda: ninguna
queda_pendiente: lead debe verificar mecánico y commitear (sub-agente no commitea por plan Wave 0)
Próxima tarea si completa: TS-05
=== END RECITATION ===

=== RECITATION TS-05 ===
Campaign ID: 39ce752f-eff8-41e9-897f-cf604d5123c9
Objetivo activo: TS-05 — Preservar engines:{node:">=22.12"} en tarball publicado
Estado: completed
Última acción: Steps 1-4 ✅ — diagnosis completo, npm pack tarball verificado, guard workflow validado, smoke-pack PASSED con engines ok
Resultado: OK
Próxima acción: Lead: verificar mecánico y commitear task file + actualizar plan file recitation; próximo task TS-06 (Wave 1)
Contrato: verificacion: package.json engines:{node:">=22.12"} ✅ + npm pack tarball contiene engines ✅ (tar extract) + workflow guard TS-05 líneas 195-201 ✅ + smoke-pack PASSED (SMOKE OK) ✅
evidencia:
- claim: package.json tiene engines:{node:">=22.12"}
  evidencia: vantadb-ts/package.json:6-8 + node -e check ✅
  confianza: alta
- claim: npm pack tarball contiene engines
  evidencia: tar -xzf tgz -O package/package.json | grep -A1 engines → ">=22.12" ✅ (2 runs)
  confianza: alta
- claim: workflow guard TS-05 existe y verifica antes de publish
  evidencia: .github/workflows/release-npm-61.yml:195-201 (node -e engines.node check)
  confianza: alta
- claim: smoke-pack PASSED con engines ok
  evidencia: node vantadb-ts/scripts/smoke-pack.mjs → SMOKE OK + engines ok log ✅
  confianza: alta
artefactos:
- vantadb-ts/package.json
- .github/workflows/release-npm-61.yml
- vantadb-ts/scripts/smoke-pack.mjs
- .opencode/skills/campaign-executor/tasks/TS-05.md
invariantes: package.json siempre debe contener engines.node === ">=22.12"; tarball siempre debe incluir ese campo; workflow nunca debe llegar a npm publish si engines falta
deuda: ninguna
queda_pendiente: lead commitea; TS-06 listo (misma rama, no paralelizar 2→3→5)
Próxima tarea si completa: TS-06
=== END RECITATION ===

=== RECITATION TS-06 ===
Campaign ID: 39ce752f-eff8-41e9-897f-cf604d5123c9
Objetivo activo: TS-06 — Gate CI para tests TS: Fast Gate (<5min) con npm ci && build && vitest
Estado: completed
Última acción: Steps 1-4 ✅ — medición 26.12s (<5min) → Fast Gate, triggers PR+push ok, sin continue-on-error, 264 passed, CI_POLICY.md §10 actualizado
Resultado: OK
Próxima acción: Lead: git add .opencode/skills/campaign-executor/tasks/TS-06.md docs/operations/CI_POLICY.md docs/plans/2026-08-25-research-vantadb-ts-quickwins.md && git commit -m 'ci: TS-06 gate Fast Gate for TS tests (<5min, PR+push, no continue-on-error)' && campaign_update_task_state completed
Contrato: verificacion: cd vantadb-ts && npm run build ✅ (0 err, 1.7s) + npx vitest run ✅ (264 passed, 9 files, ~13s) + npm ci+build+vitest total 26.12s <300s → Fast Gate ✅ + grep continue-on-error vacío ✅ + pull_request.paths incluye vantadb-ts/** ✅ + CI_POLICY.md §10 documenta gate ✅
evidencia:
- claim: Workflow CI tiene job Fast Gate que corre npm ci + build + vitest en <5min medido
  evidencia: Measure-Command total 26.12s (ci 5.6s + build 2.6s + vitest 13.8s) y 27.38s (2 runs) << 300s; .github/workflows/release-npm-61.yml:42-84 job tests timeout 10, comment TS-06 measured ~18s → Fast Gate tier
  confianza: alta
- claim: Job sin continue-on-error y respeta CI_POLICY Regla 2
  evidencia: grep -n continue-on-error .github/workflows/release-npm-61.yml → 0 resultados
  confianza: alta
- claim: Job corre en cada PR y push que toca vantadb-ts
  evidencia: .github/workflows/release-npm-61.yml:24-28 pull_request.paths [vantadb-wasm/**, vantadb-ts/**, workflow] + push.branches [main] paths [vantadb-ts/**] (PR cubre feature branches, push cubre main)
  confianza: alta
- claim: npx vitest run 264 tests pasan tras build
  evidencia: cd vantadb-ts && npx vitest run → 9 passed, 264 passed (3 runs: 14.86s, 13.50s, 12.90s)
  confianza: alta
artefactos:
- .github/workflows/release-npm-61.yml (job tests existente, verificado Fast Gate <5min, triggers PR+push, no continue-on-error)
- docs/operations/CI_POLICY.md (§10 Release Workflows row updated to document Fast Gate job)
- .opencode/skills/campaign-executor/tasks/TS-06.md (blast radius, 4 steps, SDP 5 skills)
- docs/plans/2026-08-25-research-vantadb-ts-quickwins.md (recitation TS-06)
invariantes: job CI siempre debe correr npm ci + build + vitest <5min; sin continue-on-error; triggers PR+push con paths filter; 264 tests pasan; CI_POLICY documenta gate
deuda: ninguna (task docs): +documentación CI_POLICY, -deuda observabilidad
queda_pendiente: registrar en docs/avance/activo/bindings.md cuando Wave 1 cierre; próximo task TS-08 (CDN ESM) o TS-07 (smoke-pack)
Próxima tarea si completa: TS-08
=== END RECITATION ===

=== RECITATION TS-07 ===
Campaign ID: f1748ef4-7b7d-4483-abc6-e55ed99be422
Objetivo activo: TS-07 — Smoke-test del tarball: script smoke-pack.mjs wired en release-npm-61.yml, quickstart create+put+get+close + engines verification
Estado: completed
Última acción: Steps 1-4 ✅ — script 4 pasos + wiring order 6 steps + smoke SMOKE OK (5.2s, engines ok, PASSED) + package/tarball engines >=22.12, task file sync
Resultado: OK
Próxima acción: Lead: verify full mecánico (cargo fmt/clippy/docs-coverage + npm run build + vitest) y git commit TS-07 + archivar plan Wave 2 cierre
Contrato: verificacion: node vantadb-ts/scripts/smoke-pack.mjs ✅ (packed vantadb-0.5.0.tgz + engines ok {"node":">=22.12"} + rewrote ^0.5.0 + tarball installed cleanly + SMOKE OK + PASSED, 5.2s) + wiring build(77)<rewrite(183)<engines(197)<smoke(204)<check(208)<publish(221) ✅ + tarball engines OK ✅ + package engines OK ✅
evidencia:
- claim: script smoke-pack.mjs existe y hace 4 pasos atómicos (npm pack → tar extract + engines check + file:→^ rewrite en copia → npm pack fixed → mkdtemp app limpio → npm install tgz → quickstart create+put+get+close → cleanup ambos tmps)
  evidencia: vantadb-ts/scripts/smoke-pack.mjs:1-106 (verify-step1.mjs 13/13 checks OK) + campaign_verify_cmd node C:\Users\Eros\AppData\Local\Temp\verify-step1.mjs → Step1 OK 0.8s
  confianza: alta
- claim: workflow release-npm-61.yml wirea smoke-pack.mjs después del build y antes de publish con working-directory correcto y sin continue-on-error
  evidencia: .github/workflows/release-npm-61.yml:203-207 (- name: Smoke-test packed tarball, working-directory: vantadb-ts, run: node scripts/smoke-pack.mjs) + orden verificado build(77)<rewrite(183)<engines(197)<smoke(204)<check(208)<publish(221) + campaign_verify_cmd verify-step2.mjs → wiring OK 1.4s + grep continue-on-error vacío
  confianza: alta
- claim: node vantadb-ts/scripts/smoke-pack.mjs pasa con SMOKE OK + engines ok + PASSED (hardening TS-05) — quickstart create+put+get+close funciona contra tgz instalado limpio
  evidencia: campaign_verify_cmd node vantadb-ts/scripts/smoke-pack.mjs → [smoke] packed: vantadb-0.5.0.tgz + [smoke] engines ok: {"node":">=22.12"} + [smoke] rewrote vantadb-wasm dep -> ^0.5.0 + [smoke] tarball installed cleanly + SMOKE OK + [smoke] PASSED (vantadb-0.5.0.tgz) exit 0 5.2s (2026-08-27)
  confianza: alta
- claim: tarball publicado preserva engines.node >=22.12 y package.json engines intacto
  evidencia: campaign_verify_cmd node -e package engines OK: {"node":">=22.12"} ✅ + campaign_verify_cmd node verify-step3c.mjs → tarball engines OK: {"node":">=22.12"} ✅ (npm pack + tar extract, 2.1s)
  confianza: alta
artefactos:
- vantadb-ts/scripts/smoke-pack.mjs (106 líneas, idempotente, dos tmps, finally cleanup)
- .github/workflows/release-npm-61.yml (226 líneas, job publish-ts 151-226, wiring 203-207)
- vantadb-ts/package.json (66 líneas, engines >=22.12, file:../vantadb-wasm/pkg)
- .opencode/skills/campaign-executor/tasks/TS-07.md (blast radius, 4 steps ✅, SDP 4 skills, Regla 0)
- docs/plans/2026-08-25-research-vantadb-ts-quickwins.md (recitation TS-07)
invariantes: package.json siempre engines.node === ">=22.12"; tarball siempre debe incluir engines (tar extract); smoke-pack siempre debe hacer pack→install limpio→quickstart (create+put+get+close)→cleanup con exit 0 y engines check fail-fast; workflow publish-ts nunca debe llegar a npm publish si smoke falla (exit 1 → job fail); quickstart payload hello válido para put/get sync
deuda: ninguna — task verification + task file + recitation, sin código nuevo estructural; Wave 2 cierre pendiente: registrar en docs/avance/activo/bindings.md + archivar plan (nota plan)
queda_pendiente: lead verify full + git add/commit + skill progreso + archivar plan + restaurar DESKTOP-QW5 PENDING state si aplica
Próxima tarea si completa: Ninguna — Wave 2 cierre (Waves 0-2 completas: TS-02, TS-05, TS-06, TS-08, TS-07)
=== END RECITATION ===
