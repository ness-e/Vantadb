# Plan de Ejecución: Backlog Triage 2026-09-07 (bindings + docs + benches + a11y)

> **Campaign ID:** 3d601136-c620-4ec8-947b-f1fbe050471d
> **Inicio:** 2026-09-07
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md
> **Autonomous:** false
> **SPEC:** no existe SPEC.md en raíz/docs/spec — DO set no incluye feature-add greenfield (INTG-01/02, TS-10 diferidos a DEFER); no se genera SPEC en este plan. Si un DO revela feature nueva, se pausa y se genera mini-spec vía spec-driven-development.
> **SDP:** campaign_discover_skills_v2 no invocado por tarea (triage read-only, 99 filas); skills base cargadas: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail (full), spec-driven-development.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 8 |
| 🟡 DEFER | ~75 (splits grandes, PRX/FUT/INTG/STABLE, resto UX/DESKTOP, GOV-TK, MEM, TS-10/11/12/13, WSM-14, WEB-03/09, PROV, BLOG-CTA, OLD-01, BND-08, PERF-BENCH-01, REVIEW-10, FIND-47/48/49/50, MOD-05) |
| ❌ SKIP | 5 (MOD-22, MOD-23, SRV-01, WSM-11, UX-14 — ya implementados, evidencia abajo) |
| 🔴 BLOQUEADO | 5 (MKT-18f humano PyPI, MKT-18i upstream AnythingLLM, AUD-042 upstream tantivy, DESKTOP-41/44 sesión humana/VM, PRX-10 requiere PRX-03) |

Status: ⬆️ uphill = 2 (FIND-60 alcance exacto 49 warnings; UX-A11Y scope 3 superficies) · ⬇️ downhill = 8 (contratos mecánicos definidos)

## Gate P — confirmación usuario (2026-09-07)

Set DO/SKIP/DEFER/BLOQUEADO confirmado vía `question` → "Aprobar plan (Recomendado)". Sin DEFER/SKIP disputados.

## Verificación real global (Paso 0)

- `codegraph_explore llamaindex put_batch` → firma actual `put_batch(keys, vectors, payloads, metadatas, namespace, namespaces, ttls)` (`vantadb-python/src/lib.rs:484`); call-site `integrations/llamaindex/.../vectorstore.py:134` llama `put_batch(entries)` → TypeError real. FIND-64 ✅ real.
- `codegraph_explore GraphBfsResult native` → `GraphBfsResult = bigint[]` (`vantadb-ts/src/types.ts:237`) con nota breaking; `_native` ya `async+await` con wrap (`native.ts:154-163`, comentario TS-02). MOD-22/MOD-23 ✅ ya fixeados → SKIP.
- `codegraph_explore audit` → `AuditLogger::with_rotation` + `DEFAULT_AUDIT_MAX_BYTES 10MiB` + `max_files 5` (`src/audit.rs:15-17,121-204`); `init_audit` usa rotación (`src/sdk/builder.rs:291-295`). SRV-01 ✅ ya implementado → SKIP.
- `codegraph_explore memory_record_to_js` → `record_metadata_drop(1)` + comentario WSM-11 (`vantadb-wasm/src/lib.rs:2355-2363`). WSM-11 ✅ ya implementado → SKIP.
- `codegraph_explore PersonaPanel` → `.catch(onError(vantaErrorMessage))` + comentario UX-14 (`MemoryLens.tsx:173-175`); tabs ya con `role=tablist/tab/tabpanel` + roving tabindex (`:456-486`). UX-14 ✅ ya implementado → SKIP (UX-07 tabs también resuelto).
- `grep mascota_gato|avatar_gato` → 4 refs vivas en `web/src` + evidencia contradictoria de existencia en `public/assets/` (worklog dice copiados, AGENTS/WDA-07 dicen 404) → WEB-03 no se incluye como DO; se verifica como step 0 dentro de WEB-02.
- `grep release-npm|napi|prepublish` en workflows → solo `release-npm-61.yml` + `release-npm-node.yml`, sin pipeline create-npm-dirs/artifacts/prepublish → BND-08 real pero 🔴 1-2sem → DEFER (fuera de appetite de este plan).
- `grep validateVector|_mapRecord|_buildSearchRequest` → duplicación `_mapRecord` en `vantadb.ts:134` + `native.ts:110`, `_buildSearchRequest` duplicado, `validateVector` asserts Float32Array → MOD-24 ✅ real.

