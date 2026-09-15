# FIND-79 — endurecer + completar tests TS portabilidad (export/import/reindex + importRecords estricto)

> **Plan:** `docs/plans/2026-09-15-find-correcciones.md` (Task 7, Wave2 — disjunto de FIND-78/FIND-70: `vantadb-ts/src/` vs README/workflow)
> **Estado:** ✅ DONE (2026-09-15 — verify contrato verde + review vanta-review approve + commit)
> **Commit:** `test: FIND-79 — ...` (4 archivos: `vantadb.ts` + `portability.test.ts` + `dx04` + `hardening`)
> **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡
> **Branch:** `develop` · **Commit:** `test: FIND-79 — ...` (tras verify contrato)
> **Ruta:** vanta-worker · **nextTask:** FIND-78
> **SDP:** `campaign_discover_skills_v2` archivosClave=`vantadb-ts/src/vantadb.ts vantadb-ts/src/__tests__` phase=`BUILD`
> contractKeywords=[vitest, typescript, export, import, reindex] → 8 skills (base + lifecycle, 0 keyword-mapped).
> Sugeridas del plan: `test-driven-development` ✅ cargada, `systematic-debugging` ✅ cargada.
> **SKILLS_CARGADAS:** test-driven-development, systematic-debugging, incremental-implementation, context-engineering (+ base campaign-executor, progreso, ponytail full; source-driven-development, doubt-driven-development, api-and-interface-design devueltos por SDP, no cargados — sin API nueva ni stakes prod; frontend-ui-engineering descartada — sin UI)
> **Referencias:** `docs/api/TS_SDK.md:429-436` (solo lectura — tabla Export/Import), `.opencode/rules/js-ecosystem.md` (leída — R-1/R-2: sin persistencia prometida en `connect()`/`create()`, `pkg/`/`dist/` no commiteados). Sin símbolos públicos nuevos → sin Spec (Gate D no dispara).
> **Uphill:** ⬆️ 0 / **Downhill:** ⬇️ 3 steps (fix importRecords + tests + verify/commit).

## Contrato (del plan)

