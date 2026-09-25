# Plan de Ejecución: CleanCA Fase 2 — puntos no aplicados de la guía (2026-09-13)

> **Fuente:** `.opencode/references/clean-code-clean-architecture.md` (guía) + 5 digests
> de investigación exhaustiva (2026-09-13) + validación internet (cargo-modules 0.27,
> cargo-coupling, rust-dsm con Ca/Ce/I/A/D + Tarjan; origen Screaming: Uncle Bob
> blog 2011-09-30 + Clean Architecture cap.21 (2017); VSA: Jimmy Bogard 2018;
> consenso híbrido actual: Jovanović 2024, NDepend 2025, DEV 2026 — slices por fuera,
> capas por dentro, migración incremental, nunca big-bang).
> **Alcance:** SOLO lo no aplicado en Fase 1 que es beneficioso (mejora código/estructura,
> optimiza proyecto). Lo verificado-sano no se toca (`server/`, Rust core docs, TS prod).
> **Estado:** ✅ COMPLETED (18/18, 2026-09-13) · **Rama:** `develop`
> **Norma:** guía completa + Apéndice V + `.opencode/commands/cleanCA.md` + §10 Adaptador
> del plan Fase 1 (sin campaign MCP para IDs no registrados; glob-antes-de-Read;
> ≥10 skills vía tool `skill`; verify en bash; no commitea worker).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 18 (S6 con GO explícito humano 2026-09-13 + S3b CacheLayer, slice 2 de S3, registrada 2026-09-13) |
| 🟡 DEFER | 1 (reorg física Screaming — se reactiva como Fase 3 solo con gate §Cierre) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 2 (Config monolítico intencional?, ciclo sdk↔parser alcance exacto) · ⬇️ downhill = 16

## Gate P — triage (beneficio verificado vs costo)

1. Ya-sano o ya-hecho en Fase 1 → fuera (naming, SLAP splits, E1 prod, humble handlers, Dependency Rule dominio).
2. Medición primero: M1 instala los termómetros (rust-dsm + cargo-modules --acyclic) antes de refactors.
3. Esfuerzo 🔴 solo con payoff estructural (S3 TxnManager, S6 operadores) y en waves tardías.
4. Reorg física de carpetas → DEFER (costo alto, beneficio bajo: el ciclo storage↔index y los 30+ usos de `StorageEngine` se moverían, no se eliminarían).
5. `Config` split → solo tras tarea de decisión D0 (puede ser monolito intencional).

## Tasks

### Wave 0 — medición + quick wins + decisiones (disjuntos)

**Task 1: M1 — baseline métricas Ca/Ce/I/A/D + gate ADP en CI**
- Appetite 4h · 🟢 · 🔴 Alta · `src/` (solo lectura + `dev-tools/` + workflow)
- Archivos clave: `src/lib.rs`, `src/storage/engine/mod.rs`, `src/backend.rs`
- Gate Justificación: sin números no hay mejora medible (guía §4.3); rust-dsm da Ca/Ce/I/A/D + ciclos Tarjan en JSON; cargo-modules `--acyclic` vigila ADP. Validado 2026-09-13: cargo-modules 0.27 (`dependencies --lib --acyclic`, un target por vez, reporta primer ciclo) + cargo-coupling (gates `--check --min-grade/--max-circular`; mide marco Khononov, complementa no sustituye) + rust-dsm (CLI Node `node dist/cli/commands.js`, inmaduro 0★ — solo termómetro informativo).
- Contrato: `rust-dsm -f json` archivado en `docs/dev/reviews/` + `cargo modules dependencies --lib --acyclic` verde + job CI informativo (no bloqueante aún) + tabla baseline en task file. Pins: `cargo-modules 0.27 + cargo-coupling <versión fijada en DISCOVERY> + rust-dsm@<commit fijado>`; validar JSON con `jq`; documentar alcance por target (`--lib/--bin/-p`, `--features`, `--cfg-test`) y workspace.
- Task file `docs/dev/tasks/C2M1.md` · ✅ COMPLETED · Ruta vanta-worker.
- Skills (≥10 vía `skill`): campaign-executor, progreso, codebase-memory, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, planning-and-task-breakdown, observability-and-instrumentation.
- MCP/tools: `codebase-memory-mcp_get_architecture`, `check_index_coverage`; bash: `cargo install cargo-modules rust-dsm`, `cargo modules dependencies --lib --acyclic`, `rust-dsm -f json -o docs/dev/reviews/dsm-baseline.json`.
- Dependencias: ninguna (primera). Impacto: cero código (solo tooling + docs); habilita M2/M3/S3 con números.
- Verify: JSON archivado + comando acyclic exit 0 + task file con tabla Ca/Ce/I/A/D por módulo.