## Tasks

### Task 1: FIND-64 — Fix llamaindex put_batch legacy roto

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 2-4h
- **Prioridad:** 🟠
- **Archivos clave:** `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:134`, `vantadb-python/src/lib.rs:484`
- **Verificación real:** ✅ CÓDIGO-REAL — firma nueva con 7 params vs call-site 1 posicional (lista 6-tuplas); roto desde PY-QW2.
- **Gate Justificación:** bug runtime 100% reproducible en adapter oficial; fix acotado a 1 call-site + test; desbloquea ecosistema LlamaIndex.
- **Gate Result:** ✅ DO
- **Contrato:** `python -m pytest integrations/llamaindex/tests -q` pasa + `grep -n put_batch integrations/llamaindex/vantadb_llamaindex/vectorstore.py` muestra kwargs (keys=/vectors=/payloads=/metadatas=/namespace=)
- **Pre-mortem:**
  - **Fallo probable 1:** el adapter soporta versiones viejas de vantadb-py con firma vieja → mitigar con compat shim + pin `vantadb-py>=0.5.0`.
  - **Fallo probable 2:** metadatas 6-tupla legacy no mapea 1:1 a columnas nuevas → lote piloto con fixture real.
  - **Fallo probable 3:** sin test adapter el fix regresa → test de integración obligatorio en el mismo PR.
- **Stop conditions:** appetite >1d → abortar a DEFER; premisa invalidada (firma distinta en develop) → re-triaje.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🔴 | mapping 6-tupla incorrecto | fixture real antes/después | 1 iteración sin green |
  | 🟢×🟡 | compat versión vieja | pin + shim | review |
  | 🟢×🟢 | sin cobertura adapter | test nuevo bloquea merge | CI |
- **Cynefin:** 🟦 obvio — causa-efecto claro (firma vs call-site).
- **Top 3 riesgos:** mapping tuplas; compat; regresión sin test.
- **Uphill/Downhill:** ⬇️ ejecución pendiente (approach conocido).
- **DoD task:** contrato mecánico ✅ · task file sync · recitation actualizada.
- **Shape Up:** ¿problema correcto? sí (adapter roto) · ¿appetite ok? sí (2-4h ≤1d) · ¿ahora? sí (ecosistema).
- **Task file:** `docs/tasks/FIND-64.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 61821573

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 2: MOD-24 — Nits TS agrupados (dedup + guard + JSDoc)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/guards.ts`, `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/native.ts`
- **Verificación real:** ✅ CÓDIGO-REAL — `_mapRecord` duplicado ×2, `_buildSearchRequest` duplicado, `validateVector` asserts Float32Array, JSDoc distance/score; tests `vanta.test.ts:217-236` cubren guard.
- **Gate Justificación:** deuda de contrato SDK (type-lie + duplicación = bugs futuros); acotado a 3 archivos TS sin cambio wire.
- **Gate Result:** ✅ DO
- **Contrato:** `npm ci && npm run build && npx vitest run && npx eslint .` en `vantadb-ts/` → 0 errors; `grep -rn "_mapRecord" vantadb-ts/src --include=*.ts | grep "function _mapRecord"` → 1 definición
- **Pre-mortem:**
  - **Fallo probable 1:** unificar `_mapRecord` cambia mensajes de error que tests asertan → correr tests primero, snapshot de mensajes.
  - **Fallo probable 2:** `validateVector` acepta number[] y rompe zero-copy Float32Array → mantener assert + overload documentado.
  - **Fallo probable 3:** JSDoc examples que no compilan se intentan compilar en CI → marcar como snippets, no doctests.
