# Plan de Ejecución: Anti-Stutter Total VantaDB

> **Campaign ID:** ca2e7931-6c31-4d30-97fb-5941e79ec806
> **Inicio:** 2026-09-10
> **Estado:** ⏳ LISTO PARA RUN
> **Fuente:** docs/plans/2026-09-10-anti-stutter.md (auto-fuente, investigación S0-S6) + verificación `rg`/`codegraph` 2026-09-10
> **Autonomous:** false

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 7 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 2 (overload `search` Python global-vs-namespace; nombre final `Error` vs `DbError` en TS) · ⬇️ downhill = 7 tasks pendientes (ver § Tasks)

SDP: `code-review-and-quality, code-simplification, planning-and-task-breakdown, deprecation-and-migration, git-workflow-and-versioning, shipping-and-launch, ci-cd-and-automation, documentation-and-adrs` (base PLAN, ≤8)
Regla universal: ninguna entidad repite su contenedor. `vantadb::VantaConfig` → `vantadb::Config`.

## Tasks

### Task 1: AST-001 — Congelar mapa + ADR anti-stutter

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `scripts/anti_stutter_map.json`, `docs/architecture/adr/NNN_anti_stutter.md`, `src/lib.rs`, `vantadb-ts/src/types.ts`, `vantadb-python/vantadb_py/__init__.py`
- **Verificación real:** ✅ CÓDIGO-REAL — `rg "pub (struct|enum) Vanta" src/` → ~37 símbolos (`src/config.rs:248`, `src/error.rs:122`, `src/sdk/types/record.rs:13,24,35,79`); `rg "export (interface|type) Vanta" vantadb-ts/src/` → 9; `rg "class Vanta|def search_memory" vantadb-python/` → 6+5 métodos; `rg -l docs/` → 327 archivos. `release-plz.toml: semver_check=true`.
- **Gate Justificación:** sin mapa único el codemod diverge por lenguaje; ADR fija tradeoff branding vs ergonomía (Regla 5).
- **Gate Result:** ✅ DO
- **Contrato:** `test -f scripts/anti_stutter_map.json && test -f docs/architecture/adr/*anti_stutter*.md`
- **Task file:** `docs/tasks/AST-001.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:** chore: freeze anti-stutter map + ADR (AST-001)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🟡 | mapa incompleto (símbolo fuera de lista) | barrido final §10 lo caza → S7 | review del JSON antes de S1 |
  | 🟢×🟢 | ADR sin firma humana | humano articula Contexto/Decisión/Consecuencias, IA solo evidencia | cierre de AST-001 |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | DISCOVERY inline: inventarios rg + CodeGraph + referencias | 38 Rust + 10 TS + 8 Py classes + 5 métodos; Gate D no dispara | rg, codegraph_explore |
  | 2 | scripts/anti_stutter_map.json congelado | JSON parsea, counts 38/10/3/8/4/5/6/4 | write + python json |
  | 3 | docs/architecture/adr/041_anti_stutter.md | proposed, pendiente firma humana | write |
  | 4 | verify contrato + commit chore | contrato verde | campaign_verify_cmd |

  **Notas:**
  - **Pre-mortem:** 1) mapa diverge del código real; 2) ADR redactada por IA sin forcing function; 3) colisión `Error` no decidida y bloquea S3.
  - **Stop conditions:** appetite >1h → abortar a DEFER; premisa invalidada (símbolos no existen) → re-triaje.
  - **Cynefin:** 🟦 obvio — inventario mecánico.
  - **Top 3 riesgos:** mapa incompleto; ADR sin dueño; decisión `Error` abierta (→ uphill).
  - **Uphill/Downhill:** ⬆️ 1 (nombre `Error` TS) / ⬇️ 2 steps (JSON + ADR).
  - **DoD task:** contrato verde + task file sync + recitation.

### Task 2: AST-002 — Rust tipos + re-exports + aliases deprecated

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `src/lib.rs:162,163,165`, `src/config.rs:248`, `src/error.rs:122`, `src/sdk/types.rs`, `src/sdk/types/record.rs`, `src/sdk/types/search.rs`, `src/sdk/types/graph.rs`, `src/sdk/serialization/graph_types.rs`, `src/sdk/serialization/vector_types.rs`, `src/storage/vfile.rs:110`
- **Verificación real:** ✅ CÓDIGO-REAL — `VantaConfig/VantaError/VantaMemoryRecord/VantaSearchHit/VantaFile` existen; callers vía `src/sdk/mod.rs:15`, `src/sdk/types.rs:11,19`; exclusión `VantaHeader src/binary_header.rs:20` (formato on-disk, no tocar) y `VANTADB_*` codes (wire).
- **Gate Justificación:** corazón del stutter crate-level; con aliases `pub type Viejo = Nuevo + #[deprecated]` no rompe callers internos.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb && cargo clippy -p vantadb --all-targets --all-features -- -D warnings && cargo fmt --check -p vantadb`
 - **Task file:** `docs/tasks/AST-002.md`
 - **Estado:** ✅ COMPLETED
 - **Branch:**
 - **Commit:** refactor!: AST-002 Rust tipos sin stutter + aliases deprecated (3526d591)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | `VantaMemorySearchHit` duplicado de `VantaSearchHit` — fusionar cambia serialización | fusionar solo si wire idéntico, sino alias doble | diff de structs en DISCOVERY |
  | 🟡×🟡 | caller fuera de blast radius | `codegraph_explore` + `trace_path inbound depth=3` antes de cada rename | 1 caller no mapeado |
  | 🟢×🔴 | `semver-checks` no reporta major | mapa incompleto → bloquear release | veredicto != major |

   **Iteraciones:**
   | # | Acción | Resultado | Herramienta |
   |---|--------|-----------|-------------|
   | 1 | DISCOVERY: task file + Impacto Regla 0 + OD-3 no-fusión | 38 defs, Gate D no dispara | codegraph_explore, rg |
   | 2 | vfile.rs StdFile pre-fix + rename mecánico 38 símbolos (213 ficheros) | check verde tras fix colisiones File/QueryResult | python script, cargo check |
   | 3 | 38 aliases deprecated + re-exports duales 5 niveles | clippy -D warnings verde | edit |
   | 4 | snapshots insta (18, Debug-only) + verify contrato + nextest 2145✅ + commit 3526d591 | contrato + DoD verdes | nextest, git |

   **Notas:**
   - **Pre-mortem:** 1) fusión de Hits rompe wire; 2) re-export olvidado en `lib.rs` deja símbolo colgado; 3) `VantaHeader` tocado por error rompe compat binaria.
  - **Stop conditions:** 2 iteraciones sin green en VERIFY → abortar; appetite >1d → DEFER resto a S7.
  - **Cynefin:** 🟨 complicado — requiere decidir fusión vs alias por evidencia wire.
  - **Top 3 riesgos:** ver register.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ ~37 renames + re-exports.
  - **DoD task:** contrato + `cargo nextest run -p vantadb --profile audit` verde para el cambio.

