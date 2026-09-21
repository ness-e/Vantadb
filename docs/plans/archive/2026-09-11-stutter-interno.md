# Plan de Ejecución: Stutter Interno (entity/scene + misceláneos)

> **Campaign ID:** 69be5375-44c1-4169-ad63-84589614b196
> **Inicio:** 2026-09-11
> **Estado:** ⏳ LISTO PARA RUN
> **Fuente:** sesión `ses_f710bf1ceffeYi3mGzzzlwAPxc` (auditoría 45 hallazgos) + re-verificación contra árbol actual + decisiones usuario Gate P
> **Autonomous:** false

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 4 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 0 (decisiones tomadas) · ⬇️ downhill = 4
SDP: `code-review-and-quality, code-simplification, systematic-debugging, test-driven-development, documentation-and-adrs, git-workflow-and-versioning`
Regla: rename DIRECTO (sin usuarios). Familia entity/scene con `#[serde(alias)]` para leer DBs viejas. `Error::*Error` explícitamente FUERA (contrato cross-binding).

## Análisis de dependencias y efectos

- **STU-001** (pool/prefetch/cache/TierPolicy/server-fns): callers solo internos + tests del mismo archivo (`prefetch.rs:83,93` usa `is_prefetch_enabled`; `lib.rs:813-865` usa `cache_invalid`). `lsm` es `pub(crate)` → riesgo cero externo. server/errors es `pub` pero sin usuarios.
- **STU-002** (`AsyncVantaDB→AsyncClient`): convive con `AsyncMemoryClient` sin colisión (nombres distintos); actualizar `__all__`, docstrings y tests que la usen.
- **STU-003** (entity/scene): structs con `Serialize/Deserialize` → el JSON en disco cambia de forma; `#[serde(alias="entity_id")]` / `#[serde(alias="scene_name")]` mantiene lectura de DBs viejas (escritura nueva usa nombre corto). Claves `entity:`/`scene:` NO cambian (son prefijos de partición, no campos). Callers: solo `src/entity/**` + `cli_server_auth_tests.rs` + tests propios.
- **STU-004** (`FOOTER_GROUPS→GROUPS` + verify final + cierre): local a `footer.tsx`; luego gates workspace y archivado.

## Cuestionamiento (doubt-driven) + beneficios/desventajas

- ¿Vale `is_enabled()` si `PrefetchMode` pudiera tener otros flags? Hoy el único booleano es prefetch → sí, y si aparece otro se renombra entonces (YAGNI).
- ¿`TierPolicyConfig::policy→kind` o `heuristic`? `kind` (neutro; el tipo ya dice Policy).
- ¿`Error::*Error` de verdad no? Sí se excluye: `code()`/`is_retriable`/`recovery_hint` + bindings + Display visible = contrato, no estética.
- Beneficio global: elimina el stutter interno real (14 sitios); costo: 1 migración serde menor + serie de commits mecánicos verificados.

## Tasks

### Task 1: STU-001 — Renames baratos Rust (pool/prefetch/cache/TierPolicy/server)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `src/connection_pool.rs:86`, `src/config.rs:131`, `vantadb-wasm/src/lib.rs:408,416,813`, `src/lsm.rs:221`, `src/server/errors.rs:39,122,149`, `src/index/graph/prefetch.rs:83,93`
- **Verificación real:** ✅ CÓDIGO-REAL — callers censados arriba, todos internos/tests.
- **Gate Justificación:** cero riesgo externo, beneficio inmediato.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb --all-targets && cargo clippy -p vantadb --all-targets --all-features -- -D warnings`
- **Task file:** `docs/tasks/STU-001.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🟡 | `kind` no convence como nombre | `heuristic` alternativo, 1 línea | review |
  | 🟢×🟢 | doc-comments con nombre viejo | rg post-rename | 1 hit |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) test olvidado con nombre viejo (compilador lo caza); 2) `mark_cache_invalid→mark_invalid` incluir; 3) Tanner: `query_error_response`/`thread_not_found_response` NO son stutter (califican, no repiten) → no tocar.
  - **Stop conditions:** N/A (mecánico).
  - **Cynefin:** 🟦 obvio.
  - **Top 3 riesgos:** test olvidado.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ 6 renames + callers.
  - **DoD task:** contrato + nextest del módulo.