- **Stop conditions:** >1d o divergencia con MOD-22/23 reabiertos → abortar a DEFER.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | mensajes error snapshot | baseline tests antes | primera corrida |
  | 🟢×🔴 | regresión wire nativo | solo refactor interno, sin tocar wire | review + vitest |
  | 🟢×🟢 | scope creep JSDoc | solo R4#3–#10 listados | diff >5 archivos → parar |
- **Cynefin:** 🟨 complicado — elegir dónde vive el helper compartido requiere criterio (guards vs shared).
- **Top 3 riesgos:** mensajes; wire; creep.
- **Uphill/Downhill:** ⬇️ downhill (steps atómicos claros).
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí (higiene SDK antes de BND-12).
 - **Task file:** `docs/tasks/MOD-24.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 35bb05fb

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** ejecutar antes que BND-12 (base limpia para coverage).

### Task 3: FIND-60 — 49 warnings rustdoc a cero

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 2-4h
- **Prioridad:** 🟡
- **Archivos clave:** `src/sdk/api.rs` + 49 sitios (`cargo doc` reporta)
- **Verificación real:** 🟡 VERIFICAR — reportado por certify 2026-09-03 L5; alcance exacto se confirma en DISCOVERY corriendo `cargo doc --no-deps -p vantadb -p vantadb_py`.
- **Gate Justificación:** gate docs roto (`-D warnings` fallaría); fix mecánico (links `crate::`, fully-qualified, boxear privados); sin cambio de comportamiento.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo doc --no-deps -p vantadb -p vantadb_py 2>&1 | grep -c warning` → 0; `cargo doc --no-deps -D warnings -p vantadb` exit 0
- **Pre-mortem:**
  - **Fallo probable 1:** 49 es número stale (ya bajó/subió) → re-medir en DISCOVERY y re-estimar.
  - **Fallo probable 2:** links a items privados requieren decisión diseño (hacer pub vs boxear) → regla: nunca hacer pub para callar rustdoc; boxear o `no_run`.
  - **Fallo probable 3:** `vantadb_py` doc requiere toolchain Python → si bloquea, scoping solo `-p vantadb` + nota.
- **Stop conditions:** si warnings >100 o exigen rediseño API → DEFER con evidencia.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | conteo stale | re-medir primero | DISCOVERY |
  | 🟢×🟡 | tentación hacer pub | prohibido; boxear | review |
  | 🟢×🟢 | toolchain py | scope a -p vantadb | primer intento |
- **Cynefin:** 🟦 obvio (mecánico).
- **Top 3 riesgos:** conteo; pub-accidental; toolchain.
- **Uphill/Downhill:** ⬆️ uphill: 1 (conteo exacto) · resto ⬇️.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí (quick win gate docs).
- **Task file:** `docs/tasks/FIND-60.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 0f1f5e37

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 4: BND-12 — Cobertura tests vantadb-node 8→~20

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-node/tests/`, `vantadb-node/src/lib.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — research 20260825 H-06 confirma 8 tests; gaps: search/explain_search/put_batch/capabilities/close-drain.
- **Gate Justificación:** binding Node es ruta primaria en Node (decisión PERF-BENCH-01); sin cobertura no se puede publicar (BND-08 bloqueado por calidad).
- **Gate Result:** ✅ DO
- **Contrato:** `npm test` en `vantadb-node/` → ≥18 tests, 0 failed; cobertura de `search`, `explain_search`, `put_batch`, `capabilities`, `close` presente
- **Pre-mortem:**
  - **Fallo probable 1:** tests nativos requieren build napi (toolchain) → usar matriz existente `release-npm-node.yml` como referencia.
  - **Fallo probable 2:** `put_batch` nativo tiene firma distinta a python → leer `lib.rs:117` primero.
  - **Fallo probable 3:** flakies por close-drain → orden determinístico + Issue `flaky` si aparece (Regla 2, nunca `continue-on-error`).
- **Stop conditions:** toolchain napi roto en runner limpio → BLOQUEADO con evidencia, no inventar tests mock.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | toolchain napi | validar build primero | DISCOVERY |
  | 🟢×🟡 | firma put_batch distinta | leer lib.rs:117 | primer step |
  | 🟢×🔴 | flaky close | determinismo + Issue | 2 fallas mismo error → Gate V |
- **Cynefin:** 🟨 complicado (napi + ciclo vida nativo).
- **Top 3 riesgos:** toolchain; firma; flaky.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí (pre-requisito publish).
 - **Task file:** `docs/tasks/BND-12.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 6f46b032

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** depende de MOD-24 (contrato errores estable). Si MOD-24 cambia mensajes, re-baselinear.

