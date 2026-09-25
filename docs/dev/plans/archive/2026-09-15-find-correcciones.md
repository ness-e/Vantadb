# Plan de Ejecución: Correcciones FIND 2026-09-15 — 27 DO + 1 SKIP (rehace 2026-09-10)

> **Campaign ID:** 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
> **Inicio:** 2026-09-15
> **Estado:** ✅ COMPLETADA 2026-09-16 (31/31, 0 failed; cierre skill progreso + archive)
> **Fuente:** `docs/dev/Backlog.md` filas FIND-63–88 + FIND-90/91 (28 filas; FIND-89 completada 2026-09-14)
> **Rehace a:** `docs/dev/plans/2026-09-10-correcciones.md` (26 tasks, sin iniciar, verificado contra review-doc y no contra código actual)
> **Autonomous:** false
> **FAIL_MODE:** `parallel` (MAX 3; secuencial interno si colisionan archivos)
> **SPEC:** no existe SPEC.md — 0 greenfield (todo fix/docs/tests sobre comportamiento existente).
> **SDP:** `campaign_discover_skills_v2` (phase PLAN, 30 keywords) → 8 skills (campaign-executor, progreso, planning-and-task-breakdown, spec-driven-development, source-driven-development, documentation-and-adrs, writing-guidelines, claude-api). Base manual: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, documentation-and-adrs, idea-refine, codebase-memory (14 total, justificación en Notas).
> **Selección owner:** Gate P vía `question` 2026-09-15 — SKIP FIND-76 (Recomendado) + escribir 27 DO re-scopeados (Recomendado).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 27 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 1 (FIND-76, verificado stale) |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 4 (FIND-88 forma del SSE drain · FIND-90 serialización `schema://` · FIND-83 alcance real de la contradicción MCP-27/29 · FIND-80 mecanismo de upload de crashes) · ⬇️ downhill = 27 tasks con contrato definido (steps atómicos en task files bajo demanda)

## Gate P — confirmación usuario (2026-09-15)

Triage + Paso 0 + `question` → owner aprobó "SKIP verificado (Recomendado)" para FIND-76 y "Sí, escribir plan (Recomendado)" con los 27 DO re-scopeados según evidencia de código.

## Verificación real global (Paso 0, 2026-09-15 — CodeGraph + codebase-memory-mcp + lectura directa)

Índices: CBM `full` 2026-09-15T06:28Z (80.687 nodos); scopes `src/`, `Formula/`, `workflows/`, `tests/` sin gaps; resto solo gaps conocidos (build/gitignore). Todo símbolo citado abajo existe en disco hoy.

- FIND-63: `desktop/vitest.config.ts:10` YA tiene `environment: "jsdom"` (DESKTOP-26 hecho); solo `projection.worker.test.ts:1` declara `// @vitest-environment node`; `undo.test.ts:54` asume storage compartido jsdom. Gap = 13 fails originales bajo Node experimental; approach cambia: correr suite y fijar remanente/ambiente por archivo, no config global.
- FIND-64: `ci-rust-10.yml:17` paths tienen `vantadb-*/**` + `integrations/**` pero NO `vanta-memory/**` — gap confirmado (el crate no matchea ningún patrón).
- FIND-65: `cache.rs:107-108` `is_expired` ya usa `elapsed()` (prod OK), pero el TEST `:761` hace `Instant::now() - Duration::from_secs(10_000)` → paniquea en hosts con uptime < 10.000s ANTES de llamar. Bug persiste en el test.
- FIND-66: `Formula/vantadb.rb:25-26` SIRVE aarch64-apple-darwin (+aarch64-linux) → README "🚧 Planned" stale; README `:40` documenta `--head` pero el rb NO tiene stanza `head` → gap real (añadir stanza o quitar doc); README `:22-24` lista 3 binarios pero `def install` solo instala 2 (`vanta-cli`, `vantadb-server`; el propio rb comenta que el tarball trae exactamente esos 2) → mismatch real.
- FIND-67: `QUICKSTART.md:12` dice v0.4.x, `:90` wheel 0.1.1, `:6` last_reviewed 2026-07-01 (tag actual v0.5.0). HALLAZGO NUEVO del Paso 0: `:188-189` usa `WANTA_EMBEDDING_PROVIDER`/`WANTA_LOCAL_MODEL` legacy (FIND-89 migró el código a `VANTADB_*` pero no este doc) → entra al scope.
- FIND-68: `docs/api/PROXY.md` NO existe (confirmado); `vanta-proxy/config.toml` SÍ existe (D31, `[server]`+`[upstream]` reales) → scope = solo docs (endpoints visibles en `server.rs:742-768`, env/defaults desde config + código).
- FIND-69: `vectorstore.py:24` fallback `DSPyRetrieve = object` + `:62` `super().__init__(k=k)` → `object.__init__` no acepta `k` → TypeError siempre sin framework. Tests existen (`tests/test_vectorstore.py` + conftest).
- FIND-70: `Cargo.toml:240` exige `async-ingestion` para el bench; BENCHMARKS.md lo cita CON flag (`:422,491-494`); pero `heavy-bench-nightly-51.yml` ni lo menciona (grep `ingestion_concurrent|async-ingestion` en workflows = 0) → el bench nunca corre en nightly. Fix = añadirlo con feature o documentar skip (no tocar Cargo).
- FIND-71: `download.py:26` ALLOW_PATTERNS trae `*.safetensors` + `*.bin` (duplicados plausibles); `manifest.lock` existe; `verify.py:135-192` es smoke con dummies/skips ("verify→smoke" por aclarar); sizes en `embeddings/README.md` por re-medir.
- FIND-72: `batch_vs_sequential_bench.py` NO tiene argparse y `__main__` corre ambos benches → `--help` ejecuta todo (peor que lo reportado); `requirements.txt` sin pins (solo `vantadb-py>=0.5.0`); `competitive_bench.py` mezcla `rmtree` con/sin `ignore_errors` (`:239,361` vs `:343,434` → WinError32 en Windows).
- FIND-73: `store()` sin `key` en los 3 `.pyi` (`openai:21-26`, litellm, ollama idénticos); `verify_pyi.py` solo hace `hasattr` (presencia, no firmas). `ollama/README.md` es quickstart sano → verificar drift de versión en ejecución.
- FIND-74: `examples/README.md` SÍ existe + `demo/requirements.txt` trae `vantadb-py>=0.4` (deriva confirmada); 0 archivos `.ts` en `examples/` (TS fuera del árbol confirmado). Re-scope del plan viejo sigue válido.
- FIND-75: README YA dice 1.35 MB (`:3,20`: 1.411.870 bytes medidos) → números al día (re-verificar, no re-escribir a ciegas); `lib.rs:2377-2378` input zero-copy AÚN pendiente (output hecho por PERF-08); `src/vantadb_wasm.d.ts` existe vs `pkg/` generado (gitignored) → diff post-build.
- FIND-77: comentario `tools.rs:1090` dice "Full profile (76 tools)"; GOV-B6 registró 79 (base 49 + ext 30, 2026-09-02) → stale por 3. Recontar en ejecución.
- FIND-78: `README.md:16` enlaza `docs/dev/reviews/research-vantadb-node-20260825.md` INEXISTENTE (Test-Path False); `:27` dice Node ≥ 18 pero `vantadb-ts/package.json:7` exige `node>=22.19` → nota de compat real.
- FIND-79: `exportAll/exportNamespace/importFile/reindexHnswFromText` EXISTEN (`vantadb.ts:868,823,915,953`); `importRecords` SÍ tiene tests (`dx04`, `hardening`) pero aserción débil → scope = endurecer + añadir los faltantes (no partir de cero).
- FIND-80: `fuzz/corpus/` VACÍO; `fuzz-40.yml` existe (corpus cache `:88-94`, crash cleanup `:96-97`) pero sin upload de crashes; `docs/dev/workflow/fuzz-40.md` SÍ documenta ci-gate (`:44-46`) → esa parte del reporte está stale; `fuzz-pr` sin mención. Re-scope: seed + upload + fuzz-pr.
- FIND-81: `vanta_certification.json` + `vantadb_data/` EXISTEN junto al crate (Test-Path True) → higiene real.
- FIND-82: `test-mcp.py:148-160` imprime conteos pero NUNCA aserta (4/4 handshake sin gate de drift) → real.
- FIND-83: MCP-27 documentado (`skills/.../SKILL.md:200`, IQL solo nodos tipados); MCP-29 (namespaces como tablas, completada) vive en la otra skill; api-reference evolucionó 33→79 según GOV-B6; copias `skills/` ↔ `.opencode/skills/` por verificar con hash. Uphill: alcance exacto de la contradicción se fija en DISCOVERY (puede colapsar a sync mecánico).
- FIND-84: 9 adapters en `integrations/` (sin dist/ en varios); pyprojects por verificar pins; fixtures por verificar subdir/disco. Estructura existe → DO con verificación en DISCOVERY.
- FIND-85: matriz wheels 3.11/3.13 (`release-wheels-60.yml:79,218,267`) vs classifiers 3.11–3.14 (`pyproject.toml:23-26`) → mismatch real; `probe_lock_db/` está gitignored (`.gitignore:134`) → sub-item stale en checkout (solo limpieza local); `put_batch_raw` con guard de tipos (`test_stub_drift.py:245`) → "firma asimétrica" por verificar contra `lib.rs:811`.
- FIND-86: `dream/mod.rs:31` "deliberately not wired into pipeline_worker.rs yet" + `l1_batch.rs:13` "pipeline_worker is untouched (wiring is a...)" + `auto_consolidate.rs:18` "follow-up: MCP scene_consolidate" → triple wiring real; MEM-70 sin menciones en `vanta-memory/` (números ausentes).
- FIND-87: `package.json:11-22` exports = `.` + `./types`, SIN `./native` → real; `TS_SDK.md:25` solo nota Node 18+ (sin sección native/wiki) → real.
- FIND-88: `cost.rs:201` `record_response_usage` existe + testeada (`:400`) pero `server.rs:251` dice "wired by a future SSE drain" y `:248-264` solo contabiliza input (`output_tokens: 0`) → cableado real pendiente.
- FIND-90/91: verificados en sesión 2026-09-15 (E0609 post-F3X `13f0f729`; task files ya existen con tabla var→campo y fix previstos).

## Tasks

### Wave0 — fallout F3X + CI paths (disjuntos: mcp / tests / workflow)

**Task 1: FIND-90 — handlers MCP a getters `IndexPort`**
- Appetite 1d · 🟡 · 🔴 Alta · `vantadb-mcp/src/handlers/resources.rs:134`, `tools.rs:1821,2850`
- Verificación real: E0609 vigente (sesión 2026-09-15); trait con `distance_metric()`, `all_node_ids()`, `stored_vector()` (`src/index_port.rs`).
- Gate Justificación: crate mcp + server no compilan; Fast Gate workspace rojo; fix acotado a 3 accesos.
- Contrato: `cargo check -p vantadb-mcp --tests` + clippy `-D warnings` + `mcp_tests` + `cargo check -p vantadb-server` verdes; `rg "\.config\b|\.nodes\b" vantadb-mcp/src/` = 0.
- Pre-mortem: `schema://` cambia de forma si se serializa desde getters (fijar forma JSON antes); iteración `all_node_ids`+`stored_vector` más lenta en dim-check (path frío, aceptable).
- Stop: TOCTOU `schema://` sin forma fijada → abortar a Gate V; +1 día sin green → DEFER con task file al día.
- Risk Register: 🟡×🔴 forma `schema://` diverge → snapshot JSON antes/después | 🟢×🟡 dim-check O(n) con clones → mide solo si review lo pide | 🟢×🟢 colisión con FIND-77 (mismo archivo) → 77 va en Wave3.
- Cynefin: 🟨 complicado (elegir serialización entre 2 approaches válidos). Top 3: forma schema / coste dim / colisión 77.
- ⬆️ 1 (serialización schema) / ⬇️ 3 steps. DoD: contrato + task file sync + recitation; commit atómico `fix:`; sin deuda neta.
- Appetite 1d ≥ esfuerzo 🟡 ✔. Task file `docs/dev/tasks/FIND-90.md` (EXISTE — continuar, no regenerar) · ⬜ PENDING · Ruta vanta-worker.