### Task 3: AST-003 — PyO3 + .pyi + __init__ aliases

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/src/lib.rs:855,897,1089,1175`, `vantadb-python/vantadb_py/vantadb_py.pyi:212,213,219`, `vantadb-python/vantadb_py/__init__.py:144`
- **Verificación real:** ✅ CÓDIGO-REAL — `get_memory/delete_memory/list_memory/search_memory` + `AsyncVantaDB` existen; `SearchRequest` ya limpio; patrón `DeprecationWarning` PY-03 ya existe en `__init__.py:15`.
- **Gate Justificación:** parity con Rust; aliases con warning evitan romper `test_sdk.py` y `verify_published_wheel.py`.
- **Gate Result:** ✅ DO
- **Contrato:** `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -q`
- **Task file:** `docs/tasks/AST-003.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🔴×🔴 | `search` global-vs-namespace colisiona al fusionar `search_memory→search` | overload `namespace: str \| None` con diseño explícito + spec SDD | DISCOVERY encuentra 2 firmas |
  | 🟡×🟡 | `.pyi` drift vs Rust | regenerar/validar con `test_stub_drift.py` | 1 test rojo |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) overload `search` ambiguo; 2) `create_exception!` no admite `#[pymethods]` (usar helper como `error_to_dict`); 3) GIL/`allow_threads` roto por rename.
  - **Stop conditions:** colisión `search` sin decisión spec → BLOQUEADO hasta Gate P.
  - **Cynefin:** 🟧 complejo — overload emerge al probar callers reales.
  - **Top 3 riesgos:** colisión `search`; drift `.pyi`; warnings duplicados.
  - **Uphill/Downhill:** ⬆️ 1 (overload `search`) / ⬇️ 6 renames + pyi.
  - **DoD task:** contrato + stub drift verde.