### Task 5: BND-13 — docs/api/NODE_SDK.md completa

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 3-4h
- **Prioridad:** 🟡
- **Archivos clave:** `docs/api/NODE_SDK.md` (nuevo), `vantadb-node/` README, `docs/api/`
- **Verificación real:** ✅ CÓDIGO-REAL — H-07 confirma README quickstart existe pero falta doc completa + ejemplos por runtime.
- **Gate Justificación:** sin docs no hay adopción Node aunque BND-12 suba cobertura; doc-only, cero riesgo código; Regla 11 (sin claims sin bench — referenciar TS-09 como pendiente).
- **Gate Result:** ✅ DO
- **Contrato:** `docs/api/NODE_SDK.md` existe con quickstart + matriz native-vs-wasm + ejemplos por runtime; `scripts/validate-docs-coverage.ps1` sin gaps nuevos para `vantadb-node`; sin números de performance sin fuente (grep `ms|QPS|x faster` → 0 o con cita a BENCHMARKS.md)
- **Pre-mortem:**
  - **Fallo probable 1:** documentar API que BND-12 aún no testea → cruzar con `lib.rs` real, marcar experimental.
  - **Fallo probable 2:** claims de performance sin bench → prohibido por Regla 11; matriz honesta.
  - **Fallo probable 3:** duplicar README vs docs/api → README apunta a docs/api como fuente.
- **Stop conditions:** si API nativa cambia bajo los pies (BND-12 en vuelo) → esperar a BND-12 o documentar snapshot con fecha.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | drift con BND-12 | snapshot + fecha | review |
  | 🟢×🟡 | claims sin fuente | Regla 11 checklist | pre-commit |
  | 🟢×🟢 | duplicación | fuente única docs/api | review |
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** drift; claims; duplicación.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation. DoD release: API docs sincronizadas.
- **Shape Up:** sí/sí/sí.
- **Task file:** `docs/tasks/BND-13.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** después de BND-12 (o en paralelo con sync explícito).

### Task 6: TS-09 — Bench reproducible JS/WASM

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/bench/` (nuevo), `docs/operations/BENCHMARKS.md`
- **Verificación real:** ✅ CÓDIGO-REAL — H-11 confirma ausencia de bench JS/WASM; bloquea comparativas vs Orama/vectra y todo claim web (Regla 11).
- **Gate Justificación:** habilita WEB-02 y posicionamiento honesto; dataset determinístico (seed) estilo `canonical_p99`.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run bench` (o script documentado) en `vantadb-ts/` corre insert+search con dataset determinístico y emite p50/p95/p99; resultados anexados a `docs/operations/BENCHMARKS.md` con entorno (CPU/RAM/OS) + comando + fecha
- **Pre-mortem:**
  - **Fallo probable 1:** WASM en Node vs browser divergen → scope inicial Node + nota browser pendiente.
  - **Fallo probable 2:** números no reproducibles (GC/timers) → ≥3 corridas, reportar mediana + varianza.
  - **Fallo probable 3:** tentación de comparativa head-to-head vs Orama → prohibido por D1 (estrategia conservadora); solo números propios.
- **Stop conditions:** varianza >30% entre corridas sin causa → DEFER con evidencia (entorno).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | no-reproducibilidad | seed + 3 corridas | primera corrida |
  | 🟢×🔴 | comparativa prohibida | solo propios (D1) | review |
  | 🟢×🟢 | scope browser | Node primero + nota | plan |
- **Cynefin:** 🟨 complicado (metodología bench).
- **Top 3 riesgos:** reproducibilidad; comparativa; scope.
- **Uphill/Downhill:** ⬇️ downhill (patrón canonical_p99 existe).
- **DoD task:** contrato ✅ · sync · recitation. Regla 9/11: bench + fuente.
- **Shape Up:** sí/sí/sí (desbloquea claims).
 - **Task file:** `docs/tasks/TS-09.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 55ad6488

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | DISCOVERY: task file creado + harness bench.mjs/smoke.mjs + scripts bench/bench:3x (RED Missing script → GREEN) | smoke OK + bench 200×32 emite p50/p95/p99 | node/npm |
  | 2 | 3 corridas 2000×384d×200q + mediana + append §15 BENCHMARKS.md con entorno+comando+fecha | §15 anexado, hype-check 0 nuevos | bench.mjs |
  | 3 | Verify: tsc 0 + vitest 280/280 + eslint 0 errors | verde, sin commit | tsc/vitest/eslint |

  **Notas:**