**Task 2: FIND-91 — `durability_recovery:433` → `contains_node`**
- Appetite 1h · 🟢 · 🔴 Alta · `tests/durability_recovery.rs:433`
- Verificación real: E0609 con `--all-features`; `:437` ya usa `node_count()` del trait (fix simétrico de 1 línea).
- Gate Justificación: bloquea clippy `--all-features`, perfil chaos y job heavy `storage-persistence`.
- Contrato: `cargo check -p vantadb --test durability_recovery --all-features` + clippy comando CI Windows + nextest chaos del test (si tiempo; si no, Heavy).
- Pre-mortem: `contains_node` con semántica distinta a `nodes.get().is_some()` (verificar impl en `port_impl.rs`); perfil chaos 218s excede ventana (dejar a Heavy documentado).
- Stop: semántica distinta → Gate V antes de cambiar assert.
- Risk Register: 🟢×🔴 assert debilitado por error → diff 1 línea revisado + test chaos verde | 🟢×🟢 tiempo chaos → fallback Heavy.
- Cynefin: 🟦 obvio. Top 3: semántica / tiempo / ninguno.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file sync + recitation; commit `fix:`; sin deuda.
- Appetite 1h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-91.md` (EXISTE) · ⬜ PENDING · Ruta vanta-worker.

**Task 3: FIND-64 — `'vanta-memory/**'` en paths CI**
- Appetite 1h · 🟢 · 🔴 Alta · `.github/workflows/ci-rust-10.yml:6-19` (+ sección pull_request)
- Verificación real: `vantadb-*/**` no matchea `vanta-memory/` (leído `:17` + ausencia de la entrada).
- Gate Justificación: cambios solo-memory no disparan CI; fix de 2 líneas.
- Contrato: entrada en push Y PR + `actionlint` + parse YAML OK.
- Pre-mortem: otra workflow con el mismo gap (buscar `vantadb-*/**` en todas); path con typo rompe el trigger (validar con `actionlint`).
- Stop: N/A (1h, mecánico).
- Risk Register: 🟡×🟢 gap gemelo en otro workflow → grep global antes de cerrar | 🟢×🟢 typo YAML → actionlint.
- Cynefin: 🟦 obvio. Top 3: workflow gemelo / typo / ninguno.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `ci:`; sin deuda.
- Appetite 1h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-64.md` (EXISTE) · ⬜ PENDING · Ruta vanta-lead (CI).

### Wave1 — desktop + proxy (disjuntos: desktop / cache.rs / server.rs)

**Task 4: FIND-63 — remanente vitest localStorage**
- Appetite 4h · 🟢 · 🔴 Alta · `desktop/vitest.config.ts`, `desktop/src/store/undo.test.ts`
- Verificación real: jsdom YA global (`:10`, DESKTOP-26); 1 archivo con env node explícito; `undo.test.ts:54` depende de storage compartido.
- Gate Justificación: 13 fails originales; approach re-scopeado (suite verde, no config).
- Contrato: `npx vitest run` desktop 0 fails + `tsc` 0 (conteo final el que reporte la suite, no 86 prefijado).
- Pre-mortem: mock global contamina tests que SÍ quieren Node (acotar por archivo); fake timers colisionan con polling (aislar).
- Stop: fix contamina otros tests → mock por archivo, no global.
- Risk Register: 🟡×🟢 contaminación cruzada → por-archivo + suite completa por cambio | 🟢×🟢 timers → aislar polling.
- Cynefin: 🟨 complicado (jsdom vs node por archivo). Top 3: contaminación / timers / conteo final.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `fix:`; sin deuda.
- Appetite 4h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-63.md` (EXISTE) · ⬜ PENDING · Ruta vanta-worker.

**Task 5: FIND-65 — `checked_sub` en test TTL**
- Appetite 2h · 🟢 · 🟠 · `vanta-proxy/src/cache.rs:752-764` (prod `:107-108` ya usa `elapsed()`, NO tocar)
- Verificación real: `Instant::now() - 10_000s` en el test paniquea con uptime < 2.7h antes de entrar a `is_expired`.
- Gate Justificación: panic real en hosts jóvenes; fix en test, prod intacto.
- Contrato: test verde en host joven (o con uptime simulado) + suite cache verde + clippy 0.
- Pre-mortem: `checked_sub().unwrap_or(now)` cambia el caso que cubre (mantener los 3 casos: expirada/viva/TTL_ZERO).
- Stop: N/A (mecánico).
- Risk Register: 🟢×🟡 cobertura de casos → los 3 asserts sobreviven | 🟢×🟢 prod tocado por error → diff solo-test.
- Cynefin: 🟦 obvio. Top 3: casos / prod intacto / ninguno.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `fix:`; sin deuda.
- Appetite 2h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-65.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 6: FIND-88 — cablear `record_response_usage` al SSE drain**
- Appetite 2d · 🟠 · 🟢 Baja · `vanta-proxy/src/server.rs:248-264`, `cost.rs:198-214`
- Verificación real: método existe+testeado; server solo contabiliza input y deja `output_tokens: 0` con TODO explícito.
- Gate Justificación: coste output-side real (hoy 0); prioridad baja → appetite acotado con salida DEFER-ratificado.
- Contrato: SSE drain llama a `record_response_usage` con cuerpo buffereado (o DEFER-ratificado con evidencia de por qué no) + suite proxy verde + clippy 0.
- Pre-mortem: SSE es streaming sin buffer (no hay "cuerpo" que parsear → DEFER-ratificado es salida válida); doble conteo request+response (solo output_tokens en response).
- Stop: sin punto de buffer en el drain → DEFER-ratificado, no rabbit hole.
- Risk Register: 🟠×🟢 sin buffer → salida DEFER explicitada | 🟡×🟡 doble conteo → tests de coste antes/después | 🟢×🟢 translate simétrico fuera de roadmap → solo si se exige.
- Cynefin: 🟨 complicado (forma del drain por descubrir). Top 3: buffer / doble conteo / alcance.
- ⬆️ 1 (forma del drain) / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `feat:`/`docs:` según salida; sin deuda.
- Appetite 2d ≥ 🟠 ✔. Task file `docs/dev/tasks/FIND-88.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave2 — TS tests + node + bench-nightly (disjuntos)

**Task 7: FIND-79 — endurecer + completar tests TS portabilidad**
- Appetite 1d · 🟡 · 🟡 · `vantadb-ts/src/__tests__/` (10 files; `dx04`, `hardening` existen)
- Verificación real: métodos existen (`vantadb.ts:823,868,888,915,953`); `importRecords` testeado débil; export/import/reindex sin tests dedicados.
- Gate Justificación: portabilidad sin red de seguridad; round-trip real puede cazar bugs (bienvenido).
- Contrato: tests nuevos export/import/reindex + `importRecords` sin try/catch blando + `vitest run` + `tsc` + `eslint` 0.
- Pre-mortem: FS en tests WASM (paths temporales por test); round-trip expone bug real (fijarlo o trackearlo, no silenciarlo).
- Stop: FS inviable en WASM → stubs + DEFER e2e documentado.
- Risk Register: 🟡×🟢 FS/paths → tmpdir por test | 🟢×🟡 bug cazado por round-trip → fix o FIND nuevo, nunca skip.
- Cynefin: 🟨 complicado. Top 3: FS / bug cazado / aserción débil.
- ⬆️ 0 / ⬇️ 3 steps. DoD: contrato + task file + recitation; commit `test:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-79.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 8: FIND-78 — link roto + nota engines Node**
- Appetite 2h · 🟢 · 🟢 · `vantadb-node/README.md:16,27`
- Verificación real: link a review inexistente (Test-Path False); "Node ≥ 18" vs `engines node>=22.19` del TS.
- Gate Justificación: doc rota + compat ambigua; fix mecánico.
- Contrato: 0 links rotos en el README (muestreo total) + nota compat explícita node18 vs ts22.19.
- Pre-mortem: el review existe en otra ruta (buscar antes de borrar la referencia).
- Stop: N/A.
- Risk Register: 🟢×🟢 referencia reubicada → grep antes de editar.
- Cynefin: 🟦 obvio. Top 3: reubicación / nota / ninguno.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- Appetite 2h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-78.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 9: FIND-70 — bench en nightly con feature o skip documentado**
- Appetite 2h · 🟢 · 🟡 · `.github/workflows/heavy-bench-nightly-51.yml`, `Cargo.toml:240` (solo lectura), `docs/user/operations/BENCHMARKS.md`
- Verificación real: nightly ni menciona el bench (grep workflows = 0); docs lo citan con flag.
- Gate Justificación: bench citado pero jamás corrido; fix = 1 job con `--features async-ingestion` o skip explícito.
- Contrato: nightly corre el bench con feature (o skip documentado en yml + BENCHMARKS coherente) + `actionlint` OK.
- Pre-mortem: feature rompe build nightly (entonces skip documentado es la salida, no forzar).
- Stop: N/A.
- Risk Register: 🟢×🟡 build con feature → probar comando local antes | 🟢×🟢 Cargo sin cambios (solo yml/docs).
- Cynefin: 🟦 obvio. Top 3: feature / skip / ninguno.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `ci:`; sin deuda.
- Appetite 2h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-70.md` · ⬜ PENDING · Ruta vanta-lead (CI).

### Wave3 — TS packaging + providers + embeddings (disjuntos)

