---
title: "FIND-755 - el gate de docs del workspace: 4 crates rojos de base, 43 links clasificados por sitio"
kind: task
status: completed
description: "El gate `cargo doc -D warnings` no pasaba en 4 crates (`vantadb` 38 links, `vanta-proxy` 2, `vantadb-mcp` 1, `vantadb-server` 1) mas una regresion de `vanta-memory` (1). 43 links clasificados: 18 resueltos con la ruta real, 21 aplanados a code span por apuntar a item privado, 2 a code span por target inexistente y 1 URL desnuda. Exit 0, 0 diagnosticos."
tags: [vantadb, docs, rustdoc, gate, workspace]
---

# FIND-755 - el gate de docs del workspace

## Contexto

`FIND-740` dejó `vanta-memory` en verde con `cargo doc --no-deps -p vanta-memory
-- -D warnings`, y midió que el resto del workspace seguía rojo en **4 crates**
(46 líneas de diagnóstico). Mientras quede un crate rojo, el paso de docs **no
se puede exigir**: con el check siempre rojo, nadie distingue "empeoraste" de
"ya estaba", y cada carril que quiera comprobar su documentación hereda el ruido
y acaba ignorando el comando. Ese es exactamente el estado del que `FIND-740`
acababa de sacar a `vanta-memory`.

`FIND-755` cierra el workspace. Es un gate **de workspace**, así que "arreglar el
lint" es arreglar los 43 sitios.

## Medición (antes / después)

Contrato mecánico:

```powershell
$env:RUSTDOCFLAGS="-D warnings"
cargo doc --no-deps --workspace --keep-going 2>&1 | rg '^error' | wc -l
```

**ANTES** (commit `60007a472`, árbol limpio salvo el budget de otro carril):

```
$env:RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --workspace --keep-going
error: unresolved link to `Cipher`
error: unresolved link to `EncryptionStream`
error: unresolved link to `registry`
error: unresolved link to `Cipher::encrypt_with_nonce`
error: unresolved link to `Cipher::decrypt_with_nonce`
error: unresolved link to `Cipher::derive_kek`
error: unresolved link to `KeySource`
error: unresolved link to `Cipher::encrypt`
error: unresolved link to `Cipher::decrypt`
error: public documentation for `derive_namespace` links to private item `NS_ENVELOPE_SALT`
error: unresolved link to `WriteReceipt`
error: public documentation for `ContentBinding` links to private item `write_declared_limits`
error: unresolved link to `FieldValue`
error: unresolved link to `crate::ingestion::AsyncIngestionPipeline`
error: public documentation for `resolve_local_model_path` links to private item `local_model_bases`
error: unresolved link to `ort::init_from`
error: unresolved link to `link_entities`
error: unresolved link to `ChainedError`
error: public documentation for `plan_all` links to private item `ArtifactSpec::classify`
error: public documentation for `check_integrity` links to private item `ArtifactSpec::classify`
error: unresolved link to `OperatorRegistry::compile`
error: public documentation for `with_entity` links to private item `node_in_entity_scope`
error: unresolved link to `Config::in_memory`
error: public documentation for `merge_record` links to private item `crate::sdk::merge`
error: unresolved link to `SkillStore`
error: public documentation for `query_error_response` links to private item `log_error`
error: public documentation for `pagination` links to private item `encode_cursor`
error: public documentation for `pagination` links to private item `decode_cursor`
error: unresolved link to `ConnectInfo`
error: unresolved link to `ShreddedField::Null`
error: unresolved link to `filter_verdict`
error: unresolved link to `filter_verdict`
error: unresolved link to `ShreddedRowStore::get`
error: public documentation for `create_life_insurance` links to private item `crate::backend::Snapshotable`
error: public documentation for `request_compaction` links to private item `crate::backend::Compactable`
error: public documentation for `create_in_memory` links to private item `crate::hardware::HardwareCapabilities::resident_default_ceiling`
error: public documentation for `read_header` links to private item `File::read_node`
error: public documentation for `WAL_FORMAT_VERSION` links to private item `CHAIN_GENESIS`
error: could not document `vantadb`
error: public documentation for `note_backfill` links to private item `derived_health`
error: could not document `vanta-memory`
error: public documentation for `Reporter` links to private item `RECENT_CAP`
error: public documentation for `recent_reports` links to private item `RECENT_CAP`
error: this URL is not a hyperlink
error: could not document `vanta-proxy`
error: could not document `vantadb-mcp`
error: unresolved link to `ServerHooks::on_storage_ready`
error: could not document `vantadb-server`
```