### Task 7: WEB-02 — Benchmarks propios en /benchmarks

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟡
- **Archivos clave:** `web/src/app/benchmarks/page.tsx`, `docs/operations/BENCHMARKS.md`
- **Verificación real:** ✅ CÓDIGO-REAL — H-10 confirma ruta existe pero sin datos actuales citables; depende de TS-09 para números JS (para números Rust usa BENCHMARKS.md existente).
- **Gate Justificación:** social proof honesto estilo Chroma sin claims de adopción; solo tablas p50/p99 con fuente reproducible.
- **Gate Result:** ✅ DO
- **Contrato:** `/benchmarks` renderiza tablas p50/p99 con cita a fuente (`BENCHMARKS.md` § + comando + fecha); `grep -ri "faster\|ultrafast\|blazing" web/src/app/benchmarks/` → 0 adjetivos sin número; step 0 verifica WEB-03 (assets gato: si `public/assets/mascota_gato.png` falta → quitar refs muertas en el mismo PR, no restaurar binarios)
- **Pre-mortem:**
  - **Fallo probable 1:** números TS-09 aún no listos → shippear con números Rust + placeholder marcado para JS (no inventar).
  - **Fallo probable 2:** WEB-03 revela 404s → scope incluye quitar refs (no generar imágenes).
  - **Fallo probable 3:** drift con BENCHMARKS.md futuro → link a sección, no copiar-pegar ciego (o copiar con fecha).
- **Stop conditions:** sin fuente reproducible para un número → ese número no se publica (Regla 11).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | TS-09 no listo | Rust primero + placeholder | kickoff |
  | 🟢×🟢 | 404 assets | quitar refs, no binarios | step 0 |
  | 🟢×🟡 | drift | cita con fecha | review |
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** dependencia TS-09; assets; drift.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí.
- **Task file:** `docs/tasks/WEB-02.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** después de TS-09 para números JS; independiente para números Rust.

### Task 8: UX-A11Y-01 — Slice a11y desktop (teclado + focus + labels)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `desktop/src/components/data/DataExplorer.tsx:859-871`, `ResultsList.tsx:22-27`, `ImportPaste.tsx:107-115`, `ImportDrop.tsx:124-133`, `ingest/useModalFocus.ts`, `IngestForm.tsx:44-74`
- **Verificación real:** ✅ CÓDIGO-REAL — UX-02 (`<tr onClick>` sin tabIndex), UX-03 (hook `useModalFocus.ts` huérfano), UX-04 (solo-placeholder, toast sin anclaje, `window.confirm`) todos con file:línea; UX-07/UX-14 ya resueltos (no se tocan).
- **Gate Justificación:** slice vertical a11y de mayor ROI (teclado/SR bloqueados en grid + modales + ingest); 3 superficies, 1 contrato E2E teclado; no tocar UX-06/08/09/10/11/12/13/15/17/19 (DEFER).
- **Gate Result:** ✅ DO
- **Contrato:** Playwright teclado: Tab llega al grid → Enter abre Inspector; Tab no escapa de ImportPaste/ImportDrop y foco se restaura al cerrar; IngestForm tiene `<label>` visibles + error inline (0 `window.confirm` en el path); `npx tsc --noEmit` + `vitest run` en `desktop/` verdes
- **Pre-mortem:**
  - **Fallo probable 1:** lote UX-04 fue aplicado y sobrescrito antes → re-aplicar desde cero con test, no confiar en stash.
  - **Fallo probable 2:** `useModalFocus` no encaja en ambos modales → adaptar hook, no duplicar useEffect Escape.
  - **Fallo probable 3:** grid virtualizado rompe roving tabindex → scope a paginación actual, nota para grid virtual futuro.
- **Stop conditions:** si requiere rediseño visual (UX-06 contraste, UX-12 jerarquía) → fuera de scope, DEFER con nota (necesita input owner).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | re-aplicación UX-04 | test teclado desde día 0 | primer step |
  | 🟢×🟡 | hook no encaja | adaptar, no duplicar | review |
  | 🟢×🟢 | creep visual | prohibido tocar tokens/layout | diff >6 archivos → parar |
- **Cynefin:** 🟨 complicado (ARIA APG + foco).
- **Top 3 riesgos:** re-aplicación; hook; creep.
- **Uphill/Downhill:** ⬆️ uphill: 1 (encaje del hook) · resto ⬇️.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí (a11y bloqueante uso teclado).
- **Task file:** `docs/tasks/UX-A11Y-01.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** c0cf7b31

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** UX-19 (E2E permanente) como follow-up DEFER, no en este task.

