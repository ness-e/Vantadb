---
title: "Plan de campaña: feedback Ego (EGO-01..07) para release 0.9.0"
kind: plan
description: "7 hallazgos de integración real Ego x VantaDB (bugs, inconsistencias, mejoras) con fix, contrato mecánico y gates para el tren 0.9.0."
---

# Plan de campaña: feedback Ego (EGO-01..07) para release 0.9.0

> **Inicio:** 2026-10-08
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/dev/Backlog.md` § Alta 2026-10-08 (filas `EGO-01..07`) ← `Ego/docs/VANTADB-FEEDBACK-Y-MEJORAS.md` (2026-10-07)
> **Autonomous:** false
> **SDP:** documentation-skill · writing-plans · planning-and-task-breakdown · progreso · systematic-debugging (bugs EGO-01/05/06) · source-driven-development (API/wire EGO-02/03/05)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 7 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 4 incógnitas abiertas (finalizer napi en EGO-06; wiring ONNX + prebuilts en EGO-07; shape del scope en MEMG-25) · ⬇️ downhill = ~35 steps pendientes (ver § Orden de ejecución).

> **Determinación owner-delegada 2026-10-08:** entran al tren 0.9.0 EGO-01..07 + MEMG-25 + FIND-320 + EGO-08. MEMG-26 solo-spec en ventana (implementación → 0.10): un modelo de eventos apurado es deuda de esquema permanente. EGO-04/05 rompen wire/formato y la ventana 0.x para romper es ahora — en 0.10 costarían a Ego una segunda migración.

> Verificación previa (2026-10-08, sesión de análisis): los 7 gaps se confirmaron contra código real — `vantadb-node/src/lib.rs:825-831` (cursor), `src/sdk/types.rs:106` (`Value`), `src/sdk/api/memory.rs:733` (sin inferencia), `src/sdk/serialization/mod.rs:111` (sin `*`), `vantadb-ts/src/guards.ts:220-243` (WIRE-03 parcial), `src/text_index.rs:176` (ASCII-only). Sin DEFER/SKIP que confirmar (decisión owner 2026-10-08: los 7 en 0.9.0).

## Orden de ejecución (olas, MAX_WIP=3, FAIL_MODE=parallel)

```
Ola 1: EGO-01 (lib.rs) · EGO-02 (guards.ts) · EGO-04 (text_index.rs)   ← archivos disjuntos
Ola 2: EGO-03 (multi.rs) · EGO-05 (types.rs)                          ← tras EGO-01 (lib.rs libre)
Ola 3: EGO-06 (lib.rs) · EGO-07 (config+memory+cargo)                 ← tras EGO-05 (lib.rs libre)
Ola 4: MEMG-25 (types) · FIND-320 (recall) · EGO-08 (docs/plan)        ← tras Ola 2 (tipos estables)
```

Regla: `vantadb-node/src/lib.rs` se toca en EGO-01, EGO-05 (si el wire lo exige) y EGO-06 — serializar esos tres, nunca en paralelo.

## Gates de release 0.9.0 (verificables al cerrar la campaña)

- `vantadb-ts/package.json` == `[workspace.package]` (gate FIND-230 — tren npm completo).
- `cargo test --test sdk_serialization` verde (EGO-05) + test de reindex/migración (EGO-04).
- Prebuilt `vantadb-node` reconstruido con `embed-local` + chequeo de tamaño (EGO-07).
- `scripts/verify-release.ps1 -Tag v0.9.0 -Smoke` (gate de release completo).
- Follow-up en Ego (post-publicación, fuera de este plan): bump `vantadb` 0.8.0→0.9.0 + eliminar los 5 workarounds (`EgoMemoryAdapter.ts`, `EgoMemoryLifecycle.ts:98`).

## Tasks

### Task 1: EGO-01 — cursor `undefined`/`Null` + string en `db.list()`

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `vantadb-node/src/lib.rs:825-831` (+ `get_opt_u64` existente), `vantadb-node/tests/api.test.ts`
- **Verificación real:** ✅ CÓDIGO-REAL — `obj.get("cursor")` retorna `Some(Null)` y `as_u64()` falla; sin rama string.
- **Gate Justificación:** bug medido en integración real; fix de ~6 líneas reusando helper existente; cero blast radius fuera del parse de `list`.
- **Gate Result:** ✅ DO
- **Contrato:** `db.list({namespace, limit: 100, cursor: undefined})` → página OK; `cursor: "0"` → página OK; `cursor: "abc"` → error `cursor must be a number`; suite `api.test.ts` verde.
- **Task file:** `docs/dev/tasks/EGO-01.md`
- **Estado:** ✅ COMPLETED (unit 8/8 + napi 29/29; review degradado pendiente externo)
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. `get_opt_u64` tiene semántica distinta (f64 vs Value) → adaptar, no llamar a ciegas.
  2. Cursor string con espacios/signo → normalizar con `trim()` + rechazo explícito.
  3. Otro parse cercano (`limit`) con el mismo bug latente → revisar y fijar igual si aplica.

- **Stop conditions:** >1d → fix mínimo (solo `Null`) + FIND del string.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟢 | Regresión de cursores numéricos | Test de los 3 caminos (número/undefined/string) | VERIFY |
  | 🟢×🟡 | `limit` con el mismo patrón | Grep del parse y fix simétrico | DISCOVERY |

- **Cynefin:** 🟦 obvio — causa-efecto claro, helper existente.
- **Top 3 riesgos:** (1) regresión numérica; (2) `limit` con el mismo bug; (3) string no-numérico ambiguo.
- **Uphill/Downhill:** ⬇️ (3 steps: parse + tests + verify).
- **DoD:** task = contrato + `api.test.ts` verde · commit = `fix(node):` + task ID · release = entrada de changelog.
- **Validación Appetite vs Effort:** 1d ≥ 1h ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 2: EGO-02 — `query_vector` opcional con `text_query`/`query_sparse`

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Baja
- **Archivos clave:** `vantadb-ts/src/guards.ts:220`, `vantadb-ts/src/types.ts`, `vantadb-ts/src/__tests__/d5a-validation.test.ts`
- **Verificación real:** ✅ CÓDIGO-REAL — WIRE-03 acepta `[]` con texto pero `undefined` lanza `must be an array`.
- **Gate Justificación:** fricción DX medida; cambio de 3 líneas en guard compartido; sin cambio wire (Rust ya acepta vector vacío).
- **Gate Result:** ✅ DO
- **Contrato:** `buildSearchRequestBase({namespace, text_query: "hola"})` → base con `query_vector: []`; sin `text_query` ni `sparse` y sin vector → sigue lanzando; `d5a-validation.test.ts` + suite TS verdes.
- **Task file:** `docs/dev/tasks/EGO-02.md`
- **Estado:** ✅ COMPLETED (commit `9f6bc7ab`; review degradado pendiente externo)
- **Branch:** develop
- **Commit:** `9f6bc7ab`

- **Pre-mortem:**
  1. El guard es compartido native+wasm → el cambio afecta ambos paths (deseado, pero correr ambas suites).
  2. Tipo `SearchRequest.query_vector` obligatorio en `types.ts` → hacerlo opcional o el fix no compila en consumidores.
  3. Tests que pinean el mensaje `must be an array` → actualizar snapshots con el nuevo comportamiento.

- **Stop conditions:** >1d → dejar fix mínimo + nota.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟢 | Snapshots pineados al error viejo | Grep de `must be an array` en tests antes de cerrar | VERIFY |
  | 🟢×🟢 | Divergencia native/wasm | Suites de ambos paths en el verify | CIERRE |

- **Cynefin:** 🟦 obvio — el precedente WIRE-03 marca el approach.
- **Top 3 riesgos:** (1) snapshots; (2) tipo TS; (3) divergencia de paths.
- **Uphill/Downhill:** ⬇️ (3 steps: guard + tipo + tests).
- **DoD:** task = contrato + suites TS verdes · commit = `fix(ts):` + task ID · release = entrada de changelog.
- **Validación Appetite vs Effort:** 1d ≥ 1h ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 3: EGO-03 — comodines/prefijo nativos en `search_multi`

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `src/sdk/search/multi.rs`, `src/sdk/serialization/mod.rs:111`, `vantadb-node/src/lib.rs:616`
- **Verificación real:** ✅ CÓDIGO-REAL — `validate_namespace` solo permite `[A-Za-z0-9._/-]`; no existe `search_prefix`.
- **Gate Justificación:** elimina el fan-out manual de Ego (N+1 round-trips por búsqueda); feature-add acotada a la capa search.
- **Gate Result:** ✅ DO
- **Contrato:** `search_multi(["kb/*"], req)` retorna hits mergeados de los namespaces coincidentes; `put({namespace: "kb/*"})` sigue dando VALIDATION_ERROR; tests core + node verdes.
- **Task file:** `docs/dev/tasks/EGO-03.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. `validate_namespace` compartida con put/get → NO relajarla; patrón separado solo-search.
  2. Expansión con 0 matches → definir: array vacío (no error) + documentar.
  3. Spec SDD obligatoria (símbolo público nuevo) + Gate P/D con `question` si hay decisiones abiertas.

