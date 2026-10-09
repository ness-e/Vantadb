---
title: "TASK MEMG-07: Forgetting curves sobre L1"
kind: task
description: "Curva de olvido real sobre L1 (vanta-memory): política declarada por tipo (half-life configurable) + retención exponencial por edad + pass read-only con métrica; la curva deprioriza (effective_heat), nunca purga; sin campos nuevos en el wire; descarte automático → FIND"
---

# TASK MEMG-07: Forgetting curves sobre L1

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 42, F2 — Memoria I)
- **Fuente:** Backlog `MEMG-07` (L142; "curvas de olvido reales sobre L1 — el decay existe; el descarte no") + plan Task 42 (bloque F0-expandido, L1204-1230) + FUT-10 (`backlog-futuro.md:24`) + N-09 (`backlog-notion.md:28`)
- **Esfuerzo:** 🟡 2d | **Appetite:** max 3d | **Stop (plan L1215):** 2d → entregar decay configurable por tipo + test + métrica; FIND del descarte automático si no cierra
- **Prioridad:** 🟡
- **Tipo:** Rust (crate `vanta-memory`; core SDK consumido sin modificar)
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `42` en el campaign server)
- **Incógnitas (uphill):** 1 — parametrización de la curva (resuelta en DISCOVERY: política declarada, sin canónico → ver §Spec #1/#5)
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `42`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Símbolos nuevos (aditivos — sin callers preexistentes):** `core::record::lifecycle::{DecayPolicy, retention_factor, effective_heat, scan_decay, DecayReport}` (consumidores: tests + `l1_reader::run_decay_pass` + MEMG-21 futuro) · `core::record::l1_reader::run_decay_pass` (consumidor: hosts/scheduler futuro — pull-based). **Callers de lo existente que NO cambia:** `decay_heat`/`bump_heat`/`is_prune_eligible`/`PRUNE_HEAT_THRESHOLD` (tests `heat_decay.rs` + unit tests de `lifecycle.rs`) mantienen firma y semántica — cero ripple. `MemoryType` gana derive `Hash` (aditivo, sin cambio de comportamiento; recompila usuarios del tipo). |
| Callees | Core SDK existente (sin cambios): `Embedded::list` vía `read_session_records` (`l1_reader.rs:28`). Crates ya presentes: `serde`/`serde_json`/`chrono` (parse RFC3339 — ya usado por `l1_writer.rs:541`). **Cero dependencias nuevas.** |
| Implicaciones | **API pública aditiva** en `vanta-memory` (crate publicable DIST-01) — sin breaking, **sin cambios de wire** (`MemoryRecord` no gana campos; serde defaults intactos; MEMG-13 backup/restore y MEMG-20 checkpoints no afectados). Sin locks nuevos, sin Tokio, sin I/O nuevo (el pass lee vía la superficie existente). Regresión: suite `-p vanta-memory` se re-corre; ningún test existente toca los símbolos nuevos. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `1820af61`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/core/record/lifecycle.rs` (:1-295 completo — `DEFAULT_HEAT:32`, `PRUNE_HEAT_THRESHOLD:40` (=1), `bump_heat:62`, `decay_heat:72` (shift-right por pass), `is_prune_eligible:111`, `mark_contradiction:86`, tests inline `:188-295`; header `:14` declara "no float decay curve" — MEMG-07 es esa curva).
  - `vanta-memory/tests/heat_decay.rs` (:1-188 completo — fixture canónico de `MemoryRecord` + test de persistencia bump→decay→prune; **no se modifica**).
  - `vanta-memory/src/core/record/mod.rs` (:1-53 completo — re-exports del pipeline L1; doc `:11-15` describe `decay_heat` en el "periodic maintenance pass").
  - `vanta-memory/src/core/abstractions/types.rs` (:1-199 — `MemoryType:25` (7 variantes, sin `Hash`), `MemoryRecord:71-130` (`created_at`/`updated_at` ISO, `heat:125`, `priority`, `superseded_by`)).
  - `vanta-memory/src/core/record/l1_reader.rs` (:1-104 + :244-390 — `l1_namespace:23`, `read_session_records:28` (paginado, `include_quarantined:false` SCH-05), `read_namespace_records:38`, `recall_candidates:247`, `rrf_merge:329`; surface que MEMG-21 extiende).
  - `vanta-memory/src/core/record/l1_writer.rs` (secciones :31-52, :282-291, :488-557 — `L1Error`, `EmbedFn`, `persist_planned`, `record_input`, `put_record:521`, `rfc3339_to_epoch_ms:540` (parse RFC3339 con chrono — precedente de parseo)).
  - `vanta-memory/src/utils/managed_timer.rs` (:1-70 — trait `Clock:15`, `SystemClock:22`, `FakeClock:35`; patrón de tiempo inyectable).
  - `vanta-memory/src/utils/mod.rs` (:1-30 — re-exports; patrón aditivo).
  - `vanta-memory/Cargo.toml` (:1-80 — deps: serde/serde_json/thiserror/toml/tracing/chrono; dev-deps: tempfile; features lean).
  - `src/eviction.rs` (:90-160 — `BayesianDecay` feature-gated `bayesian_decay`; scoring de **eviction del core**, no L1 — referenciado, no consumido).
  - Docs/specs: master plan Task 42 (L1204-1230) + F2 (L1059-1061) + Task 44/MEMG-21 (L1260-1287), `docs/dev/Backlog.md` (MEMG-07:142, MEMG-21:169, FUT-10:509, FIND-285..288:444-447), `docs/dev/backlog-futuro.md:24` (FUT-10), `docs/dev/backlog-notion.md:28` (N-09), `docs/api/VANTA_MEMORY.md` (:300-507 — formato de sección + Debts), `docs/dev/tasks/MEMG-20.md` (formato canónico), `.opencode/rules/api-contract.md` (R-6/R-7/R-8), `.opencode/references/clean-code-clean-architecture.md` (Apéndice V).
  - Web (fuente de la forma, no de la calibración): Wikipedia "Forgetting curve" (https://en.wikipedia.org/wiki/Forgetting_curve, revisión oldid=1369227085) — forma exponencial `R = e^(−t/S)` y la nota "simple equations such as this one were not found to provide a good fit" (soporta política declarada, no calibración).
- **Archivos referenciados hacia dentro (imports/deps):** `crate::core::abstractions::{MemoryRecord, MemoryType}` (existentes), `crate::core::record::L1Error` (existente), `crate::core::record::l1_reader::read_session_records` (existente, usado desde `l1_reader` mismo), `serde`/`serde_json`/`chrono` (ya en el crate), `std::collections::HashMap`.
- **Referencias entrantes (grep HEAD):** `DecayPolicy|retention_factor|effective_heat|scan_decay|run_decay_pass|DecayReport` en `vanta-memory/` = **0 hits** (verificado 2026-10-05; solo un comentario contiene "forgetting" en `mod.rs:13`). `decay_heat` = callers solo tests (`lifecycle.rs` inline + `heat_decay.rs`); sin caller de producción (no existe pass in-repo). `MemoryType` = usuarios múltiples (derive aditivo).
- **Veredicto impacto:** **BAJO (aditivo puro)** — 2 archivos de código tocados por extensión (`lifecycle.rs`, `l1_reader.rs`), 1 derive aditivo (`types.rs`), 1 línea de re-export (`mod.rs`), 1 test nuevo, docs. Sin cambios de firma/semántica en lo existente, sin wire, sin deps, sin locks. Pre-mortems mitigados: (1) fórmula sin canónico → política declarada + [a verificar en DISCOVERY] cumplido (FUT-10/N-09 + fuente web; sin claims de precisión); (2) compat con heat/umbrales → **extender por capas** (heat intacto como señal; curva read-side; shift legacy sin tocar); (3) borrado por curva → **read-only por diseño** (nunca escribe; el gate explícito queda intacto).

## Contrato

"Curva de olvido real sobre L1 (plan L1213): (a) **decay por tipo/edad, configurable** — `DecayPolicy` con half-life por `MemoryType` (defaults declarados) y `retention_factor(record, policy, now) = 2^(−age/half_life)` (familia exponencial `R=e^(−t/S)`; edad = tiempo desde el último toque `updated_at`); (b) **integración con el pass** — `run_decay_pass(db, session_key, policy, now)` sobre `read_session_records` + `scan_decay(records, policy, now)` puro (pull-based, patrón `TimerScanner::run_once`), **read-only**; (c) **métrica** — `DecayReport { scanned, decayed, unchanged, exempt, below_threshold, heat_total, heat_effective }` + `heat_forgotten()` (cuánto decae / qué no expira / qué cruzaría el umbral del gate explícito); (d) **sin pérdida** — la curva deprioriza (`effective_heat`) y **jamás borra ni muta** registros; el descarte sigue siendo el gate explícito (`PRUNE_HEAT_THRESHOLD` intacto); (e) **fórmula documentada como política** (no claim calibrado — N-09 abierto; FUT-10 trackea la visión completa frecuencia/importancia → MEMG-21). Parámetros declarados y testeados con valores conocidos (half-lives pinneados + retención exacta en 0/1/2 half-lives + report con totales exactos). Verify: `cargo nextest run --profile audit -p vanta-memory --test forgetting_curve --build-jobs 2` + suite `-p vanta-memory --build-jobs 2` + `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` verdes."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato del plan (Task 42, Gate Result ✅ DO) sanciona la "curva de olvido real sobre L1 (decay por tipo/edad, configurable; parámetros declarados)" + "integración con el pass" + "métrica", y su stop condition fija el mínimo ("decay configurable por tipo + test + métrica; FIND del descarte automático"). Los símbolos nuevos SON el mecanismo sancionado por el plan; el pre-mortem #3 fija el límite duro ("la curva jamás borra"). Sin símbolos fuera de la sanción. Precedente de campaña idéntico: MEMG-20/MEMG-13 ("Gate D: pre-respondido por el plan F0").

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Fórmula de la curva | A) **exponencial parametrizada por half-life: `R = 2^(−age/hl)`** (familia `R=e^(−t/S)` de Wikipedia §Equations; half-life interpretable y testeable con valores exactos en 0/1/2 half-lives) / B) ecuación original de Ebbinghaus `b=100k/((log t)^c+k)` (fit histórico; `t` en minutos, constantes sin sentido para L1) / C) shift-right actual `decay_heat` (no es curva por edad — por-pass) | ✅ **A** — decidido-por-evidencia: forma exponencial más simple (Wikipedia: "perhaps the simplest being an exponential curve"); half-life declarable; la misma fuente advierte que incluso esta forma "was not found to provide a good fit" → política declarada, **no calibración** (N-09) |
| 2 | Señal de edad | A) **`updated_at`** (último toque; `bump_heat` lo actualiza en cada acceso → la retención se resetea con el uso, Ebbinghaus-style review) / B) `created_at` (no refleja accesos) / C) campo nuevo `last_access_at` (wire change innecesario) | ✅ **A** — decidido-por-evidencia: `bump_heat:64` ya escribe `updated_at = now`; es la señal de recencia existente (MEMG-21 la usa como base) |
| 3 | Materialización del decay | A) **read-side: `heat` almacenado intacto; `effective_heat` derivado de (heat, edad, tipo)** — idempotente por construcción, sin writes, sin wire change, "deprioriza sin purgar" (contrato) / B) mutar `heat` en el pass (no idempotente sin campo-ancla nuevo — doble decay al re-correr; o doble-decay aceptado como el shift legacy) | ✅ **A** — decidido-por-evidencia: contrato "la curva deprioriza, nunca purga"; idempotencia exigible a un pass de mantenimiento (re-run/restart); el shift legacy `decay_heat` queda intacto (compat pre-mortem #2) |
| 4 | Configurabilidad | A) **`DecayPolicy { half_life_ms: HashMap<MemoryType, u64> }`** (privado + getter/setter; tipo ausente = exento) / B) constante global (no configurable) / C) env vars (fuera del patrón del crate) | ✅ **A** — decidido-por-evidencia: "configurable" del contrato; `HashMap` con `MemoryType` key (requiere derive `Hash` aditivo); exención = ausencia (expresa "qué no expira") |
| 5 | Defaults por tipo | A) **tabla declarada**: Persona 90d · Episodic 7d · Instruction **exento** · WorkFact 30d · WorkTask 14d · WorkMethod 60d · WorkArtifact 30d / B) half-life única global / C) valores "calibrados" (no existe canónico → inventar precisión) | ✅ **A** — decidido-por-evidencia: FUT-10 pide curva Ebbinghaus con frecuencia/importancia/confirmaciones/saliencia (visión completa); N-09: "ninguna fórmula canónica validada" → half-lives son **política declarada** (redondas, tuneables); frecuencia/importancia quedan para MEMG-21 (scoring compuesto); [a verificar en DISCOVERY] cumplido contra FUT-10 |
| 6 | Exención de `Instruction` | A) **exento (sin half-life)** — instrucciones estrictas (`priority = -1`) se siguen hasta ser contradichas; olvidarlas por curva = falla de corrección / B) half-life larga (180d) | ✅ **A** — decidido-por-evidencia: semántica de `Instruction` en `types.rs:30-31`; la exención es configurable (cualquier tipo puede exentarse con `clear_half_life`) |
| 7 | Pass | A) **`run_decay_pass(db, session_key, policy, now)` = `read_session_records` + `scan_decay` (read-only; en `l1_reader.rs`, dirección de import limpia) + `scan_decay(records, policy, now)` puro** / B) pass mutante (ver #3) / C) scheduler propio (fuera: WIRE-15 es el servicio futuro) | ✅ **A** — decidido-por-evidencia: no existe pass productivo in-repo (`decay_heat` sin callers; `services/` = pipeline_worker + conversation_hook); pull-based como `TimerScanner::run_once` ("the owner calls run_once"); el wrapper db vive en `l1_reader` (lifecycle→l1_reader crearía ciclo con `l1_writer→lifecycle`) |
| 8 | Métrica | A) **`DecayReport { scanned, decayed, unchanged, exempt, below_threshold, heat_total, heat_effective }` + `heat_forgotten()`** — "cuánto decae" (totales), "qué no expira" (exempt), input del gate (below_threshold) / B) solo un contador de decayed (pierde los totales) | ✅ **A** — decidido-por-evidencia: contrato pide "métrica (cuánto decae / qué no expira)"; categorías disjuntas (`decayed`/`unchanged`/`exempt`) + `below_threshold` ortogonal |
| 9 | Umbral del gate | A) **reutilizar `PRUNE_HEAT_THRESHOLD` (=1) sobre `effective_heat`** para `below_threshold`; `is_prune_eligible` (raw) sin cambios / B) umbral nuevo (doble semántica) | ✅ **A** — decidido-por-evidencia: un solo umbral en el dominio (R-8/reuse); el descarte sigue siendo gate explícito con el mismo umbral |
| 10 | Wire/compat | A) **sin campos nuevos en `MemoryRecord`; `decay_heat`/`bump_heat`/`is_prune_eligible`/`PRUNE_HEAT_THRESHOLD` intactos** / B) reemplazar el shift por la curva (rompe tests/umbrales — pre-mortem #2) | ✅ **A** — decidido-por-evidencia: pre-mortem #2 ("extender por capas"); cero cambios de wire (backup/restore MEMG-13 y checkpoints MEMG-20 intactos) |
| 11 | Consumo in-repo | A) **API + tests + docs + FIND del descarte automático (stop L1215)** — el pass no se wirea a ningún scheduler (no existe; WIRE-15) / B) wire especulativo a `pipeline_worker` | ✅ **A** — decidido-por-evidencia: stop condition del plan corta exactamente ahí; sin consumer verificado (patrón FIND-288) |
| 12 | `Hash` en `MemoryType` | A) **derive `Hash` aditivo** (requerido por `HashMap` key) / B) `BTreeMap` + `PartialOrd/Ord` (más derives) / C) array indexado (match manual, más código) | ✅ **A** — decidido-por-evidencia: cambio mínimo; derives no afectan comportamiento ni wire |
| 13 | Parseo de timestamps | A) **`chrono::DateTime::parse_from_rfc3339`** (ya en el crate — `l1_writer.rs:541`, `md_import.rs:270`) con fallback `updated_at` → `created_at` → sin decay (retención 1.0) / B) inverso hand-rolled de `millis_to_iso8601` (~30 líneas, error-prone) | ✅ **A** — decidido-por-evidencia: reuse de dependencia existente; el fallback "no se puede envejecer → no se olvida" es el sesgo seguro (sin pérdida) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **La curva jamás borra ni muta:** `scan_decay`/`run_decay_pass` son read-only; ningún registro se elimina, y `heat`/`updated_at` almacenados no cambian. El descarte sigue siendo el gate explícito (`is_prune_eligible` + `PRUNE_HEAT_THRESHOLD` intactos).
  2. **Compat de capas:** `bump_heat`/`decay_heat`/`is_prune_eligible`/`PRUNE_HEAT_THRESHOLD` mantienen firma y semántica (tests `heat_decay.rs` + inline verdes sin editarse); `MemoryRecord` no gana campos (wire intacto).
  3. **Idempotencia del pass:** correr `run_decay_pass` dos veces con el mismo `now_ms` produce el mismo `DecayReport` y no altera el store (test lo pinea).
  4. **Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción**; sin dependencias nuevas; `src/sdk/**`/`src/wal.rs`/`src/storage/**`/`src/vector/` **no se tocan** (consumo puro).
  5. **Fórmula como política, no calibración:** docs y rustdoc declaran explícitamente que los half-lives son defaults de política (N-09 abierto; sin claim de precisión).
  6. **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  7. **Coordinación MEMG-21 (reservada, espera a MEMG-07):** mi superficie (`retention_factor`/`effective_heat`/`DecayPolicy`/`DecayReport`) es el punto de extensión para su scoring compuesto (recencia); los símbolos quedan públicos y documentados para que MEMG-21 los consuma sin re-tocar el diseño.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test forgetting_curve --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings`.
- **Deuda pendiente:** descarte automático + wiring del pass a un scheduler (FIND — stop L1215). Sin otra deuda.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin clones en hot path (fuera del path de recall; pass explícito pull-based), sin abstracciones especulativas (1 struct de config + 2 funciones puras + 1 report + 1 pass + tests). El cambio **elimina** deuda de "curva de olvido por tipo/edad inexistente" (MEMG-07/DELTA P1) y agrega tests de contrato. Sin deps nuevas. Nota: el parser RFC3339 se duplica funcionalmente con `l1_writer::rfc3339_to_epoch_ms` (privado) — `NOTICED BUT NOT TOUCHING` (dedup futuro si se toca `l1_writer`; hoy evita ripple).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: curva con parámetros declarados y testeados con valores conocidos (retención exacta 0/1/2 half-lives + report con totales exactos) + pass idempotente + nunca-borra/nunca-muta; suites scoped verdes + fmt/clippy verdes |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) + `codebase-memory-mcp_check_index_coverage` (7 paths, `no_recorded_issue` ✅) + grep puntual
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — `VANTA_MEMORY.md` + task file editados

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `test-driven-development` · `systematic-debugging` (pins) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` (SDP phase=BUILD) + rol: `rust-write-tests`, `coordinated-web-search`. `documentation-skill` se carga al editar `docs/**` (step 3). `security-and-hardening` excluida (sin trust boundary nuevo: lectura de store propio, sin input externo nuevo). `performance-optimization` excluida (no hot path — pass explícito; Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo; el pass lee registros del store propio vía superficie existente (quarantine gate SCH-05 heredado de `read_session_records`); sin auth/secrets/deps/FFI. Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — no aplica: fuera del path de recall/pipeline (scan explícito pull-based por sesión); sin loops nuevos en hot paths; `powf` por registro (O(n) del pass, mismo orden que el read existente). Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — RED+GREEN: política + curva + effective heat (lifecycle.rs)

- **Archivos:** `vanta-memory/src/core/record/lifecycle.rs` (tests inline + código), `vanta-memory/src/core/abstractions/types.rs` (derive `Hash`)
- **Acción:** RED — tests unitarios en el `mod tests` existente de `lifecycle.rs`: (1) `retention_factor_zero_age_is_one`; (2) `retention_factor_at_one_half_life_is_half` (Episodic, `2026-01-01T00:00:00.000Z` + 7d → `0.5` exacto); (3) `retention_factor_at_two_half_lives_is_quarter` (`0.25` exacto); (4) `retention_factor_exempt_type_is_one` (Instruction); (5) `retention_factor_clock_skew_is_one` (now < updated); (6) `retention_factor_custom_half_life` (`set_half_life`); (7) `retention_factor_unparseable_timestamps_is_one`; (8) `default_policy_declares_per_type_half_lives` (pinnea los 7 valores declarados); (9) `effective_heat_rounds_known_values` (8→4 en 1hl; 3→1 en 2hl); (10) `effective_heat_exempt_type_keeps_raw_heat`. Falla por compilación (símbolos ausentes → RED correcto). GREEN — `types.rs`: `Hash` en el derive de `MemoryType`; `lifecycle.rs`: `DecayPolicy` (mapa privado + `half_life_ms`/`set_half_life`/`clear_half_life` + `Default` declarado), `retention_factor` (parse RFC3339 con fallback `updated_at`→`created_at`→1.0; `hl.max(1)`; `saturating_sub`), `effective_heat` (`round`); header `:14` actualizado (la curva float ahora existe como capa de política; el shift entero queda intacto).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --lib --build-jobs 2` → RED correcto (compile error por símbolos ausentes) → GREEN (unit tests nuevos + 295 existentes verdes)
- **Evidencia:** ✅ RED verificado: `error[E0433] cannot find type DecayPolicy` + `error[E0425] cannot find function retention_factor` — falla por la razón correcta (símbolos ausentes, no assertion). ✅ GREEN: `366 tests run: 366 passed` (+11 unit tests MEMG-07: retención exacta 0/1/2 half-lives, exención Instruction, clock skew, half-life custom/clear, timestamps ilegibles → 1.0, hl=0 clamped sin NaN, defaults declarados pinneados, `effective_heat` con redondeo conocido 8→4 y 3→1, roundtrip serde de la política). `cargo fmt -p vanta-memory` OK; `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` OK (exit 0).
- **Estado:** ✅ COMPLETED

### Step 2 — RED+GREEN: report + pass + integración (scan + run_decay_pass)

- **Archivos:** `vanta-memory/src/core/record/lifecycle.rs` (`DecayReport` + `scan_decay`), `vanta-memory/src/core/record/l1_reader.rs` (`run_decay_pass`), `vanta-memory/src/core/record/mod.rs` (re-export), `vanta-memory/tests/forgetting_curve.rs` (nuevo)
- **Acción:** RED — integración `tests/forgetting_curve.rs` (fixture propio; ISO hardcodeado + `now_ms` fijo `1_769_817_600_000` = 2026-01-31T00:00:00Z, sin chrono en el crate de test): (1) `decay_pass_reports_known_values_for_seeded_session` — 4 records en `l1/sess-1` (Episodic heat=8 @2026-01-01 → eff 0, decayed+below; Persona heat=4 @2026-01-31 → eff 4, unchanged; Instruction heat=2 @2020 → exempt; WorkTask heat=1 @2026-01-01 → eff 0, decayed+below) → `scanned=4, decayed=2, unchanged=1, exempt=1, below_threshold=2, heat_total=15, heat_effective=6, heat_forgotten()=9`; (2) `decay_pass_is_idempotent_and_never_mutates` — correr dos veces con el mismo `now_ms` → reportes iguales + payloads crudos idénticos (comparación byte a byte vía `db.get`); (3) `decay_pass_never_deletes_records` — los 4 siguen presentes post-pass; (4) `scan_decay_classifies_exempt_separately` (unit-ish sobre slice). Falla por compilación (símbolos ausentes → RED correcto). GREEN — `DecayReport` (campos + `Default` + `heat_forgotten`), `scan_decay` (categorías disjuntas + totales + `below_threshold` con `PRUNE_HEAT_THRESHOLD`), `run_decay_pass` en `l1_reader.rs` (`read_session_records` + `scan_decay`; read-only), re-export en `mod.rs`.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test forgetting_curve --build-jobs 2` → RED correcto → GREEN (4 tests)
- **Evidencia:** ✅ RED verificado: `error[E0432] unresolved import scan_decay` + `run_decay_pass` — falla por la razón correcta (símbolos ausentes). ✅ GREEN: `4 tests run: 4 passed` — (1) valores conocidos exactos del pass sobre sesión seedeada (`scanned=4, decayed=2, unchanged=1, exempt=1, below_threshold=2, heat_total=15, heat_effective=6, heat_forgotten()=9`); (2) **idempotencia** — dos pasadas con el mismo `now_ms` → reportes iguales + payloads crudos byte-idénticos (sin mutación); (3) **nunca borra** — los 3 registros siguen presentes post-pass; (4) clasificación `exempt` separada de `unchanged`/`decayed`. **Anti-regresión:** suite `-p vanta-memory` = `635 tests run: 635 passed, 2 skipped` (0 regresiones; números finales post NIT-4 del review); `cargo fmt -p vanta-memory` OK; `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` OK (exit 0).
- **Estado:** ✅ COMPLETED

### Step 3 — Docs + FIND (descarte automático) + gates

- **Archivos:** `docs/api/VANTA_MEMORY.md` (§Forgetting curve), `docs/dev/Backlog.md` (FIND), `vanta-memory/src/core/record/lifecycle.rs` + `mod.rs` (rustdoc)
- **Acción:** documentar la superficie en `VANTA_MEMORY.md` (firmas reales, fórmula como política con la fuente de la forma + N-09 abierto, tabla de defaults declarados, ejemplo de pass, límite: read-only/no-purga/consumidor pendiente) con `documentation-skill` cargada; registrar la fila FIND (descarte automático + scheduler del pass — sin consumer in-repo; stop L1215) con el formato de `prompts/findings.md`; correr gates docs.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` exit 0 + `pwsh scripts/validate-docs-coverage.ps1` 0 gaps
- **Evidencia:** ✅ `VANTA_MEMORY.md` §Forgetting curve (MEMG-07): tabla API (5 símbolos + contrato), fórmula como política (familia exponencial + fuente + N-09), tabla de defaults declarados por tipo con rationale, ejemplo de pass, garantías pinneadas por test + consumidor in-repo: none → FIND-289. ✅ FIND-289 registrado en `Backlog.md` (esquema 10-col, tras FIND-288; descarte automático + scheduler del pass). ✅ Gates: check-links exit 0 (0 broken markdown; wikilinks 20/20 budget) · check-docs exit 0 (GATING all clear) · gen-index `--write` regeneró `docs/index.md` + `llms.txt` → `--check` exit 0 · validate-docs-coverage: 0 gaps (exit 0).
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-07.md` (§Review + RESULTADO §7)
- **Acción:** `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `pwsh scripts/validate-docs-coverage.ps1` · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → revisar por Rule Group (Critical/High bloquean; Medium → FIND); clasificar tier HARD-02 (paths `vanta-memory/**` + `docs/api/**` → **Adversarial**); fork `vanta-review` con el diff (fresh context); registrar veredicto en §Review; commit **LOCAL** `feat(memory):` con pathspec de archivos propios; `campaign_update_task_state(completed, taskId:"42")` con recitation + payload `review`.
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** (pendiente)
- **Estado:** ⬜ PENDING

## Dependencias

- MEMG-01 ✅, MEMG-02 ✅, MEMG-11 ✅ (`508e211e`), MEMG-12 ✅ (`9d0e371d`), MEMG-13 ✅ (`eb842343`), MEMG-20 ✅ (`c1373528`) — releídos frescos desde HEAD (`1820af61`): `lifecycle.rs`/`l1_reader.rs`/`types.rs` sin cambios pendientes de otras tareas.
- **MEMG-21 (Task 44) RESERVADA y en espera** — consume mi `retention_factor` (recencia) + comparte `lifecycle.rs`; los símbolos quedan públicos y documentados. Sin solape de paths en esta tarea.
- FUT-10 (`backlog-futuro.md:24`) + N-09 (`backlog-notion.md:28`) — fuentes de la política (sin canónico).
- nextTask: Task 44 (MEMG-21) — lo decide el orquestador (MEMG-07 es su dependencia dura).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Adversarial** (HARD-02: el diff incluye `docs/api/**` + `vanta-memory/**`). Sesión reviewer: `ses_ef4078f35ffeknsGryHe2RWk8s`. Veredicto: ✅ **APPROVE** (ronda única; 0 Critical / 0 High / 0 Medium).
- **Enfoque:** contrato L1213 punto por punto (curva por tipo/edad configurable, pass read-only, métrica, sin pérdida/compat, doc↔código) + RBI adversarial (idempotencia, wire intacto, overflow/saturación, O(n), serde del HashMap privado) + OCR Rule Group 1 pasada independiente + estado WIP ajeno.
- **Cómo se probó:** corridas propias del reviewer — `forgetting_curve` 4/4 · suite `-p vanta-memory` 635/635 (2 skipped pre-existentes) · `clippy --all-targets --all-features -D warnings` exit 0 · `fmt --check` exit 0 · gates docs 0 (check-links/check-docs/gen-index) · `git diff --cached` vacío + WIP ajeno intacto (verificado inicio y fin). Spot-check de evidencia del task file (365→366 por el test serde agregado post-evidencia — NIT-4).
- **Hallazgos + disposición:**
  1. [NIT] `VANTA_MEMORY.md` "Guarantees" atribuía los asserts exactos a `forgetting_curve.rs` (viven en los inline de `lifecycle.rs`) → **APLICADO** (doc-only): atribución corregida a "`lifecycle.rs` inline tests + `tests/forgetting_curve.rs`".
  2. [NIT] Rango `R ∈ (0, 1]` impreciso (underflow a `0.0` en edades extremas) → **APLICADO** (doc-only): `R ∈ [0, 1]` en rustdoc + `VANTA_MEMORY.md`.
  3. [NIT] Rustdoc de `unchanged` ("full retention") impreciso para heat=0/redondeo → **APLICADO** (doc-only): "effective heat equals stored heat".
  4. [NIT] Números de evidencia stale (365/634 vs 366/635 final) → **APLICADO** (doc-only): actualizados a los finales verificados.
- **Veredicto:** ✅ **APPROVE** — contrato, DoD Task y gates verdes; 0 Critical/0 Required; NITs doc-only aplicados (código sin cambios post-review); changeset listo para commit local (Step 4).

## Context Save Point

- **Última acción:** Steps 1-3 ✅ (RED→GREEN: 11 unit + 4 integración; suite 635/635; fmt/clippy OK; docs gates 0; FIND-289; OCR sin Critical/High; review P2-01 APPROVE con NITs doc-only aplicados). Step 4: commit local en curso.
- **Próximo paso:** commit **LOCAL** `feat(memory):` con pathspec de archivos propios → commit docs de cierre (RESULTADO + hash) → campaign completed taskId `42`.
- **Estado del worktree:** HEAD `1820af61`; WIP ajeno (`opencode.jsonc`, master plan) NO se stagea.

## Notas

- **Por qué read-side (decisión clave):** mutar `heat` por pass requeriría un campo-ancla nuevo (`last_decay_at`) o aceptar doble-decay al re-correr (no idempotente); el contrato dice "la curva deprioriza, nunca purga" y el stop del plan deja el descarte fuera. Read-side: idempotente por construcción, cero wire change, cero riesgo de pérdida; el futuro gate de descarte consume `effective_heat` (misma función que el report).
- **Fórmula (política declarada, no calibración):** `R = 2^(−age/hl)` — familia exponencial `R = e^(−t/S)` (Wikipedia "Forgetting curve", §Equations; Woźniak et al. 1995), parametrizada por half-life. La misma fuente advierte que la forma simple "was not found to provide a good fit" → N-09 abierto, sin claim de precisión; half-lives = defaults tuneables.
- **Edad = último toque (`updated_at`):** `bump_heat` lo actualiza en cada acceso → el uso resetea la retención (patrón review de Ebbinghaus). La señal no se contamina: el pass no escribe.
- **Pass pull-based:** no existe pass productivo in-repo (`decay_heat` sin callers; WIRE-15 = scheduler futuro); `run_decay_pass` es el punto de entrada por sesión (patrón `TimerScanner::run_once`: "the owner calls run_once").
- **Exención `Instruction`:** instrucciones estrictas (`priority=-1`) se siguen hasta ser contradichas; configurable por tipo (`clear_half_life`).
- **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean. PROHIBIDO tocar `docs/pipeline-state.json`.
- **Disco:** si el linker falla por espacio → `dev-tools/target-cleanup.ps1 -Clean -Yes` (patrón MEMG-02/13/20).
- **NOTICED BUT NOT TOUCHING:** (1) `l1_writer::rfc3339_to_epoch_ms` (privado) duplica funcionalmente el parser nuevo — dedup futuro; (2) `src/eviction.rs` `BayesianDecay` (feature-gated) queda como scoring de eviction del core — no se consume (pre-mortem: eviction ≠ L1); (3) `decay_heat` (shift legacy) queda sin callers de producción — no se deprecia (compat).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 7e84244f (feat local, sin push; + commit docs de cierre)
ARCHIVOS: vanta-memory/src/core/record/lifecycle.rs, vanta-memory/src/core/record/l1_reader.rs, vanta-memory/src/core/record/mod.rs, vanta-memory/src/core/abstractions/types.rs, vanta-memory/tests/forgetting_curve.rs (nuevo), docs/api/VANTA_MEMORY.md, docs/dev/Backlog.md (FIND-289), docs/dev/tasks/MEMG-07.md (nuevo), docs/index.md + llms.txt (generados)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:plan Task 42 sanciona la curva (Gate Result ✅ DO) · D:no disparado (pre-respondido por plan F0 — símbolos dentro de la sanción, precedente MEMG-13/20) · V:no disparado (verde al primer intento; 0 fallas mismo-error) · C:no disparado (FIND-289 registrado; WIP ajeno no stageado)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base) · test-driven-development, systematic-debugging (pins) · source-driven-development, doubt-driven-development, incremental-implementation (SDP v3 BUILD) · rust-write-tests, coordinated-web-search, documentation-skill (rol/cierre)
```
