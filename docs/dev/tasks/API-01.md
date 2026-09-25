# API-01: W0 fundación — tipos base + error envelope + casing + u128 wire

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 1 · investigación `docs/dev/plans/2026-09-24-api-estandarizacion.md` (API-STD-02/05/06/15)
- **Fuente:** Backlog Phase 51 (fila `API-01`)
- **Esfuerzo:** 🔴 3-5d (tool estimate: 30-60 turns)
- **Prioridad:** 🔴
- **Tipo:** Mixto (Rust core + Python + TypeScript + Node + WASM)
- **Turns estimados:** 30-60
- **Creado:** 2026-09-25
- **last-synced:** 2026-09-25
- **Estado:** ⏳ IN PROGRESS (Steps 0-8 ✅ · contrato mecánico ✅) — pendiente Review P2-01 + cierre
- **Incógnitas (uphill):** 0 abiertas (codegen DEFER por stop condition del plan; ADR-041 decidido B 2026-09-25)
- **Pendientes (downhill):** 0 (pendiente de gate: Review P2-01 + cierre)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `QueryResult` lo leen los 4 bindings + HTTP (`src/server/handlers.rs:1164` doc) + MCP; `VantaHeader` 4 callers (`src/migration.rs`, `src/lib.rs:167` re-export, `src/storage/vfile.rs`) + `validate_compat` 5 callers (`src/index/serialize/bytes.rs`, `binary_header.rs`, `vfile.rs`, `wal.rs`); `FilterOp` en `vantadb-ts/src/types.ts:207`, `vantadb-wasm/src/vantadb_wasm.d.ts:198`, `vantadb-node/index.d.ts:228` |
| Callees | `u128_serde` (`src/sdk/types.rs:32`, crate-private) usado por `record.rs:95`, `graph.rs:29`, `serialization/*`; `VantaError` core (`src/error.rs`) consumido por todos los bindings vía `map_vanta_error`/`to_js_err` |
| Implicaciones | `Write.node_id` sin `u128_serde` pierde IDs >2^53 en wire JSON (solo writes); cambiarlo es breaking wire (`feat!:`); `validate_compat` es hot path de lectura on-disk (no romper); `VantaHeader` no es serde (serialización custom por magic bytes → renombrar NO afecta on-disk) |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN:** el ejecutor debe leer COMPLETOS los archivos a modificar antes del primer edit y completar esta tabla (los paths ya fueron leídos parcialmente en discovery; el gate exige lectura completa en ejecución).

- **Archivos leídos (completos):** `src/sdk/types/graph.rs` ✅ (leído completo en discovery), `src/binary_header.rs` ✅ (leído completo 2026-09-25), `src/error.rs` ✅ (completo 1348L), `src/sdk/types/record.rs` ✅, `src/sdk/types.rs` ✅ (u128_serde), `vantadb-wasm/src/lib.rs` ✅ (función P2-8 + módulos de test vecinos), `vantadb-python/src/types.rs` ✅, `vantadb-ts/src/types.ts` ✅, `vantadb-node/index.d.ts` ✅, `vantadb-wasm/src/vantadb_wasm.d.ts` ✅ (header + tipos W0), `docs/dev/architecture/adr/041_anti_stutter.md` ✅, `docs/api/ERROR_HANDLING.md` ✅ (417L completo), `docs/api/BINDINGS_NAMESPACES.md` ✅ (secciones iniciales), `src/lib.rs` ✅ (completo 215L). Pendientes de lectura completa (no modificados en esta iteración): `src/sdk/version_history.rs`, `src/cli_handlers/crud.rs:445` — inspeccionados; `version_history.rs` ya usa `node_id_str` postcard-safe (no requiere cambio W0).
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `use super::u128_serde` (graph.rs:9, record.rs:6); `map_vanta_error`/`to_js_err` en bindings; `crate::error::{Error, Result}` en 161+ sitios del core
- **Archivos que referencian a los editados (referencias entrantes):** `git grep -n 'QueryResult'` (bindings + server + mcp); `git grep -n 'VantaHeader'` (migration/vfile/wal/index-serialize); `git grep -n 'FilterOp'` (record.rs + 3 .d.ts + TS)
- **Veredicto impacto:** **alto** en wire (4 bindings + HTTP + MCP leen `QueryResult`; cambio breaking `feat!:`), **medio** en errores (envelope + doc; no eliminar variantes con callers), **bajo** en `VantaHeader` (renombre no afecta bytes on-disk)