- **Stop conditions:** >3d → expansión solo en binding node + FIND del core.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Wildcard llega a write path | Tests de `put`/`get` con `*` → error | VERIFY |
  | 🟡×🟡 | Semántica 0-matches ambigua | Decisión escrita en Spec antes de codear | DISCOVERY |
  | 🟢×🟡 | Fan-out duplica top_k por namespace | Límite por namespace + merge acotado | diseño |

- **Cynefin:** 🟨 complicado — requiere decidir semántica (pattern syntax, 0-matches, límites) entre approaches válidos.
- **Top 3 riesgos:** (1) fuga a write path; (2) semántica 0-matches; (3) explosión de fan-out.
- **Uphill/Downhill:** ⬇️ (5 steps: spec + validación + expansión + tests + verify).
- **DoD:** task = contrato + suites core/node verdes · commit = `feat(search):` + task ID · release = changelog (feature → minor) + docs API.
- **Validación Appetite vs Effort:** 3d ≥ 1d ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 4: EGO-04 — tokenizador default Unicode + bump de versión

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `src/text_index.rs:176` (+ `TOKENIZER_NAME`/`TOKENIZER_VERSION` + spec), tests del índice
- **Verificación real:** ✅ CÓDIGO-REAL — `is_ascii_alphanumeric` parte `cuáles`; el path `advanced-tokenizer` no es el default.
- **Gate Justificación:** fix de 1 línea + versionado; corrige recall en español (idioma primario de Ego); breaking on-disk cubierto por minor 0.x.
- **Gate Result:** ✅ DO
- **Contrato:** `tokenize("¿Cuáles son los principios?")` contiene `cuáles` y `principios` (sin fragmentos `cu`/`les`); `TOKENIZER_VERSION` bumpeada; índice viejo → rebuild/reindex verificado (no panic, no corrupción silenciosa); suite text_index verde.
- **Task file:** `docs/dev/tasks/EGO-04.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. Cambio de formato on-disk sin migración → DBs 0.8.0 ilegibles o, peor, leídas a medias; definir rebuild automático o error explícito.
  2. `debug_assert_eq!` de spec/version en readers viejos → verificar el camino de apertura de índices existentes.
  3. Benchmark de ingesta (Regla 9 si hay claim de perf — no lo hay; skip justificado).

- **Stop conditions:** >3d → fix + migración como FIND separado con trigger explícito.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Índice 0.8.0 corrupto tras upgrade | Test de apertura de índice viejo + camino de rebuild | VERIFY |
  | 🟢×🟡 | Stopwords/stemming en español fuera de scope | Declarar fuera de scope en Notas (default tokenizer, no advanced) | cierre |

- **Cynefin:** 🟦 obvio — fix conocido, el costo está en la migración.
- **Top 3 riesgos:** (1) migración on-disk; (2) readers viejos; (3) scope creep a stemming.
- **Uphill/Downhill:** ⬇️ (4 steps: tokenizer + versión + migración + tests).
- **DoD:** task = contrato + suite verde · commit = `fix(index):` + task ID · release = changelog (breaking 0.x documentado).
- **Validación Appetite vs Effort:** 3d ≥ 1d ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 5: EGO-05 — `Deserialize` tolerante en `Value` (plano + tagged)

- **Appetite:** max 3d
- **Esfuerzo:** 🟠 2-3d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `src/sdk/types.rs:106` (+ tests `sdk_serialization`), `vantadb-node/src/lib.rs` (si el wire lo exige), smoke Node
- **Verificación real:** ✅ CÓDIGO-REAL — `Value` es externally-tagged; escalares planos de JS no deserializan; Ego taggea a mano (`EgoMemoryAdapter.ts:72`).
- **Gate Justificación:** elimina la fricción más invasiva del binding (todo `put` con metadata numérica); retrocompatible por diseño (acepta ambas formas).
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --test sdk_serialization` verde; `put` con metadata plana `{ts: Date.now(), n: 3, s: "x", b: true}` → round-trip igual; forma tagged `{Float: n}` sigue funcionando; suites WASM/Python sin regresión.
- **Task file:** `docs/dev/tasks/EGO-05.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. Ambigüedad int/float en el límite i64 (f64 > 2^53) → regla escrita: integral + rango → `Int`, resto → `Float`.
  2. Cambio wire cross-binding → paridad WASM/Python verificada, no asumida.
  3. Serialización asimétrica (leo plano, escribo tagged) → fijar forma canónica de escritura en el task file.

- **Stop conditions:** >3d → entregar solo lectura tolerante + FIND de escritura canónica.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Precisión i64>f64 silenciosa | Test de borde 2^53 + error/Float explícito | VERIFY |
  | 🟡×🟡 | Drift WASM/Python | Suites de los 3 bindings en el verify | CIERRE |
  | 🟢×🟡 | Listas mixtas (`ListInt` vs `ListFloat`) | Regla por-elemento documentada + tests | diseño |

- **Cynefin:** 🟨 complicado — decisión de coerción entre approaches válidos con borde de precisión.
- **Top 3 riesgos:** (1) precisión; (2) drift cross-binding; (3) forma canónica de escritura.
- **Uphill/Downhill:** ⬇️ (6 steps: deserializer + tests serialización + smoke node + paridad + verify).
- **DoD:** task = contrato + 3 bindings verdes · commit = `fix(sdk):` + task ID · release = changelog (breaking 0.x si aplica) + docs de metadata.
- **Validación Appetite vs Effort:** 3d ≥ 2-3d ✓ (límite justo — stop condition activa)

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 6: EGO-06 — finalizer NAPI + registro por proceso (lock Fjall)

- **Appetite:** max 1sem
- **Esfuerzo:** 🟠 2-3d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `vantadb-node/src/lib.rs` (finalizer + registry), test Node open/abandon/reopen
- **Verificación real:** ✅ CÓDIGO-REAL — sin `Drop`/`finalizer`/`InstanceRegistry` en el binding; lock retenido tras excepción.
- **Gate Justificación:** crash-recovery en Windows (plataforma primaria de Ego/Electron); sin esto cada test fallido bloquea la DB.
- **Gate Result:** ✅ DO
- **Contrato:** test Node open → abandonar sin `close()` (GC/finalizer) → reopen mismo path → OK (sin `FjallError: Locked`); doble-open concurrente → error claro (no corrupción silenciosa); suite node verde.
- **Task file:** `docs/dev/tasks/EGO-06.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. El finalizer corre en GC (timing no determinista) → el test debe forzar GC o el contrato es flaky (Regla 2: sin flakys).
  2. Finalizer + `close()` explícito = doble-free del lock → guardia idempotente.
  3. Sin Windows real el fix no se valida → liga con `DIST-21` (CI Electron); si no hay runner, documentar + test best-effort.