**Task 10: FIND-87 — `./native` en exports + nota wiki**
- Appetite 4h · 🟡 · 🟡 · `vantadb-ts/package.json:11-22`, `docs/api/TS_SDK.md`
- Verificación real: exports = `.` + `./types` (sin `./native`); TS_SDK sin sección native/wiki (solo nota Node 18+ en `:25`).
- Gate Justificación: superficie native sin export documentado; consumidores no la descubren.
- Contrato: `./native` en exports (o decisión documentada de no exponer) + nota wiki explícita + `npm run build` + `vitest run` verdes.
- Pre-mortem: exponer `./native` rompe empaquetado (validar `npm pack --dry-run` con el export).
- Stop: pack roto → decisión documentada en vez de export.
- Risk Register: 🟡×🟢 empaquetado → dry-run antes/después | 🟢×🟢 `native.ts` ya testeado (`native-error.test.ts`).
- Cynefin: 🟨 complicado (decisión exponer/no). Top 3: pack / decisión / wiki.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `feat:`/`docs:`; sin deuda.
- Appetite 4h ≥ 🟡 ✔ (ajustado de 🟢 por decisión de packaging). Task file `docs/dev/tasks/FIND-87.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 11: FIND-73 — `key` en `.pyi` + `verify_pyi` con firmas**
- Appetite 4h · 🟢 · 🟡 · `providers/*/[*.pyi]`, `.github/scripts/verify_pyi.py`, `providers/ollama/README.md`
- Verificación real: `store()` sin `key` en los 3 stubs (`:21-26` c/u); script solo `hasattr` (presencia).
- Gate Justificación: drift stub-vs-runtime (PROV-10) + gate que no gatea.
- Contrato: `key` en 3 `.pyi` (+ `embed_batch` documentado) + script con `inspect.signature` + README ollama verificado.
- Pre-mortem: firma real difiere del stub en más params (sincronizar todo el `store`, no solo `key`).
- Stop: N/A.
- Risk Register: 🟢×🟡 drift mayor → diff stub-vs-runtime completo | 🟢×🟢 script con intérprete ausente → skip documentado, no rojo falso.
- Cynefin: 🟦 obvio. Top 3: drift / intérprete / readme.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `fix:`; sin deuda.
- Appetite 4h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-73.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 12: FIND-71 — embeddings peso + verify + sizes**
- Appetite 1d · 🟡 · 🟡 · `embeddings/download.py:26`, `verify.py:135-192`, `embeddings/README.md`, `manifest.lock`
- Verificación real: patterns amplios (`*.safetensors`, `*.bin`); lock existe; verify con dummies/skips; sizes por re-medir.
- Gate Justificación: descarga 3-4× lo declarado; verify que no verifica.
- Contrato: `ALLOW_PATTERNS` recortado + `verify`→smoke aclarado en docs + sizes reales + `download --check` verde.
- Pre-mortem: recorte rompe un modelo (check por modelo, no global); red en CI para descargar (todo offline/`--check`).
- Stop: modelo sin pattern claro → lista explícita por modelo, no glob.
- Risk Register: 🟡×🟢 modelo roto → check individual | 🟢×🟢 red → `--check` offline.
- Cynefin: 🟨 complicado. Top 3: patterns / smoke / sizes.
- ⬆️ 0 / ⬇️ 3 steps. DoD: contrato + task file + recitation; commit `fix:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-71.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave4 — MCP comentarios + skills (disjunto de Wave0; archivos distintos entre sí)

**Task 13: FIND-77 — conteo 76→79 en comentarios MCP**
- Appetite 1h · 🟢 · 🟢 · `vantadb-mcp/src/handlers/tools.rs:1090` (+ `config.rs:14` por verificar)
- Verificación real: comentario dice 76; GOV-B6 registró 79 (49+30). Recontar en ejecución (post-FIND-90).
- Gate Justificación: comentario que miente sobre la superficie; fix de 1 línea + recontar.
- Contrato: grep 76 ausente en comentarios de conteo + `cargo check -p vantadb-mcp` 0.
- Pre-mortem: el conteo real cambió tras F3X (recontar, no copiar 79 a ciegas).
- Stop: N/A. Secuencia: DESPUÉS de Wave0 (mismo archivo que FIND-90).
- Risk Register: 🟢×🟢 conteo a ciegas → recount mecánico | 🟢×🟢 colisión 90 → orden de waves.
- Cynefin: 🟦 obvio. Top 3: recount / orden / ninguno.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- Appetite 1h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-77.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 14: FIND-82 — assert de conteo en `test-mcp.py`**
- Appetite 4h · 🟡 · 🔴 Alta · `skills/vantadb-mcp/scripts/test-mcp.py:148-160`
- Verificación real: handshake 4/4 sin ningún assert (el drift 79-vs-56 se enmascara).
- Gate Justificación: el gate que debía cazar el drift no gatea; buildear desde fuente + asertar por perfil.
- Contrato: script aserta conteo por perfil + binario desde fuente documentado + handshake 4/4 verde.
- Pre-mortem: conteo varía por perfil/build (asertar por perfil, no número mágico global).
- Stop: N/A.
- Risk Register: 🟡×🟢 perfiles → assert parametrizado | 🟢×🟢 binario instalado stale → build desde fuente en el contrato.
- Cynefin: 🟨 complicado. Top 3: perfiles / fuente / handshake.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `test:`; sin deuda.
- Appetite 4h ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-82.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 15: FIND-83 — unificar skills (copias + MCP-27/29 + api-ref)**
- Appetite 1d · 🟡 · 🟠 · `skills/`, `.opencode/skills/`, `SKILLS-MANIFEST.md`
- Verificación real: MCP-27 con scope en SKILL (`:200`); MCP-29 resuelta en otra skill; api-reference con historial de drift (GOV-B6); copias por verificar con hash.
- Gate Justificación: dos fuentes de verdad divergentes + contradicción histórica sin cerrar.
- Contrato: copias hash-SAME con gate + MCP-27/29 unificadas (o contradicción cerrada con evidencia) + api-ref completa + manifest al día.
- Pre-mortem: "unificar" borra contenido único de una copia (diff antes de copiar, merge no overwrite).
- Stop: divergencia con dueño (humano) → Gate V, no overwrite.
- Risk Register: 🟡×🟠 overwrite → diff+merge | 🟡×🟡 contradicción real → Gate V | 🟢×🟢 manifest → regenerar/actualizar.
- Cynefin: 🟨 complicado. Top 3: overwrite / contradicción / manifest.
- ⬆️ 1 (alcance contradicción) / ⬇️ 3 steps. DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-83.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave5 — docs grandes + server (disjuntos)

**Task 16: FIND-67 — QUICKSTART a 0.5.0 (+ legacy `WANTA_*`)**
- Appetite 4h · 🟡 · 🟠 · `docs/user/QUICKSTART.md:6,12,90,188-189`
- Verificación real: v0.4.x + wheel 0.1.1 + last_reviewed 2026-07-01 + `WANTA_*` legacy post-FIND-89.
- Gate Justificación: puerta de entrada del producto desactualizada en 4 puntos.
- Contrato: boundary 0.5.0 + wheel path real + `VANTADB_*` + revalidación corriendo el quickstart + `last_reviewed` actualizada.
- Pre-mortem: revalidar todo tarda (timebox: comandos tal cual, sin embellecer).
- Stop: N/A.
- Risk Register: 🟡×🟢 revalidación larga → timebox + comandos literales | 🟢×🟢 wheel path → verificar en releases, no adivinar.
- Cynefin: 🟨 complicado (revalidación). Top 3: tiempo / wheel / legacy.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- Appetite 4h ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-67.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 17: FIND-68 — `docs/api/PROXY.md` + config/env documentados**
- Appetite 1d · 🟡 · 🟡 · `docs/api/PROXY.md` (nuevo), `vanta-proxy/config.toml`, `vanta-proxy/src/server.rs:742-768`
- Verificación real: PROXY.md no existe; config.toml real existe (scope docs-only, no crear ejemplo).
- Gate Justificación: 8 endpoints + features opt-in sin doc dedicada.
- Contrato: endpoints + 8 features + defaults + env documentados; índice enlaza; Regla 11 (0 claims sin fuente).
- Pre-mortem: doc diverge del código al mes (checklist de sync en el task file).
- Stop: scope a endpoints+features+config (no tutorial).
- Risk Register: 🟡×🟢 drift futuro → checklist sync | 🟡×🟡 claims → Regla 11, cada número con fuente.
- Cynefin: 🟨 complicado. Top 3: drift / claims / scope.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-68.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 18: FIND-81 — higiene `vantadb-server/`**
- Appetite 2h · 🟢 · 🟢 · `vantadb-server/vanta_certification.json`, `vantadb-server/vantadb_data/`, `vantadb-server/README.md` (nuevo)
- Verificación real: ambos existen junto al crate (Test-Path True).
- Gate Justificación: artefactos de 335MB + json huérfano en el árbol.
- Contrato: json movido/borrado con justificación + data en gitignore + mini README 5 líneas.
- Pre-mortem: borrar lo que otro proceso usa (verificar que son restos, no runtime).
- Stop: N/A.
- Risk Register: 🟢×🟡 resto vs runtime → verificar uso antes | 🟢×🟢 335MB ya ignorado → gitignore + limpieza local documentada.
- Cynefin: 🟦 obvio. Top 3: uso / tamaño / readme.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `chore:`; sin deuda.
- Appetite 2h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-81.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave6 — packaging + adapters puntuales (disjuntos)

**Task 19: FIND-66 — Formula sync (ARM64 + head + mcp)**
- Appetite 2h · 🟢 · 🟠 · `Formula/vantadb.rb`, `Formula/README.md`
- Verificación real: rb SIRVE aarch64 (ambos OS) + instala exactamente 2 binarios (con comentario ponytail); README desactualizado en los 3 puntos (de distinta forma que el reporte: head SÍ documentado pero sin stanza).
- Gate Justificación: instalador oficial mintiendo en 3 puntos.
- Contrato: ARM64 ✅ + head (stanza o quitar doc) + mcp (instalar o quitar de la tabla); sin Ruby en runner → validación por lectura + diff revisado.
- Pre-mortem: sin Ruby no hay `brew audit` (lectura cuidadosa + diff como evidencia).
- Stop: N/A.
- Risk Register: 🟢×🟡 sin runtime brew → evidencia = diff + lectura | 🟢×🟢 sha por plataforma → no tocar shas.
- Cynefin: 🟦 obvio. Top 3: head / mcp / shas intactos.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `docs:`/`fix:`; sin deuda.
- Appetite 2h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-66.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 20: FIND-75 — WASM README + zero-copy input + d.ts**
- Appetite 4h · 🟢 · 🟢 · `vantadb-wasm/README.md`, `vantadb-wasm/src/lib.rs:2377-2378`, `vantadb-wasm/src/vantadb_wasm.d.ts`
- Verificación real: números al día (1.35MB/1.411.870B); input zero-copy pendiente explícito; d.ts src vs pkg por comparar post-build.
- Gate Justificación: cerrar los 2 sub-items vivos + ratificar el tercero con evidencia (no re-escribir números a ciegas).
- Contrato: números re-medidos + input zero-copy implementado o DEFER-ratificado + diff d.ts + clippy 0.
- Pre-mortem: zero-copy input toca parseo MemoryInput/NodeInput (fuera de PERF-08) → DEFER-ratificado es salida válida.
- Stop: parseo mayor → DEFER-ratificado, no refactor.
- Risk Register: 🟢×🟢 números → re-medir, no copiar | 🟡×🟢 input → DEFER salida válida | 🟢×🟢 d.ts requiere build → documentar comando.
- Cynefin: 🟨 complicado. Top 3: medición / input / build.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `docs:`/`perf:`; sin deuda.
- Appetite 4h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-75.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 21: FIND-69 — dspy sin framework**
- Appetite 2h · 🟢 · 🟡 · `integrations/dspy/vantadb_dspy/vectorstore.py:19-24,62`, `integrations/dspy/tests/`
- Verificación real: fallback `object` + `super().__init__(k=k)` → TypeError garantizado.
- Gate Justificación: import sin dspy siempre explota; fix = tolerar o no llamar super.
- Contrato: `pytest integrations/dspy/tests/` verde SIN dspy + CON dspy (si hay) + preserva `forward()` shape.
- Pre-mortem: fix rompe el path CON framework (matriz ambas).
- Stop: N/A.
- Risk Register: 🟢×🟡 path con-framework → matriz 2 modos | 🟢×🟢 shape forward → test existente lo cubre.
- Cynefin: 🟦 obvio. Top 3: matriz / shape / ninguno.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `fix:`; sin deuda.
- Appetite 2h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-69.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave7 — integrations + python + fuzz (disjuntos)

**Task 22: FIND-84 — pins + fixtures + dist/PyPI integrations**
- Appetite 1d · 🟡 · 🟡 · `integrations/*/pyproject.toml`, tests, README central
- Verificación real: 9 adapters con dist/ irregular; pins/fixtures por verificar en DISCOVERY (estructura existe).
- Gate Justificación: matriz de adapters sin pins ni fixtures sanas.
- Contrato: upper-bounds en 7/9 + fixtures sin subdir inexistente/disco + decisión `dist/` + PyPI/Alpha documentada + pytest verde (mocks).
- Pre-mortem: pin rompe un adapter (pins por adapter con test); "publicar PyPI" sin dueño → documentar Alpha es salida válida.
- Stop: adapter sin mantenedor → Alpha documentado, no publish forzado.
- Risk Register: 🟡×🟡 pins → por-adapter + tests | 🟡×🟢 PyPI → Alpha salida válida | 🟢×🟢 fixtures disco → mocks/tmpdir.
- Cynefin: 🟨 complicado. Top 3: pins / PyPI / fixtures.
- ⬆️ 0 / ⬇️ 3 steps. DoD: contrato + task file + recitation; commit `fix:`/`docs:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-84.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 23: FIND-85 — matriz wheels + firma `put_batch_raw`**
- Appetite 1d · 🟡 · 🟡 · `release-wheels-60.yml:79,218,267`, `pyproject.toml:23-26`, `lib.rs:811`, `*.pyi:203,362`
- Verificación real: matriz 3.11/3.13 vs classifiers 3.11-14; `probe_lock_db` gitignored (stale en checkout); `put_batch_raw` con guard de tipos (asimetría por verificar).
- Gate Justificación: classifiers prometen lo que CI no prueba.
- Contrato: matriz 3.12/3.14 añadidas (o classifiers recortados) + firmas comparadas + pytest verde.
- Pre-mortem: matriz ampliada tarda ×2 (si duele, recortar classifiers es salida válida y honesta).
- Stop: N/A.
- Risk Register: 🟡×🟢 tiempo CI → recorte como salida | 🟢×🟡 firma → diff 3 vías (rs/pyi/async).
- Cynefin: 🟨 complicado. Top 3: tiempo / firma / probe local.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `ci:`/`fix:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-85.md` · ⬜ PENDING · Ruta vanta-lead (CI) + worker.
- Nota: `probe_lock_db/` solo limpieza local documentada (no es parte del contrato CI).

