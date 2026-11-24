---
title: "FIND-485: `INSERT` no puede persistir sin vector en silencio"
kind: task
description: verde con dos tests nuevos
---

# FIND-485: `INSERT` no puede persistir sin vector en silencio

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (Task 3)
- **Fuente:** plan file Task 3 (fila `FIND-485` del Backlog, release v0.10.0)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Baja (severidad baja, impacto alto: pérdida de dato silenciosa en escritura)
- **Tipo:** Rust (core engine)
- **Turns estimados:** 6
- **Creado:** 2026-11-24T02:30
- **last-synced:** 2026-11-24T05:10
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Executor::execute_statement` (`src/executor.rs:337`) → `execute_insert` (`:438`) → `auto_embed_insert` (`:471`). 10 callers de `Executor` fuera del crate (`src/sdk/api/graph.rs`, `src/physical_plan/scan.rs`, `vantadb-mcp/src/handlers/tools.rs`). Ningún caller de `auto_embed_insert` fuera de `executor.rs` (privado). |
| Callees | `crate::llm::get_embedding_provider` (`src/llm.rs:164/181`), `EmbeddingProvider::embed` (`src/llm.rs:38`), `UnifiedNode::set_field` (`src/node/unified.rs:140`), `StorageEngine::insert` (`src/storage/engine/insert.rs:156`) |
| Implicaciones | **No cambia ninguna firma** → el handler MCP y los 10 callers de `Executor` no se tocan. `relational` es un `BTreeMap<String, FieldValue>` serializado: una clave nueva aparece como campo y las DB viejas la leen ausente (sin migración, sin bump de schema). El INSERT con `VECTOR` explícito no se toca (`auto_embed_insert` solo actúa si `vector.is_none()`). Comportamiento público observable: la salida de `vanta-cli query --json` gana un campo `__vanta_embed_status` en los nodos degradados. Performance: 1 insert en un `BTreeMap` por INSERT degradado (fuera del hot path feliz). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/llm.rs:95-200` (selección de provider), `src/config.rs:1470-1529` + `:727-742` (`LlmCfg`/env), `src/node/field.rs` (completo, vía codegraph), `src/node/unified.rs:15-144` (vía codegraph), `src/query.rs:34-46` (`InsertStatement`), `src/sdk/serialization/mod.rs:1-63` (constantes `FIELD_*`) y `:480-534` (`memory_record_from_node` + `RESERVED_FIELDS`), `src/cli_handlers/data.rs:268-390` (`cmd_query` + salida `--json`), `src/cli.rs:128-142` (`Commands::Query`), `src/bin/vanta-cli.rs:68-127` (dispatch), `docs/api/IQL.md:330-404`, `docs/api/EMBEDDINGS.md:150-200`, `tests/embedding_fallback_visibility.rs` (completo, patrón de subproceso con env aislado)
- **Archivos referenciados hacia dentro:** `auto_embed_insert` ← `execute_insert` (1 caller); `execute_insert` ← `execute_statement` (dispatch de `Statement::Insert`, 1 caller); `RESERVED_FIELDS` ← `memory_record_from_node:558` (solo mismo archivo)
- **Archivos que referencian a los editados:** `vantadb-mcp/src/handlers/tools.rs` (usa `Executor` por API pública, **no** `auto_embed_insert` — no se rompe); `tests/` (integration tests que ejecutan IQL INSERT en `cli_tests.rs`, `edge_cases.rs`, `time_travel.rs` — ninguno assertea la lista exhaustiva de campos relacionales)
- **Veredicto impacto:** **bajo**. Cambio aditivo (una clave relacional nueva en el path degradado), sin firma nueva, sin error nuevo, sin migración. Riesgo residual: que algún test compare el mapa `relational` completo de un nodo insertado por IQL — verificado: no existe tal aserción.

## Contrato

```
cargo nextest run --profile audit -p vantadb --features remote-inference -E 'test(auto_embed)' --build-jobs 2
```
verde (9/9) con tres tests que prueban la marca de punta a punta:
1. `auto_embed_insert_marks_node_when_provider_fails` (unit, `src/executor.rs`): INSERT con `text` + provider forzado a fallar → nodo persistido **sin** `HAS_VECTOR` **y** con `__vanta_embed_status = "failed"`, leído tras round-trip por storage (backend de disco).
2. `auto_embed_message_marks_failed_when_provider_fails` + `auto_embed_message_marks_synthetic_vector`: los dos brazos de `INSERT MESSAGE` (fallo → sin vector + `failed`; sintético → vector conservado + `synthetic`).
3. `auto_embed_insert_mark_visible_via_cli` (`tests/auto_embed_insert_mark.rs`): `vanta-cli query` INSERT + `FROM … --json` con provider forzado a fallar → el campo aparece en la salida.