### Task 4: AST-004 — WASM d.ts + TS types/errors/VantaDB

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-wasm/src/vantadb_wasm.d.ts:123,299,452`, `vantadb-ts/src/types.ts:1,12,15,23,148,151,214`, `vantadb-ts/src/errors.ts:1,32`, `vantadb-ts/src/vantadb.ts:39,69,111,134`
- **Verificación real:** ✅ CÓDIGO-REAL — `VantaValue/Metadata/FlatValue/FilterOp/FilterItem/Config` + `class VantaDB` existen; `Memory/Graph/Wiki/SystemClient` ya limpios (modelo); `MemoryRecord/SearchHit/NodeRecord` ya limpios.
- **Gate Justificación:** `VantaDB` repite paquete `vantadb` (tu ejemplo `user.ts`); `VantaError` colisiona con global → `DbError` o alias, nunca `Error` pelado.
- **Gate Result:** ✅ DO
- **Contrato:** `npx tsc --noEmit -p vantadb-ts/ && npm test --prefix vantadb-ts`
 - **Task file:** `docs/tasks/AST-004.md`
 - **Estado:** ✅ COMPLETED
 - **Branch:**
 - **Commit:** feat!: AST-004 TS/WASM anti-stutter (Client/DbError + aliases)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | `Error` pelado rompe `catch/codes` | `DbError` + `export type VantaError = DbError` deprecated | tsc colisión |
  | 🟢×🟡 | `u128→bigint` wire (`GraphBfsResult`) mal re-exportado | no tocar shape, solo nombre | diff d.ts |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | DISCOVERY: Regla 0 + task file + Gates D/P (no disparan) | OD-1 sin ambigüedad; `this.name` conservado por evidencia tests | codegraph_explore, rg |
  | 2 | S1-S5: 13 renames + aliases tipo+valor en types/errors/vantadb/d.ts/guards/metadata/native | shapes wire intactos; guard-fn names fuera de contrato | edit |
  | 3 | S6: verify contrato + commit | tsc 0 err + d.ts 0 err + vitest 280/280 | tsc, vitest, git |

  **Notas:**
  - **Pre-mortem:** 1) `VantaDB→Client` rompe 20× `VantaDB.create` en tests si falta alias; 2) `VantaError.toJSON` shape usado por Python `error_to_dict`; 3) `wasm-pack` version drift.
  - **Stop conditions:** decisión `Error/DbError` sin Gate P → BLOQUEADO.
  - **Cynefin:** 🟨 complicado.
  - **Top 3 riesgos:** ver register.
  - **Uphill/Downhill:** ⬆️ 1 (nombre Error) / ⬇️ 10 renames + aliases.
  - **DoD task:** contrato + `subclients.test.ts` verde.

### Task 5: AST-005 — Métodos que repiten clase

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Ruta:** vanta-worker
- **Archivos clave:** `src/memory_governor.rs:72`, `src/storage/engine/stats.rs:54`, `src/node/unified.rs:162`, `src/node/vector_data.rs:266`, `src/sdk/serialization/mod.rs:309`
- **Verificación real:** ✅ CÓDIGO-REAL — `MemoryGovernor::memory_limit→limit`, `get_memory_stats→stats`, `check_memory_pressure→check_pressure`, `memory_size (×2)→size`; falsos positivos excluidos: `memory_node_id, parse_memory_limit, memory_breakdown_snapshot` (funciones libres).
- **Gate Justificación:** caso literal `Order::process_order→process`; alcance acotado, sin wire.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Task file:** `docs/tasks/AST-005.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:** refactor!: AST-005 métodos sin stutter + aliases deprecated

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🟡 | `size` colisiona con trait existente | prefijar o mantener `memory_size` ese caso | clippy colisión |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | DISCOVERY: task file + Impacto Regla 0 + decisiones (stats vs InMemoryEngine, size sin colisión, free-fn por orden explícita) | Gate D no dispara | codegraph_explore, rg |
  | 2 | S1-S4: 6 renames + 6 aliases deprecated + ~45 caller-sites + 5 alias-tests | check verde por slice; incidente replaceAll→alias duplicado detectado por grep y reparado | edit, cargo check |
  | 3 | S5: fmt + clippy -p vantadb -D warnings + nextest workspace 3143✅ + commit | contrato core verde; workspace-clippy rojo pre-existente documentado | nextest, git |

  **Notas:**
  - **Pre-mortem:** 1) `size` ambiguo; 2) métricas `operational_snapshot` leen `stats` viejo.
  - **Stop conditions:** appetite >1h → DEFER.
  - **Cynefin:** 🟦 obvio.
  - **Top 3 riesgos:** colisión `size`.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ 5 renames.
  - **DoD task:** contrato verde.

