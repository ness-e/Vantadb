# Plan de Ejecución: Anti-Stutter Cierre Directo (sin usuarios, sin aliases)

> **Campaign ID:** 11eb06a6-bcc5-4894-ad55-697637426a63
> **Inicio:** 2026-09-11
> **Estado:** ⏳ LISTO PARA RUN
> **Fuente:** docs/plans/archive/2026-09-10-anti-stutter.md + auditoría 7 remanentes + decisión Gate P OD-2 = opción B
> **Autonomous:** false

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 4 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 0 · ⬇️ downhill = 4 tasks
SDP: `code-review-and-quality, code-simplification, deprecation-and-migration, git-workflow-and-versioning, shipping-and-launch, ci-cd-and-automation, documentation-and-adrs, systematic-debugging`
Regla: rename DIRECTO, se eliminan aliases deprecated (no hay usuarios que migrar). OD-2 resuelto: B (`search_vector` ANN puro + `search` namespaced, paridad TS/WASM).

## Tasks

### Task 1: AST-008 — Python directo: pyclass + VantaPy* + search namespaced

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/src/lib.rs:86`, `vantadb-python/src/types.rs:48,175,287`, `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/vantadb_py/vantadb_py.pyi`
- **Verificación real:** ✅ CÓDIGO-REAL — `pub struct VantaDB` pyclass + `VantaPyMemoryRecord/ListResult/SearchHit` existen; AST-003 dejó `search_vector` entregado y aliases aditivos; decisión B: `search_memory→search` (namespaced), `search_vector` queda ANN puro, viejo `search` nodo-plano se elimina o redirige a `search_vector`.
- **Gate Justificación:** OD-4 + OD-2 cerrados por decisión usuario (sin usuarios, directo, opción B).
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb_py && cargo fmt --check -p vantadb_py`
- **Task file:** `docs/tasks/AST-008.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:** (ver `git log --oneline --grep=AST-008`)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | pickling/`m.add`/repr rompen al renombrar pyclass | reconstruir + pytest completo incl. subclients/async | 1 test rojo |
  | 🟡×🟡 | `.pyi` drift | `test_stub_drift.py` verde | 1 test rojo |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | S1–S3 Rust: struct VantaDB→Client, search_memory→search, search→search_vector, types name=Record/ListResult/SearchHit | check+fmt+clippy verdes | cargo |
  | 2 | S4 stubs+wrapper: class Client, sin aliases (puente Vector), Async search/search_vector | .pyi espejo + __all__ canónicos | edit |
  | 3 | S5 tests (11 archivos) + maturin develop + pytest 135 passed | 29 drift/subclients, 75 sdk, 26 misc, 5 load | pytest |
  | 4 | S6 commit (sin push); Gate C: integrations/examples/docs-API con nombres viejos → deuda lead | — | git |

  **Notas:**
  - **Pre-mortem:** 1) `#[pyclass]` name vs struct name; 2) colisión `search` nodo-plano vs namespaced; 3) GIL/rebuild maturin.
  - **Stop conditions:** sin usuarios no hay BLOQUEO por compat; appetite >1d → dividir.
  - **Cynefin:** 🟨 complicado.
  - **Top 3 riesgos:** pyclass rename; colisión search; pyi drift.
  - **Uphill/Downhill:** ⬆️ 0 (decisiones tomadas) / ⬇️ renames + rebuild + pytest.
  - **DoD task:** contrato + `test_sdk.py` + `test_stub_drift.py` verdes.