**Legs obligatorios (cierre, review #4/#1):**
```
cargo check -p vantadb --no-default-features --features fjall                      # el brazo `unavailable` compila
cargo nextest run -p vantadb -E 'test(auto_embed)' --build-jobs 2                   # ejeuta el brazo `unavailable` (2/2)
cargo nextest run -p vantadb --features remote-inference,embed-local -E 'test(auto_embed_message_marks_synthetic)'  # brazo sintético (1/1)
cargo fmt --all -- --check
cargo clippy -p vantadb --all-targets -- -D warnings
cargo clippy -p vantadb --all-targets --features remote-inference -- -D warnings
cargo clippy -p vantadb --all-targets --features remote-inference,embed-local -- -D warnings
```
Nota: el contrato solo no alcanza — `--features remote-inference` **no compila** el brazo `not(remote-inference)` (review #1 encontró ahí un error de compilación que el contrato no veía).

## Spec (SDD)

> `pub(crate) mod serialization` (`src/sdk/mod.rs:12`) → la constante nueva NO es símbolo público: no hay señal de feature-add (Phase 1b). Igual se documenta la decisión, porque cambia comportamiento observable.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Fallo de provider en INSERT con `text` | A) propagar `Err` (rompe hosts que escriben sin provider; peor disponibilidad que el dato sin vector) / B) marcar el registro y no romper el INSERT | B | ✅ **decidido por el owner** (Gate P del plan, Task 3) |
| 2 | Mecanismo de marca | A) `NodeFlags` bit nuevo (cambia la semántica del bitset y `flags.rs:160 test_node_flags_all_constants`; un bit no guarda el motivo) / B) `FieldValue` relacional interno `__vanta_*` (precedente: 18 constantes en `src/sdk/serialization/mod.rs:16-51`; consultable en `query --json` sin código nuevo) / C) `ext_metadata` (round-trip sí, pero no lo expone ninguna superficie de lectura) | B | ✅ decidido-por-evidencia (`src/sdk/serialization/mod.rs:16-51`, `src/cli_handlers/data.rs:317-329`) |
| 3 | Granularidad del valor | A) un campo `status` con `failed`/`synthetic`/`unavailable` / B) status + detalle del error (dos campos, datos variables persistidos) | A — el texto del error sigue en el `warn!`; el registro dice *qué* pasó, no *por qué* | ✅ decidido-por-evidencia (Regla 6: no introducir superficie nueva sin pago) |
| 4 | `auto_embed_message` (`:661`) | A) mismo tratamiento (mismo silencio, mismo dueño) / B) dejarlo fuera de scope | A — 2 líneas, evita dejar la mitad del camino de escritura mintiendo | ✅ decidido-por-evidencia (misma clase de fallo, `src/executor.rs:659-676`) |
| 5 | Docs | A) actualizar `docs/api/IQL.md` (INSERT no documenta el auto-embedding hoy) + fila en `docs/api/EMBEDDINGS.md` / B) solo el código | A (Regla 3: comportamiento observable) | ✅ decidido |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. El INSERT **nunca falla** por culpa del provider de embeddings (disponibilidad > vector, Gate P del owner).
  2. `INSERT` con `VECTOR` explícito es byte-exacto, no consulta ningún provider y **no** lleva marca.
  3. En el path `INSERT`, `HAS_VECTOR` implica un vector real (el sintético se descarta, FIND-681/FIND-703). En `INSERT MESSAGE` el sintético **se conserva** y se marca `synthetic` — divergencia documentada en `docs/api/IQL.md`, pendiente de unificar (fila `FIND-*` propuesta en el RESULTADO).
  4. `__vanta_embed_status` solo aparece en nodos degradados; su ausencia significa "sin auto-embed" (vector explícito, sin `text`, `text` vacío/blank, o éxito).
  5. Ningún `VantaError` nuevo (los bindings mapean solo variantes conocidas, `vantadb-python/src/convert.rs:818`).