## Contrato

"`cargo test --test sdk_serialization` verde Y test wire `u128` >2^53 redondo en los 4 bindings (Py/TS/Node/WASM) Y `rg 'Generic\(' src/error.rs` con tipado o doc-diseño Y `dev-tools/verify_changed.ps1` verde"

## Spec (SDD — feature-add: cambia contratos públicos de tipos/errores)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Wire `u128` en `Write.node_id` | A: `#[serde(with = "u128_serde")]` → string decimal (consistente con `MemoryRecord`/`StaleContext`; breaking wire) / B: dejarlo numérico (pierde >2^53) | A | ✅ decidido-por-evidencia: API-STD-02 R4 + Gate P "Tipos base: `u128→string` decimal" (API-STD-15 tabla Eje Tipos base; modelo Node `index.d.ts:63-64`) |
| 2 | Errores catch-all (`Generic`/`ResourceLimit`/`InvalidInput`/`Schema`) | A: envelope `code+message+context` documentado + tipado incremental por variante / B: tipar todo ya (rompe 161+ callers) / C: dejar como está | A | ✅ Gate P Eje Errores (`code+message+context`, sin panic; RFC 9457 solo HTTP) — alcance W0: envelope + doc; variantes sin callers que lo justifiquen se tipan, el resto se documenta by-design |
| 3 | Casing en fronteras | camelCase JSON/MCP · kebab CLI/tools-nuevos · nativo interior (snake Rust/Python, camel TS) | — | ✅ Gate P Eje Casing (API-STD-15) — W0 aplica a tipos de bindings tocados + normativa |
| 4 | Codegen single-schema | A: codegen desde JSON Schema (dueño sin resolver) / B: manual 1 vez + DEFER codegen | B si no hay consenso | ⏳ incógnita — stop condition del plan: "codegen sin consenso → manual 1 vez + DEFER codegen" |
| 5 | `VantaHeader` / ADR-041 | A: firmar ADR-041 (excepción permanente) / B: renombrar (anti-stutter; no afecta on-disk — serialización custom por magic bytes) / C: dejar proposed | decidir con ADR-041 leído completo | 🟡 evidencia 2026-09-25: la exclusión cita "formato on-disk, compat binaria" pero el nombre del struct NO se serializa (sin serde + `serialize()` bytes crudos) → **recomendación B (rename `Header` + alias deprecated)**; **firma owner pendiente (Regla 5, BLOQUEO)** — evidencia en ADR §Evidencia adicional + task file Step 5 |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `u128_serde` es el ÚNICO patrón de `u128` en wire (string decimal) — no cambiar el formato de `MemoryRecord`/`StaleContext`/`SnapshotRecord` (`node_id_str` postcard binario); no romper `validate_compat` (range-based, hot path de lectura); no tocar `vantadb-pro` (open-core); no renombrar símbolos Rust internos por gusto (snake Rust / camel TS/JSON); `map_vanta_error`/`to_js_err` siguen mapeando a excepciones sin panic.
- **Comandos de verificación:** `cargo test --test sdk_serialization` · `cargo nextest run --profile audit --workspace --build-jobs 2` · `dev-tools/verify_changed.ps1` · pytest/npm test por binding.
- **Deuda pendiente:** P2-8 (`collect_all_deduped()` O(n) en `vantadb-wasm/src/lib.rs:564-596`) se paga en este task (Regla 6, moneda declarada en Gate P "P2-8 con batch fundación"); P2-5 (`put_batch` dual Python) NO es de W0 (se paga en API-02 con la migración array-objetos).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|---------------------------|
| `activeGoal` | Encabezado `# API-01: W0 fundación — tipos base + error envelope + casing + u128 wire` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | API-02 (W1 bindings) |

    contract:
      verificacion: cargo test --test sdk_serialization → (pendiente); wire u128 >2^53 en 4 bindings → (pendiente); rg Generic( → (pendiente); verify_changed.ps1 → (pendiente)
      evidencia:
        - claim: "Write.node_id sin u128_serde (graph.rs:18-24) vs StaleContext con él (:29)"
          evidencia: src/sdk/types/graph.rs:18-31 (leído verbatim)
          confianza: alta
        - claim: "Gate P decidió u128→string decimal + camelCase/kebab/nativo + errores code+message+context"
          evidencia: docs/dev/tasks/API-STD-15.md:13-32
          confianza: alta
        - claim: "VantaHeader tiene 4 callers; renombrar no afecta on-disk (serialización custom)"
          evidencia: src/binary_header.rs:20,49-84 + codegraph blast radius
          confianza: media (confirmar leyendo ADR-041 completo)
      artefactos:
        - docs/dev/tasks/API-01.md
      invariantes: u128_serde único patrón wire; validate_compat intacto; vantadb-pro intocable; sin panic en bindings
      deuda: P2-8 pagada en este task; P2-5 diferida a API-02 (declarada)
      queda_pendiente: codegen single-schema (DEFER si no hay consenso); tipado por-variante completo de errores (incremental)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** pagar P2-8 (eliminar `collect_all_deduped()` O(n) con `HashSet` amortizado) — saldo neto ≤0.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ + `cargo test --test sdk_serialization` + tests wire por binding + fmt/clippy/nextest |
| **Commit** | Atómico por step (~100 líneas), conventional commit `feat!:` + `API-01` (breaking wire), verificación mecánica (`campaign_verify_cmd`) |
| **Release** | `dev-tools/verify.ps1` completo; docs mismo-PR (Regla 3); release-plz decide versión/tag (Regla 7 — no tocar Cargo.toml/CHANGELOG/tags) |

## Herramientas necesarias
- Terminal: `cargo` (check/test/clippy/fmt), `pytest`, `npx tsc`, `npm test`
- `codegraph_explore` (blast radius), `campaign_*` (verify/state), `dev-tools/verify_changed.ps1`
- **Skills cargadas (SDP):** campaign-executor (base task system) · source-driven-development (APIs externas: serde/FFI) · incremental-implementation (slices verticales) · test-driven-development (RED→GREEN wire tests) · context-engineering (tarea multi-toolchain) · doubt-driven-development (stakes altos: breaking wire) · api-and-interface-design (contratos públicos). Descartadas: frontend-ui-engineering (no toca `web/`), performance-optimization (solo P2-8 complejidad, sin claim de perf), systematic-debugging (no es bug con repro), documentation-and-adrs (solo si ADR-041 se firma).

## Investigation Notes
- Gate P (API-STD-15) aprobó 4/4 recomendadas; decisiones de W0 citadas en §Spec con fuente por fila.
- API-STD-02 confirmó 5/5 fallos en código (R1 stutter `VantaHeader`, R2 catch-all, R3 docs `FilterOp`, R4 wire >2^53, R5 breaking históricos).
- TS `types.ts:43-46` ya declara `created_at_ms: string | number` (tolera ambos) — el wire target es string decimal.
- WASM `.d.ts:142-153` ya documenta "policy string-u64" para timestamps y `node_id: string`.
- `json_to_vanta_value` (`cli_handlers/crud.rs:445`) convierte números JSON a `Value::Int/Float` — verificar si afecta el roundtrip de ids (no es u128, pero es el conversor de filtros CLI).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 2 — codegen dueño; firma ADR-041 owner |
| Pendientes de ejecución (downhill) | 1 — Step 5 (firma owner) |
| % completado | 100% (9/9 steps ✅ · contrato mecánico 100%) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — toca fronteras FFI (wire u128 en 4 bindings) y contrato de errores de bindings (`to_js_err`/`map_vanta_error`): cargar `security-and-hardening` si se toca el mapeo de excepciones; sin `unsafe` nuevo; no relajar validación.
- [x] **PERFORMANCE** — P2-8: `collect_all_deduped()` O(n) por dedup → `HashSet<u128>` (complejidad, no claim de perf; sin benchmark por Regla 9 al no declarar mejora cuantificada). `u128_serde` en `Write` es raro (no hot path).

## Steps

### Step 0: Lectura completa + Impacto mapeado (GATE Regla 0)
- **Archivos:** todos los de §Impacto mapeado (pendientes)
- **Acción:** leer completos los archivos a modificar; completar la tabla de impacto con referencias entrantes (`git grep`) y veredicto
- **Verify:** tabla §Impacto mapeado completa + `git grep -c` por archivo
- **Estado:** ✅ DONE 2026-09-25 — leídos completos: `record.rs`, `error.rs` (1348L), `binary_header.rs`, `lib.rs`, `vantadb-ts/src/types.ts`, `vantadb-node/index.d.ts`, `vantadb-python/src/types.rs`, `vantadb-wasm/src/lib.rs` (función objetivo + tests vecinos), `adr/041`, `ERROR_HANDLING.md`. Nota: `vantadb-wasm/src/lib.rs:752-784` (no `:564-596` — líneas del task file stale, ver Step 7).

### Step 1: RED — test wire u128 >2^53 para `Write.node_id`
- **Archivos:** `tests/sdk_serialization.rs`
- **Acción:** test que serializa `QueryResult::Write { node_id: Some(u128 > 2^53) }` a JSON y afirma string decimal; verificar que HOY FALLA (RED)
- **Verify:** `cargo test --test sdk_serialization` → fallo esperado en el test nuevo
- **Estado:** ✅ DONE 2026-09-25 (RED verificado: JSON emitió número `9223372036854775815`)

### Step 2: GREEN — `u128_serde` en `Write.node_id`
- **Archivos:** `src/sdk/types/graph.rs:24`
- **Acción:** `#[serde(serialize_with/deserialize_with)]` con helpers Option-aware nuevos (`serialize_opt`/`deserialize_opt` — el helper original solo cubría `u128` no-Option)
- **Verify:** `cargo test --test sdk_serialization` verde
- **Estado:** ✅ DONE 2026-09-25 (17/17 tests verdes; `verify_changed` 4/4: fmt/check/clippy/docs-coverage)

### Step 3: docs por variante `FilterOp`
- **Archivos:** `src/sdk/types/record.rs:12-20`
- **Acción:** `///` por variante (Eq/Neq/Gt/Lt/Gte/Lte) con semántica y tipo de valor esperado
- **Verify:** `cargo doc -p vantadb --no-deps` sin warnings de missing_docs en `FilterOp`
- **Estado:** ✅ DONE 2026-09-25 — docs por variante + semántica `PartialOrd` derivado sobre `Value` (same-variant natural; cross-variant = orden de declaración). Verify: `cargo doc -p vantadb --no-deps` exit 0, 36.7s, sin warnings.

### Step 4: error envelope + doc-diseño del catch-all
- **Archivos:** `docs/api/ERROR_HANDLING.md`, `src/error.rs:182-288`
- **Acción:** documentar envelope `code+message+context` (sin panic) y el rol by-design de `Generic`; tipar solo variantes sin callers que lo justifiquen (evaluar `ResourceLimit(String)`/`Schema(String)`/`InvalidInput(String)`); `#[non_exhaustive]` si aplica (P2-6 ya resuelto — no reintroducir)
- **Verify:** `rg 'Generic\(' src/error.rs` con tipado o doc-diseño citado en ERROR_HANDLING.md
- **Estado:** ✅ DONE 2026-09-25 — envelope documentado en `error.rs` (module doc) + ERROR_HANDLING.md (principio 7 + §"The `Generic` catch-all (by design)" + changelog). Evaluación de tipado: `ResourceLimit` 12 callers, `Schema` 14, `InvalidInput` 64, `DatabaseBusy` 14, `NoVectorForKey` 4 → **doc-diseño by-design** en los 5 String + `Generic` (R-6 deuda incremental, NO romper callers). `#[non_exhaustive]` ya presente (`error.rs:121`). Verify: `cargo doc` exit 0 (17.9s); `rg -n -B4 'Generic\('` muestra doc-diseño.

### Step 5: cierre ADR-041 + decisión `VantaHeader`
- **Archivos:** `docs/dev/architecture/adr/041_anti_stutter.md`, `src/binary_header.rs:20`, `src/lib.rs:167`
- **Acción:** leer ADR completo; decidir A/B/C del Spec #5; si renombrar → `git grep VantaHeader` completo + actualizar re-export; si firmar → estado `accepted` + firma
- **Verify:** ADR sin `proposed`; `cargo check --workspace` verde; `git grep VantaHeader` sin residuos
- **Estado:** ✅ DONE 2026-09-25 (decisión owner B: rename `VantaHeader`→`Header` + alias deprecated; 66 refs código + 17 docs; ADR exclusión retirada + decisión registrada; mapa `resolved`). Verify: check/clippy/verify_changed verdes (commit del lead).

### Step 6: fundación casing (tipos de bindings)
- **Archivos:** `vantadb-ts/src/types.ts`, `vantadb-wasm/src/vantadb_wasm.d.ts`, `vantadb-node/index.d.ts`, `vantadb-python/src/types.rs`
- **Acción:** aplicar la norma Gate P en los tipos tocados por W0 (camelCase JSON/MCP; nativo snake/camel; `js_name` PyO3/NAPI si aplica); NO migrar aún los métodos (eso es W1/API-02)
- **Verify:** `npx tsc --noEmit` (ts) + `cargo check -p vantadb-python` (si aplica) + diff revisado
- **Estado:** ✅ DONE 2026-09-25 — norma declarada en `docs/api/BINDINGS_NAMESPACES.md` § Casing Contract (tabla por superficie + reglas de ventana de migración "one casing per payload") + punteros en los 4 archivos de tipos. **Cero renames** de métodos/campos (migración = W1/API-02, por diseño). Verify: `npx tsc --noEmit` exit 0 (3.8s); `cargo check -p vantadb_py` exit 0 (22s).

### Step 7: pagar P2-8 (`collect_all_deduped` O(n) → HashSet)
- **Archivos:** `vantadb-wasm/src/lib.rs:564-596`
- **Acción:** dedup por `node_id` con `HashSet<u128>`; conservar orden de primera aparición
- **Verify:** `wasm-pack build --release` o `cargo check -p vantadb-wasm` + tests wasm existentes
- **Estado:** ✅ DONE (pre-existente, verificado 2026-09-25) — **el task file referenciaba líneas stale**. El fix real vive en `vantadb-wasm/src/lib.rs:752-784`: `seen: HashSet<u128>` + `seen.insert(record.node_id)` conservando orden de primera aparición (commit `9dcbff5a perf(wasm): dedup collect_all_deduped by u128 node_id (AUD-043)`, 2026-08-16). Test existente: `test_collect_all_deduped_no_duplicates` (`:2720-2759`). Sin cambios necesarios → deuda neta ≤0 confirmada.

### Step 8: tests wire u128 en 4 bindings + verify final
- **Archivos:** `vantadb-python/tests/`, `vantadb-ts/`, `vantadb-node/`, `vantadb-wasm/` (test files por binding)
- **Acción:** test por binding: id >2^53 hace roundtrip como string decimal (Py: `json.dumps`; TS/WASM: serialización wasm; Node: napi); correr verificación mecánica completa
- **Verify:** `cargo test --test sdk_serialization` + tests por binding verdes + `dev-tools/verify_changed.ps1` verde
- **Estado:** ✅ DONE 2026-09-25 — core ✅ 17/17; **Python ✅ 2/2** (`tests/test_wire_u128.py`: node_id int exacto + `query_structured` node_id string); **Node ✅ 28/28** (`api.test.ts`: roundtrip exacto string, `BigInt > 2^53`); **TS ✅ 1/1** (`tests/wire-u128.test.ts`); **WASM ✅ 30/30** (`wasm-pack test --node`, incluye `api01_wire_u128_tests::query_write_node_id_above_2_53_is_decimal_string` y `test_collect_all_deduped_no_duplicates`). Rebuild `wasm-pack build --release` (3m05s) + `node dev-tools/build-wasm-types.mjs`; verificación manual del pkg fresco: `typeof: string | value: 9007199254740993` (sin pérdida f64). El test TS/WASM aceptan `string|bigint` porque representación depende del serializer del boundary; un `number` (f64) se rechaza siempre.

## Dependencias
- Ninguna (primera de la ola). Desbloquea: API-02, API-03, API-06, API-08.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (subagente, contexto fresco) — pendiente al cierre
- **Enfoque:** ¿el approach del wire (string decimal) y el envelope de errores es el correcto? ¿alternativas mejores?
- **Cómo se probó:** evidencia de verificación real (comandos + outputs), no auto-reporte
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica antes de aprobar):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⏳ pendiente