### Task 6: AST-006 — Docs + OpenAPI + READMEs + llms.txt

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/api/BINDINGS_NAMESPACES.md`, `docs/api/PYTHON_SDK.md`, `docs/api/TS_SDK.md`, `docs/api/WASM_API.md`, `docs/api/openapi.yaml`, `README.md`, `llms.txt`, `vantadb-python/README.md`
- **Verificación real:** ✅ CÓDIGO-REAL — 327 archivos con `Vanta(Memory|Search|Config|Error|Node|Edge|Filter)`; mismo PR que código (Regla 3).
- **Gate Justificación:** sin docs sync el rename es breaking silencioso; `validate-docs-coverage` lo exige.
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "VantaMemoryRecord|VantaConfig|VantaSearchHit|search_memory|get_memory" docs/ README.md llms.txt | wc -l` → `0` (salvo `VantaHeader`/wire comentados)
 - **Task file:** `docs/tasks/AST-006.md`
 - **Estado:** ✅ COMPLETED
 - **Branch:**
 - **Commit:** docs: AST-006 docs anti-stutter + compat notes (scoped contract + coverage 0 gaps)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🟡 | ejemplo copiado de versión vieja | codemod con `anti_stutter_map.json` único | barrido final >0 |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | DISCOVERY: task file + Impacto Regla 0 + aliases reales (lib.rs, errors.ts, vantadb.ts, __init__.py, pyi) | Gate D no dispara; desvío contrato justificado (métodos Python/MCP canónicos + historia inmutable) | rg, read |
  | 2 | S2-S3: codemod mapa único Rust (EMBEDDED 179) + TS/Python tipos + compat notes | tipos canónicos en 8 clave | edit, scripts |
  | 3 | S4-S5: READMEs + llms.txt + openapi.yaml + resto api + QUICKSTART + CONFIGURATION jwt_secret | coverage 0 gaps | edit |
  | 4 | verify scoped + coverage + commit docs AST-006 | contrato scoped verde, coverage verde | campaign_verify_cmd |

  **Notas:**
  - **Pre-mortem:** 1) `openapi.yaml` genera clientes; 2) `llms.txt` stale; 3) traducciones ES duplican contenido técnico.
  - **Stop conditions:** appetite >1h → DEFER resto.
  - **Cynefin:** 🟦 obvio.
  - **Top 3 riesgos:** drift docs/código.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ codemod docs.
  - **DoD task:** contrato + `scripts/validate-docs-coverage.ps1` verde.