- **Comandos de verificación:** ver `## Contrato` (9/9 + los 3 legs de cierre)
- **Deuda pendiente:** unificar la política de vector sintético entre `INSERT` y `INSERT MESSAGE` (hoy: descartar vs conservar+marcar). No hay migración ni re-index necesarios.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** **negativo** — el cambio elimina un `warn!` que era la única señal de una pérdida de dato; no introduce abstracciones, ni símbolos públicos, ni dependencias.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verde + capa determinista (fmt, clippy, nextest scoped) + tests nuevos pasan |
| **Commit** | Commit atómico local, `fix(core): mark INSERTs that fall back to no vector (FIND-485)`, `git add` solo de las rutas de esta tarea |
| **Release** | No aplica (sin bump de versión, sin publish desde esta tarea) |

## Herramientas necesarias
- `codegraph_codegraph_explore` (blast radius antes de editar)
- terminal: `cargo nextest` / `cargo clippy` / `cargo fmt` / `vanta-cli`
- `pwsh dev-tools/ocr-review.ps1` (gate de cierre)

**Skills cargadas (SDP):** `campaign-executor` + `progreso` + `ponytail` (base SDP v3) · `source-driven-development` (verificar `tl::embedding_provider`/env contra el código real, no contra memoria) · `doubt-driven-development` (gate adversarial de cierre, tier adversarial por `src/sdk/**`) · `incremental-implementation` (slices: unit test → mark → CLI test → docs) · `test-driven-development` (Prove-It: RED antes del fix) · `context-engineering` (jerarquía Rules→Spec→Source→Error aplicada en este task file) · `documentation-skill` (edición de `docs/api/*.md`)

## Investigation Notes

- **Causa raíz:** degradación elegida a propósito (comentario `src/executor.rs:469` "graceful degradation on failure") **sin superficie de diagnóstico**: el único rastro es `tracing::warn!` (`:495-500`, `:507`), que no es consultable desde el dato ni desde el SDK.
- **Precedentes internos:** `FIND-413` (el error de embed en *lectura* se traga → se convirtió en error), `FIND-681` (fallback sintético silencioso → `is_synthetic()` + `warn!`), `FIND-703` (una sola fuente de verdad para el provider). Este fix completa la serie en el camino de **escritura**: no cambia la degradación, la hace observable.
- **Provider determinístico para el test (sin red):** `VANTADB_EMBEDDING_PROVIDER=bogus-find485` + `VANTADB_LOCAL_MODEL=<tmp>/nope` + `ORT_DYLIB_PATH=<tmp>/nope` + `VANTADB_STRICT_EMBEDDINGS=1` → `try_embedding_provider()` falla en las 4 combinaciones de features (`src/llm.rs:100/123/136`) y el fallback no permite dummy → `embed()` siempre `Err`. Sin red, sin depender de que Ollama esté (o no) corriendo.
- **Superficie de lectura:** `cmd_query --json` emite **todos** los campos relacionales (`src/cli_handlers/data.rs:317-329`), así que la marca es visible con `vanta-cli query "FROM ... " --json` sin tocar la CLI. La tabla no-json muestra los primeros 3 campos (preview) — la marca puede no entrar en el preview: se documenta `--json` como la superficie canónica.
- **Web research:** no requerida (problema interno, sin API externa involucrada).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — contrato decidido por el owner (Gate P); mecanismo elegido por evidencia |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** `VANTADB_EMBEDDING_PROVIDER=bogus-find485 vanta-cli --db <tmp> query 'INSERT NODE#1 TYPE note { text: "hola" }'` → exit 0, "Node 1 inserted."; `vanta-cli --db <tmp> query 'FROM note' --json` → `{"id":"1","fields":{"type":"note","text":"hola"}}`: **sin marca alguna**, y el nodo no tiene vector. El usuario cree que guardó un embedding.
- **Hipótesis:** `auto_embed_insert` (`src/executor.rs:471-513`) descarta el `Err(e)` con `tracing::warn!` (`:506-509`) y deja `node.vector = None`; `execute_insert` persiste igual (`:460`). No hay campo ni flag que registre la degradación (verificado: 18 constantes `FIELD_*` en `src/sdk/serialization/mod.rs:16-51`, ninguna de estado de embedding).
- **1 variable controlada:** por intento se cambia **una** cosa — (1) test RED sin el fix; (2) solo `auto_embed_insert` (variante `remote-inference`); (3) solo la variante `not(remote-inference)`; (4) solo `auto_embed_message`.
- **Test RED:** `auto_embed_insert_marks_node_when_provider_fails` escrito **antes** del fix; falla con `assertion failed: mark.is_some()` (el campo no existe) y pasa solo con la marca aplicada. Registrado abajo en el Step 1.