- **Stop conditions:** >1sem o sin runner Windows → registro + `close()` defensivo documentado + FIND con trigger `DIST-21`.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Test flaky por timing de GC | Forzar GC en el test (`--expose-gc`) o sincronizar vía registry | VERIFY |
  | 🟡×🟡 | Doble liberación | Flag idempotente + test close→finalizer | diseño |
  | 🟢×🔴 | Sin validación Windows | Reutilizar `DIST-21` cuando exista | cierre |

- **Cynefin:** 🟨 complicado — 1 incógnita (API de finalizer napi + timing GC) → se resuelve en step 1.
- **Top 3 riesgos:** (1) flaky GC; (2) doble-free; (3) falta de Windows real.
- **Uphill/Downhill:** ⬆️ 1 incógnita (finalizer napi) · ⬇️ 5 steps.
- **DoD:** task = contrato + suite node verde · commit = `fix(node):` + task ID · release = changelog.
- **Validación Appetite vs Effort:** 1sem ≥ 2-3d ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 7: EGO-07 — auto-embed ONNX en `put` (`embed-on-put` opcional)

- **Appetite:** max 3sem
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🟠 Media
- **Archivos clave:** `src/config.rs`, `src/sdk/api/memory.rs:733`, `vantadb-node/Cargo.toml`, `vantadb-node/src/lib.rs`, `vantadb-ts/src/native.ts` (`NativeConnectOptions`)
- **Verificación real:** ✅ CÓDIGO-REAL — `put_one` filtra sin inferir; `embed-local` no compilado en el binding; ONNX solo en `llm`/`vanta-memory`/`vantadb-mcp`.
- **Gate Justificación:** la doc promete auto-embed y Ego lo necesita para HNSW real in-process; es la única feature (resto = fixes); Ego tiene workaround (no bloquea) → P1 dentro de la campaña.
- **Gate Result:** ✅ DO
- **Contrato:** con `embed_on_put: true` + modelo presente, `put` sin vector → search vectorial lo encuentra (recall > 0); con flag off → comportamiento idéntico al actual; sin `model.onnx`/`tokenizer.json` → error accionable (no panic); `cargo check -p` con/sin feature verde; prebuilt `.node` reconstruido + tamaño registrado.
- **Task file:** `docs/dev/tasks/EGO-07.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. ONNX Runtime en prebuilts multi-plataforma (tamaño/ABI/dylib) → spike de empaquetado ANTES del wiring.
  2. Latencia de `put` con inferencia síncrona → inferencia en `spawn_blocking`, nunca en event loop.
  3. Modelo de 220 MB dentro del paquete npm → NO empaquetar; wizard `setup-embeddings` + `embedding_health` como fuente de verdad.
  4. Spec SDD obligatoria (feature + símbolo público) + Gate P/D con `question` para las decisiones abiertas (default del flag, estrategia de carga, mensaje de error).

- **Stop conditions:** prebuilts inviables en 1sem → flag + wiring con backend pluggable + FIND de distribución (desacopla el fix del empaquetado).

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🔴×🔴 | Prebuilts ONNX por plataforma inviables | Spike de empaquetado primero; stop a 1sem | DISCOVERY |
  | 🟡×🟡 | `put` se vuelve lento | `spawn_blocking` + bench before/after del write path | VERIFY |
  | 🟡×🟢 | Modelo ausente = error críptico | Error accionable con ruta esperada + link al wizard | diseño |
  | 🟢×🟡 | Flag default cambia comportamiento | Default `false` + test de paridad off | CIERRE |

- **Cynefin:** 🟧 complejo — empaquetado/ABI solo emergen al probar por plataforma: spike corto + steps con verify frecuente.
- **Top 3 riesgos:** (1) prebuilts; (2) latencia de `put`; (3) modelo ausente.
- **Uphill/Downhill:** ⬆️ 2 incógnitas (wiring sesión ONNX en node; prebuilts+tamaño) · ⬇️ 8 steps.
- **DoD:** task = contrato + prebuilt medido · commit = `feat(embed):` + task ID (commits por slice) · release = changelog (feature → minor) + docs + ADR si hay tradeoff de distribución.
- **Validación Appetite vs Effort:** 3sem ≥ 1-2sem ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 8: MEMG-25 — semántica formal de `MemoryScope`

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 3-5d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `src/sdk/types/record.rs`, `vanta-memory/` (tipos), `docs/api/` (spec)
- **Verificación real:** ✅ CÓDIGO-REAL — solo namespaces planos; el aislamiento de Sub-Egos vive en el adapter de Ego, no en el motor.
- **Gate Justificación:** sin scopes en el motor, Ego no puede delegar aislamiento (todo queda en TS, fuera de garantías Rust); spec-first, sin enforcement en este slice.
- **Gate Result:** ✅ DO
- **Contrato:** tipos `MemoryScope` (tenant/org/project/principal/agent/session/task + visibility/trust/authority/provenance/validity/retention/owner/writer) compilando + spec en docs + test de construcción/defaults; cero cambio de comportamiento (aditivo puro).
- **Task file:** `docs/dev/tasks/MEMG-25.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. Scope creep a enforcement (eso es MGR-04/FIND-301) → este slice = tipos + spec, enforcement explícitamente fuera.
  2. Nombres de campos que colisionan con metadata existente → prefijo/reserva documentada en spec.
  3. Spec SDD + Gate P/D con `question` para el shape (campos obligatorios vs opcionales).