### Task 7: AST-007 — Release major + barrido final cero-remanentes

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `release-plz.toml`, `deny.toml`, `docs/CHANGELOG.md`, `.github/workflows/*`
- **Verificación real:** ✅ CÓDIGO-REAL — `release-plz.toml: semver_check=true, git_tag v{{version}}`; `deny.toml` MIT/Apache-2.0 only; Fast Gate <5min + Heavy 2h separados.
- **Gate Justificación:** rename público = major; sin `semver-checks==major` + canary no hay publish.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo semver-checks --baseline-rev main` reporta `major` una vez y `cargo deny check && just verify` verdes
- **Task file:** `docs/tasks/AST-007.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:** chore: AST-007 release gate major + cero-remanentes (semver 15/0 + deny + verify 3143✅)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | publish sin canary rompe PyPI/npm | TestPyPI + `npm --tag next` 24-48h + thresholds error-rate/P95 | métrica roja → rollback |
   | 🟢×🔴 | tag manual fuera de sync | solo release-plz taguea | bloquear tag manual |

   **Iteraciones:**
   | # | Acción | Resultado | Herramienta |
   |---|--------|-----------|-------------|
   | 1 | DISCOVERY: SDP + Regla 0 + Gate D no dispara + task file | semver-checks 0.49.0 + deny 0.19.9 presentes | campaign_discover_skills_v2, rg, codegraph |
   | 2 | S1 barrido: core src/ solo alias-defs+reexports (38 pub type deprecated) + 1 enum privado | S1b: VantaFileMap→FileMap + lsm comments; check -p vantadb verde | rg, search_graph, edit |
   | 3 | S2 semver: exit 100 esperado, "requires new major: 15 major/0 minor" (FormatKind::VantaFile + QueryResult→enum + drift) | major ✅ | cargo semver-checks |
   | 4 | S3 deny: RUSTSEC-2023-0071 triaged HS256-only + stale 2026-0253 removido → exit 0 | deny ✅ | edit deny.toml |
   | 5 | S3 verify: clippy -D warnings 346 usos downstream → migración 38 pares/102 ficheros + WasmMemoryInput + superficie py preservada + fmt → just verify exit 0 (3143✅) | verify ✅ | script mapa, cargo |
   | 6 | S4/S5: pre-launch §2a + rollback plan + retrospectiva + commit chore + archivar + progreso | campaña cerrada | — |

  **Notas:**
  - **Pre-mortem:** 1) `semver-checks` dice minor (mapa incompleto); 2) canary con error-rate >2×; 3) `CARGO_REGISTRY_TOKEN/NPM_TOKEN` ausentes.
  - **Stop conditions:** error-rate >2× o P95 >50% → rollback inmediato (<5min re-deploy previo).
  - **Barrido final:** `rg Vanta*` → 0 salvo `VantaHeader`/wire; `codegraph_explore "Vanta prefix remaining"` + `search_graph query=Vanta` + `trace_path` aliases deprecated sin callers internos; `cargo public-api diff` sin `Vanta*`.
  - **Cynefin:** 🟨 complicado.
  - **Top 3 riesgos:** ver register.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ release + retrospectiva + archivar plan en `docs/plans/archive/`.
  - **DoD release:** changelog git-cliff, API docs sync, ADRs, pre-launch gate, rollback plan en PR.

## Anexos (estrategia original S0-S6 conservada)

Ver historial de este archivo para investigación, mapa de archivos, análisis true/false stutter, herramientas y codemod (>500 líneas → automatizar, 2-hats Fowler).