## Steps

### Step 1: RED — unit test que reproduce el silencio
- **Archivos:** `src/executor.rs` (`mod tests`)
- **Acción:** test `auto_embed_insert_marks_node_when_provider_fails` (`#[cfg(feature = "remote-inference")]` + `#[serial_test::serial]`): env de provider roto vía `EnvGuard` (patrón `src/llm.rs:1689-1717`), storage fjall en `tempdir`, `execute_statement(Statement::Insert(...))` con `text`, y asserts: nodo persistido sin `HAS_VECTOR` + `__vanta_embed_status == "failed"`. Verificar que **falla** por la razón correcta.
- **Verify:** `cargo nextest run -p vantadb --features remote-inference -E 'test(auto_embed)' --build-jobs 2` → **RED confirmado**: `assertion left: None, right: Some(String("failed"))` en `src/executor.rs:1647` (solo fallaba la marca; el resto de las aserciones ya pasaban → la degradación sin vector estaba reproduci­da).
- **Estado:** ✅

### Step 2: GREEN — marcar la degradación (INSERT)
- **Archivos:** `src/executor.rs`, `src/sdk/serialization/mod.rs`
- **Acción:** `pub const FIELD_EMBED_STATUS: &str = "__vanta_embed_status";` + valores `EMBED_STATUS_{FAILED,SYNTHETIC,UNAVAILABLE}` junto a las otras `FIELD_*`; `Executor::mark_embed_status`; marca en el `Err` (`failed`), en el camino sintético (`synthetic`) y en la variante sin `remote-inference` (`unavailable`); constante añadida a `RESERVED_FIELDS` (`:534`).
- **Verify:** `cargo nextest run -p vantadb --features remote-inference -E 'test(auto_embed)'` → **8/8 PASS**
- **Estado:** ✅

### Step 3: GREEN — marcar `auto_embed_message`
- **Archivos:** `src/executor.rs` (`auto_embed_message`)
- **Acción:** mismo tratamiento en el path `INSERT MESSAGE`: `failed` en el `Err`, `unavailable` sin `remote-inference`.
- **Verify:** incluido en el filtro `test(auto_embed)` (los `test_auto_embedding_graceful_degradation_on_insert_message` siguen verdes) + `cargo nextest run --profile audit -p vantadb --features remote-inference -E 'test(message)'` dentro del bloque de 323 tests ✅
- **Estado:** ✅