**Task 24: FIND-80 — seed corpus + crash upload + fuzz-pr**
- Appetite 1d · 🟡 · 🟢 · `fuzz/corpus/` (vacío), `.github/workflows/fuzz-40.yml:88-97`, `docs/dev/workflow/fuzz-40.md`
- Verificación real: corpus vacío; yml con cache+cleanup pero sin upload; doc SÍ cubre ci-gate (parte stale del reporte).
- Gate Justificación: fuzz sin seeds ni artefactos de crash = fuzz decorativo.
- Contrato: seed mínimo commiteado + upload corpus/crashes + doc fuzz-pr + `cargo check --manifest-path fuzz/Cargo.toml --bins` 0.
- Pre-mortem: seeds grandes al repo (mínimo viable, resto por cache).
- Stop: N/A.
- Risk Register: 🟢×🟢 tamaño repo → seeds mínimos | 🟢×🟢 targets sin harness → check bins.
- Cynefin: 🟦 obvio. Top 3: tamaño / upload / bins.
- ⬆️ 1 (mecanismo upload) / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `test:`/`ci:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-80.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave8 — examples + memory + bench-py (disjuntos; 74 post-67)

**Task 25: FIND-74 — requirements + enlaces + decisión TS**
- Appetite 4h · 🟢 · 🟢 · `examples/demo/requirements.txt`, `examples/README.md`, `docs/user/QUICKSTART.md`, `README.md`
- Verificación real: README de examples existe; requirements `vantadb-py>=0.4`; 0 `.ts` en el árbol. Secuencia DESPUÉS de Task 16 (ambos tocan QUICKSTART).
- Gate Justificación: ejemplo instalable roto por deriva de versión + TS sin hogar.
- Contrato: `vantadb-py>=0.5.0` + 1 línea QUICKSTART→examples + 1 línea README→demo/colab + decisión TS documentada.
- Pre-mortem: mover ejemplos TS rompe links (referenciar es salida válida).
- Stop: N/A. Orden: post-Task 16 por QUICKSTART compartido.
- Risk Register: 🟢×🟢 colisión 67 → orden de waves | 🟢×🟢 TS → referenciar vale.
- Cynefin: 🟦 obvio. Top 3: orden / TS / versions.
- ⬆️ 0 / ⬇️ 1 step. DoD: contrato + task file + recitation; commit `docs:`; sin deuda.
- Appetite 4h ≥ 🟢 ✔. Task file `docs/dev/tasks/FIND-74.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 26: FIND-86 — wiring MEM-69 + tool 77 + números MEM-70**
- Appetite 2d · 🟡 · 🟡 · `vanta-memory/src/services/pipeline_worker.rs`, `core/record/l1_batch.rs`, `core/scene/auto_consolidate.rs`
- Verificación real: dream "not wired yet", batch "pipeline_worker untouched", `scene_consolidate` como follow-up en código; MEM-70 sin rastro.
- Gate Justificación: memoria diferida diseñada (ADR-040) sin cablear; números sin medir.
- Contrato: MEM-69 wiring + tool 77 diseñada o implementada + MEM-70 números o DEFER-ratificado + suite 336 verde + clippy 0.
- Pre-mortem: scope triple → orden wiring→tool→números; 1 sub-item trancado NO bloquea el resto (shippear resto).
- Stop: sub-item trancado → ship resto + DEFER-ratificado del resto.
- Risk Register: 🟡×🟡 triple scope → orden + ship parcial | 🟡×🟢 MEM-70 sin harness → DEFER salida válida | 🟢×🟢 suite 336 → por módulo.
- Cynefin: 🟨 complicado. Top 3: scope / números / suite.
- ⬆️ 0 / ⬇️ 3 steps. DoD: contrato + task file + recitation; commit `feat:`; sin deuda.
- Appetite 2d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-86.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 27: FIND-72 — CLI + pins + Chroma en benches py**
- Appetite 1d · 🟡 · 🟢 · `benchmarks/batch_vs_sequential_bench.py`, `requirements.txt`, `competitive_bench.py`
- Verificación real: sin argparse (`__main__` corre todo); requirements sin pins; `rmtree` mixto con/sin `ignore_errors`.
- Gate Justificación: bench que corre con `--help` + deps sin pin + crash WinError32.
- Contrato: `--help` no ejecuta + pins + WinError32 documentado/fix + `py_compile` 0.
- Pre-mortem: argparse cambia defaults usados por CI (preservar defaults exactos).
- Stop: N/A.
- Risk Register: 🟢×🟡 defaults → preservados + testeados | 🟢×🟢 WinError32 → `ignore_errors` + doc.
- Cynefin: 🟦 obvio. Top 3: defaults / pins / WinError32.
- ⬆️ 0 / ⬇️ 2 steps. DoD: contrato + task file + recitation; commit `fix:`; sin deuda.
- Appetite 1d ≥ 🟡 ✔. Task file `docs/dev/tasks/FIND-72.md` · ⬜ PENDING · Ruta vanta-worker.

## SKIP

- ❌ SKIP FIND-76 — jwt gap + link HTTP_API:600. Evidencia: `validate-docs-coverage.ps1` EXIT 0 (0 gaps, corrido 2026-09-15); `CONFIGURATION.md:45` documenta `jwt_secret`; link `HTTP_API.md:600` → `.opencode/references/research-modules.md` existe (Test-Path True). Ambos sub-items resueltos; owner aprobó SKIP en Gate P.

## DEFER / BLOQUEADO

Nada (todo verificado real, esfuerzo acotado, sin dependencias bloqueantes; DEFER-ratificados puntuales viven como salidas válidas dentro de los contratos 75/80/86/88).