## Notas
- Commits: `feat!:` + `API-01` (breaking wire — Regla 7); docs mismo-PR (Regla 3); deuda neta ≤0 (Regla 6, P2-8).
- El plan padre (`api-ejecucion.md`) marca `Autonomous: false`: la ejecución se lanza con GO explícito del owner.
- Stop conditions del plan: appetite >1sem → partir tipos vs errores; codegen sin consenso → manual 1 vez + DEFER codegen.
- Gate P ya aprobó 4/4 recomendadas (score-todos, array-objetos, cursor+has_more, core-only memory) — no re-abrir.
- **Progreso 2026-09-25 (lead):** Steps 1-2 ✅ (RED→GREEN). Nota de entorno: el MCP server de esta sesión corre `target/debug/vanta-cli.exe` → el relink del bin está bloqueado; los builds de test usan target dir dedicado `target/session-api01` (check/clippy/fmt del gate no lockean). Evidencia: RED `node_id: 9223372036854775815` → GREEN 17/17 + `verify_changed` 4/4. Pendiente: Steps 3-8 (docs `FilterOp`, error envelope, ADR-041, casing, P2-8, tests wire 4 bindings).
- **Progreso 2026-09-25 (sesión 2 / vanta-worker):** Steps 3, 4, 6 ✅; Step 0 ✅ (lecturas completas). Step 5 evidencia+recomendación B listas (firma owner = BLOQUEO). Step 7 ✅ pre-existente (commit `9dcbff5a`, HashSet en `lib.rs:752-784`, test `:2720`). Step 8: core 17/17, Python 2/2, Node 28/28, TS 1/1 (pkg wasm prebuilt = bigint exacto; rebuild `wasm-pack build --release` en curso para verificar string del `u128_serde`); WASM test inline añadido, run `wasm-pack test --node` pendiente. Verify: `cargo doc` ×2 ✅, `tsc --noEmit` ✅, `cargo check -p vantadb_py` ✅.
- **Cierre verificado 2026-09-25 (sesión 2):** contrato mecánico COMPLETO — `cargo test --test sdk_serialization` 17/17 ✅ · wire u128 >2^53 en los 4 bindings ✅ (Py `test_wire_u128.py` 2/2; TS `wire-u128.test.ts` 1/1; Node `api.test.ts` 28/28; WASM `wasm-pack test --node` 30/30) · `rg 'Generic\('` con doc-diseño ✅ · `dev-tools/verify_changed.ps1` 4/4 ✅ (fmt/check/clippy/docs-coverage, `target/session-api01`). Rebuild wasm: `wasm-pack build --release` 3m05s + `node dev-tools/build-wasm-types.mjs`; pkg fresco verificado manualmente → `typeof string, value 9007199254740993`. OCR preview ejecutado (advisory; sin API key no hay rule delegation) → sin Critical/High detectables; self-check contra `api-contract.md` R-1..R-8 OK (solo doc comments + tests; sin unwrap/unsafe/deps nuevas en código no-test). **WIP ajeno detectado en el working tree** (sesión concurrente WIRE-09: `src/sdk/api.rs`, `src/storage/engine/mod.rs`, `vanta-proxy/*`, etc.) — NO tocar/commitear como parte de API-01.
- **Deuda/hallazgos de esta iteración (FIND candidatos, no creados aún):** (a) `vantadb-ts/src/types.ts` declara `Write.node_id?: string` mientras el pkg WASM prebuilt devuelve `bigint` — alinear tras rebuild verificado; (b) `vantadb-wasm/src/vantadb_wasm.d.ts:421-425` documenta shape `IqlResult {kind}` que no coincide con el `QueryResult` externamente etiquetado (`{Read}|{Write}|…`) — drift ya anotado en `vantadb.ts:1112`; (c) `scripts/anti_stutter_map.json:94` cita razón incorrecta para la exclusión `VantaHeader` (ver ADR §Evidencia adicional).