### Task 2: STU-002 — AsyncVantaDB→AsyncClient

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/vantadb_py/__init__.py:198`, `vantadb-python/tests/test_async_smoke.py`
- **Verificación real:** ✅ CÓDIGO-REAL — `AsyncVantaDB` envuelve `Client`; `AsyncMemoryClient` convive sin colisión.
- **Gate Justificación:** decisión usuario A (Gate P).
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "AsyncVantaDB" vantadb-python/ | wc -l` → `0` + pytest async smoke verde
- **Task file:** `docs/tasks/STU-002.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🟡 | test/script con nombre viejo | rg + migrar (son nuestros) | 1 hit |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) `__all__`/docstring con resto; 2) pickle no aplica (ver AST-008).
  - **Stop conditions:** N/A.
  - **Cynefin:** 🟦 obvio.
  - **Top 3 riesgos:** resto en docs.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ rename + tests.
  - **DoD task:** contrato + stub drift verde.

### Task 3: STU-003 — Familia entity/scene con serde alias

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `src/entity/mod.rs`, `src/entity/scene.rs`, `src/entity/tests.rs`, `src/entity/scene_tests.rs`, `src/entity/checker.rs`, `src/cli_server_auth_tests.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — métodos `entity_*/scene_node_*`, campos con Serialize/Deserialize, callers solo internos/tests.
- **Gate Justificación:** decisión usuario A (Gate P); familia más grande de stutter real.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb --all-targets && cargo nextest run --profile audit -p vantadb -- entity` (o filtro equivalente con evidencia)
- **Task file:** `docs/tasks/STU-003.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** `b3ea6028` — `refactor: STU-003 entity/scene sin stutter + serde alias` (24 archivos, sin push)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | DB vieja ilegible | `#[serde(alias)]` + test roundtrip viejo→nuevo | 1 test rojo |
  | 🟡×🟡 | doc-links `entity_list` rotos | actualizar a `list` | rustdoc warning |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) JSON viejo sin alias; 2) `checker.rs` usa nombres; 3) `EntityPage` doc-link.
  - **Stop conditions:** roundtrip viejo falla → STOP y revisar alias.
  - **Cynefin:** 🟨 complicado (formato en disco).
  - **Top 3 riesgos:** compat lectura; callers ocultos (grep exhaustivo).
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ renames + alias + roundtrip test + callers.
  - **DoD task:** contrato + test roundtrip formato viejo verde.

### Task 4: STU-004 — FOOTER_GROUPS + verify final + cierre

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Ruta:** vanta-docs
- **Archivos clave:** `web/src/components/vanta/footer.tsx:24`
- **Verificación real:** ✅ CÓDIGO-REAL — const local, 1 uso `:143`.
- **Gate Justificación:** único frontend real; cierre con gates workspace.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run lint --prefix web` (o eslint equivalente con evidencia) + `cargo clippy --workspace -- -D warnings` verde
- **Task file:** `docs/tasks/STU-004.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟢×🟢 | ninguno vivo | — | — |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - **Pre-mortem:** 1) lint web pesado (aceptar o scoped).
  - **Stop conditions:** N/A.
  - **Cynefin:** 🟦 obvio.
  - **Top 3 riesgos:** ninguno.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ micro-rename + gates + retrospectiva + archivar.
  - **DoD release:** retrospectiva + archivar + progreso.

=== RECITATION STU-003 ===
Campaign ID: 69be5375-44c1-4169-ad63-84589614b196
Objetivo activo: STU-003 — Familia entity/scene con serde alias
Estado: completed
Última acción: Retry verificó código heredado S1-S4, corrió S5 verify completo + fmt-fix + commit blast radius 24 archivos
Resultado: ✅
Próxima acción: Ninguna en STU-003; STU-004 desbloqueado para archivar
Contrato: check vantadb+proxy ✅, nextest entity 46/46 ✅, gate auth/checker/skills 45/45 ✅, clippy ✅, fmt ✅, rg cero ✅ (H7 inject.rs:115). Invariantes: claves byte-idénticas, alias lee-viejo/escribe-nuevo, checker/ct_eq intactos, firmas ajenas intactas. Deuda: ninguna viva. Commit b3ea6028, sin push.
Próxima tarea si completa: STU-004
=== END RECITATION ===

=== RECITATION 1 ===
Campaign ID: 69be5375-44c1-4169-ad63-84589614b196
Objetivo activo: STU-001/002/004 Wave0 + STU-003
Estado: completed
Última acción: STU-003 RETRY COMPLETO commit b3ea6028; docs a8cba614
Resultado: ✅
Próxima acción: archivar plan
Contrato: todos verdes + commits
Próxima tarea si completa: ninguna
=== END RECITATION ===

=== RECITATION 2 ===
Campaign ID: 69be5375-44c1-4169-ad63-84589614b196
Objetivo activo: STU-002 AsyncClient
Estado: completed
Última acción: COMPLETO commit b2d6e1f4
Resultado: ✅
Próxima acción: archivar
Contrato: rg0 + smoke + drift
Próxima tarea si completa: ninguna
=== END RECITATION ===

=== RECITATION 4 ===
Campaign ID: 69be5375-44c1-4169-ad63-84589614b196
Objetivo activo: STU-004 footer + review
Estado: completed
Última acción: COMPLETO commits 5e7f6fd2 + review approve
Resultado: ✅
Próxima acción: archivar
Contrato: eslint + clippy + review approve
Próxima tarea si completa: ninguna
=== END RECITATION ===

=== RECITATION 3 ===
Campaign ID: 69be5375-44c1-4169-ad63-84589614b196
Objetivo activo: STU-003 entity/scene + alias
Estado: completed
Última acción: RETRY COMPLETO commit b3ea6028
Resultado: ✅
Próxima acción: archivar
Contrato: check + nextest entity 46 + auth/checker 45 + rg0
Próxima tarea si completa: ninguna
=== END RECITATION ===