**Task 2: C1 — limpieza comentarios (código comentado + redundantes + separadores)**
- Appetite 2h · 🟢 · 🟢 · `src/backends/in_memory.rs:175`, `src/cli_handlers/crud.rs:57`, `src/accumulator.rs:143`, `src/index/flat.rs:57`, separadores en archivos citados
- Gate Justificación: guía §2.4/CC cap.4 (cero código comentado, cero ruido). Validado 2026-09-13: `rg TODO|FIXME|HACK|XXX src/` = 1 (`server/handlers.rs:392`, informativo — exceptuar); código comentado = 0 bloques; separadores `───/──/==` = 100+ en `src/` (no ~35: re-alcance a archivos citados).
- Criterio de borrado obligatorio: solo redundante literal (`// Put` antes de `put`) + separador decorativo + código comentado. Se conservan: `SAFETY/ERR-*/AUDREP-*/ADR/INVARIANT/MEM/ponytail:` y en particular `src/index/search/nearest.rs:55-63` (AUDREP-55, por qué sin fallback), `src/planner.rs:315-319` (MEM-01), `src/index/search/tests.rs:282-285` (conservar texto, solo fix mojibake de encoding).
- Contrato: los redundantes reales borrados + `cargo fmt --check` + nextest módulo.
- Task file `docs/dev/tasks/C2C1.md` · ✅ COMPLETED · Ruta vanta-worker.
- Skills: base 9 + `code-simplification`/`ponytail-review` (mínimo código).
- Verify: diff solo-borrado + fmt + nextest `index::search` + `eviction`/`planner` tocados.

**Task 3: D0 — decisión Config monolítico (research + ADR-datos, cero código)**
- Appetite 3h · 🟢 · 🟡 · `src/config.rs:250-466` (~52 campos pub, no ~60), parseo env `parse_env_or` desde `:538` (continúa tras 743), hot-reload `apply_to` en `:203` (8 campos)
- Gate Justificación: partir un monolito intencional es daño; primero evidencia (quién cambia qué junto: git log por sección) + ADR-datos; la decisión la escribe el humano (Regla 5). Opciones a evaluar: `config-rs` vs `figment` anidado por dominios vs mantener+documentar, con `apply_to` hot-reload como constraint que el split debe preservar.
- Contrato: `docs/dev/tasks/C2D0.md` con tabla cambio-conjunto por sección + recomendación (partir por dominios storage/server/llm/evicción/pool/rbac vs mantener) + `question` al usuario.
- Task file `docs/dev/tasks/C2D0.md` · ✅ COMPLETED · Ruta vanta-arch.
- Skills: base + `documentation-and-adrs`, `doubt-driven-development`.
- Verify: ADR-datos completo + pregunta respondida (BLOQUEA S-split-config futuro, no esta wave).
> **Decisión humana 2026-09-13 (D0): B+B** — split anidado por dominios + fachada plana
> + unificación `VANTA_*→VANTADB_*` breaking en el mismo cambio (`apply_to` como
> constraint; ADR humano por Regla 5). Task futuro S-split-config tras Fase 2.