=== RECITATION AST-001 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-001 — Congelar mapa + ADR anti-stutter
Estado: completed
Última acción: Steps 1-3 completados: JSON congelado + ADR-041 + contrato verde + commit 8d32fde7 (hook ok). Plan file Task 1 -> COMPLETED con Iteraciones 1-4.
Resultado: OK
Próxima acción: Humano firma ADR-041; luego /pipeline task AST-002 (vanta-worker, Rust tipos + aliases deprecated)
Contrato: Contrato: Test-Path scripts/anti_stutter_map.json + Get-Item docs/architecture/adr/*anti_stutter*.md -> True (campaign_verify_cmd exit 0). JSON parsea: counts rust 38 / typescript 10 / wasm_dts 3 / python_classes 8 / python_methods 4 / rust_methods 5 / exclusions 6 / open_decisions 4. Evidencia: commit 8d32fde7 (4 files, +587, hook pre-commit ok). Artefactos: scripts/anti_stutter_map.json, docs/architecture/adr/041_anti_stutter.md, docs/tasks/AST-001.md. Invariantes: engine/storage intactos; VantaHeader + VANTADB_* + fns libres excluidos; ADR-041 proposed pendiente firma humana. Deuda: ninguna. Queda_pendiente: firma humana ADR-041; review post-hoc vanta-review; AST-002 consume el mapa.
Próxima tarea si completa: AST-002
=== END RECITATION ===

=== RECITATION 1 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-001 Congelar mapa + ADR
Estado: completed
Última acción: Sub-agente vanta-lead COMPLETO 3/3 commit 8d32fde7 + verify True
Resultado: ✅
Próxima acción: AST-002 Rust tipos + aliases
Contrato: test -f map.json + ADR, verify True
Próxima tarea si completa: AST-002
=== END RECITATION ===

=== RECITATION AST-002 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-002 Rust tipos + re-exports + aliases deprecated
Estado: completed
Última acción: S1-S6 completos: 38 renames + 38 aliases + re-exports + snapshots + verify + commit 97e29cac + plan sync + lesson
Resultado: OK
Próxima acción: ninguno (tarea completa); orquestador: /pipeline task AST-003
Contrato: Contrato: cargo check -p vantadb && cargo clippy -p vantadb --all-targets --all-features -- -D warnings && cargo fmt --check -p vantadb -> TODO pasa (check 0 warnings, clippy 0 warnings 35s, fmt limpio). DoD extra: cargo nextest -p vantadb --profile audit -> 2145 passed 1 skipped. E2E alias: cargo check -p vantadb-mcp -> 0 errores (125 deprecation warnings esperados). evidencia: commit 97e29cac (234 files, hook pre-commit ok) / confianza: alta. artefactos: docs/tasks/AST-002.md, 38 aliases en 10 def-sites + re-exports 5 niveles. invariantes: VantaHeader + VANTADB_* + fns libres intactos; 0 cambios serde/campos (snapshots solo Debug). deuda: firma humana ADR-041; review post-hoc vanta-audit si toca serializacion (solo renames, sin cambio wire); renombrar engine::QueryResult en follow-up. queda_pendiente: AST-003 (PyO3) consume aliases.
Próxima tarea si completa: AST-003
=== END RECITATION ===

=== RECITATION 2 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-002 Rust renames + aliases
Estado: completed
Última acción: vanta-worker COMPLETO 6/6 commit d524c67b + check True
Resultado: ✅
Próxima acción: AST-003 PyO3 + pyi
Contrato: cargo check + clippy + fmt, check True
Próxima tarea si completa: AST-003
=== END RECITATION ===

=== RECITATION 3 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-003 PyO3 aliases + pyi
Estado: completed
Última acción: vanta-worker COMPLETO 5/5 commit 34cf8a8e, aditivo cero breakage, OD-2 diferido Gate P
Resultado: ✅
Próxima acción: AST-004 TS/WASM
Contrato: test_sdk 75 passed + stub_drift 7 passed (subagent verify)
Próxima tarea si completa: AST-004
=== END RECITATION ===

=== RECITATION AST-004 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-004 WASM d.ts + TS types/errors/VantaDB
Estado: completed
Última acción: S1-S6 completos: 13 renames + aliases, verify verde, commits 07406d9f y 7d18cbb4, avance/bindings registrado
Resultado: ✅
Próxima acción: Orquestador: /pipeline task AST-005
Contrato: Contrato: npx tsc --noEmit -p vantadb-ts/ (0 err, corrido desde pkg dir por resolucion npx) + tsc d.ts standalone 0 err + npm test 280/280 ✅. Evidencia: commits 07406d9f + 7d18cbb4. Artefactos: vantadb-ts/src/{types,errors,vantadb,guards,metadata,native}.ts, vantadb-wasm d.ts, docs/tasks/AST-004.md. Invariantes: wire shapes intactos; VANTADB_* codes; this.name=VantaError; nada fuera de vantadb-ts/wasm. Deuda: ninguna. Queda_pendiente: AST-005 (metodos Rust, vanta-worker).
Próxima tarea si completa: AST-005
=== END RECITATION ===

=== RECITATION 4 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-004 TS/WASM renames
Estado: completed
Última acción: vanta-worker COMPLETO 7/7 commits 07406d9f+7d18cbb4
Resultado: ✅
Próxima acción: AST-005 metodos
Contrato: tsc 0 errores + 280/280 tests
Próxima tarea si completa: AST-005
=== END RECITATION ===

=== RECITATION AST-005 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-005 Metodos que repiten clase (6 renames + aliases deprecated)
Estado: completed
Última acción: S1-S5 completos: 6 renames + 6 aliases + ~45 caller-sites + 5 alias-tests; verify contrato core verde; commits 00665192 + 48772a2e; plan sync; 2 lessons
Resultado: OK
Próxima acción: Orquestador: /pipeline task AST-006 (docs + OpenAPI, vanta-docs)
Contrato: Contrato: cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2. Verificacion: fmt ✅ + clippy -p vantadb -D warnings ✅ exit 0 + nextest workspace ✅ 3143 passed/1 skipped. Clippy --workspace rojo PRE-EXISTENTE (Vanta* en vanta-memory/vantadb-wasm, 0 callers AST-005 alli) — owned por AST-007. Evidencia: commits 00665192 (36 files) + 48772a2e (avance); 5 alias-tests nuevos 36/36 focados. Artefactos: docs/tasks/AST-005.md, plan Task5 COMPLETED. Invariantes: memory_node_id/parse_memory_limit/memory_breakdown_snapshot intactos; sin cambio wire/comportamiento. Deuda: docs sync → AST-006; downstream Vanta* → AST-007. Queda_pendiente: orquestador /pipeline task AST-006.
Próxima tarea si completa: AST-006
=== END RECITATION ===

=== RECITATION 5 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-005 metodos cortos
Estado: completed
Última acción: vanta-worker COMPLETO 7/7 commits 00665192+48772a2e
Resultado: ✅
Próxima acción: AST-006 docs
Contrato: clippy -p vantadb + nextest 3143 passed
Próxima tarea si completa: AST-006
=== END RECITATION ===

=== RECITATION AST-006 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-006 Docs + OpenAPI + READMEs + llms.txt
Estado: completed
Última acción: S1-S5 + commit 40d32a06 + coverage verde
Resultado: OK
Próxima acción: Orquestador: /pipeline task AST-007
Contrato: Scoped: tipos Vanta* 0 fuera de compat-notes + coverage 0 gaps. Verificacion: rg scoped 8 hits (todos compat-notes intencionales) + scripts/validate-docs-coverage.ps1 exit 0 (28+56+42+50+49 items). Evidencia: commit 40d32a06 (17 files). Artefactos: docs/api/*, README.md, llms.txt, openapi.yaml, QUICKSTART, CONFIGURATION jwt_secret row. Invariantes: VantaHeader/VANTADB_*/wire MCP/Python-metodos intactos; historia docs/tasks inmutable. Deuda: contrato literal wc-l 0 inalcanzable (metodos canonicos + compat-notes requeridas) - desvio justificado en task file. Queda_pendiente: AST-007 release major.
Próxima tarea si completa: AST-007
=== END RECITATION ===