- [x] Tests nuevos export/import/reindex con asserts fuertes (sin try/catch blando) — `portability.test.ts` 14 tests
- [x] `importRecords` sin try/catch blando (asserts estrictos inserted/updated/errors) — 8 estrictos + 4 endurecidos
- [x] `npx vitest run` verde en `vantadb-ts/` — 12 files, 311/311 ✅ (04:00 UTC)
- [x] `npx tsc --noEmit` 0 errores ✅ · `npm run build` ✅
- [x] `npx eslint .` 0 errores en `vantadb-ts` ✅ (exit 0, misma cadena)
- [x] Round-trip cazó bug real H1 → FIJADO (put-loop TS, Prove-It RED 6 fails → GREEN), nunca silenciado
- [x] FS inviable en WASM → pins + DEFER e2e documentado (H2, salida válida del plan)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-ts/src/vantadb.ts:823-956` (`exportNamespace/exportAll/importRecords/importFile/rebuildIndex/reindexHnswFromText` + `system` subcliente `:329-351` que delega) + `:142-149` (`_wasm` error boundary) + `:452-490` (`putBatch`/`get` patrón a reutilizar), `vantadb-ts/src/types.ts:140-164` (`ExportReport`/`ImportReport`/`FilterItem`), `vantadb-ts/src/__tests__/dx04.test.ts:352-374` (2 tests débiles `importRecords`), `vantadb-ts/src/__tests__/hardening.test.ts:310-342` (2 tests débiles `importRecords` con `console.error` y 0 asserts en catch), `vantadb-ts/src/__tests__/integration.test.ts` (sin portabilidad), `vantadb-ts/package.json` (scripts `build`/`test`/`lint`, engines `node>=22.19`), `vantadb-ts/vitest.config.ts` (load.test excluido por default), `vantadb-ts/eslint.config.js` (tests con `no-explicit-any` off), `docs/api/TS_SDK.md:429-436` (solo lectura), `.opencode/rules/js-ecosystem.md` (34L).
- **Referencias hacia dentro:** `importRecords` ← 3 callers en `vantadb.ts` (def + `system` delegate + tipo); `exportAll/exportNamespace/importFile/reindexHnswFromText` ← 1 caller c/u (codegraph: "no covering tests found" para los 4). WASM: `import_records` espera `Vec<MemoryRecord>` (`vantadb-wasm/src/lib.rs:1530-1543`), `export_all/import_file` → `std::fs` vía `impl_export.rs:228-378`, `reindex_hnsw_from_text` → `admin.rs` (IO en WASM).
- **Referencias entrantes:** ningún test fuerte depende de estos métodos (solo los 4 débiles citados); `docs/api/BINDINGS_NAMESPACES.md:175-181` lista los 4 como ✅ (tabla de paridad, no test).
- **Veredicto:** blast radius = `vantadb-ts/src/vantadb.ts` (1 método con fix) + `src/__tests__/` (2 archivos a endurecer + 1 nuevo). Sin Rust, sin docs técnicas, sin workflows. Riesgo 🟢 (TS-only, `put`/`get` ya testeados como base del fix).
- **Gate D:** no dispara (0 símbolos `pub` nuevos, 0 endpoints, fix de binding + tests; contrato con salida DEFER explícita para FS).

## Hallazgos DISCOVERY (evidencia, no silenciados)

### H1 (bug real cazado por round-trip): `importRecords(MemoryInput[])` siempre falla

- TS tipa `importRecords(records: MemoryInput[])` (`vantadb.ts:888`) y lo pasa crudo a `inner.import_records`, que deserializa `Vec<MemoryRecord>` (`lib.rs:1532`).
- Repro Node directo (dist actual): `importRecords([{namespace:'t',key:'a',payload:'pa',metadata:{}}])` → `VantaError: importRecords: Error: missing field created_at_ms` (CODE `VANTADB_WASM_ERROR`).
- Peor: ni siquiera un `MemoryRecord` completo de `get()` round-tripea — `importRecords([got])` → `invalid type: string "1789...", expected u64` (el get serializa u64 como string por JS number-safety, el import exige u64). **El path WASM está roto en ambas direcciones.**
- Los 4 tests existentes lo ocultan con try/catch blando (`dx04:358-363,369-373`; `hardening:326-331,335-340` con `console.error` y 0 asserts si throws).
- **Fix (scope TS-only, sin rebuild wasm):** reimplementar `importRecords` sobre el path `put` ya testeado — por cada input: `existed = get()!==null`, `put()` → inserted/updated, catch por-record → errors (semántica = core `impl_export.rs:307-314`: existed→updated, ok→inserted, err→errors). `put` ignora campos extra (verificado: `put(got-full-record)` → OK), así que acepta `MemoryInput` y `MemoryRecord` sin cambio de firma (estructuralmente compatible). Sin FS, sin Rust, ~25 líneas.

### H2 (FS inviable en WASM → DEFER e2e, salida válida del plan)

- Repro Node: `exportAll('./tmp.jsonl')` → `exportAll: IO error: operation not supported on this platform` (`impl_export.rs:227`); `reindexHnswFromText('t')` → mismo IO (`admin.rs:44`). `exportNamespace`/`importFile` comparten `resolve_export_path`/`std::fs` → mismo destino.
- Por R-1 js-ecosystem, `create()`/`connect()` son InMemory sin persistencia FS — el error es honesto, no bug.
- **Tests:** asserts fuertes de que lanzan `DbError` (sin try/catch blando), con paths `os.tmpdir()` + `mkdtemp` por test (aislamiento, pre-mortem FS del plan). **DEFER e2e:** round-trip real export→import solo posible vía binding nativo/futuro WASI-OPFS — documentado aquí, no nuevo ticket (el plan lo prevé como salida válida; el wiring FS-WASM es feature, no fix 1d).

## Steps atómicos

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | Fix `importRecords` en `vantadb.ts` vía path `put` (inserted/updated/errors + `duration_ms`, JSDoc honesto) | ✅ DONE | smoke Node R1-R4 + build |
| 2 | Endurecer 4 tests débiles (asserts estrictos) + nuevo `portability.test.ts` (importRecords estricto + export/import/reindex asserts platform-error con tmpdir, 0 try/catch blando) | ✅ DONE | RED 6 fails old-impl → GREEN 14/14 |
| 3 | Verify contrato + reviewer + commit `test:` + recitation + RESULTADO | ✅ DONE | vitest 311 + tsc + eslint + review approve |

## Verify + Review (evidencia)

- `npm run build && npx vitest run && npx tsc --noEmit && npx eslint .` (cadena única, exit 0): build ✅, **12 files 311/311 passed** ✅, tsc ✅, eslint 0 ✅.
- Prove-It RED: con impl vieja restaurada temporalmente (solo Edit, sin git), `portability.test.ts` → **6 failed / 8 passed** (`missing field metadata|created_at_ms`, `string-u64`) — los tests cazan H1; con fix → 14/14.
- Reviewer `vanta-review` (P2-01, contexto fresco): ✅ **APPROVE**, 0 critical/required. Subset re-verificado por reviewer: 14/14 + 109/109 + tsc exit 0. Opcionales no bloqueantes → NOTICED BUT NOT TOUCHING (ver abajo).
- Incidente `git stash pop` (mid-task): el worktree es compartido con agentes Wave2 paralelos + 15 stashes históricos (incl. GOV-C4 prohibido). Mi `stash push` falló por pathspec y el `pop` incondicional intentó popear `stash@{0}` ajeno → conflicto en `docs/plans/archive/2026-09-03-quality-gtm-wave.md` (estaba limpio). Recuperado con `git restore --source=HEAD` (solo ese archivo); stash ajeno intacto (15 entradas verificadas). **Lección:** jamás `git stash` en repo compartido con agentes paralelos — RED-proof por Edit round-trip en su lugar (lesson registrada).

## NOTICED BUT NOT TOUCHING (opcionales del reviewer, no bloqueantes)

- Guard `MAX_BATCH_SIZE` (100k) del binding viejo no replicado en put-loop → sin regresión real (el path viejo nunca funcionó); loop degrada en gracia. Sin acción.
- Divergencia `read_only` upfront (core throw vs TS errors:N) → alcanzable solo vía `create({read_only:true})`, sin contrato que lo exija. Sin acción.
- `system.*` FS asserts usan `toThrow(DbError)` genérico vs regex en flats → estrictos igual. Sin acción.
- Try/catch restantes en `dx04`/`hardening` pertenecen a describes no tocados (edge/query/D5a/putBatch-mixed) — pre-existentes, fuera de scope.

## Pre-mortem (del plan — verificado en DISCOVERY)

1. FS/tmpdir en tests WASM → **confirmado H2**: paths `mkdtemp(os.tmpdir())` por test; asserts sobre el throw (no sobre archivos).
2. Round-trip expone bug real → **confirmado H1**: se fija (put-loop TS, sin Rust), nunca silenciado; los 4 try/catch blandos se eliminan.

## Herramientas

- Blast: `codegraph_explore "vantadb-ts exportAll exportNamespace importFile reindexHnswFromText importRecords"` (57 símbolos, 6 archivos; 4 métodos sin tests) + 2ª explore core (`impl_export.rs:228-378` leído verbatim).
- Evidencia: `node -e dist/` ×4 (missing field / string-u64 / IO-platform ×2 / put-full OK).
- Verify: `npx vitest run` + `npx tsc --noEmit` + `npx eslint .` en `vantadb-ts/`; `campaign_verify_cmd` si disponible.
- Prohibidos (M): `.opencode/` (M submodule, no tocar), `completions/` (M), `desktop/src-tauri/Cargo.lock` (M), `docs/pipeline-state.json` (M), `docs/api/TS_SDK.md` (solo lectura), `desktop/` entero (FIND-63 cerrado), `vantadb-wasm/` (sin rebuild — fix es TS-only).

## Context Save Point

- 2026-09-15 DISCOVERY: plan Task 7 + `vantadb.ts:823-956` + types + 3 test files + package/vitest/eslint + TS_SDK + js-ecosystem leídos; `campaign_detect_task_type`=typescript; SDP BUILD 8 skills; TDD + systematic-debugging + incremental + context cargadas; H1 (bug importRecords, fix put-loop decidido) + H2 (FS DEFER-e2e) con repros Node; siguiente: Step 1 (editar `vantadb.ts`).
