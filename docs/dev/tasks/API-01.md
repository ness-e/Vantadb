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
- **Estado:** ⏳ IN PROGRESS (Steps 1-2 ✅ 2026-09-25)
- **Incógnitas (uphill):** 3 abiertas (dueño codegen single-schema; alcance tipado errores; alcance fundación casing)
- **Pendientes (downhill):** 6 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `QueryResult` lo leen los 4 bindings + HTTP (`src/server/handlers.rs:1164` doc) + MCP; `VantaHeader` 4 callers (`src/migration.rs`, `src/lib.rs:167` re-export, `src/storage/vfile.rs`) + `validate_compat` 5 callers (`src/index/serialize/bytes.rs`, `binary_header.rs`, `vfile.rs`, `wal.rs`); `FilterOp` en `vantadb-ts/src/types.ts:207`, `vantadb-wasm/src/vantadb_wasm.d.ts:198`, `vantadb-node/index.d.ts:228` |
| Callees | `u128_serde` (`src/sdk/types.rs:32`, crate-private) usado por `record.rs:95`, `graph.rs:29`, `serialization/*`; `VantaError` core (`src/error.rs`) consumido por todos los bindings vía `map_vanta_error`/`to_js_err` |
| Implicaciones | `Write.node_id` sin `u128_serde` pierde IDs >2^53 en wire JSON (solo writes); cambiarlo es breaking wire (`feat!:`); `validate_compat` es hot path de lectura on-disk (no romper); `VantaHeader` no es serde (serialización custom por magic bytes → renombrar NO afecta on-disk) |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN:** el ejecutor debe leer COMPLETOS los archivos a modificar antes del primer edit y completar esta tabla (los paths ya fueron leídos parcialmente en discovery; el gate exige lectura completa en ejecución).

- **Archivos leídos (completos):** `src/sdk/types/graph.rs` ✅ (leído completo en discovery), `src/binary_header.rs` ✅ (vía codegraph verbatim), `src/error.rs:178-297` ✅ (rango objetivo); **pendiente de lectura completa en ejecución:** `src/sdk/types/record.rs`, `src/sdk/types.rs`, `src/error.rs` (completo), `src/sdk/version_history.rs`, `src/cli_handlers/crud.rs:445` (`json_to_vanta_value`), `vantadb-wasm/src/lib.rs:564-596` (P2-8), `vantadb-python/src/types.rs`, `vantadb-ts/src/types.ts`, `vantadb-node/index.d.ts`, `docs/dev/architecture/adr/041_anti_stutter.md`
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
| 5 | `VantaHeader` / ADR-041 | A: firmar ADR-041 (excepción permanente) / B: renombrar (anti-stutter; no afecta on-disk — serialización custom por magic bytes) / C: dejar proposed | decidir con ADR-041 leído completo | ⏳ incógnita — resolver en ejecución con evidencia (`adr/041_anti_stutter.md:43,56,64`) |

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
| Incógnitas abiertas (uphill) | 3 — codegen dueño; alcance tipado errores; alcance fundación casing |
| Pendientes de ejecución (downhill) | 8 — steps de abajo |
| % completado | 22% (2/9 steps) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — toca fronteras FFI (wire u128 en 4 bindings) y contrato de errores de bindings (`to_js_err`/`map_vanta_error`): cargar `security-and-hardening` si se toca el mapeo de excepciones; sin `unsafe` nuevo; no relajar validación.
- [x] **PERFORMANCE** — P2-8: `collect_all_deduped()` O(n) por dedup → `HashSet<u128>` (complejidad, no claim de perf; sin benchmark por Regla 9 al no declarar mejora cuantificada). `u128_serde` en `Write` es raro (no hot path).

## Steps

### Step 0: Lectura completa + Impacto mapeado (GATE Regla 0)
- **Archivos:** todos los de §Impacto mapeado (pendientes)
- **Acción:** leer completos los archivos a modificar; completar la tabla de impacto con referencias entrantes (`git grep`) y veredicto
- **Verify:** tabla §Impacto mapeado completa + `git grep -c` por archivo
- **Estado:** ⬜ PENDING

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
- **Estado:** ⬜ PENDING

### Step 4: error envelope + doc-diseño del catch-all
- **Archivos:** `docs/api/ERROR_HANDLING.md`, `src/error.rs:182-288`
- **Acción:** documentar envelope `code+message+context` (sin panic) y el rol by-design de `Generic`; tipar solo variantes sin callers que lo justifiquen (evaluar `ResourceLimit(String)`/`Schema(String)`/`InvalidInput(String)`); `#[non_exhaustive]` si aplica (P2-6 ya resuelto — no reintroducir)
- **Verify:** `rg 'Generic\(' src/error.rs` con tipado o doc-diseño citado en ERROR_HANDLING.md
- **Estado:** ⬜ PENDING

### Step 5: cierre ADR-041 + decisión `VantaHeader`
- **Archivos:** `docs/dev/architecture/adr/041_anti_stutter.md`, `src/binary_header.rs:20`, `src/lib.rs:167`
- **Acción:** leer ADR completo; decidir A/B/C del Spec #5; si renombrar → `git grep VantaHeader` completo + actualizar re-export; si firmar → estado `accepted` + firma
- **Verify:** ADR sin `proposed`; `cargo check --workspace` verde; `git grep VantaHeader` sin residuos
- **Estado:** ⬜ PENDING

### Step 6: fundación casing (tipos de bindings)
- **Archivos:** `vantadb-ts/src/types.ts`, `vantadb-wasm/src/vantadb_wasm.d.ts`, `vantadb-node/index.d.ts`, `vantadb-python/src/types.rs`
- **Acción:** aplicar la norma Gate P en los tipos tocados por W0 (camelCase JSON/MCP; nativo snake/camel; `js_name` PyO3/NAPI si aplica); NO migrar aún los métodos (eso es W1/API-02)
- **Verify:** `npx tsc --noEmit` (ts) + `cargo check -p vantadb-python` (si aplica) + diff revisado
- **Estado:** ⬜ PENDING

### Step 7: pagar P2-8 (`collect_all_deduped` O(n) → HashSet)
- **Archivos:** `vantadb-wasm/src/lib.rs:564-596`
- **Acción:** dedup por `node_id` con `HashSet<u128>`; conservar orden de primera aparición
- **Verify:** `wasm-pack build --release` o `cargo check -p vantadb-wasm` + tests wasm existentes
- **Estado:** ⬜ PENDING

### Step 8: tests wire u128 en 4 bindings + verify final
- **Archivos:** `vantadb-python/tests/`, `vantadb-ts/`, `vantadb-node/`, `vantadb-wasm/` (test files por binding)
- **Acción:** test por binding: id >2^53 hace roundtrip como string decimal (Py: `json.dumps`; TS/WASM: serialización wasm; Node: napi); correr verificación mecánica completa
- **Verify:** `cargo test --test sdk_serialization` + tests por binding verdes + `dev-tools/verify_changed.ps1` verde
- **Estado:** ⬜ PENDING

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