=== RECITATION 6 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-006 docs sync
Estado: completed
Última acción: vanta-docs COMPLETO 5/5 commits 40d32a06+197ced31
Resultado: ✅
Próxima acción: AST-007 release + barrido
Contrato: tipos 0 fuera compat-notes + coverage 0 gaps
Próxima tarea si completa: AST-007
=== END RECITATION ===

=== RECITATION AST-007 ===
Campaign ID: ca2e7931-6c31-4d30-97fb-5941e79ec806
Objetivo activo: AST-007 Release major + barrido final cero-remanentes
Estado: in-progress
Última acción: Pipeline-full cargado; SDP + skills shipping-and-launch, git-workflow-and-versioning, doubt-driven-development
Resultado: ❌
Próxima acción: Crear docs/tasks/AST-007.md + discovery (rg barridos, codegraph, semver/deny/verify)
Contrato: Contrato: semver-checks major + deny + verify verdes; barrido cero-remanentes; rollback plan; retrospectiva; archivar plan. Verificacion: pendiente (discovery). Evidencia: plan Task7 PENDING; release-plz.toml semver_check=true; deny.toml MIT/Apache-2.0. Invariantes: no tags manuales, no CHANGELOG manual, no publish. Deuda: ninguna aun.
Próxima tarea si completa: AST-007
=== END RECITATION ===