## SKIP con evidencia (no re-proponer sin evidencia nueva)

| ID | Evidencia de SKIP |
|----|-------------------|
| MOD-22 | `GraphBfsResult = bigint[]` + nota breaking (`types.ts:237`); sin `as GraphBfsResult` en grep → shape real ya afirmado |
| MOD-23 | `_native` async+await con wrap (`native.ts:154-163`, comentario TS-02) → rechazos async ya envueltos |
| SRV-01 | `AuditLogger::with_rotation` + defaults 10MiB/5 (`audit.rs:15-17,121-204`) + `init_audit` con rotación (`builder.rs:291-295`) → rotación ya implementada |
| WSM-11 | `record_metadata_drop(1)` + comentario WSM-11 (`lib.rs:2355-2363`) + contador en `operational_metrics()` → ya señalizado |
| UX-14 | `.catch(onError(vantaErrorMessage))` + comentario UX-14 (`MemoryLens.tsx:173-175`) → error real ya propagado |

## DEFER (resumen por grupo — detalle en Backlog.md)

- Splits god-file: REVIEW-10, FIND-48/49/50 (appetite 2-3d, fuera de este plan; re-slicear por concern antes de DO).
- Proxy: PRX-02..13 (gateway completo; cada uno ≥1d; requiere plan propio con decisiones D11).
- Futuro: FUT-02..14, OLD-01, PROV-11 (roadmap, sin urgencia Shape Up).
- Estrategia/distribución: BND-08 (🔴 pipeline npm), TS-10/11/12/13, WSM-14, INTG-01/02 (feature-add → mini-spec + owner), STABLE-05/07/09 (validación 3 corridas).
- Desktop resto: DESKTOP-40/42/43/45, FIND-20/21, resto UX (06/08/09/10/11/12/13/15/17/19), MOD-05.
- Docs/gov: GOV-TK2/5/8, BLOG-CTA (contenido editorial), RES/DEC residuales, MEM-66/68/69/70 + MCP-41 (requieren DISCOVERY vanta-arch).
- Cobertura/perf: FIND-47 (no hotspot), PERF-BENCH-01 (condicionado a números TS-09/BND), SRV-06 (DISCOVERY OIDC), WEB-03 (fold en WEB-02 step 0), WEB-09 (requiere input visual owner).

## BLOQUEADO (con causa)

| ID | Causa |
|----|-------|
| MKT-18f | acción humana PyPI (checklist en task file, env `pypi`) — no agentizable |
| MKT-18i | upstream AnythingLLM sin backend VantaDB — feature-request humana |
| AUD-042 | upstream tantivy ≥0.27 sin publicar (lru 0.18 bloqueado) |
| DESKTOP-41 | VM Windows limpia + instaladores — sesión humana |
| DESKTOP-44 | validación proxy con LLM vivo — sesión guiada owner+agente |
| PRX-10 | requiere PRX-03 (keys) — orden |
| DISC-03 | ICEBOX (dependencias externas 1000+ miembros/SaaS) |