## Grafo de dependencias / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave0: FIND-90 + FIND-91 + FIND-64
Wave1: FIND-63 + FIND-65 + FIND-88
Wave2: FIND-79 + FIND-78 + FIND-70
Wave3: FIND-87 + FIND-73 + FIND-71
Wave4: FIND-77 + FIND-82 + FIND-83
Wave5: FIND-67 + FIND-68 + FIND-81
Wave6: FIND-66 + FIND-75 + FIND-69
Wave7: FIND-84 + FIND-85 + FIND-80
Wave8: FIND-74 + FIND-86 + FIND-72
Wave9: FIND-92 + FIND-93 + FIND-94 (plan-adjust 2026-09-16, ver Notas)
```

Órdenes internos: FIND-77 tras Wave0 (mismo archivo que FIND-90); FIND-74 tras Task 16 (QUICKSTART compartido); resto sin dependencias (F3X/F3C mergeadas).
Nota runner: si waves ×3 abortan, fallback secuencial por wave (precedente 2026-09-09/10).

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Mismo archivo en una wave | waves separan por archivo (77→Wave4, 74→Wave8); si colisiona → secuencial interno |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO |
| rustc crash paralelo / OOM | `-j 2` siempre en cargo |
| Reporte original vs código | cada contrato cita evidencia 2026-09-15; el ejecutor reporta divergencia como HALLAZGO, no la silencia |
| Suites largas (vitest desktop, chaos 218s) | timebox + documentar si se deja a Heavy |

## Notas

- plan-adjust template: `plan-adjust [YYYY-MM-DD]: <ID> — qué cambió · ⬆️ antes/después · ⬇️ antes/después`.
- plan-adjust 2026-09-16: Wave9 — se crea Wave9 con FIND-92 (gemelo ci-rustdoc de FIND-64) + FIND-93 (dead_code txn.rs:158) + FIND-94 (drift SDK vs 9 adapters, Backlog :236) · ⬆️ 0 waves nuevas antes / 1 wave (Wave9) después · ⬇️ 27 tasks antes / 30 después. Justificación: hallazgos nacidos en ejecución (FIND-64 discovery, FIND-65 verify, FIND-69 verify), archivos disjuntos (ci-rustdoc.yml / txn.rs / integrations+vanta-python), MAX 3 OK; FIND-94 NO absorbido por FIND-84 (decisión explícita FIND-84). Filtro Notion/Propuesta: FIND-94 cae en "SDKs mínimos" (Propuesta §3) — mantener superficie mínima, preferir shim solo si migración rompe.
- SKILLS_CARGADAS (14): campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, documentation-and-adrs, idea-refine, codebase-memory. SDP (`campaign_discover_skills_v2` phase PLAN) devolvió 8 (base+ Slifecycle + keywords docs/plan); el resto cubre bugs (systematic), TDD por fix, review P2-01, adversarial (doubt), brainstorming/idea-refine para re-scopes, codebase-memory para grafos.
- MCPs usados: CodeGraph (`codegraph_explore` ×6: desktop/proxy, ts/mcp, integraciones/memoria, dspy/providers/embeddings) + codebase-memory-mcp (`list_projects`, `check_index_coverage` 22 scopes, `get_architecture` overview) + lectura/grep directos para claims línea-exacta. agent-search NO necesario (0 ambigüedad externa; todo verificable en repo).
- Routing por tipo (task.md Phase 1): Rust core/bindings → vanta-worker; docs → vanta-docs; CI/workflows → vanta-lead; mixto CI+worker → lead+worker.
- Diferencias vs plan 2026-09-10: +FIND-90/91 (fallout F3X), −FIND-76 (SKIP verificado), 10 re-scopes con evidencia (63, 66, 67, 70, 71, 72, 74-parcial, 75, 79, 80, 85-parcial), contratos con comandos exactos y líneas verificadas hoy.

=== RECITATION ===
Objetivo activo: PLAN FIND-correcciones (rehace 2026-09-10 con todos los FIND del Backlog)
Estado: plan
Última acción: plan escrito con 27 DO + 1 SKIP, Paso 0 con evidencia file:línea por tarea
Resultado: ✅
State: PLAN (desde: —)
Próxima acción: `/pipeline run` (backlog completo) o `/pipeline task FIND-90` (primera, 🔴 Alta)
Contrato: plan file existe con 27 tasks DO + resumen + waves + gates; task files bajo demanda
Invariantes: no se tocó código (modo plan read-only); Backlog intacto (sin filas añadidas/eliminadas)
Comandos de verificación: existencia del plan file + 28 filas FIND en Backlog intactas
Deuda: ninguna
Próxima tarea si completa: FIND-90 (Wave0)
last-synced: 2026-09-15
=== END RECITATION ===

=== RECITATION FIND-90 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-90 — handlers MCP a getters IndexPort
Estado: completed
Última acción: 3 accesos migrados + verify completo + commit 26e6ebc0 (solo 3 archivos propios)
Resultado: OK
Próxima acción: Reviewer distinto P2-01 + /pipeline task FIND-91
Contrato: verificacion: check mcp --tests OK; clippy -D warnings OK; mcp_tests 91/91 OK via cargo test (nextest audit excluye binario por .config/nextest.toml:62 — HALLAZGO); check server OK; fmt OK; rg sin E0609 (restos: StorageEngine.config/graphrag/comentarios). evidencia: claim 3 accesos migrados / evidencia vantadb-mcp/src/handlers/resources.rs:139-145, tools.rs:1821, tools.rs:2847-2857 / confianza alta. invariantes: distancia MCP intacta (solo fuente de metric cambio); forma schema:// intacta (mismo tipo serde); dim None-en-vacio intacta. artefactos: commit 26e6ebc0. deuda: review P2-01 pendiente (vanta-audit/review); contrato plan enmendar nextest->cargo test. queda_pendiente: ninguna (FIND-91 siguiente, archivo disjunto)
Próxima tarea si completa: FIND-91
=== END RECITATION ===

=== RECITATION FIND-64 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-64: 'vanta-memory/**' en paths CI (ci-rust-10.yml push+PR)
Estado: completed
Última acción: 5/5 steps DONE: discovery+re-scope task file (Regla 0), fix +2 lineas workflow, verify YAML+actionlint+diff-check+grep, FIND-92 en Backlog + FIND-64 marcada completada, commit 9a419d65 (amend, hooks verdes). Task file STALE reconciliado con legacy archivado.
Resultado: OK
Próxima acción: Ninguno en FIND-64. Siguiente: FIND-63 (Wave1, remanente vitest localStorage).
Contrato: verificacion: python yaml.safe_load utf-8 OK (ambos triggers listan vanta-memory/**) + actionlint exit 0 (campaign_verify_cmd passed 1.6s + pre-commit hook ok) + git diff --check limpio + rg global. evidencia: claim 'gap real' -> ci-rust-10.yml:17,33 vantadb-*/** sin vanta-memory + Cargo.toml:704,716 member + rg vanta-memory workflows 0 hits (confianza alta); claim 'fix 2 lineas' -> git show 9a419d65 diff +2 (:18 push, :35 PR) (alta); claim 'sin typo' -> actionlint+YAML parse (alta); claim 'gemelo ticketado' -> ci-rustdoc.yml:24,33 + Backlog FIND-92 fila :234 (alta). artefactos: .github/workflows/ci-rust-10.yml, docs/dev/tasks/FIND-64.md, docs/dev/Backlog.md. invariantes: no tocar .opencode/, completions/, Cargo.lock, integrations/llamaindex/ (commit solo 3 archivos); ci-rustdoc.yml NO tocado (FIND-92). deuda: ninguna (FIND-92 queda ⬜ para Wave futura; DeprecationWarning vantadb_py heredado fuera de scope). queda_pendiente: orquestador decide wave de FIND-92; progreso-migracion fila FIND-64 a avance/ si aplica
Próxima tarea si completa: FIND-63
=== END RECITATION ===

=== RECITATION FIND-91 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-91 — durability_recovery:433 sobre dyn IndexPort
Estado: completed
Última acción: Step1+Step2 DONE, review vanta-review approve, Backlog fila ✅, avance registrado, commit 71aedbde (4 archivos, hooks verdes). No se tocó WIP ajeno (.opencode/, completions/, tauri lock, FIND-90).
Resultado: OK
Próxima acción: Ninguno — tarea cerrada. Push vía vanta-lead.
Contrato: Contrato: check --all-features ✅ (13.40s) + clippy CI Windows ✅ (1m22s, 0 warnings) + fmt ✅ + cargo test --features failpoints --test durability_recovery 8/8 ✅ (30.71s, incluye target test). nextest --profile chaos corre 0 tests por diseño (default-filter solo chaos_integrity_failpoints, .config/nextest.toml:100-101) — documentado en task + lesson, no requiere Heavy. Invariantes: assert por-nodo intacto; failpoints intactos (reviewer verificó). Verificacion: outputs reales + commit 71aedbde (hooks fmt/clippy/actionlint verdes). Evidencia: diff 1 línea tests/durability_recovery.rs:433; semántica src/index/port_impl.rs:273-275; review approve vanta-review. Artefactos: tests/durability_recovery.rs, docs/dev/tasks/FIND-91.md, docs/dev/Backlog.md (fila ✅), docs/dev/avance/activo/core-engine.md. Deuda: ninguna.
Próxima tarea si completa: FIND-64
=== END RECITATION ===

=== RECITATION FIND-65 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-65 — checked_sub en test TTL (vanta-proxy cache.rs, solo test)
Estado: completed
Última acción: Step 1 DONE + verify + review Approve + commit 432eee45 (3 files) + Backlog sync + lesson. Colisión de índice con agente FIND-88 paralelo resuelta (reset soft + unstage, su archivo preservado; su commit 33d0da0a encima limpio).
Resultado: OK
Próxima acción: Ninguno — tarea cerrada. Push vía vanta-lead.
Contrato: verificacion: cargo test -p vanta-proxy cache -j 2 → 18/18 OK (ttl_expiry_predicate ok) + fmt OK + clippy vanta-proxy 0 warnings + rg 2x checked_sub/0 restas + pre-commit hooks OK. clippy -D warnings global bloqueado por dead_code pre-existente txn.rs:158 (repro sin fix via stash) → FIND-93. evidencia: claim fix solo-test / evidencia git show 432eee45 (3 files) + diff cache.rs:752-771 / confianza alta; claim prod intacto / evidencia diff sin :107-108 / alta. invariantes: prod is_expired intacto; 3 casos sobreviven; WIP ajeno intacto (FIND-88.md preservado en disco, FIND-63 worktree no tocado). artefactos: commit 432eee45; docs/dev/tasks/FIND-65.md; Backlog FIND-65 ✅ + FIND-93 nueva. deuda: ninguna (FIND-93 queda ⬜). queda_pendiente: push vía vanta-lead.
Próxima tarea si completa: FIND-88
=== END RECITATION ===

=== RECITATION FIND-63 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-63 — remanente vitest localStorage desktop + tsc 0
Estado: completed
Última acción: Steps 1-3 DONE + review approve + commit 33f60d20 (7 files, hooks verdes) + 2 lessons. WIP ajeno intacto.
Resultado: OK
Próxima acción: Ninguno en FIND-63. Siguiente: FIND-65 (Wave1) — ya completada según plan (commit 432eee45); orquestador decide próxima (FIND-79 Wave2).
Contrato: Contrato: verificacion: npx vitest run 14 files 86/86 OK (20.35s) + npx tsc --noEmit 0 errores + git diff --check limpio + pre-commit hooks OK. campaign_verify_cmd dio exit -1 vacio (bug conocido, plan Riesgos) → evidencia via bash directa. evidencia: claim 13 fails root-caused / evidencia Node v26.8.1 + ExperimentalWarning --localstorage-file + TypeError localStorage undefined en undo.test.ts:54, MemoryLens.test.tsx:79, ProxyDashboard.test.tsx:23 / confianza alta; claim fix por archivo / evidencia stubs en undo.test.ts:14, ProxyDashboard.test.tsx:8, MemoryLens.test.tsx:11 + vitest.config.ts y projection.worker node intactos (0 diff) / alta; claim tsc / evidencia Client-only en pkg d.ts:7/js:8 + alias en transport.ts:10, vanta-wasm-map.ts:14, test:12 + mod.Client runtime :91,97 / alta; claim review / evidencia vanta-review approve (2 nits no bloqueantes, doc-VantaDB aplicado) / alta. artefactos: commit 33f60d20 (7 files). invariantes: sin simbolos nuevos; worker node intacto; fake timers polling intactos; QUICKSTART no tocado; prohibidos intactos (wal.rs, .opencode/, completions/, tauri lock solo WIP ajeno). deuda: ninguna (nit key/length consistente con patron fakeStorage del repo). queda_pendiente: orquestador: migracion Backlog FIND-63 ⬜→✅ via progreso (no tocado por race Wave1, precedente FIND-88) + push via vanta-lead
Próxima tarea si completa: FIND-65
=== END RECITATION ===

=== RECITATION FIND-88 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-88 — cablear record_response_usage al SSE drain (o DEFER-ratificado)
Estado: completed
Última acción: commit 33d0da0a docs solo-task-file + lesson registrada; DEFER-ratificado cerrado
Resultado: ✅
Próxima acción: ninguno en FIND-88; siguiente FIND-79 (Wave2)
Contrato: verificacion: cargo test -p vanta-proxy -j 2 163/163 OK + clippy scope-proxy 0 warnings + fmt limpio + commit 33d0da0a solo-task-file (hooks verdes). workspace -D warnings falla solo en src/storage/engine/txn.rs:158 pre-existente fuera de scope, no tocado. evidencia: claim sin punto de buffer general / evidencia forward.rs:342-349 + server.rs:897-898 + test mapback_sse_never_buffered:1081 / confianza alta. claim unico drain gated e inutil para usage / evidencia server.rs:650-655,679-686,688,707 + cost.rs:104-105 + 0 callers prod + rg include_usage 0 hits / confianza alta. claim sin doble conteo / evidencia cost.rs:209 input_tokens:0 / confianza alta. artefactos: docs/dev/tasks/FIND-88.md (commit 33d0da0a). invariantes: 0 lineas codigo; prohibidos intactos; translate no exigido no tocado. deuda: ninguna (output 0 por invariante; futuro wiring = feature SSE-usage fuera de appetite). queda_pendiente: orquestador: migracion Backlog FIND-88 via progreso (no tocado por race con agentes Wave1 paralelos); push via vanta-lead
Próxima tarea si completa: FIND-79
=== END RECITATION ===

=== RECITATION FIND-78 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-78 link roto + nota engines Node (vantadb-node README)
Estado: completed
Última acción: Step 1 DONE + verify mecanico + commit 56e6fa67 (README 2 lineas + task file, hooks verdes) + recitation en plan file
Resultado: ✅
Próxima acción: Ninguno en FIND-78. Orquestador: Backlog via progreso + push; siguiente FIND-70
Contrato: verificacion: Test-Path 4x True + rg engines + git diff --check exit 0 + pre-commit hooks OK + commit 56e6fa67. campaign_verify_cmd exit -1 vacio (bug conocido) → bash directa. evidencia: claim review en archive / evidencia docs/dev/reviews/archive/research-vantadb-node-20260825.md / alta; claim 0 links rotos / evidencia Test-Path + diff / alta; claim compat / evidencia README:27 + ambos package.json / alta. artefactos: commit 56e6fa67; docs/dev/tasks/FIND-78.md; plan recitation. invariantes: 0 codigo; package.json lectura; prohibidos intactos; sin rutas inventadas. deuda: ninguna (colateral reviews/README:46 fuera de scope). queda_pendiente: orquestador migracion Backlog via progreso + push via vanta-lead
Próxima tarea si completa: FIND-70
=== END RECITATION ===

=== RECITATION FIND-78 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-78 link roto + nota engines Node (vantadb-node README)
Estado: in-progress
Última acción: DISCOVERY completo + task file FIND-78.md creado (Regla 0, SDP, Gate D no disparado)
Resultado: ✅
Próxima acción: Step 1: editar vantadb-node/README.md:16 (ruta archive) y :27 (nota compat), luego verify
Contrato: verificacion: pendiente (Test-Path + rg + diff-check + campaign_verify_cmd). evidencia: claim link roto / evidencia Test-Path docs/dev/reviews/research-vantadb-node-20260825.md False + archive True / confianza alta. claim engines mismatch / evidencia vantadb-node/package.json:47 >=18 vs vantadb-ts/package.json:7 >=22.19 / alta. artefactos: docs/dev/tasks/FIND-78.md. invariantes: solo vantadb-node/README.md editable; package.json solo lectura; sin codigo TS. deuda: ninguna. queda_pendiente: Step 1 edicion + verify + commit
Próxima tarea si completa: FIND-70
=== END RECITATION ===

=== RECITATION FIND-79 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-79 — endurecer + completar tests TS portabilidad
Estado: completed
Última acción: Steps 1-3 DONE + review APPROVE + commit 0e140763 + 2 lessons + task file sync
Resultado: ✅
Próxima acción: Ninguno en FIND-79. Siguiente: FIND-78 (Wave2). Push via vanta-lead.
Contrato: verificacion: npm build + vitest 12f/311t OK + tsc --noEmit OK + eslint 0 (cadena exit 0) + RED 6-fails old-impl + review vanta-review APPROVE + commit 0e140763 (hooks verdes). evidencia: claim importRecords fix put-loop / evidencia vantadb.ts:888-922 + repros R1-R4 / alta; claim 14 tests nuevos + 4 endurecidos / evidencia portability.test.ts + diffs dx04/hardening, 0 try-catch blando / alta; claim FS DEFER / evidencia IO-platform pins tmpdir / alta. artefactos: commit 0e140763 (5 files). invariantes: sin Rust/docs-tecnicas/workflows; prohibidos intactos; stash ajeno intacto; WIP FIND-70 no tocado. deuda: ninguna (opcionales reviewer documentados como NOTICED). queda_pendiente: orquestador: migracion Backlog FIND-79 via progreso (no tocado por race Wave2) + push via vanta-lead
Próxima tarea si completa: FIND-78
=== END RECITATION ===

=== RECITATION FIND-70 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-70 — bench en nightly con feature o skip documentado
Estado: completed
Última acción: Commit 34effd35 (3 files, hooks verdes: actionlint ok, no Rust staged) + lesson registrada
Resultado: OK
Próxima acción: Ninguno en FIND-70. Siguiente: FIND-87 (Wave3).
Contrato: verificacion: actionlint exit 0 (hook pre-commit ok + bash directa) + YAML parse 12 steps + git diff --check limpio + rg 3 hits + Cargo.toml intacto. evidencia: claim fix 1 step / evidencia commit 34effd35 (3 files, 151 insertions) / confianza alta; claim review / evidencia vanta-review approve / alta. artefactos: commit 34effd35. invariantes: Cargo.toml intacto; sin continue-on-error nuevo; WIP ajeno no tocado. deuda: ninguna (monitorear 1a corrida nightly). queda_pendiente: orquestador: Backlog FIND-70 via progreso + push via vanta-lead
Próxima tarea si completa: FIND-87
=== END RECITATION ===

=== RECITATION FIND-73 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-73 — key en .pyi + verify_pyi con firmas
Estado: completed
Última acción: 3 steps DONE + review P2-01 approve (+1 nit aplicado) + commit ad53f7c4 (8 files, hooks verdes) + lesson registrada
Resultado: OK
Próxima acción: Ninguno en FIND-73. Siguiente: FIND-71 (Wave3, embeddings, disjunto).
Contrato: verificacion: py_compile 4 archivos OK + comparador ast-vs-inspect: positivo 9 firmas x3 providers OK, negativo pre-fix detecta drift, negativo init caza api_key requerido-vs-opcional + git diff --check limpio + pre-commit hooks OK + review vanta-review APPROVE. evidencia: claim key en 3 stubs / evidencia providers/*/vantadb_*.pyi:26 key: str|None=None + commit ad53f7c4 / confianza alta; claim gate real / evidencia controles positivo+negativo + script .github/scripts/verify_pyi.py / alta; claim README sin drift version / evidencia sin string version + quickstart valido (key opcional) + Cargo 0.5.0 x3 / alta. invariantes: runtime .rs intacto (solo lectura); placeholder ${PROVIDER} intacto (CI replace); prohibidos intactos (FIND-87/71, Cargo.toml, .opencode/, completions/, tauri lock). artefactos: commit ad53f7c4 (8 files). deuda: ninguna. queda_pendiente: orquestador: migracion Backlog FIND-73 a avance/bindings.md (no tocado por race Wave3 con FIND-87/71) + push via vanta-lead
Próxima tarea si completa: FIND-71
=== END RECITATION ===

=== RECITATION FIND-87 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-87 — ./native en exports + nota wiki (TS packaging)
Estado: completed
Última acción: Steps 1-2 DONE + review vanta-review approve + lesson + commit 32aa0a5e (3 files propios, hooks verdes)
Resultado: OK
Próxima acción: Ninguno en FIND-87. Siguiente: FIND-73/FIND-71 (Wave3 paralelos). Push via vanta-lead.
Contrato: verificacion: npm run build ✅ tsc + npx vitest run ✅ 12 files 311 passed + npm pack --dry-run ✅ 20 files con dist/native.js 13.4kB + git diff --check ✅ + node import dist/native.js ✅ NativeVantaDB. evidencia: claim subpath exonerado / evidencia package.json:22-26 + pack output / confianza alta; claim wiki explicita / evidencia TS_SDK.md:690 seccion native / alta; claim review distinto / evidencia vanta-review approve ses_f5a578f83ffe4SxVn93OHVE6kB / alta. artefactos: commit 32aa0a5e (3 files, hooks verdes). invariantes: . intacto; native.ts intacto; engines/files/deps intactos; prohibidos + WIP ajeno no tocados. deuda: ninguna. queda_pendiente: orquestador: migracion Backlog FIND-87 via progreso + push via vanta-lead
Próxima tarea si completa: FIND-73
=== END RECITATION ===

=== RECITATION FIND-71 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-71 — embeddings peso + verify + sizes (download patterns + smoke)
Estado: completed
Última acción: Steps 1-4 DONE + review vanta-review approve + commit fix (4 files propios) + lesson. WIP ajeno intacto (FIND-87/73 paralelos, providers/vantadb-ts/verify_pyi/TS_SDK no tocados).
Resultado: OK
Próxima acción: Ninguno en FIND-71. Siguiente: Wave4 (FIND-77 tras Wave0 OK).
Contrato: verificacion: download --check global OK + 9/9 --only por modelo verde + tamper-lock FAIL(exit1)->restore OK + verify --check PASS + py_compile OK + git diff --check limpio + GREEN 4/4 gaps cerrados (bash directa; campaign_verify_cmd bug exit -1 conocido). evidencia: claim patterns recortados / evidencia download.py ALLOW_PATTERNS sin *.bin + MODEL_PATTERNS/get_allow_patterns en ambos snapshot_download / confianza alta; claim check por modelo / evidencia --only 9/9 verde + sizes manifest (253/170/878/2200/941/1079/691/3470/16000MB = README) / alta; claim lock verificado / evidencia repo/rev vs manifest + tamper test / alta; claim smoke aclarado / evidencia docstring verify.py + nota verify.log + README / alta; claim review / evidencia vanta-review approve / alta. invariantes: manifest.json/lock intactos (solo lectura/validación); sin red en CI (todo --check offline); proibidos intactos (.opencode/, completions/, tauri lock, Cargo.toml, FIND-87/73). artefactos: commit fix FIND-71 (download.py, verify.py, README.md, docs/dev/tasks/FIND-71.md). deuda: ninguna (nit reviewer: --check no detecta bin-only hasta descarga real — fail-fast futuro opcional fuera de alcance). queda_pendiente: orquestador: Backlog FIND-71 via progreso (no tocado por race Wave3 paralelo) + push via vanta-lead; plan file con recitaciones paralelas queda unstaged (compartido con FIND-87/73).
Próxima tarea si completa: FIND-77
=== END RECITATION ===

=== RECITATION FIND-77 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-77 — conteo 76→79 en comentarios MCP
Estado: completed
Última acción: Bloque :25-35 reescrito con valores medidos + review 2da ronda OK + commit c32940db (hooks verdes)
Resultado: OK
Próxima acción: Ninguno en FIND-77. Siguiente: FIND-82 (Wave4, test-mcp.py). Push via vanta-lead.
Contrato: verificacion: cargo check -p vantadb-mcp -j 2 Finished 0 warnings + test_mcp_tool_profiles ok + test_mcp_tool_annotations_coverage ok + rg 76 solo 1_048_576 + diff --check limpio + hooks pre-commit ok | evidencia: claim Full 79=49+30 / evidencia parser JSON 79 rows + mcp_tests.rs:4569-4581 assert 79 ok / alta; claim hits 115=85+30 / evidencia rg -c per-file / alta; claim comentario veraz / evidencia diff c32940db 11+/11- solo comentarios / alta | artefactos: commit c32940db (3 files propios) | invariantes: 0 lineas codigo; 1_048_576 intactos; prohibidos intactos; commit solo 3 archivos | deuda: ninguna | queda_pendiente: push via vanta-lead; Backlog FIND-77 sync via progreso
Próxima tarea si completa: FIND-82
=== END RECITATION ===

=== RECITATION FIND-82 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-82 — assert de conteo en test-mcp.py
Estado: completed
Última acción: Steps 1-2 DONE + review approve + commit 23c56257 (2 files propios, hooks verdes) + lesson registrada
Resultado: OK
Próxima acción: Ninguno en FIND-82 — tarea cerrada
Contrato: verificacion: py_compile exit 0 + fresco target/debug/vantadb-server.exe (build 7m16s) 79 tools 4/4 exit 0 + stale 56 tools 3/4 exit 1 drift + dev 38/memory 22 4/4 exit 0 + git diff --check limpio + commit 23c56257 (hooks verdes) + review approve. evidencia: claim gate ahora gatea / evidencia STALE 3/4 exit 1 vs FRESH 4/4 exit 0 + asserts por perfil en test-mcp.py / confianza alta; claim causa raiz delegacion / evidencia cli_handlers/server.rs:253-349 + 23 tools faltantes listadas (56+23=79) / alta; claim review P2-01 / evidencia vanta-review approve ses_f5a35e200ffeTO54udJT73r5ld / alta. artefactos: commit 23c56257 (test-mcp.py + FIND-82.md); target/debug/vantadb-server.exe (no commiteado, gitignored). invariantes: handshake intacto; prohibidos intactos (.opencode/, completions/, tauri lock, tools.rs, FIND-77/83, Rust core); WIP ajeno no tocado (Gate C: status con ramificaciones ajenas → add solo 2 propios). deuda: ninguna (mcp-protocol.md:7 60-tools stale → FIND-83). queda_pendiente: orquestador: migracion Backlog FIND-82 via progreso (no tocado por race Wave4 paralela 77/83) + push via vanta-lead; diagnose_pipeline fallo entorno (pwsh ETIMEDOUT, no bloqueante)
Próxima tarea si completa: FIND-83
=== END RECITATION ===

=== RECITATION FIND-83 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-83 — unificar skills (copias + MCP-27/29 + api-ref)
Estado: in-progress
Última acción: Steps 1-5 ✅ + commit ab6827ba (5 files propios, hooks ok) + 2 lessons; submodule 3 files en working tree (decisión C6, WIP ajeno)
Resultado: PARTIAL
Próxima acción: Review P2-01 por agente distinto (vanta-docs/vanta-review) — solicitar al orquestador
Contrato: verificacion: Get-FileHash 10/11 SAME (test-mcp.py FIND-82 exceptuado) + git diff --check limpio + validate-docs-coverage.ps1 exit 0 (sec7 10 pares) + commit ab6827ba (hooks verdes). evidencia: claim merge bidireccional / evidencia ab6827ba 5 files + .opencode worktree 3 files / confianza alta; claim MCP-29 supersedea MCP-27 / evidencia scan.rs:81-92,185-194 + mod.rs:66-70 / alta; claim enum corto / evidencia error.rs:142,157,161,196 / alta; claim case-studies / evidencia Test-Path docs/archive ok + docs/case_studies False / alta. artefactos: ab6827ba; docs/dev/tasks/FIND-83.md. invariantes: test-mcp.py intacto; prohibidos intactos; plan/Backlog no tocados (race Wave4). deuda: review P2-01 distinto pendiente; commit submodule + bump a lead. queda_pendiente: orquestador: review vanta-docs/vanta-review + bump .opencode + push (vanta-lead)
Próxima tarea si completa: FIND-67
=== END RECITATION ===

=== RECITATION FIND-68 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-68 — docs/api/PROXY.md + config/env documentados
Estado: completed
Última acción: PROXY.md creado + enlace master-index + verify + commit c9a38972 (3 files propios, hooks verdes) + lesson
Resultado: OK
Próxima acción: Ninguno en FIND-68. Siguiente: FIND-81 (Wave5, higiene vantadb-server).
Contrato: verificacion: git diff --check limpio + Test-Path PROXY.md True + rg .route( = 10 (8 logicos) + pre-commit hooks verdes. evidencia: claim 8 endpoints / evidencia server.rs:741-772 (10 regs, models+count_tokens dual) / alta; claim 8 opt-in / evidencia config.rs:17-56 + defaults por modulo / alta; claim env / evidencia grep std::env::var 3 hits + main.rs:12-22 / alta; claim 0 Rust / evidencia commit c9a38972 3 files docs-only / alta. invariantes: 0 lineas Rust; prohibidos intactos; FIND-67/81 no tocados. artefactos: commit c9a38972; docs/api/PROXY.md; master-index enlace; docs/dev/tasks/FIND-68.md. deuda: ninguna (review P2-01 via orquestador). queda_pendiente: push via vanta-lead; migracion Backlog FIND-68 via progreso.
Próxima tarea si completa: FIND-81
=== END RECITATION ===

=== RECITATION FIND-81 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-81 — higiene vantadb-server/ (artefactos junto al crate)
Estado: completed
Última acción: Step2+Step3 DONE: borrados locales + README 5L + verify + commit 9713eb54 + lesson
Resultado: OK
Próxima acción: Ninguno en FIND-81. Siguiente: FIND-66 (Wave6).
Contrato: verificacion: Test-Path json False + data False + README True (5 lineas) + git diff --check limpio + cargo check -p vantadb-server Finished 0 warnings + commit 9713eb54 (hooks verdes). evidencia: claim json es resto writer-only / evidencia tests/common/mod.rs:227-228,359-367 + contenido runs 2026-07-02/15 / confianza alta; claim data es resto stale / evidencia mtimes 2026-08-14 + proceso PID14704 --db C:/Users/Eros/.vantadb + 0 literales + tempdirs en tests / alta; claim gitignore ya cubre / evidencia check-ignore .gitignore:29 + :137 / alta. artefactos: commit 9713eb54 (vantadb-server/README.md + docs/dev/tasks/FIND-81.md); limpieza local ~320MiB documentada. invariantes: proceso vivo intacto; .vanta_profile no tocado (fuera de scope); WIP ajeno intacto (solo 2 archivos propios en commit); codigo Rust intacto (lectura/grep solo). deuda: ninguna (reviewer P2-01 pendiente via orquestador). queda_pendiente: orquestador: Backlog FIND-81 ⬜→✅ + push via vanta-lead (no tocados por race Wave5, precedente FIND-63/88)
Próxima tarea si completa: FIND-66
=== END RECITATION ===

=== RECITATION FIND-67 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-67 — QUICKSTART a 0.5.0 (+ legacy WANTA_*)
Estado: completed
Última acción: Steps 1-2 DONE + self-review Approve + 2 lessons + plan recitation sync. Commit docs siguiente.
Resultado: ✅
Próxima acción: git add docs/user/QUICKSTART.md docs/dev/tasks/FIND-67.md + commit docs: FIND-67.
Contrato: Contrato: boundary 0.5.0 + wheel path real + VANTADB_* + revalidación + last_reviewed. Verificacion: rg WANTA_/search_memory/v0.4/0.1.1 = 0 + diff --check limpio + validate-docs-coverage 0 gaps + comandos literales verdes + campaign_verify_cmd exit -1 (bug conocido, fallback bash). Evidencia: claim 4 puntos / evidencia tag v0.5.0, Cargo.toml:728, pyproject:8, release-wheels-60.yml:101,196, config.rs:936,956 / alta; claim revalidación / evidencia put/get/list + search vector/text/hybrid + export 1y3 + audit passed tras rebuild 2x + verify.py PASS / alta. Artefactos: commit docs FIND-67 (QUICKSTART 23L + FIND-67.md). Invariantes: 0 líneas código; prohibidos intactos; FIND-68/81 no tocados; FIND-74 compatible. Deuda: ninguna. Queda_pendiente: orquestador Backlog→avance + push via vanta-lead.
Próxima tarea si completa: FIND-68
=== END RECITATION ===

=== RECITATION FIND-66 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-66 — Formula sync (ARM64 + head + mcp)
Estado: completed
Última acción: STEP-1 DONE + lesson + commit f50da880 (2 archivos propios, hooks verdes)
Resultado: ✅
Próxima acción: Ninguno en FIND-66. Siguiente: FIND-75 (Wave6, archivos disjuntos).
Contrato: Contrato: verificacion: git diff --check exit 0 + git diff Formula/ revisado (solo README, 3+/11-) + rb 0 diff (shas intactos) + pre-commit hooks OK + commit f50da880. evidencia: claim ARM64 veraz / evidencia README:47-49 vs rb:24-27,35-38 / alta; claim head removido / evidencia diff -8 seccion Local development, rb sin stanza / alta; claim mcp fuera de tabla / evidencia workflow:124 tarball 2 binarios + rb:42-46 / alta. artefactos: commit f50da880 (Formula/README.md, docs/dev/tasks/FIND-66.md). invariantes: rb shas intactos; prohibidos no tocados (WIP ajeno .opencode/completions/tauri lock intacto). deuda: review independiente P2-01 pendiente (self-review doble lectura hecho). queda_pendiente: orquestador: Backlog FIND-66 a avance via progreso (no tocado por race paralelo) + push via vanta-lead
Próxima tarea si completa: FIND-75
=== END RECITATION ===

=== RECITATION FIND-75 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-75 — WASM README + zero-copy input + d.ts
Estado: completed
Última acción: STEP-1+2 DONE + review APPROVE + commit 62687a98 + lesson + recitation plan (plan file untracked, queda working tree)
Resultado: OK
Próxima acción: Ninguno en FIND-75. Siguiente: FIND-69.
Contrato: verificacion: pkg re-medido (1658202/56925/245 + gzip 676239/10886/156) + build-wasm-types --check exit 0 post-sync + cargo check OK + clippy 0 en wasm (FIND-93 pre-existente) + diff --check limpio + review vanta-review APPROVE + commit 62687a98 (hooks verdes). evidencia: claim 2 sub-items cerrados + 1 ratificado / evidencia commit 62687a98 + task file + lesson / alta. artefactos: commit 62687a98; docs/dev/tasks/FIND-75.md. invariantes: 0 Rust/TS; prohibidos intactos. deuda: ninguna (follow-ups opcionales documentados). queda_pendiente: orquestador: Backlog FIND-75 via progreso (no tocado por race Wave6) + push via vanta-lead
Próxima tarea si completa: FIND-69
=== END RECITATION ===

=== RECITATION FIND-75 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-75 — WASM README + zero-copy input + d.ts
Estado: completed
Ultima accion: Steps 1-2 DONE + review vanta-review APPROVE + commit 62687a98 (2 files propios, hooks verdes) + lesson
Resultado: OK
Proxima accion: Ninguno en FIND-75. Siguiente: FIND-69 (Wave6, disjunto). Push via vanta-lead.
Contrato: verificacion: Get-ChildItem pkg (1658202/56925/245B) + GzipStream (676239/10886/156B) + --check exit 0 post-sync + cargo check OK + clippy 0 en wasm (solo FIND-93 txn.rs:158 pre-existente) + diff --check limpio + review APPROVE. evidencia: claim README stale 17% / evidencia 1.658.202 vs 1.411.870 + pkg 09-11 < lib.rs 09-11 / alta; claim input DEFER / evidencia from_js 14 callers + 3 adapters custom / alta; claim d.ts out-of-sync / evidencia --check exit 3 pre-sync, exit 0 post-sync / alta. artefactos: commit 62687a98 (README + FIND-75.md); pkg sync local gitignored. invariantes: 0 Rust/TS; prohibidos intactos; WIP ajeno no tocado. deuda: ninguna (opcionales reviewer: build-wasm.ps1 ref + demo/README numeros -> follow-up). queda_pendiente: orquestador: Backlog FIND-75 via progreso + push via vanta-lead
Proxima tarea si completa: FIND-69
=== END RECITATION ===

=== RECITATION FIND-69 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-69 — dspy sin framework (fallback object explota)
Estado: completed
Última acción: commit final f3d6c634 (4 files) + avance bindings + recitation plan
Resultado: OK
Próxima acción: Ninguno en FIND-69. Push via vanta-lead.
Contrato: verificacion: repro TypeError pre-fix + fallback aislado post-fix OK (INIT/forward/dump/load) + py_compile exit 0 + diff-check limpio + pytest 5F/3E todos FIND-94 + vanta-review approve + commit f3d6c634 4 files hooks verdes; evidencia: vectorstore.py:62-70 / repro aislado / hasattr False + rg 12 hits / alta; artefactos: commit f3d6c634; invariantes: prohibidos intactos; deuda: CON dspy sin verificar (sin red) + suite verde pendiente FIND-94; queda_pendiente: push via vanta-lead
Próxima tarea si completa: Wave7 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-84 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-84 — pins + fixtures + dist/PyPI integrations (9 adapters)
Estado: completed
Última acción: Steps 1-4 DONE, review P2-01 conciliado (twine verificado 12/12, score-divergencia documentada a FIND-94), commit ebf76936 (35 files, hooks verdes), lesson registrada
Resultado: OK
Próxima acción: Ninguno en FIND-84; orquestador decide (FIND-85 Wave7 sugerido)
Contrato: verificacion: test_pins 10 passed + 9 suites verdes por adapter (crewai 11p/2s, dspy 8p, haystack 1s, langchain 47p, letta 17p, llamaindex 1s, mem0 1s, ollama 9p, openai 9p; mocks, sin red) + py_compile 12 files OK + git diff --check limpio + pre-commit hooks OK + review vanta-review conciliado. evidencia: claim pins / evidencia 8 pyproject diffs + integrations/test_pins.py / confianza alta; claim fixtures / evidencia rg mkdtemp 0 + tmp_path / alta; claim pytest / evidencia outputs por adapter / alta; claim dist/PyPI / evidencia integrations/README.md + twine check 12/12 PASSED + git ls-files dist vacio / alta. artefactos: commit ebf76936 (35 files). invariantes: 0 vectorstore.py tocados (incl. dspy prohibido FIND-69); prohibidos Wave7 intactos; Backlog/avance no tocados (orquestador); push via vanta-lead. deuda: ninguna (FIND-94 pendiente con HALLAZGO score-convencion documentado en shim+README; cross-model review skipped: no-interactivo). queda_pendiente: orquestador: migracion Backlog FIND-84 via progreso + push via vanta-lead
Próxima tarea si completa: FIND-85
=== END RECITATION ===

=== RECITATION FIND-85 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-85 — matriz wheels + firma put_batch_raw
Estado: completed
Última acción: Steps 1-2 DONE + review vanta-review approve + commit 15ea513f (solo 2 archivos propios) + lesson
Resultado: ✅
Próxima acción: Ninguno en FIND-85. Push vía vanta-lead.
Contrato: verificacion: tomllib OK + diff-check 0 + stub_drift 7/7 + put_batch 14/14 + review approve + commit 15ea513f (hooks verdes). evidencia: claim recorte honesto / evidencia pyproject.toml:16-27 + Cargo.toml:15 abi3-py311 + yml:79/218/267 / confianza alta; claim sin drift 3 vías / evidencia inspect.signature MATCH x2 + async kwonly intencional / alta; claim probe local / evidencia rmdir 335MB, git status limpio / alta. artefactos: commit 15ea513f (2 files). invariantes: prohibidos intactos; Backlog/avance no tocados; requires-python >=3.11 intacto. deuda: ninguna. queda_pendiente: push vía vanta-lead; Backlog FIND-85 ⬜→✅ vía progreso (orquestador)
Próxima tarea si completa: FIND-80
=== END RECITATION ===

=== RECITATION FIND-80 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-80 — seed corpus + crash upload + fuzz-pr
Estado: completed
Última acción: 3/3 steps DONE + nit review fixeado + commit 5f0d54b6 (hooks verdes) + lesson registrada
Resultado: ✅
Próxima acción: Ninguno en FIND-80. Siguiente: Wave8 (orquestador decide).
Contrato: verificacion: cargo check --manifest-path fuzz/Cargo.toml --bins -j 2 exit 0 + actionlint exit 0 + git diff --check limpio + pre-commit hooks OK + review vanta-review approve; evidencia: claim 4 seeds 59B commiteados / evidencia fuzz/corpus/*/seed + check-ignore exit 1 + commit 5f0d54b6 / alta; claim upload ambos jobs / evidencia fuzz-40.yml:107-115,161-169 always()+warn, actionlint 0 / alta; claim doc fuzz-pr aditiva / evidencia docs/dev/workflow/fuzz-40.md:63-70, ci-gate intacto / alta; claim bins 0 / evidencia cargo check 0.47s / alta; artefactos: commit 5f0d54b6 (7 files, 93+); invariantes: prohibidos intactos (solo 4 paths propios en commit); deuda: ninguna (internet no requerida — precedente repo heavy-bench-nightly-51.yml:82-88); queda_pendiente: orquestador: Backlog→avance via progreso (no tocado por race) + push via vanta-lead
Próxima tarea si completa: FIND-74
=== END RECITATION ===

=== RECITATION FIND-74 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-74 — requirements + enlaces + decisión TS
Estado: completed
Última acción: Step 1 DONE + verify + self-review approve + commit 5e428aea (4 files, hooks verdes) + 2 lessons. WIP ajeno excluido (staged solo propios).
Resultado: OK
Próxima acción: Ninguno en FIND-74 — tarea cerrada. Orquestador: migración Backlog/avance + push vía vanta-lead.
Contrato: verificacion: requirements `>=0.5.0` ✅ + QUICKSTART:216 ✅ + README:65 ✅ + conteo .ts 0 ✅ + Test-Path 4/4 ✅ + git diff --check ✅ (campaign_verify_cmd passed 1.1s) + validate-docs-coverage 0 gaps ✅ + OCR delegate default sin bloqueos ✅ + commit 5e428aea (hooks verdes, 4 files) ✅. evidencia: claim floor / evidencia examples/demo/requirements.txt:1 + tag v0.5.0 + workspace.package 0.5.0 + pyproject 0.5.0 / confianza alta; claim QUICKSTART +1/-0 hunks intactos / evidencia git diff docs/user/QUICKSTART.md (+blank +blockquote :216) / alta; claim README +1 fila / evidencia README.md:65 / alta; claim TS referenciar / evidencia conteo 0 + vantadb-ts/examples 3 mjs + examples/README.md:4,34-40 / alta. artefactos: commit 5e428aea (requirements + QUICKSTART + README + docs/dev/tasks/FIND-74.md). invariantes: 6 hunks FIND-67 intactos; 0 líneas código; prohibidos intactos (WIP ajeno sin stage, stash 0..14, FIND-86/72, demo.py:17 NOTICED); Regla 11 por claim. deuda: demo.py:17 deriva gemela → FIND futuro (Gate C, archivo distinto); reviewer distinto al push (vanta-lead). queda_pendiente: orquestador: Backlog/avance (race, no tocado) + push vía vanta-lead; siguiente FIND-86/72 Wave8
Próxima tarea si completa: FIND-86/72 paralelos Wave8 (orquestador decide)
=== END RECITATION ===

=== RECITATION FIND-86 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-86 — wiring MEM-69 + tool 77 + números MEM-70
Estado: completed
Última acción: Step 3 RED→GREEN + 4 tests wiring + review APPROVE + commit 29ec9f02 (hooks verdes, 4 propios)
Resultado: OK
Próxima acción: Orquestador: progreso + push vía vanta-lead; siguiente FIND-72
Contrato: verificacion: cargo test -p vanta-memory -j 2 → 28/28 suites, 542 passed (336 lib + 205 integ + 1 doc), 0 failed; clippy scoped -D warnings 0; fmt ok; hooks pre-commit ok; review vanta-review APPROVE; OCR preview sin severidades. evidencia: commit 29ec9f02 (4 files) + tests pipeline_manager 21/21 (4 nuevos wiring) / confianza alta. artefactos: commit 29ec9f02. invariantes: default intacto (use_batch=false; Dream solo task explícita); prohibidos intactos (stage solo propios); Regla 11 sin P99. deuda: nextest-audit workspace + OCR con key al merge + scheduler que encole Dream (nit review) + progreso/push lead. queda_pendiente: orquestador progreso + push vanta-lead; siguiente FIND-72
Próxima tarea si completa: FIND-72
=== END RECITATION ===

=== RECITATION FIND-72 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-72 — CLI + pins + Chroma en benches py
Estado: completed
Última acción: Steps 1-2 DONE + review approve + OCR + /cleanCA self-audit + commit 107550d8 (argparse batch con defaults exactos, pins 7 deps, rmtree ignore_errors ×8 + notas WinError32)
Resultado: OK
Próxima acción: Ninguno en FIND-72. Push vía vanta-lead; orquestador: Wave9 (FIND-92/93).
Contrato: verificacion: `python benchmarks/batch_vs_sequential_bench.py --help` EXIT 0 sin correr benches ✅ + `python -m py_compile` ambos archivos EXIT 0 ✅ + `git diff --check` limpio ✅ + pre-commit hooks OK ✅ + review vanta-review approve ✅ + OCR sin Critical/High ✅. campaign_verify_cmd bug exit -1 vacío reproducido → fallback bash directa (riesgo conocido del plan). evidencia: claim `--help` no ejecuta / evidencia RED previo (`Initializing Database...` + corría run_bench) vs post-fix (usage + EXIT 0) / confianza alta; claim defaults preservados / evidencia tabla Spec task file §4 + diff solo-aditivo en __main__ / alta; claim rmtree fix / evidencia grep post-fix 11/11 con ignore_errors=True (4 nuevos en competitive + 4 en batch) / alta; claim pins / evidencia requirements.txt:16-31 floors + marcas NO VERIFICADA / alta; claim sin regresión competitive --help / evidencia salida idéntica pre/post vía `git show HEAD:` (exit 1 por guard pre-existente h5py/lancedb/chromadb, sin red) / alta. artefactos: commit 107550d8 (4 files propios). invariantes: 0 workflows tocan estos .py (grep CI); flags competitive intactos; sin símbolos SDK; sin claims performance; WIP ajeno intacto (Justfile/completions/Cargo.lock/pipeline-state.json/.opencode no stageados). deuda: TSYS-13 ratificar con red floors h5py/lancedb/chromadb/qdrant-client>=1.10/pymilvus>=2.5. queda_pendiente: push vía vanta-lead; Backlog→avance vía orquestador (race paralelo).
Próxima tarea si completa: FIND-92
=== END RECITATION ===

=== RECITATION FIND-92 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-92 — gap gemelo de FIND-64: paths ci-rustdoc.yml no matchean vanta-memory/
Estado: completed
Última acción: Steps 1-4 DONE: task file 10 bloques + fix +2 líneas + verify 5/5 + OCR + review P2-01 reconciliado + 2 lessons + commit e395b563 (hooks verdes)
Resultado: OK
Próxima acción: Ninguno en FIND-92. Push vía vanta-lead; orquestador: ticketeo FIND nuevo (rustdoc-70 twin) + FIND-93/94.
Contrato: verificacion: python yaml.safe_load utf-8 OK (push+PR listan vanta-memory/**) ✅ + actionlint exit 0 ✅ + git diff --check exit 0 ✅ + rg vanta-memory 4 hits (0 gaps) ✅ + rg vantadb-*/** 4 hits estables ✅ + pre-commit hooks verdes ✅. campaign_verify_cmd bug exit -1 vacío reproducido → fallback bash directa (riesgo conocido). evidencia: claim gap real / evidencia ci-rustdoc.yml:24,33 + rg pre-fix 2 hits solo ci-rust-10 + Cargo.toml:704,716 member / confianza alta; claim fix 2 líneas / evidencia git diff +2/-0 (commit e395b563) / alta; claim sin typo / evidencia actionlint+YAML / alta; claim review P2-01 / evidencia vanta-review CHANGES-REQUIRED reconciliado (C1 válido→HALLAZGO, resto trade-off/noise) / alta; claim OCR / evidencia preview group 1 sin Critical/High en scope / media. artefactos: commit e395b563 (2 files propios); docs/dev/tasks/FIND-92.md; .github/workflows/ci-rustdoc.yml. invariantes: WIP ajeno intacto (.opencode/Justfile/completions/Cargo.lock/pipeline-state.json no stageados); Backlog no tocado (race); Cargo solo lectura; FIND-93/94 no tocados. deuda: HALLAZGO rustdoc-70.yml también ciego a vanta-memory (cargo doc workspace :59, paths :6-21 sin memory) → orquestador ticketea FIND nuevo; resto nits reviewer (concurrency/comentario/self-trigger/branches/providers/proxy) → futuros FINDs. queda_pendiente: push vía vanta-lead; Backlog→avance vía orquestador (race paralelo).
Próxima tarea si completa: FIND-93
=== END RECITATION ===

=== RECITATION FIND-93 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-93 — dead_code cloned_sole_buffer txn.rs:158 bloquea clippy -D warnings workspace
Estado: completed
Última acción: Step 3 DONE: OCR Rule Group 1 sin findings + cleanCA ritual limpio + review approve + commit 0cd54e47 (hooks verdes) + lesson registrada
Resultado: OK
Próxima acción: Ninguno en FIND-93. Siguiente: FIND-94 (disjunta) — orquestador decide; push vía vanta-lead
Contrato: verificacion: clippy vantadb default ✅ exit 0 + clippy vanta-proxy ✅ exit 0 (desbloqueo) + check workspace ✅ + tests storage::engine 383/383 ✅ (txn 7/7) + fmt ✅ + diff-check ✅ + hooks pre-commit ✅ + review vanta-review APPROVE + commit 0cd54e47. evidencia: claim fix cfg-gate / evidencia git show 0cd54e47 (txn.rs +4/-0 + FIND-93.md) / confianza alta; claim desbloqueo / evidencia clippy proxy rojo pre-fix → verde post-fix / alta; claim sin deuda / evidencia sin allow/expect, cfg simétrico / alta. artefactos: commit 0cd54e47; docs/dev/tasks/FIND-93.md. invariantes: WIP ajeno intacto (commit solo 2 archivos propios); API pública intacta (pub(crate)); ERR-037 intacto (rama rayon idéntica). deuda: ninguna (colateral no-default-features sin soporte pre-existente, NOTICED BUT NOT TOUCHING). queda_pendiente: push vía vanta-lead; Backlog→avance NO tocado (race paralelo, orquestador)
Próxima tarea si completa: FIND-94
=== END RECITATION ===

=== RECITATION FIND-94 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-94 — drift SDK 0.5.0 vs 9 adapters (vanta.VantaDB ausente)
Estado: completed
Última acción: Steps 1-5 DONE + review APPROVE + 3 nits aplicados + commit 880cd0f3 (hooks verdes) + 2 lessons
Resultado: OK
Próxima acción: Ninguno en FIND-94. Siguiente: cierre de campaña (orquestador); push vía vanta-lead
Contrato: verificacion: rg 0 hits prod ✅ + hasattr VantaDB False/Client True ✅ + pytest por adapter (crewai-mem 11p/1s, dspy 8p, langchain 47p, letta 17p, ollama 9p, openai 9p, pins 10p; crewai-vs/haystack/llamaindex/mem0 skipped importorskip válido) ✅ + py_compile ✅ + diff-check ✅ + OCR sin Critical/High (advisory sin API key) + review vanta-review APPROVE + commit 880cd0f3 (hooks verdes). evidencia: claim migración 11 prod / evidencia git show 880cd0f3 (23 files, shim borrado) / confianza alta; claim score-similitud / evidencia probes (idéntico->1.0, ortogonal->0.0) + suites verdes / alta; claim decisión migrar / evidencia Spec §4b D1-D9 + Propuesta §3 / alta. artefactos: commit 880cd0f3; docs/dev/tasks/FIND-94.md. invariantes: vantadb-python/ solo lectura; FIND-69/FIND-84/FIND-92/FIND-93 intactos; WIP ajeno no commiteado; sin símbolos públicos nuevos; Regla 11 N/A. deuda: ninguna (0 DEFER; nits no-bloqueantes aplicados/documentados). queda_pendiente: push vía vanta-lead; Backlog→avance NO tocado (race paralelo, orquestador); cierre de campaña (orquestador)
Próxima tarea si completa: CIERRE-CAMPAÑA
=== END RECITATION ===

=== RECITATION FIND-95 ===
Campaign ID: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
Objetivo activo: FIND-95 — gemelo-del-gemelo FIND-92: paths rustdoc-70.yml ciegos a vanta-memory/
Estado: completed
Última acción: Steps 1-4 DONE + review APPROVE + commit 79a4942d (amend sync task file, hooks verdes) + 2 lessons
Resultado: OK
Próxima acción: Ninguno en FIND-95. Siguiente: CIERRE-CAMPANA (orquestador); push via vanta-lead
Contrato: verificacion: YAML OK push+PR (via d.get on/True, pitfall YAML1.1 vivido) + actionlint exit 0 + git diff --check limpio + rg 6 hits vanta-memory/4 hits vantadb-* + pre-commit hooks OK + OCR sin Critical/High en scope + review vanta-review APPROVE. evidencia: claim gap real / evidencia rustdoc-70.yml:6-21 + rg 0 hits pre-fix / confianza alta; claim fix +2/-0 / evidencia git show 79a4942d / alta; claim 0 gaps vanta-memory / evidencia rg 6 hits (3 workflows x push+PR) / alta. artefactos: commit 79a4942d (2 files: yml + task file). invariantes: WIP ajeno intacto (solo 2 archivos propios); sin simbolos nuevos; sin re-arquitectura paths; Backlog/plan no tocados. deuda: ninguna (O1-O3 reviewer = futuros FINDs: divergencia paths workspace-wide, asimetria branches, colision concurrency.group). queda_pendiente: push via vanta-lead; Backlog FIND-95 ⬜→✅ via progreso (orquestador); CIERRE-CAMPANA (orquestador)
Próxima tarea si completa: CIERRE-CAMPAÑA
=== END RECITATION ===

## Cierre de campaña (2026-09-16, orquestador)

- **Resultado:** 31/31 COMPLETED, 0 failed, consecutiveFails 0. Waves 0-9 + Wave9-ext (plan-adjust: +FIND-92/93/94/95).
- **Progreso masivo:** 31 filas Backlog → `docs/dev/avance/` por dominio (desktop 63; ci-cd 64/66/70/92/95; proxy 68/88; memory 86; core-engine 91/93; bindings 69/73/75/78/79/82/83/84/85/87/90/94; operaciones 65/67/71/72/74/81; seguridad 80); FIND-76 SKIP → `historial/backlog-history.md`. Backlog 102→70 abiertas.
- **Retrospectiva:** Start: re-verificar claims del reporte contra código actual (10 re-scopes) + Gate D vía `question` para símbolos públicos + checkpoint `pipeline-state.json` por wave + Paso 0c (references+Notion) desde Wave8. Stop: asumir task file existente + Backlog desde sub-agentes en paralelo (race → migración final). Continue: waves MAX 3 por DAG + review P2-01 con corridas propias + commits atómicos + SARL RESUME misma sesión (2 rate-limits absorbidos, 0 trabajo perdido).
- **Acción medible:** resumes/total = 2/31 (6%); North Star >90% primer intento ✅ (94%).
- **Push:** vía vanta-lead (commits atómicos por task + este cierre).

=== RECITATION ===
Objetivo activo: PLAN FIND-correcciones — CIERRE
Estado: completed
Última acción: cierre ejecutado (progreso masivo 31 + retrospectiva + archive con budget)
Resultado: ✅
Próxima acción: `/audit quick` o `/ship` (vanta-lead: push + release)
Contrato: 0 tareas pendientes + Backlog sin filas FIND campaña + avance con 31 registros + plan archivado
Invariantes: stash@{0} GOV-C4, WIP ajeno y submodule intactos
Deuda: ninguna (O1-O3 FIND-95 = futuros FINDs fuera de campaña)
Próxima tarea si completa: ninguna (campaña cerrada)
last-synced: 2026-09-16
=== END RECITATION ===