**Task 4: A1 — documentar boundaries + doctrina híbrida + regla anti-ciclo (cero código)**
- Appetite 3h · 🟢 · 🟡 · nuevo `docs/dev/architecture/BOUNDARIES.md` + diagrama
- Gate Justificación: el digest probó que reorg física no paga; lo que paga es boundary explícito (owners + "search no importa storage directo, solo vía trait" + ciclo storage↔index documentado como deuda con plan). Doctrina validada en internet: híbrido (slices por fuera, capas Clean por dentro, dominio compartido al centro); layout ≠ arquitectura; migración incremental (nuevo en `Features/`, migrar lo viejo solo al tocarlo, estable no se toca); slices no se llaman entre sí (eventos/servicios finos); a `Shared/` solo lo estable+común (se tolera duplicación pequeña).
- Contrato: doc creado + enlazado desde `docs/dev/architecture/` + regla citable por `/cleanCA` futuro (A3) + sección "Fase 3: gate de reactivación de reorg física". Añadir convención `pub use` re-export (conservar API al mover) + regla "ciclos entre siblings = error" para que A3 tenga qué citar.
- Task file `docs/dev/tasks/C2A1.md` · ✅ COMPLETED · Ruta vanta-docs (con revisión vanta-arch).
- Skills: base + `documentation-and-adrs`, `writing-plans`.

### Wave 1 — traits e ISP/LSP (tras M1; disjuntos entre sí)