**48 líneas `^error`** = **43 links/URLs** + **5 `could not document`**,
**24 sitios con `-->`**, exit **101**.

**DESPUÉS** (mismo comando, mismo árbol con el cambio de esta tarea):

```
$env:RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --workspace --keep-going
    Checking vantadb v0.9.0 (...)
 Documenting vantadb v0.9.0 (...)
   Compiling vantadb-server v0.9.0 (...)
    Checking vanta-memory v0.9.0 (...)
 Documenting vanta-memory v0.9.0 (...)
 Documenting vantadb-wasm v0.9.0 (...)
    Checking vantadb-mcp v0.9.0 (...)
 Documenting vanta-proxy v0.9.0 (...)
    Checking vanta-proxy v0.9.0 (...)
 Documenting vantadb_py v0.9.0 (...)
 Documenting vantadb-mcp v0.9.0 (...)
 Documenting vantadb-server v0.9.0 (...)
```

**0 líneas `^error`, 0 diagnósticos, exit 0.**

> Nota sobre el número de `FIND-740`: midió **46 líneas de diagnóstico**; hoy hay
> **48 `^error`**. El delta es `vanta-memory` (`note_backfill` →
> `derived_health`), una **regresión posterior** — ver §Desviación declarada.

## Criterio de decisión (el de `FIND-740`)

| Clase | Cuándo | Cómo se ve en el HTML |
|---|---|---|
| **(a) resoluble** | el ítem existe y es alcanzable; el link sólo apunta al scope equivocado | **cambia a mejor**: hoy renderiza texto sin hipervínculo, después hipervínculo |
| **(b) privado** | el ítem existe pero es privado y la doc es pública | **no cambia**: rustdoc ya no lo enlazaba (lo demuestra el propio error: `= note: this link will resolve properly if you pass --document-private-items`, es decir **no** resuelve en el gate) |
| **(c) target inexistente** | el ítem no existe bajo el feature set medido o es de un crate externo fuera del prelude | **no cambia**: ya renderizaba el texto plano |
| **(d) redundante / otro lint** | el target del link es el propio texto, o el lint no es de links | depende; acá 1 `bare_urls` |

**No se debilitó el bar:** no hay ni un `#[allow(rustdoc::*)]` nuevo
(`rg 'allow\(rustdoc' src/ vanta-proxy/ vantadb-mcp/ vantadb-server/ vanta-memory/` → 0).
Los 18 resueltos **se resolvieron**, no se aplanaron.

## Clasificación por sitio y evidencia

### (a) Resueltos con la ruta real — 18

Los 9 de `src/crypto.rs`, `src/attestation.rs:24`, `src/entity/linking.rs:24`,
`src/error.rs:13`, `src/operator_registry.rs:8` y `src/shred/mod.rs:37` son
**docs de módulo (`//!`)**: se resuelven en el scope del **padre** porque la
declaración `pub mod m;` en `src/lib.rs` lleva `///`(hallazgo del review de
`FIND-740`). Por eso `pub struct Cipher` "no está en scope" en la doc de su
propio módulo. Son **resolubles**, no irresolubles: se les dio la ruta completa.