### Task 2: AST-009 — WASM struct + validadores TS + engine QueryResult + anexos

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-wasm/src/lib.rs:370`, `vantadb-ts/src/guards.ts`, `vantadb-ts/src/metadata.ts`, `src/engine.rs`, `vanta-memory/`, `vantadb-wasm/`
- **Verificación real:** ✅ CÓDIGO-REAL — `pub struct VantaDB` wasm + `isValidVantaValue/isVantaMetadata` + `engine::QueryResult` + `Vanta*` en anexos existen (censados en AST-005/007).
- **Gate Justificación:** remanentes 3, 4, 5, 6 de la auditoría; archivos disjuntos de AST-008 → paralelizable.
- **Gate Result:** ✅ DO
- **Contrato:** `npx tsc --noEmit -p vantadb-ts/ && cargo check -p vantadb --all-targets`
 - **Task file:** `docs/tasks/AST-009.md`
 - **Estado:** ✅ COMPLETO
 - **Branch:** develop
 - **Commit:** a786d5f

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🟡 | rebuild wasm-pack pesado | solo si toca `lib.rs` runtime; si no, d.ts + lib basta | build >10min |
   | 🟢×🟡 | `size`/nombres colisionan en anexos | prefijar con justificación | clippy colisión |

   **Iteraciones:**
   | # | Acción | Resultado | Herramienta |
   |---|--------|-----------|-------------|
   | 1 | S1 guards.ts: isValidVantaValue→isValidValue, isVantaMetadata→isMetadata (+VALID_VALUE_TYPES, re-export, 2 test files) | tsc ✅ | edit |
   | 2 | S2 tests VantaDB.*→Client.* (7 files, nuestros) | tsc ✅ | edit |
   | 3 | S3 lib.rs struct→Client + wasm_tests.rs (+1 fix import olvidado) | cargo check wasm ✅ | edit |
   | 4 | S4 wasm-pack build --release --target bundler (2.9min) | pkg exporta Client ✅ | bash |
   | 5 | S5 vantadb.ts WasmClient + docs (native.ts, TS_SDK.md, d.ts doc-words) | tsc ✅ | edit |
   | 6 | S6 contrato + 280 tests + commit a786d5f + lessons | npm test 280/280 ✅ | bash |

  **Notas:**
  - **Pre-mortem:** 1) `.wasm` binario cambia → e2e; 2) anexos con dueños distintos (avisar en commit); 3) ejemplos `VantaDB.create` en tests TS ya van por alias (revisar tras quitar aliases en AST-010).
  - **Stop conditions:** appetite >1d → dividir por anexo.
  - **Cynefin:** 🟨 complicado.
  - **Top 3 riesgos:** rebuild wasm; anexos; ejemplos TS.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ renames + checks.
  - **DoD task:** contrato + `npm test --prefix vantadb-ts` verde.

### Task 3: AST-010 — Quitar aliases deprecated + compat-notes

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `src/lib.rs`, `src/sdk/**`, `src/storage/vfile.rs`, `src/memory_governor.rs`, `vantadb-ts/src/**`, `vantadb-python/**`, `docs/api/**`, `README.md`, `llms.txt`
- **Verificación real:** ✅ CÓDIGO-REAL — 38 `pub type Vanta*` + aliases TS/Python + 6 compat-notes documentados en AST-007 (barrido: 0 usos internos).
- **Gate Justificación:** sin usuarios, los shims se eliminan en vez de esperar 2 semanas; depende de AST-008/009 (nombres finales).
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "Vanta[A-Z]\w+" src/ vantadb-ts/src/ vantadb-python/vantadb_py/__init__.py | grep -v "VantaHeader\|VANTADB_" | wc -l` → `0`
- **Task file:** `docs/tasks/AST-010.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | queda 1 caller interno usando alias → no compila | `cargo check` lo caza antes del commit | 1 error |
  | 🟢×🟡 | docs con ejemplo viejo | barrido rg del contrato | conteo >0 |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) alias usado en test olvidado; 2) compat-note huérfana; 3) historia docs/tasks inmutable (no tocar).
  - **Stop conditions:** conteo >0 tras 2 pasadas → dividir por lenguaje.
  - **Cynefin:** 🟦 obvio (mecánico).
  - **Top 3 riesgos:** caller olvidado.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ borrado + barrido.
  - **DoD task:** contrato 0 + verify full.

### Task 4: AST-011 — Verify final + cierre

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `release-plz.toml`, `deny.toml`, `scripts/anti_stutter_map.json`
- **Verificación real:** ✅ CÓDIGO-REAL — gates AST-007 verdes pre-cierre; este task re-verifica tras borrado de aliases.
- **Gate Justificación:** semver + deny + nextest + tsc + pytest + coverage deben seguir verdes sin shims.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo deny check && cargo fmt --check -p vantadb && cargo check -p vantadb`
- **Task file:** `docs/tasks/AST-011.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🔴 | semver ya no reporta major (todo limpio) | esperado: verde estable post-cierre | — |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) nextest largo (aceptar); 2) tsc en pkg dir (nota AST-004).
  - **Stop conditions:** N/A (solo lectura + comandos).
  - **Cynefin:** 🟦 obvio.
  - **Top 3 riesgos:** ninguno vivo.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ gates + retrospectiva + archivar plan.
  - **DoD release:** retrospectiva + archivar en `docs/plans/archive/` + progreso.

=== RECITATION AST-009 ===
Campaign ID: 11eb06a6-bcc5-4894-ad55-697637426a63
Objetivo activo: AST-009 WASM struct + validadores TS + engine QueryResult + anexos
Estado: completed
Última acción: S1-S6 completos: guards rename, tests Client, wasm Client + rebuild 2.9min, TS consume, verify full, commit a786d5f, lessons, plan actualizado
Resultado: OK
Próxima acción: ninguno (AST-009 cerrada)
Contrato: verificacion: tsc exit 0 + cargo check -p vantadb --all-targets OK (25s) + npm test 280/280 + clippy/fmt wasm OK + pre-commit hook verde | evidencia: commit a786d5f (15 files) | artefactos: docs/tasks/AST-009.md, pkg/ rebuilt (gitignored) | invariantes: python/sdk/aliases-vivos/vantadb-node intactos para AST-008/AST-010 | deuda: bench+5 examples+desktop usan VantaDB.* (duenos distintos, avisado en commit) | queda_pendiente: AST-010 (quitar aliases)
Próxima tarea si completa: AST-010
=== END RECITATION ===

=== RECITATION AST-008 ===
Campaign ID: 11eb06a6-bcc5-4894-ad55-697637426a63
Objetivo activo: AST-008 Python directo: pyclass Client + Record/SearchHit/ListResult + search namespaced
Estado: in-progress
Última acción: Discovery completo: reglas, mapa, ADR-041, lib.rs/types.rs/vector.rs/__init__/.pyi/tests leidos; task file creado con Spec + Regla 0 + 7 steps
Resultado: ✅
Próxima acción: S1: rename struct VantaDB->Client en lib.rs + cargo check
Contrato: verificacion: cargo check -p vantadb_py && cargo fmt --check -p vantadb_py (pendiente) + test_sdk/stub_drift/subclients/async verdes; evidencia: task file docs/tasks/AST-008.md creado con Spec D1-D7 + Regla 0 | claim: discovery completo sin editar codigo | evidencia: codegraph_explore + coverage no_recorded_issue + README naming note como canonico | confianza: alta; artefactos: docs/tasks/AST-008.md; invariantes: no tocar src/ core, TS, WASM, vector.rs, convert.rs, integrations, docs; deuda: integrations/examples/docs-API quedan con nombres viejos (Gate C al lead)
Próxima tarea si completa: AST-010
=== END RECITATION ===