## Grafo de dependencias (orden)

```
MOD-24 ──→ BND-12 ──→ BND-13
TS-09 ──→ WEB-02 (JS numbers; Rust numbers independientes)
FIND-64 (independiente) ┐
FIND-60 (independiente) ┤→ waves paralelas con lo anterior
UX-A11Y-01 (independiente) ┘
```

Waves sugeridas (`FAIL_MODE=parallel`, MAX 3): Wave0 {FIND-64, MOD-24, FIND-60} · Wave1 {BND-12, TS-09, UX-A11Y-01} · Wave2 {BND-13, WEB-02}.

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Backlog con filas stale (5 SKIP hallados hoy) | Paso 0 obligatorio por tarea futura; si DISCOVERY contradice backlog, manda código |
| BND-12/TS-09 toolchain (napi/wasm) | validar build en DISCOVERY antes de implementar |
| Drift docs vs código (BND-13/WEB-02) | citas con fecha + fuente reproducible (Regla 11) |

=== RECITATION FIND-64 ===
Campaign ID: 3d601136-c620-4ec8-947b-f1fbe050471d
Objetivo activo: FIND-64: migrar llamaindex put_batch legacy a firma kwargs
Estado: completed
Última acción: Steps 1-3 ✅: repro TypeError confirmado, fix a kwargs, contrato verde 23/23. Diff listo sin commit
Resultado: ✅
Próxima acción: vanta-lead: commit fix: migrate llamaindex put_batch to kwargs signature (FIND-64)
Contrato: verificacion: PYTHONPATH=vantadb-python py -3.11 -m pytest integrations/llamaindex/tests -q → 23 passed ✅; grep put_batch → kwargs keys=/vectors=/payloads=/metadatas=/namespace= ✅ | evidencia: integrations/llamaindex/vantadb_llamaindex/vectorstore.py:118-147 | artefactos: docs/tasks/FIND-64.md (nuevo) | invariantes: API publica intacta; archivos MOD-24/FIND-60 no tocados | deuda: ninguna | queda_pendiente: commit via vanta-lead
Próxima tarea si completa: ninguno (Wave0: MOD-24, FIND-60 en paralelo externo)
=== END RECITATION ===

=== RECITATION MOD-24 ===
Campaign ID: 3d601136-c620-4ec8-947b-f1fbe050471d
Objetivo activo: MOD-24 nits TS: dedup _mapRecord/_buildSearchRequest + fix validateVector + JSDoc
Estado: completed
Última acción: Steps 1-4 completos: RED (1 fallo Float32Array) -> GREEN guards -> dedup vantadb.ts -> dedup native.ts; verify full verde
Resultado: ✅
Próxima acción: vanta-lead: git add 4 archivos + commit refactor MOD-24
Contrato: verificacion: npm run build exit 0 + vitest 280/280 + eslint 0 + grep function _mapRecord = 1 (guards.ts:123) | evidencia: claim dedup -> diff 4 archivos +72/-47; claim type-lie fix -> RED 1 fallo + GREEN 280 verdes; claim sin cambio wire -> base wire-neutral, backends conservan null/undefined/{}/exclude_superseded | artefactos: vantadb-ts/src/guards.ts, vantadb.ts, native.ts, __tests__/vanta.test.ts, docs/tasks/MOD-24.md | invariantes: mensajes _mapRecord byte-identicos; wire nativo intacto; tests existentes sin modificar (solo +2 nuevos) | deuda: ninguna | queda_pendiente: commit por vanta-lead (refactor: dedup TS guards and map helpers MOD-24)
Próxima tarea si completa: FIND-60
=== END RECITATION ===