**Task 5: S1 — segregar `StorageBackend` (12 métodos → roles) + tipar `checkpoint`/`compact`**
- Appetite 1d · 🟡 · 🔴 · `src/backend.rs:131` (`pub(crate)` — riesgo semver MENOR, no major: sin ADR) (+ `in_memory.rs:138`, `fjall_backend.rs:248,258`, `rocksdb_backend.rs:328`)
- Gate Justificación: elimina 1 ISP-grave + 2 LSP de un golpe (digest #1–3); es el contrato más implementado (4 backends) y el más estable.
- Contrato: traits `Snapshotable`/`Scannable`/`Compactable` (nombres a validar en DISCOVERY; `Compactable` no `Compaction`) + `checkpoint()->Result` honesto por backend (in_memory declara no-soportado en tipo, no en runtime) + `compact()->Result<bool|Outcome>` (hoy `->()` no-op) + 4 backends compilan + nextest backends + clippy 0. Sin cambio semántico.
- Task file `docs/dev/tasks/C2S1.md` · ✅ COMPLETED · Ruta vanta-worker. Depende de: M1 (números antes/después).
- Skills: base 9 + `api-and-interface-design`, `test-driven-development`.
- MCP/tools: `codegraph_explore StorageBackend` (callers), `detect_changes`; `cargo check -p vantadb`, nextest backends, `cargo modules dependencies --lib --acyclic` (sin ciclos nuevos).
- Impacto: padres = todo `storage/engine` + `entity` (vía `StorageEngine`); hijos = 4 backends; riesgo = firmas `pub(crate)` → semver menor (verificar en DISCOVERY con 1 línea, sin ADR salvo que algo sea pub).
- Verify:workspace check + suites + acyclic + task file con tabla Ca/Ce/I/A/D después.

**Task 6: S7 — partir `AccessTracker` (lectura vs mutación)**
- Appetite 3h · 🟢 · 🟡 · `src/node/flags.rs:112`
- Gate Justificación: ISP barato (split `AccessStats` + `Pinnable`); quick win que financia Regla 6.
- Contrato: lectores de evicción dependen solo de `AccessStats`; tests evicción verdes.
- Task file `docs/dev/tasks/C2S7.md` · ✅ COMPLETED · Ruta vanta-worker.

**Task 7: M2 — trait `EntityRepository` en `entity/`**
- Appetite 4h · 🟢 · 🟡 · `src/entity/` (6 fich.), consumidores vía `StorageEngine` directo
- Gate Justificación: digest métricas (entity/ D=0.50, A=0.0 — consumidores atados a structs concretos); mismo patrón que backends.
- Contrato: trait puerto + `StorageEngine` lo implementa (o adaptador) + callers migran + nextest entity + sin cambio semántico.
- Task file `docs/dev/tasks/C2M2.md` · ✅ COMPLETED · Ruta vanta-worker. Depende de: M1.

### Wave 2 — OCP + SRP grandes (tras S1)

**Task 8: S2 — registry/factory de backends (elimina `match BackendKind`)**
- Appetite 1d · 🟡 · 🟠 · `src/storage/engine/init.rs:281-301` (construcción), `src/config.rs:661-672` (env `VANTA_BACKEND`). `src/server/handlers.rs:212-216` es solo `backend_label` display — NO entra al registry, solo necesita `as_str()`.
- Gate Justificación: OCP — nuevo backend hoy = editar init probado + N matches; registry lo vuelve aditivo.
- Contrato: `BackendRegistry` (nombre a validar) con registro en composición + init/config consumen registry + nuevo backend ficticio de prueba que no toca init + suites verdes.
- Task file `docs/dev/tasks/C2S2.md` · ✅ COMPLETED · Ruta vanta-worker (diseño previo vanta-arch si DISCOVERY lo pide). Depende de: S1.
- Verify: check + nextest storage + acyclic + test del backend ficticio.

**Task 9: S3 — extraer `TxnManager` (+ `CacheLayer` si cabe) de `StorageEngine`**
- Appetite 2–3d · 🔴 · 🔴 · `src/storage/engine/mod.rs:319` (26 campos contados, no ~30: `next_txn_id/active_txns/txn_buffers`, 3 caches texto; 221 callers codegraph)
- Gate Justificación: SRP god-object, Ca~35/A~0.05 (Zona de Dolor); es la rigidez raíz del sistema.
- Contrato: `TxnManager` extraído con tests propios + `StorageEngine` delega + suites storage verdes + bench **en CI al mergear** (decisión humana 2026-09-13: H2 justifica implementar con tests + partición verificada; el número `canonical_p99` se exige en CI sana antes del merge a main) + M1 post-números (D debe bajar).
- Task file `docs/dev/tasks/C2S3.md` · ✅ COMPLETED · Ruta vanta-worker. Depende de: S1, M1. Slices obligatorios (txn primero, cache después).
- Skills: base + `performance-optimization` (bench), `planning-and-task-breakdown`.
- Verify: nextest storage + bench delta + fmt/clippy + M1 re-medición.

**Task 10: S5 — partir `Executor::execute_statement` (por variante de Statement + helpers de embedding)**
- Appetite 1d · 🟡 · 🟠 · `src/executor.rs:175` (~225 líneas, 7 ramas Select/Query/Insert/Update/Delete/Relate/InsertMessage). Corrección validada: parse vive en `execute_hybrid`/parser y auth-RBAC en `execute_plan:439-451`, NO en esta fn — el split es por variante + `execute_insert`/`embed_if_needed`, no "parse/auth/embedding/mutación".
- Gate Justificación: SRP — 7 ramas con razones de cambio distintas en un fn; helpers por variante con tests por variante.
- Contrato: helpers por variante + tests + suites executor verdes + sin cambio semántico.
- Task file `docs/dev/tasks/C2S5.md` · ✅ COMPLETED · Ruta vanta-worker.

### Wave 3 — tests FIRST/AAA (disjuntos; pueden correr con Wave 2 si hay RAM)

**Task 11: T1 — aislar tests lentos intra-binario (`#[ignore]` + doc)**
- Appetite 4h · 🟢 · 🟡 · `tests/message_thread_test.rs:187` (sleep TTL 2s fijos), `src/index/core.rs:137` (sleep 1000ms barrera stress), `vantadb-ts/.../load.test.ts` (batches 5k/2k, timeout 30s), suites certification. Ya hecho: `test_load.py` (4× `@pytest.mark.slow` + `addopts="-m 'not slow'"` por defecto) y binarios pesados Rust (excluidos por `default-filter` en `.config/nextest.toml`, van a `heavy-certification-50.yml`).
- Gate Justificación: FIRST-Fast — la suite rápida debe correr en segundos siempre; lo lento va a gate separado.
- Contrato: `#[ignore]` en lentos intra-binario (se excluye automáticamente, sin perfil) + doc de cómo correr suite completa (`--run-ignored` / `-m slow` / proyecto vitest separado). Vitest: `projects`/exclude, no tocar `timeout`-vs-`workers` (son ejes distintos).
- Task file `docs/dev/tasks/C2T1.md` · ✅ COMPLETED · Ruta vanta-worker.

**Task 12: T2 — determinismo (seeds + clock inyectable)**
- Appetite 1d · 🟡 · 🟡 · `src/index/core.rs:121` (`rand::rng()` en 2 fns), `message_thread_test.rs:187` (sleep TTL), `test_sdk.py:864` (valor bajo). No tocar: `src/wal.rs:828` (rand solo para nombre temp, inocuo), `mem0/test_vectorstore.py` (fixture `function`-scoped ya aislada; renombrar `test_1..11` es cosmético), `tests/common/mod.rs` (sin estado mutable global).
- Gate Justificación: FIRST-Repeatable/Independent — `sleep(TTL+1)`, `rand` sin seed.
- Contrato: `StdRng::seed_from_u64(42)` (misma toolchain; `ChaCha8Rng` si se exige byte-idéntico cross-versión) + clock inyectable para TTL (NO `TTL=0`, cambia semántica) + suites verdes 3 corridas seguidas.
- Task file `docs/dev/tasks/C2T2.md` · ✅ COMPLETED · Ruta vanta-worker (mem0: con cuidado integrations).

**Task 13: T3 — AAA en load/certification (Act bulk único)**
- Appetite 1d · 🟡 · 🟢 · suites load TS + certification Rust. Reporter ya mitigado (`ProgressDrawTarget::hidden()`, sin `steady_tick`; quedan solo banners `println` por ordenar).
- Gate Justificación: tests legibles = tests que protegen (guía §7.3). Loops act+assert en load son integración legítima (Khorikov): AAA = colapsar a un Act bulk + asserts finales, no partir el loop.
- Contrato: un Act bulk por test + asserts finales + banners ordenados + suites verdes.
- Task file `docs/dev/tasks/C2T3.md` · ✅ COMPLETED · Ruta vanta-worker.

### Wave 3.5 — CacheLayer slice 2 de S3 (sesión propia, tras Wave 3, antes de Wave 4)

**Task 13b: S3b — extraer `CacheLayer` de `StorageEngine` (slice 2 de S3, deuda registrada 2026-09-13)**
- Appetite 1d · 🟡 · 🟠 · `src/storage/engine/mod.rs` (24 campos tras S3-txn `cdb15b3e`) + nuevo `cache.rs`
- Alcance acotado (de `docs/dev/tasks/C2S3.md` Step 2, verificado en DISCOVERY de S3): 5 campos (`volatile_cache`, `text_stats_cache`, `text_ns_cache`, `cardinality_stats`, `cache_warmer`) + probes en `get.rs` (`lookup_volatile_cache`) + inserts en `insert.rs`/apply paths + evicción en `maintenance.rs` + writers `text_index` + `stats`. Tamaño ≈ slice txn → sesión propia, NO junto a otra tarea del mismo `storage/engine/`.
- Patrón obligatorio (mismo que slice txn): `pub(crate)` sellado + delegación 1:1 + `cloned_*` en paths batch (preservar ERR-037 en rayon); sin cambio semántico, sin optimización (hot path — bench en CI al mergear, decisión humana 2026-09-13).
- Gate Justificación: completa la cura del god-object (24→~19 campos) y la baja de D que M1 ratifica; sin este slice la rigidez raíz queda a medias.
- Contrato: `CacheLayer` extraído con tests propios + `StorageEngine` delega + suites storage verdes + `acyclic` sin ciclos nuevos + M1 post-números finales (D debe bajar vs 24 campos) + bench en CI al mergear.
- Task file `docs/dev/tasks/C2S3b.md` · ✅ COMPLETED · Ruta vanta-worker. Depende de: S3-txn (`cdb15b3e`), Wave 3 cerrada (suite rápida+determinista como red). Bloquea: nada de Wave 4 (disjunta), pero debe cerrar antes de Wave 5 (M3/M1-full exigen la D final).
- Skills: base 9 + `performance-optimization` (bench), `planning-and-task-breakdown`.
- Verify: `cargo check -p vantadb --tests --all-targets` + `clippy -D warnings` + `fmt` + `nextest -p vantadb --lib storage --build-jobs 2` + `acyclic` + conteo campos después en task file.
- NOTA RUNNER (`/pipeline run`): ejecutar esta task inmediatamente al terminar Wave 3 (T1+T2+T3 verdes), antes de Wave 4. No saltearla: es deuda con dueño y alcance cerrado, no opcional.

### Wave 4 — docs API + frontend (disjuntos)

**Task 14: C2 — documentar API pública bindings (TS ~11 exports + Python stubs + WASM baseline)**
- Appetite 1–2d · 🟡 · 🟡 · TS: `vantadb-ts/src/errors.ts` (`ErrorJSON`, `DbError`, ctor, `toJSON`, `wrapWasmError`) + `guards.ts` (6 guards sin JSDoc) = gap real ≈11 exports acotados; Python: fijar UN estilo (numpydoc) + `ruff/pydocstyle` en CI + auditar stubs `.pyi` por método; WASM: supuesto "40-50 sin `///`" invertido por spot-check — auditar con `cargo doc --no-deps` por crate en vez de asumir.
- Gate Justificación: guía §2.4 (toda API pública documentada) + Regla 3 (docs/api al día); Rust core ya está al ~100% (no tocar).
- Contrato: `cargo doc --no-deps` con `warn` primero (baseline) y `deny` cuando verde + JSDoc en los 11 exports TS (`stripInternal` solo si hay `@internal` reales) + docstrings numpydoc Python + `validate-docs-coverage` verde.
- Task file `docs/dev/tasks/C2C2.md` · ✅ COMPLETED · Ruta vanta-docs (con vanta-worker para firmas). Slices por binding.
- Skills: base + `documentation-and-adrs`, `writing-guidelines`.

**Task 15: A2 — convención nombres utilities web (prefijo + `--modificador`, sin BEM total)**
- Appetite 4h · 🟢 · 🟢 · `web/src/app/globals.css` (28 clases custom + 10 `@keyframes vanta-*`; `animate-*` envuelve `vanta-*`: documentar, no es bug; `glow-neon` texto vs `glow-box-neon` caja son ejes distintos)
- Gate Justificación: digest recomienda convención ligera (BEM completo no paga en Tailwind); `prefijo--modificador` solo en familias con variantes (`press/glow/animate`).
- Contrato: orphan-check primero (`rg` por clase contra `web/src`+`public/`; candidatas: `speed-lines-radial`, `halftone-fade`, `text-stencil`, `ink-divider`, `stagger-children`, `animated-gradient-border`, `scanlines`, `neon-underline`, `.press` base) + tabla de renames + norma de 10 líneas en `web/` (o `.opencode/rules/frontend-web.md`) + build+lint web verdes + Playwright smoke (los renames rompen `className` dinámicos).
- Task file `docs/dev/tasks/C2A2.md` · ✅ COMPLETED · Ruta vanta-worker (frontend) o docs.
- Verify: `npm run build`, `npm run lint`, Playwright smoke si toca clases usadas en tests.

### Wave 5 — rupturas controladas (tras todo; solo si Waves 1–4 verdes)

**Task 16: M3 — romper ciclos reales sdk↔planner y sdk↔query (re-scope tras M1)**
- Appetite 1d · 🟡 · 🟠 · Ciclos reales: (a) `sdk↔planner` (`planner.rs:23` importa `sdk` Y `sdk/search/{mod,explain,debug_ops,hybrid}.rs` llaman `crate::planner::*` en ~30 sitios); (b) `sdk↔query` (`query.rs:7` importa `sdk::SearchProfileConfig`, `sdk/search/lexical.rs:9` importa `query::RelOp`). El citado `grammar.rs:24→builder.rs:1→planner.rs:23` es unidireccional, NO ciclo — mover solo `SearchProfileConfig` no basta.
- Gate Justificación: ADP — ciclos reales entre módulos; `--acyclic` debe quedar verde.
- Contrato: re-scope exacto en DISCOVERY tras M1 (Tarjan/`--acyclic`); módulo neutral con `SearchProfileConfig`+`SearchProfileMode` (+ evaluar `RelOp`/funciones `planner::*` o inversión de dirección; `Box`/re-export que oculte el síntoma NO vale) + suites parser/sdk/planner verdes.
- Task file `docs/dev/tasks/C2M3.md` · ✅ COMPLETED · Ruta vanta-worker. Depende de: M1 (medir ciclo), A1 (boundaries). BLOQUEADA hasta re-medición.

**Task 17: S6 — cadena `LogicalOperator` extensible (GO explícito humano 2026-09-13, inversión aceptada)**
- Appetite 2–3d · 🔴 · 🟠 · `src/planner.rs:261-308`, `src/executor.rs:175,184`, `src/cost_estimator.rs:201`, `src/physical_plan/*`, `src/query.rs:334,495`
- Gate Justificación: OCP-grave único del sistema (agregar operador = editar 5 archivos probados); GO como inversión aunque no haya operador nuevo a la vista.
- Contrato: nuevo operador registrable sin editar planner/executor probados + operador ejemplo obligatorio + test "nuevo operador sin tocar planner/executor" + suites verdes. Matiz: la capa física extensible YA existe (`PhysicalOperator` Volcano `open/next/close` en `query.rs:495`) — S6 extiende la lógica, no la crea. Precedente YAGNI en contra salvado por tu GO explícito.
- Task file `docs/dev/tasks/C2S6.md` · ✅ COMPLETED · Ruta vanta-arch (diseño) → vanta-worker.

## SKIP / DEFER / BLOQUEADO

- DEFER: reorg física Screaming (costo>beneficio probado — ver gate de reactivación en §Cierre).
- GO explícito humano 2026-09-13: S6 se ejecuta en Wave 5 (aceptado como inversión) + bench-en-CI al mergear para S3/S5 (el número se exige en CI sana antes del merge a main; implementamos con tests + partición verificada).
- BLOQUEADO: split `Config` hasta D0+respuesta humana.
- SKIP: tocar `server/` (sano D=0.05), Rust core docs (~100%), TS prod (`any`=0).

## Grafo / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave0: M1 + C1 + D0 + A1
Wave1: S1 + S7 + M2        (tras M1)
Wave2: S2 + S5 (+ S3 arranca, slices largos)
Wave3: T1 + T2 + T3
Wave3.5: S3b (CacheLayer, sesión propia — INMEDIATAMENTE tras Wave 3, antes de Wave 4)
Wave4: C2 + A2
Wave5: M3 + S6
```

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Traits `pub(crate)` = semver menor | S1 verificado `pub(crate)`: sin ADR/major; verificar en DISCOVERY con 1 línea |
| Hot path (S3, S5) | bench en CI al mergear (decisión humana 2026-09-13); H2 justificado solo para splits mecánicos |
| Estimaciones research (confianza media) | cada ejecutor re-mide en DISCOVERY y reporta divergencia como HALLAZGO (los 5 digests 2026-09-13 ya corrigieron conteos: 26 campos, 12 métodos, 52 campos Config, 100+ separadores) |
| 3 muertes infra en Fase 1 | §10 Adaptador vigente; CARGO_TARGET_DIR separada si hay lock contention; no copiar repos |

## Cierre

`/cleanCA` global (0 🔴) + M1 post-números (D de storage/engine y sdk debe bajar) + retrospectiva + archivar plan + `skill progreso`.

## Cierre (2026-09-13, 18/18 + retrospectiva)

> Verificación de cierre: 18/18 task files con verify verde + 18 commits (uno por tarea:
> `90aeca88`→`c2cdbf3e`; S3-txn = `cdb15b3e`) + `cargo fmt --check` limpio + tree limpio salvo
> ajeno (`.opencode` submodule + `desktop/src-tauri/Cargo.lock`). Etiquetas ⏳/⬜ stale
> flipadas a ✅ en este commit. `skill progreso`: campagne registrada en `docs/dev/avance/`
> + nota en `meta.md`; plan archivado a `docs/dev/plans/archive/`.

### Retrospectiva Start/Stop/Continue
- **Start:** verificar estado real por git+task files antes de despachar (el plan seguía en
  PENDING mientras el trabajo ya estaba commiteado; el lookup MCP devolvía 0 parseables).
- **Stop:** re-despachar waves sin `git log --grep` previo; copiar repos enteros como sandbox.
- **Continue:** waves por DAG + adaptador §10 + commit atómico por tarea + verify del lead.
- **Acción medible:** sincronizar estados PENDING→COMPLETED al commitear cada tarea
  (métrica: 0 divergencias plan-vs-git al cierre; baseline este plan: 19 etiquetas stale).

### DEFER que terminó haciéndose (investigado 2026-09-13)
- **S6 cadena LogicalOperator (Task 17):** nació DEFER-activo ("solo si hay apetito", costo 🔴
  sin operador a la vista) → GO explícito humano 2026-09-13 como inversión → diseño
  `docs/dev/tasks/C2S6.md` (vanta-arch) → implementación slices S6-1/S6-2/S6-3
  (`src/operator_registry.rs` + `dedup.rs` + test extensión 4/4) → commit `c2cdbf3e`.
- **S3 slice 2 CacheLayer:** nació condicional en C2S3.md Step 2 (⬜ PENDING "no cabe en esta
  sesión, deuda registrada") → task propia C2S3b (Task 13b, registrada `5e5761b0`) →
  `src/storage/engine/cache.rs` + 456 storage + 2078 lib + 24→20 campos → commit `836aece3`.
- **Sigue DEFER:** reorg física Screaming (gate §Fase 3, 4 condiciones). **Futuro con dueño:**
  S-split-config (decisión D0 B+B registrada) + bench `canonical_p99` en CI sana al mergear S3/S5.

## Fase 3 (futura, NO parte de este plan): gate de reactivación de reorg física

La mudanza Screaming solo se cotiza si TODO esto es verde tras Fase 2: (1) S3/M2/S1 cerrados
(ventanillas reales, no promesas); (2) M1 re-medido con D de storage/engine y sdk a la baja;
(3) `cargo modules dependencies --lib --acyclic` verde + ciclos sdk↔planner/sdk↔query rotos (M3);
(4) A1 con owners y boundaries firmados. El ciclo storage↔index (bidireccional verificado:
`storage→index` en archive/engine-init-maintenance, `index→storage` en flat/layer/mod/serialize/graph)
NO lo rompe M3 — requiere task propia (trait-split storage↔index) o DEFER explícito con su plan.
Aun así: incremental por dominio (uno por vez, empezando por el más limpio), nunca big-bang;
lo estable no se toca.