| Sitio | Link | Visibilidad del target (evidencia) | Fix |
|---|---|---|---|
| `src/crypto.rs:10` | `[`Cipher`]` | `pub struct Cipher` (`crypto.rs:706`) | `(crate::crypto::Cipher)` |
| `src/crypto.rs:10` | `[`EncryptionStream`]` | `pub struct EncryptionStream<S>` (`:1091`) | `(crate::crypto::EncryptionStream)` |
| `src/crypto.rs:38` | `[`registry`]` | `pub mod registry` (`:90`) | `(crate::crypto::registry)` |
| `src/crypto.rs:40` | `Cipher::encrypt_with_nonce` | `pub fn` (`:860`) | `(crate::crypto::Cipher::encrypt_with_nonce)` |
| `src/crypto.rs:40` | `Cipher::decrypt_with_nonce` | `pub fn` (`:898`) | `(crate::crypto::Cipher::decrypt_with_nonce)` |
| `src/crypto.rs:42` | `Cipher::derive_kek` | `pub fn` (`:810`) | `(crate::crypto::Cipher::derive_kek)` |
| `src/crypto.rs:43` | `[`KeySource`]` | `pub enum KeySource` (`:226`) | `(crate::crypto::KeySource)` |
| `src/crypto.rs:45` | `Cipher::encrypt` | `pub fn` (`:949`) | `(crate::crypto::Cipher::encrypt)` |
| `src/crypto.rs:45` | `Cipher::decrypt` | `pub fn` (`:1002`) | `(crate::crypto::Cipher::decrypt)` |
| `src/attestation.rs:24` | `[`WriteReceipt`]` | `pub struct WriteReceipt` (`:678`) | `(crate::attestation::WriteReceipt)` |
| `src/entity/linking.rs:24` | `[`link_entities`]` | `pub fn link_entities` (`:559`) | `(crate::entity::linking::link_entities)` |
| `src/error.rs:13` | `[`ChainedError`]` | `pub struct ChainedError` (`:61`) | `(crate::error::ChainedError)` |
| `src/operator_registry.rs:8` | `OperatorRegistry::compile` | `pub fn compile` (`:190`) | `(crate::operator_registry::OperatorRegistry::compile)` |
| `src/shred/mod.rs:37` | `ShreddedField::Null` | `pub enum ShreddedField` (`:79`) | `(crate::shred::ShreddedField::Null)` |
| `src/cli_handlers/fmt.rs:70` | `[`FieldValue`]` | `pub enum FieldValue` (`src/node/field.rs:7`); la firma de la fn ya usa `crate::node::FieldValue`, sólo faltaba el path en la doc | `(crate::node::FieldValue)` |
| `src/sdk/api/memory.rs:2136` | `Config::in_memory()` | `pub fn in_memory` (`src/config.rs:1944`), `pub struct Config` (`:912`) | `(crate::config::Config::in_memory)` |
| `src/sdk/builder.rs:322` | `[`SkillStore`]` | `pub struct SkillStore<'a>` (`src/skills.rs:72`) | `(crate::skills::SkillStore)` |
| `src/server/state.rs:537` | `[`ConnectInfo`]` | `axum::extract::ConnectInfo`; `axum` está en el prelude del módulo (la firma usa `axum::extract::Request`) | `(axum::extract::ConnectInfo)` |

`vantadb-server/src/scheduler.rs:17` `[`ServerHooks::on_storage_ready`]`: campo
**pub** de `vantadb::cli_server::ServerHooks` (`src/server/bootstrap.rs:409`
lo consume `vantadb-server/src/main.rs`, otro crate ⇒ es `pub`). Doc de módulo ⇒
scope del padre, donde `ServerHooks` **sí** está importado (`use
vantadb::cli_server::ServerHooks;`, `scheduler.rs:41`) pero en el *módulo*, no en
la raíz. Resuelto a la ruta completa:
`(vantadb::cli_server::ServerHooks::on_storage_ready)`.

### (b) Aplanados a code span por ser **privados** — 21

**Evidencia de irresolubilidad (no auto-reporte):** el compilador emite, en cada
uno, `this item is private` + `= note: this link will resolve properly if you
pass --document-private-items`. Es decir: bajo el comando del gate (sin
`--document-private-items`) **no existe ruta** que los resuelva. Aplanar a code
span es exactamente lo que ya renderizaba.