=== RECITATION BND-12 ===
Campaign ID: 3d601136-c620-4ec8-947b-f1fbe050471d
Objetivo activo: BND-12: cobertura tests vantadb-node (search/explain_search/put_batch/capabilities/close-drain)
Estado: completed
Última acción: GREEN test-only: bigint 0n/2n + supersede vía get(); npm test 34/34, sin commit
Resultado: OK
Próxima acción: vanta-lead: commit test: green node coverage 34/34 (BND-12)
Contrato: verificacion: npm test en vantadb-node/ → 34 passed, 0 failed ✅ | evidencia: 34 tests (api 26+graph 5+persistence 3), cobertura search/explain/put_batch/capabilities/close en api.test.ts | artefactos: vantadb-node/tests/api.test.ts, docs/tasks/BND-12.md | invariantes: src/lib.rs e index.d.ts intactos; archivos MOD-24/TS-09/UX-A11Y-01 no tocados | deuda: FIND-BND12-01 (d.ts number vs bigint runtime); commit pendiente vanta-lead | queda_pendiente: commit via vanta-lead
Próxima tarea si completa: ninguno (Wave1 externa: TS-09, UX-A11Y-01)
=== END RECITATION ===

=== RECITATION FIND-60 ===
Campaign ID: 3d601136-c620-4ec8-947b-f1fbe050471d
Objetivo activo: FIND-60: 49 warnings rustdoc a cero
Estado: completed
Última acción: 5 slices: 47 warnings → 0 (46 vantadb + 1 vantadb_py connect). Diff 100% comentarios doc verificado mecanicamente. NO commiteado por regla de tarea.
Resultado: OK
Próxima acción: Orquestador: git add solo 23 archivos FIND-60 + docs/tasks/FIND-60.md y commit 'docs: fix rustdoc warnings to zero (FIND-60)'
Contrato: verificacion: RUSTDOCFLAGS='-D warnings' cargo doc --no-deps -p vantadb -p vantadb_py exit 0 ✅ + grep -c warning → 0 ✅ + cargo fmt --check ✅ | evidencia: find60-verify.txt (0 warnings), diff 100% lineas /// ///! | artefactos: docs/tasks/FIND-60.md | invariantes: doc-only, ningun pub agregado, ningun cambio comportamiento | deuda: ninguna | queda_pendiente: commit selectivo de 23 archivos + task file (orquestador/vanta-lead; hay cambios ajenos FIND-64/MOD-24 en worktree, NO commitear todo)
Próxima tarea si completa: ninguno (Wave0: FIND-64, MOD-24 en paralelo por otros agentes)
=== END RECITATION ===

=== RECITATION TS-09 ===
Campaign ID: 3d601136-c620-4ec8-947b-f1fbe050471d
Objetivo activo: TS-09 bench reproducible JS/WASM insert+search p50/p95/p99
Estado: completed
Última acción: Steps 1-3 ✅: harness + 3 corridas + §15 + verify full TS verde. SIN commit por orden explicita
Resultado: OK
Próxima acción: Orquestador/vanta-lead: commit selectivo + skill progreso; WEB-02 puede consumir §15
Contrato: verificacion: npm run bench emite p50/p95/p99 + JSON ✅; node bench/smoke.mjs OK ✅; npx tsc --noEmit 0 ✅; vitest 280/280 ✅; eslint bench/ 0 errors ✅; BENCHMARKS.md §15 con entorno+comando+fecha ✅ | evidencia: claim bench reproducible -> vantadb-ts/bench/bench.mjs + smoke.mjs + package.json scripts; claim numeros citables -> BENCHMARKS.md §15 (mediana x3 corridas 2000x384d); confianza: alta | artefactos: vantadb-ts/bench/bench.mjs, vantadb-ts/bench/smoke.mjs, docs/tasks/TS-09.md | invariantes: sin comparativa externa (D1); archivos BND-12/UX-A11Y-01 no tocados | deuda: browser OPFS/IDB no medido (nota en §15); skill progreso + commit pendientes de orquestador | queda_pendiente: vanta-lead: git add SOLO vantadb-ts/bench/ + vantadb-ts/package.json + docs/operations/BENCHMARKS.md + docs/tasks/TS-09.md + plan file y commit (hay archivos ajenos BND-12/UX-A11Y-01 en worktree, NO commitear todo)
Próxima tarea si completa: WEB-02
=== END RECITATION ===