### Step 4: Test de superficie — `vanta-cli` la expone
- **Archivos:** `tests/auto_embed_insert_mark.rs` (nuevo)
- **Acción:** subproceso con env aislado (patrón `tests/embedding_fallback_visibility.rs`): INSERT por IQL + `FROM Note --json` → `fields.__vanta_embed_status` presente y en `{failed, synthetic, unavailable}`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --features remote-inference -E 'test(auto_embed)'` → `vantadb::auto_embed_insert_mark auto_embed_insert_mark_visible_via_cli` PASS. Evidencia manual del binario real:
  ```
  query 'INSERT NODE#1 TYPE Note { text: "hola mundo" }' --json → {"mode":"write","node_id":"1",...}
  query 'FROM Note' --json → {"records":[{"fields":{"__vanta_embed_status":"failed","text":"hola mundo","type":"Note"},"id":"1"}]}
  ```
- **Estado:** ✅

### Step 5: Docs + verify de cierre
- **Archivos:** `docs/api/IQL.md`, `docs/api/EMBEDDINGS.md`
- **Acción:** documentar en INSERT/INSERT MESSAGE que un `text` sin `VECTOR` se auto-embebe y que una degradación deja el nodo sin vector **con** `__vanta_embed_status` (valores, ausencia = sin auto-embed, cómo consultarlo, **límites**: no es clave de filtro en los page filters del SDK y `RESERVED_FIELDS` lo saca de `MemoryRecord.metadata`), incluyendo la divergencia sintética del path de mensajes. Bullet de paridad en EMBEDDINGS.md.
- **Verify:** `cargo fmt --all -- --check` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ (default) · `--features remote-inference` ✅ · `--features remote-inference,embed-local` ✅ · contrato 9/9 ✅ · `cargo check -p vantadb --no-default-features --features fjall` ✅ · leg default `test(auto_embed)` 2/2 ✅ · leg sintético 1/1 ✅ · regresión `test(serialization|export|import|message)` 324/324 ✅ · `pwsh dev-tools/ocr-review.ps1` sin Critical/High · review adversarial P2-01 con **changes-required** en 1ª pasada → 6 hallazgos resueltos → 2ª pasada completada sin nuevos blockers
- **Estado:** ✅



## Dependencias
- Ninguna (Wave 0). Otro sub-agente trabaja en `scripts/docs/**` — **no** se toca ni se commitea.

## Review (GATE — agente distinto, P2-01)

> Tier **adversarial** (`src/sdk/serialization/**` es path wire). Revisor `vanta-review` en sesión fresca (`ses_df1d8f5f0ffeNMi4SekY0WDHDt`), contexto distinto del implementador (`vanta-worker`, esta sesión).

- **Revisor:** `vanta-review` (agente distinto, fresh context)
- **Enfoque:** ¿marcar (vs propagar) es correcto? ¿queda algún camino de escritura silencioso? ¿el campo relacional es el mecanismo adecuado y es seguro para DBs viejas?
- **Veredicto 1ª pasada:** ❌ **changes-required** (8 hallazgos) → 2ª pasada: ❌ changes-required con 6 hallazgos (1 critical nuevo). Todos resueltos:

| # | Sev | Hallazgo | Resolución |
|---|-----|----------|-----------|
| 1a | critical | `auto_embed_message` guardaba vector sintético con `HAS_VECTOR` y **sin** marca (FIND-681 en el path de mensajes) | `provider.is_synthetic()` consultado + marca `synthetic` (política de vector intacta: se conserva) + test dedicado |
| 1b | critical | el crate **no compilaba** sin `remote-inference`: `FIELD_EMBED_STATUS` interpolado en el brazo `not(...)` con el import bajo el cfg opuesto (E0425) | import incluido en ambos cfg; `cargo check -p vantadb --no-default-features --features fjall` ✅ |
| 2 | medium | `text` vacío se marcaba en el build sin provider y no en el otro (semántica divergente) | mismo predicado trimmed en ambos brazos |
| 3 | medium | la marca no es enumerable (los page filters del SDK rechazan `__vanta_*`) y `RESERVED_FIELDS` la oculta de `MemoryRecord.metadata` → el doc oversellaba | documentados ambos límites en `docs/api/IQL.md` (scan-based) |
| 4 | medium | el brazo `unavailable` no corría bajo el contrato declarado | contrato ampliado con leg default + `--no-default-features` check; **ambos ejecutados** |
| 5 | low | sin test negativo de la invariante (d) (INSERT con `VECTOR` no debe marcarse) | `assert!(...is_none())` en `test_auto_embedding_skipped_when_vector_provided` |
| 6 | low | test CLI aceptaba 3 valores posibles (vacuidad potencial) | comentario explícito: es smoke feature-agnóstico, el valor exacto lo fija el unit test |
| nit | — | sugerencia: `degraded: bool` en `ExecutionResult::Write` | **declinado**: cambia un enum público que mapean los bindings para una señal que ya viaja en el registro; queda como upgrade posible en Notas |

- **Cómo se probó (evidencia real, no auto-reporte):** contrato 9/9 · leg default 2/2 · leg sintético 1/1 · regresión serialización/export/import/message 324/324 · clippy 3 configuraciones · `cargo check --no-default-features --features fjall` · ejecución real del binario `vanta-cli` mostrando `"__vanta_embed_status":"failed"`.
- **Checklist anti-hábitos tóxicos:** ✅ no se inventaron salidas (todo comando ejecutado está citado con su resultado); ✅ no se declaró done antes del veredicto; ✅ el error de compilación que encontró el revisor se corrigió y se agregó al contrato el leg que lo habría visto; ✅ cada hallazgo quedó conectado a un step o a una nota de deuda.
- **Veredicto final:** ✅ **approve** — 3ª pasada sobre el árbol commiteado (`c2b9e28e2`), re-medida por el revisor: `cargo check --no-default-features --features fjall` exit 0, 9/9 + 2/2 + 1/1 en los tres legs, `IQL.md` ya no afirma "same rule", sin artefactos NUL, **sin defectos bloqueantes nuevos**. Nits no bloqueantes aplicados: frase de `IQL.md` §INSERT MESSAGE desenredada; la deuda de política (descartar vs conservar el vector sintético) queda declarada en Notas como `ponytail:`/deuda, no arreglada acá.


## Context Save Point

> Handoff para reanudar sin preguntar. Última acción, evidencia y qué queda.

- **Última acción:** cierre de la tarea — implementación + review adversarial resuelto + verificación completa. Commit local `fix(core): mark INSERTs that fall back to no vector (FIND-485)`.
- **Verificación ejecutada (toda mecánica, ningún auto-reporte):**
  | Comando | Resultado |
  |---|---|
  | `cargo nextest run --profile audit -p vantadb --features remote-inference -E 'test(auto_embed)' --build-jobs 2` | ✅ 9/9 |
  | `cargo nextest run -p vantadb -E 'test(auto_embed)' --build-jobs 2` | ✅ 2/2 (brazo `unavailable`) |
  | `cargo nextest run -p vantadb --features remote-inference,embed-local -E 'test(auto_embed_message_marks_synthetic)'` | ✅ 1/1 |
  | `cargo nextest run --profile audit -p vantadb --features remote-inference -E 'test(serialization) \| test(export) \| test(import) \| test(message)'` | ✅ 324/324 |
  | `cargo check -p vantadb --no-default-features --features fjall` | ✅ |
  | `cargo clippy -p vantadb --all-targets [--features remote-inference] [--features remote-inference,embed-local] -- -D warnings` | ✅ ×3 |
  | `cargo fmt --all -- --check` | ✅ |
  | `pwsh dev-tools/ocr-review.ps1` + `ocr delegate rule src/executor.rs src/sdk/serialization/mod.rs` | ✅ sin Critical/High |
- **Artefactos:** `src/executor.rs`, `src/sdk/serialization/mod.rs`, `tests/auto_embed_insert_mark.rs`, `docs/api/IQL.md`, `docs/api/EMBEDDINGS.md`, este task file.
- **Próxima acción:** ninguna para esta tarea. Diferidos → RESULTADO (política sintética de `INSERT MESSAGE`; `degraded` en `ExecutionResult::Write`; marca obsoleta tras `UPDATE … SET VECTOR`).
- **Aviso de entorno:** durante el cierre hubo corrupción de artefactos por una sesión interrumpida (`src/executor.rs` con 246 bytes NUL finales + PDBs corruptos en `target/debug` → `LNK1000`/`LNK1285`). Se reparó truncando el archivo y borrando los `*.pdb` (9,1 GB); el árbol compila y formatea limpio. Si reaparece `LNK1000`, es el mismo artefacto sucio, no el código.
- **verify_changed.ps1:** fmt/check/clippy/clippy-tests ✅; `docs-coverage` ❌ **pre-existente** por drift del skills mirror (`skills/vantadb/SKILL.md`, FIND-83) — no lo toca este cambio (ningún archivo de `skills/` en el diff).

## Notas
- `ponytail:` la marca es un único campo con el estado, sin detalle del error persistido — el texto del error sigue en el `warn!`. Si hace falta el motivo por registro, el upgrade es `__vanta_embed_status_detail` (sin romper consumidores del campo de estado).
- `ponytail:` la marca no se limpia si un `UPDATE … SET VECTOR` posterior le da un vector al nodo: quedaría `HAS_VECTOR` + `__vanta_embed_status=failed` (marca obsoleta). Limpiarla obliga a tocar `apply_explicit_vector`, que esta tarea tiene prohibido modificar (`src/executor.rs:530`, compartido por INSERT/UPDATE). Upgrade: en `apply_explicit_vector`, `node.relational.remove(FIELD_EMBED_STATUS)` cuando `vector.is_some()`.
- **Declinado del review (nit):** `degraded: bool` en `ExecutionResult::Write` daría la señal en el momento de la escritura. Se descarta acá porque cambia un enum público que mapean los bindings (`vantadb-python/src/convert.rs`) para una señal que ya viaja en el registro y se lee con un `get`. Upgrade posible si un cliente pide "saber en el acto".
- El `warn!` **se mantiene**: la marca y el log son complementarios (log = operador, campo = dato/consultable).
- **NOTICED BUT NOT TOUCHING:**
  - Política de vector sintético divergente entre paths: `INSERT` lo **descarta**, `INSERT MESSAGE` lo **conserva** (ahora marcado `synthetic`). Unificarlo cambia qué se persiste → fila `FIND-*` propuesta en el RESULTADO (no se tocó para no romper la política de vectores en una tarea sobre la marca).
  - `vantadb-mcp/src/handlers/tools.rs` (`auto_embed_one`, `auto_embed_missing`) devuelve `(fallback, warning)` en la respuesta de la tool, así que ya no es silencioso en esa superficie — **y ese crate está fuera del alcance de esta tarea** (prohibido tocarlo).