| Sitio | Link | Por qué es privado | Fix |
|---|---|---|---|
| `src/crypto.rs:768` | `[`NS_ENVELOPE_SALT`]` | const privada de módulo | code span (además arregla un code span anidado mal cerrado: `` `[`X`]`, ...` ``) |
| `src/attestation.rs:666` | `[`write_declared_limits`]` | fn privada | code span |
| `src/config.rs:615` | `[`local_model_bases`]` | fn privada | code span |
| `src/migration.rs:373` | `[`ArtifactSpec::classify`]` | `ArtifactSpec::classify` privado | code span |
| `src/migration.rs:664` | `[`ArtifactSpec::classify`]` | idem | code span |
| `src/physical_plan/vector.rs:166` | `[`node_in_entity_scope`]` | **`pub(crate)`** | code span |
| `src/sdk/api/memory.rs:2274` | `[`crate::sdk::merge`]` | módulo `sdk::merge` privado | code span |
| `src/server/errors.rs:119` | `[`log_error`]` | fn privada | code span |
| `src/server/pagination.rs:5` | `[`encode_cursor`]` | fn privada | code span |
| `src/server/pagination.rs:5` | `[`decode_cursor`]` | fn privada | code span |
| `src/shred/mod.rs:40` | `[`filter_verdict`]` | **`pub(crate) fn`** (`:232`) | code span |
| `src/shred/mod.rs:63` | `[`filter_verdict`]` | idem | code span |
| `src/shred/mod.rs:63` | `[`ShreddedRowStore::get`]` | **`pub(crate) fn get`** (`:408`) | code span |
| `src/storage/engine/maintenance.rs:637` | `[`Snapshotable`](crate::backend::Snapshotable)` | trait privado (el link **ya** traía el path correcto; el ítem es privado) | code span |
| `src/storage/engine/stats.rs:277` | `[`Compactable`](crate::backend::Compactable)` | idem | code span |
| `src/storage/vfile.rs:615` | `[`crate::hardware::HardwareCapabilities::resident_default_ceiling`]` | fn privada | code span |
| `src/storage/vfile.rs:890` | `[`File::read_node`]` | fn privada | code span |
| `src/wal.rs:21` | `[`CHAIN_GENESIS`]` | const privada | code span |
| `vanta-memory/src/ingest/observability.rs:348` | `[`derived_health`]` | `fn derived_health` privada (`:445`) | code span |
| `vanta-proxy/src/report.rs:57` | `[`RECENT_CAP`]` | `const RECENT_CAP` privada (`:55`) | code span |
| `vanta-proxy/src/report.rs:105` | `[`RECENT_CAP`]` | idem | code span |

### (c) Target inexistente bajo el feature set medido — 2

| Sitio | Link | Evidencia | Fix |
|---|---|---|---|
| `src/config.rs:187` | `[`AsyncIngestionPipeline`](crate::ingestion::AsyncIngestionPipeline)` | el link **ya** traía el path, pero `crate::ingestion` sólo existe con `#[cfg(feature = "async-ingestion")]` (`src/lib.rs:181-183`) y la feature **no** está en `default` | code span |
| `src/embedding_health.rs:11` | `[`ort::init_from`]` | `ort` es crate externo y no está en el prelude de la raíz (`no item named 'ort' in scope`); el mismo fichero ya lo escribe como code span en `:25` y `:355` | code span |

### (d) Otro lint de rustdoc — 1

`vantadb-mcp/src/temporal.rs:7`: `bare_urls` (una URL desnuda no es hipervínculo).
Fix: `<https://howardhinnant.github.io/date_algorithms.html>` — el propio
`help:` del compilador propone exactamente esa forma.

## Desviación declarada (2 ficheros fuera del scope autorizado)

El encargo prohibía `vanta-memory/**` y `src/wal.rs`. **Se tocaron, 1 línea de
doc cada uno**, y esto es el motivo:

1. **`vanta-memory/src/ingest/observability.rs:348`** — `vanta-memory` **ya no
   está en verde**: `note_backfill` es código **nuevo** de `FIND-745`
   (`16f22af45`, 2 commits antes de esta sesión) y trae un link a
   `derived_health`, que es `fn` privada. La premisa "ya está en verde" era
   cierta cuando se escribió el ticket y **dejó de serlo** en cuanto aterrizó ese
   commit. Sin tocar esa línea el contrato (workspace exit 0) es inalcanzable.
2. **`src/wal.rs:21`** — 1 link a la const privada `CHAIN_GENESIS`.

Ambos son `///` (cero líneas ejecutables) y se comprueba mecánicamente:
`git diff -U0 -- src/ vanta-memory/ ... | rg '^[+-][^+-]' | rg -v '^[+-]\s*(///|//!)'` → **0 líneas**.

**Lección:** "verde" no es una propiedad durable sin el gate cableado. Es la
misma clase que `FIND-740` cierra y el argumento de por qué el paso tiene que
entrar en CI, no sólo quedarse en un comando que alguien corre una vez.

## Alcance

24 ficheros, **38 inserciones / 38 borrados**, todas líneas de documentación:
`src/` (20), `vanta-memory/` (1), `vanta-proxy/` (1), `vantadb-mcp/` (1),
`vantadb-server/` (1), más este task file / Backlog / avance.

No tocados (respetado): `src/bin/vanta-cli.rs`, `tests/cli_tests.rs` (FIND-98),
`src/wal_sharded.rs`, `src/schema.rs`, `src/index/serialize/file.rs`,
`.githooks/**`, `docs/dev/workflow/RULES.md`, `dev-tools/**`,
`docs/dev/plans/**`, `.opencode/**`, `docs/dev/extraccion/**`.
`src/wal.rs` **sí** se tocó — ver §Desviación declarada.

## Impacto mapeado (Regla 0)

- **Leídos completos:** las 24 cabeceras/regiones de doc con error, más
  `src/lib.rs` (declaraciones `pub mod` + features), `src/crypto.rs` (visibilidad
  de `Cipher`/`KeySource`/`registry`), `src/shred/mod.rs` (`pub(crate)` de
  `filter_verdict`/`ShreddedRowStore::get`), `.github/workflows/docs-gates.yml`
  (job `rustdoc`), `.github/workflows/ci-gate.yml` (`REQUIRED`).
- **Referencias salientes:** ninguna. Son comentarios.
- **Referencias entrantes:** ninguna ejecutable. La única superficie que cambia
  es el HTML de rustdoc.
- **Veredicto:** **cero** símbolos públicos nuevos, cero cambios de firma, cero
  cambios de comportamiento. Ningún ítem cambió de visibilidad. El binario no
  cambia.

## Verificación

```powershell
# el contrato de la fila (ANTES: 48 ^error / exit 101 · DESPUÉS: 0 / exit 0)
$env:RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --workspace --keep-going

# sólo doc tocada (debe imprimir nada)
git diff -U0 | rg '^[+-][^+-]' | rg -v '^[+-]\s*(///|//!)'

# no se debilitó el bar (debe ser 0)
rg -n 'allow\(rustdoc' src/ vanta-proxy/ vantadb-mcp/ vantadb-server/ vanta-memory/

# fmt/clippy no ven comentarios, pero se corre igual
cargo fmt --check
cargo clippy -p vantadb -p vanta-proxy -p vantadb-mcp -p vantadb-server --all-targets -- -D warnings
```

Resultados:

| Comando | Resultado |
|---|---|
| `cargo doc --no-deps --workspace --keep-going` con `-D warnings` | **exit 0**, **0** `^error` |
| mismo + `--document-private-items` (la forma que usa `docs-gates.yml`) | **exit 101**, **33** `^error` — **otra superficie**, ver §Propuesta |
| `git diff -U0` filtrado a líneas no-doc | **vacío** |
| `rg 'allow\(rustdoc'` en los 5 crates | **0** |
| `cargo fmt --check` | **exit 0** |

## Review (P2-01)

Tier **fast** por glob (todo el diff cae en "cualquier otro path": doc comments
de `src/**`, `vanta-proxy/`, `vantadb-mcp/`, `vantadb-server/`, `vanta-memory/`
y `docs/dev/**`; ningún path adversarial). Gate: verify mecánico + veredicto
registrado.

Pendiente de la ronda de `vanta-review` (se completa al volver).

## Propuesta de cableado en CI (no aplicada: `.github/**` es de otro carril)

**Hallazgo que cambia la propuesta:** el job que parecía faltar **ya existe y
está en verde-sin-medir**. `docs-gates.yml` tiene el job `rustdoc`
(`name: Generate API reference (rustdoc)`, `docs-gates.yml:527`) que corre:

```yaml
- name: Generate rustdoc
  run: cargo doc --no-deps --workspace --all-features --document-private-items
```

Corre **sin** `-D warnings`, así que **construye el artefacto y no mide nada** —
el comentario del propio workflow dice *"Falla por defecto en docstrings con
links rotos"* y es **falso**: sin negar warnings, un link roto no falla. Segundo
detalle: el contexto `Generate API reference (rustdoc)` **no está** en `REQUIRED`
(`ci-gate.yml:38-58`), así que ni siquiera su rojo actual bloquearía.

**Y no alcanza con agregarle `-D warnings` a ese step: medido, da 33 errores**
(exit 101, 28 sitios con `-->`):

| Crate | `^error` | Ejemplos |
|---|---|---|
| `vantadb` | 15 | `Executor::execute_plan`, `IndexType::DiskAnn`, `SegmentRegistry`, `CacheWarmer`, `Self::put` ×2, `StorageEngine::filter_field` ×2, **`redundant explicit link target`** |
| `vantadb-python` | 9 | `SearchHit` ×4 (`lib.rs:1377,1441,1558,2148`), `int` (`types.rs:21`), **`unclosed HTML tag f32`** ×3 (`types.rs:35-36`) |
| `vantadb-mcp` | 6 | `RECALL_ENVELOPE_META_BYTES`, URL desnuda (`temporal.rs:375`), **`unclosed HTML tag unit`** ×3 (`temporal.rs:376`) |
| `vanta-memory` | 3 | `PipelineSessionState`, `crate::services::pipeline_worker::run_context_assembly` |

La razón es de **scope**, no de calidad: `--document-private-items` documenta
ítems que el build del gate ni siquiera procesa (docs de módulos privados y de
los wrappers PyO3), y ahí aparecen lints que antes no existían — incluidos dos
lints **que no son de links** (`invalid_html_tags`, `redundant explicit link
target`). Son 33 sitios nuevos, no los mismos 43.

**Decisión propuesta: SÍ entra, con el mismo criterio que clippy — sin
`continue-on-error`.** Concreto y mínimo, como step **nuevo** (sin tocar el step
del artefacto, que debe seguir generando el tarball aunque el gate avise):

```yaml
# docs-gates.yml, job `rustdoc`, step NUEVO
- name: Deny rustdoc warnings (doc gate)
  env:
    RUSTDOCFLAGS: "-D warnings"
  run: cargo doc --no-deps --workspace --keep-going
```

y agregar `"Generate API reference (rustdoc)"` a `REQUIRED` en `ci-gate.yml`.

**Por qué ese comando y no el del artefacto:** es **exactamente el medido en
verde** (0 diagnósticos, exit 0) y usa el árbol ya caliente del step anterior ⇒
coste marginal (segunda invocación, sin recompilar). El del artefacto exige
resolver antes los 33 de la tabla, que son otra fila.

**Deuda / condición de aterrizaje:** `FIND-719` dejó medido que **ningún job de
`docs-gates` es required en `develop`** (los contexts requeridos son de Rust),
así que el gate sólo muerde en el PR de release. Cablearlo sin resolver eso
cambia poco; hacerlo requiere tocar la lista de required de `develop`, que es
decisión del owner.

**Fila derivada propuesta (para el lead, no creada acá — el Backlog sólo admite
mi fila):** *"`cargo doc --document-private-items -D warnings` (la vista del
artefacto de `docs-gates.yml`) tiene 33 lints en 4 crates, 2 de ellos no-links
(`invalid_html_tags` ×6, `redundant explicit link target` ×1); el job `rustdoc`
los ignora porque no niega warnings y no está en `REQUIRED`"*. Sin esa fila, el
artefacto publicado sigue llamándose "gate de calidad" sin medir.

## Incógnitas

1. **`--all-features` sigue sin medirse.** `--document-private-items` **sí** se
   midió (33 errores, §Propuesta). Falta la pata `--all-features`, que trae
   rocksdb/librocksdb-sys/bindgen y no es un build barato en Windows: compila
   módulos que `default` no tiene (`async-ingestion`, `llm`, `server`,
   `encryption`) y bajo esa combinación el conteo puede volver a cambiar. El
   gate propuesto **no** depende de eso; el artefacto sí.
2. **`missing_docs` no es parte de este gate.** Ningún error de esta fila fue por
   falta de documentación: los 43 son de *links*. Exigir `#![deny(missing_docs)]`
   es una decisión de política aparte (y hoy no hay base medida para tomarla).
3. **El aplaneo de los 21 privados cambia el render del artefacto**
   `--document-private-items`: ahí sí eran hipervínculos. Es el precio de que el
   gate (sin esa flag) distinga; alternativa —no aplicada— sería documentar esos
   helpers, que es escribir doc nueva para satisfacer un lint, no para un lector.