- **Stop conditions:** >1sem → spec sola + FIND de tipos.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Creep a enforcement | Contrato lo excluye por escrito | DISCOVERY |
  | 🟢×🟢 | Colisión con metadata | Reserva de namespace en spec | diseño |

- **Cynefin:** 🟨 complicado — 1 incógnita (shape exacto del scope) → Gate P/D la resuelve.
- **Top 3 riesgos:** (1) creep; (2) colisión; (3) shape.
- **Uphill/Downhill:** ⬆️ 1 incógnita · ⬇️ 4 steps.
- **DoD:** task = contrato + spec · commit = `feat(sdk):` + task ID · release = changelog + docs API.
- **Validación Appetite vs Effort:** 1sem ≥ 3-5d ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 9: FIND-320 — contrato `retrieve()` trust-aware unificado

- **Appetite:** max 1sem
- **Esfuerzo:** 🟡 3-5d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs`, `src/rbac.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — piezas dispersas (trust gate MEMG-10, MGR-06, MGR-18, FIND-286) sin contrato único.
- **Gate Justificación:** antes de guardar datos sensibles (producción Ego), el retrieval debe filtrar por trust/conflicto y abstenerse — hoy eso no existe como contrato.
- **Gate Result:** ✅ DO
- **Contrato:** spec de `retrieve()` (search + trust + confidence + freshness + provenance + conflict + abstain) + wiring opt-in byte-idéntico por default (precedente MEMG-21) + tests; consumer declarado: `DIST-19`.
- **Task file:** `docs/dev/tasks/FIND-320.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. Cambiar semántica de recall por default → opt-in estricto, default = comportamiento actual.
  2. Solapamiento con FIND-301(c) (trust HTTP) → coordinar, no duplicar.
  3. Abstención sin umbrales calibrados → umbrales por config de host, nunca hardcodeados.

- **Stop conditions:** >1sem → spec + FIND de wiring.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Recall cambia por default | Opt-in + test de paridad byte-idéntica | VERIFY |
  | 🟡×🟡 | Duplicación con FIND-301 | Mapeo pieza→fila en la spec | DISCOVERY |

- **Cynefin:** 🟨 complicado — integración de piezas existentes con semántica a decidir.
- **Top 3 riesgos:** (1) cambio de default; (2) duplicación; (3) umbrales.
- **Uphill/Downhill:** ⬇️ (5 steps: spec + wiring + tests + paridad + verify).
- **DoD:** task = contrato + suite verde · commit = `feat(memory):` + task ID · release = changelog + docs.
- **Validación Appetite vs Effort:** 1sem ≥ 3-5d ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 10: EGO-08 — Ego Readiness Baseline v1

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `docs/dev/plans/` (baseline), `docs/dev/Backlog.md` (reorden)
- **Verificación real:** 🟡 VERIFICAR — artefacto nuevo; insumos existen (este plan + análisis §24 + filas Fase A–G).
- **Gate Justificación:** fija el contrato emergente del develop estabilizado y ordena el backlog por fases Ego (A–G) en vez de historia del repo; es el mapa que decide qué entra a cada release.
- **Gate Result:** ✅ DO
- **Contrato:** baseline con matriz fase→filas + huecos declarados + criterio de tren por release; `check-docs`/`check-links` verdes.
- **Task file:** `docs/dev/tasks/EGO-08.md`
- **Estado:** ⬜ PENDING
- **Branch:** develop
- **Commit:**

- **Pre-mortem:**
  1. Baseline que duplica este plan → EGO-08 = mapa de releases, este plan = ejecución 0.9.0 (referenciar, no copiar).
  2. Reorden sin dueño → Gate P con owner antes de mover filas.

- **Stop conditions:** >1d → matriz fase→filas sola + nota.

- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟢 | Duplicación con este plan | Referencia cruzada explícita | cierre |

- **Cynefin:** 🟦 obvio — ordenar información existente.
- **Top 3 riesgos:** (1) duplicación.
- **Uphill/Downhill:** ⬇️ (2 steps).
- **DoD:** task = contrato + gates docs · commit = `docs:` + task ID · release = n/a.
- **Validación Appetite vs Effort:** 1d ≥ 1d ✓

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

> MEMG-26 (Semantic Event Log) queda **spec-only en ventana** por determinación 2026-10-08 — sin task de implementación en este plan; su spec puede correr en paralelo a cualquier ola.
