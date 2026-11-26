---
title: "FIND-761 - la vista privada del artefacto de docs: 36 lints en 5 crates, y el step que no mide"
kind: task
status: completed
description: "El job `rustdoc` de `docs-gates.yml:527` construye el artefacto y no mide: corre `cargo doc` sin `-D warnings` y su comentario dice lo contrario. Negar warnings no alcanza: la vista del artefacto (`--all-features --document-private-items`) trae 36 lints en 5 crates. Los 36 cerrados: gate exit 0 / 0 errores."
tags: [vantadb, docs, rustdoc, gate, workspace]
---

# FIND-761 - la vista privada del artefacto de docs

## Contexto

`FIND-755` cerró **una** superficie: `cargo doc -D warnings` sin
`--document-private-items`, la que ve un lector de docs.rs. Quedó declarada la
otra, la del **artefacto que el repo publica** (`docs-gates.yml:557-558`:
`cargo doc --no-deps --workspace --all-features --document-private-items`), con
un número heredado que nadie había medido: *"33 lints en 4 crates"*.

Esta tarea cierra esa superficie. Son dos cosas distintas y las dos importan:

1. **El job no mide.** Corre `cargo doc` **sin** negar warnings (`:556` afirma
   *"Falla por defecto en docstrings con links rotos"*, y es falso: `cargo doc`
   sin `RUSTDOCFLAGS` sale **exit 0** con 36 warnings, medido), y su contexto
   `Generate API reference (rustdoc)` no está en `REQUIRED`
   (`ci-gate.yml:36-55`). Genera un tarball y no juzga nada.
2. **Negar warnings ahí no alcanza**, porque esa vista documenta ítems que el
   gate público ni siquiera procesa — y ahí aparecen errores que el otro gate no
   puede ver: 36, no 33.

**Las dos superficies necesitan gate propio y no se pueden unificar** (§Por qué
dos steps).

## Medición (antes / después)

Contrato mecánico (el del ticket):

```powershell
cargo doc --no-deps --workspace --all-features --document-private-items -- -D warnings 2>&1 | rg '^error' | wc -l   # 0
```

En Windows, forma equivalente y la que se corrió (`-- -D warnings` y
`RUSTDOCFLAGS` son el mismo canal para rustdoc; se usó el segundo porque es el
que va a usar el step de CI). `rg`/`wc` no son comandos nativos de pwsh: se
cuenta con `(Select-String -Path <log> -Pattern '^error').Count` sobre la salida
redirigida, que es lo que se reporta abajo:

```powershell
$env:RUSTDOCFLAGS="-D warnings"
cargo doc --no-deps --workspace --all-features --document-private-items --keep-going *> "$env:TEMP\gate.log"
(Select-String -Path "$env:TEMP\gate.log" -Pattern '^error').Count
```

**ANTES** (commit `cd7d8c5f7`, árbol limpio salvo el budget de otro carril):

```
EXIT=101
41        # líneas ^error
```

36 diagnósticos + 5 `error: could not document` (`vantadb`, `vanta-memory`,
`vantadb-wasm`, `vantadb_py`, `vantadb-mcp`), 34 líneas `-->`.

**DESPUÉS** (mismo comando, mismo árbol con el cambio de esta tarea):

```
EXIT=0
0
```

### El número del ticket no se reproduce: la superficie es **mayor**

Las dos mediciones, lado a lado (no se intenta reconciliar crate a crate, porque
el desglose del ticket **no dice con qué contaba**):

| Crate | Medido acá (con `--all-features`) | Ticket |
|---|---|---|
| `vantadb` | **20** | 15 |
| `vantadb-python` | **8** | 9 |
| `vantadb-mcp` | **5** | 6 |
| `vanta-memory` | **2** | 3 |
| `vantadb-wasm` | **1** | — (no figuraba) |
| **total** | **36** (+5 `could not document` = **41** `^error`) | 33 |

Dos cosas sí están probadas por el propio diagnóstico, y son las que importan:

- **El 33 se midió sin `--all-features`.** `FIND-755` lo dice en su
  §Incógnitas 1 (`FIND-755.md:436-437`): *"`--all-features` sigue sin medirse.
  `--document-private-items` **sí** se midió (33 errores)"*, y su review ronda 1
  lo registra reproducido (`FIND-755.md:309`: `EXIT=101, 33 ^error, 28 -->`).
  O sea: **33 sin la pata de features, 36 con ella**. La diferencia entra por
  código gateado que sólo compila con `--all-features`: `src/llm.rs` completo
  (`#[cfg(any(feature = "remote-inference", feature = "embed-local"))]`,
  `src/lib.rs:120-121`, ninguno de los dos en `default` — `Cargo.toml:156`) y
  `watch_config` (`#[cfg(feature = "hot-reload")]`, `src/config.rs:2284`).
- **`vantadb-wasm` no estaba en la lista de crates del ticket** y aporta
  `invalid_rust_codeblocks` ×1. Ése es el 5.º crate.

El desglose por crate del ticket (15/9/6/3) no cuadra con ninguna lectura
uniforme —contar `^error` incluye un `could not document` por crate, contar
lints no—, así que **no se usa para reconciliar**: quedan pegados los dos
totales medidos y la causa de los dos sumandos que sí están probados.

**Aritmética de las 36:** 34 líneas `-->` + **2** diagnósticos sin `-->`
(`src/lsm.rs:5`, doc de módulo, que apunta con *"the link appears in this line"*;
y `vantadb-wasm/src/worker.rs:7`, un code block) = 36. Y un mismo **sitio**
(`vantadb-mcp/src/temporal.rs:376`) lleva 2 lints en la misma línea, por eso
contar `file:line` tampoco da 36.

Conclusión honesta: la superficie era **36, no 33**, y estaba en **5 crates, no
4**. Se cierra entera.

## Criterio de decisión (el mismo de `FIND-740`/`FIND-755`)

| Clase | Cuándo | Cómo se ve |
|---|---|---|
| **(a) resuelto** | el ítem existe y es alcanzable; faltaba la ruta | cambia a mejor: hipervínculo donde había texto |
| **(b) privado real** | el ítem existe pero es privado y el doc que lo nombra es público | no cambia en el artefacto **público** (ahí ya era texto) |
| **(c) target inexistente** | el ítem no existe bajo el feature set / la cfg medida | no cambia: ya renderizaba texto plano |
| **(d) otro lint** | no es de links: HTML inválido, URL desnuda, code block, target redundante | se arregla el defecto, no se aplana |

**No se debilitó el bar:** `rg 'allow\(rustdoc' src/ vantadb-python/ vantadb-mcp/ vanta-memory/ vantadb-wasm/` → **0**.
Ningún `#[allow(rustdoc::*)]` nuevo, ningún `continue-on-error`, ningún
`#![doc(...)]` que silencie. Los 14 de (a) se **resolvieron**.

## Clasificación por sitio y evidencia

### (a) Resueltos con la ruta real — 14 lints, 14 líneas

| Sitio | Link | Evidencia de que el ítem existe y es alcanzable | Fix |
|---|---|---|---|
| `src/config.rs:2283` | `watch::Sender` | **doc obsoleto**: el canal real es `mpsc::channel::<()>()` (`:2296`) y el retorno declarado es `std::sync::mpsc::Sender<()>` (`:2289`); no hay tokio `watch` en el cuerpo | texto corregido + `(std::sync::mpsc::Sender)` |
| `src/governor.rs:102` | `Executor::execute_plan` | `pub fn` en `pub struct Executor<'a>` (`src/executor.rs:287` y `:796`); el doc vive en `governor.rs`, donde no está importado | `(crate::executor::Executor::execute_plan)` |
| `src/index/diskann.rs:19` | `IndexType::DiskAnn` | `pub enum IndexType` (`src/index_port.rs:43`) re-exportado por `pub use crate::index_port::IndexType;` (`src/index/mod.rs:25`) | `(crate::index::IndexType::DiskAnn)` |
| `src/llm.rs:508,1246,1328,1442` | `LlmCfg` ×4 | `crate::config::LlmCfg` es `pub` y **la firma de al lado lo escribe con esa ruta** (`&crate::config::LlmCfg`) | `(crate::config::LlmCfg)` |
| `src/lsm.rs:5` | `SegmentRegistry` | `pub(crate) struct` (`src/lsm.rs:87`). **Documentado bajo `--document-private-items`** — exactamente la diferencia con el gate de `FIND-755`, donde el módulo privado ni se procesa | `(crate::lsm::SegmentRegistry)` |
| `src/physical_plan/mod.rs:94` | `Ordering` | `std::cmp::Ordering` sólo se importa **dentro del cuerpo** de la fn (`:98`) | `(std::cmp::Ordering)` |
| `src/sdk/search/lexical.rs:174` | `metadata_hash_bit` | `pub fn` en `crate::sdk::serialization` (`src/sdk/serialization/mod.rs:128`); el call site ya usa esa ruta (`:181`) | `(crate::sdk::serialization::metadata_hash_bit)` |
| `src/storage/engine/cache.rs:5` | `CacheWarmer` | `pub(crate) struct` (`src/cache_warmer.rs:34`) | `(crate::cache_warmer::CacheWarmer)` |
| `src/storage/engine/coordinators.rs:1` | `StorageEngine` | `pub struct` del módulo padre; el archivo ya lo importa para sus firmas | `(crate::storage::engine::StorageEngine)` |
| `vanta-memory/src/services/pipeline_worker.rs:628` | `PipelineSessionState` | `pub` en `crate::core::state`; el `use` está **dentro del cuerpo** (`:637`) | `(crate::core::state::PipelineSessionState)` |
| `vanta-memory/src/context_engine/report_store.rs:4` | `…::run_context_assembly` | método privado de **`MemoryTaskHandler`** (`impl` en `:380`, `fn` en `:795`), no de `PipelineWorker` ni fn de módulo: la ruta del ticket apuntaba mal | `(…::MemoryTaskHandler::run_context_assembly)` |

### (b) Privado real — 1

| Sitio | Link | Evidencia | Fix |
|---|---|---|---|
| `src/ingestion.rs:46` | `Self::worker_loop_batched` | `async fn` privada (`:146`). El diagnóstico lo prueba: `this link resolves only because you passed --document-private-items, but will break without` | code span |

No cambia el artefacto público: ahí el ítem ni se documenta y el texto ya salía
plano. En el privado se pierde un hipervínculo interno — mismo precio ya
declarado en `FIND-755` §Incógnitas 3.

### (c) Target inexistente bajo la cfg/feature medida — 11

| Sitio | Link | Evidencia | Fix |
|---|---|---|---|
| `src/shred/mod.rs:280` y `:350` | `Self::put` ×2 | el `put` que existe está tras **`#[cfg(test)]`** (`:334-335`): en un build de doc no existe | code span |
| `src/scalar_index.rs:8` | `crate::storage::engine::StorageEngine::filter_field` | `rg 'fn filter_field' src/` → **0 hits**: el método no existe (el link ya traía la ruta correcta; el target se fue) | code span |
| `src/storage/engine/mod.rs:474` | `StorageEngine::filter_field` | ídem | code span |
| `src/storage/engine/insert.rs:792` | `Self::existing_for_batch` | el doc está en `existing_for_batch_many` (`#[cfg(feature = "rayon")]`, `:801`) y linkea al hermano `#[cfg(not(feature = "rayon"))]` (`:787-788`): **con `--all-features` no existe** | code span + la cfg nombrada en el texto |
| `vantadb-mcp/src/handlers/tools.rs:4523` | `RECALL_ENVELOPE_META_BYTES` | es una `const` **dentro del cuerpo** de una fn (`:4529`), no un ítem del módulo: no hay ruta que la linkee | code span |
| `vantadb-python/src/lib.rs:1377,1441,1558,2148` | `SearchHit` ×4 | en `list[SearchHit]` el nombre es el de **Python** (`#[pyclass(name = "SearchHit")]`, `vantadb-python/src/types.rs:367`) sobre un `pub(crate) struct VantaPySearchHit` (`:368`): no existe ítem `SearchHit`, y el texto es una expresión de typing de Python. Se podría linkear `VantaPySearchHit`, pero **el docstring que PyO3 expone a Python no renderiza Markdown**: un `[X](path)` aparecería literal en `help()`, y los backticks que sí se agregaron también se ven (más baratos de leer que una ruta de módulo). El code span es lo único que no corrompe ninguna de las dos superficies | code span del bloque `list[…]` |
| `vantadb-python/src/lib.rs:2613` | `int` | tipo de Python | code span |

El de `vantadb-python` es el caso que el ticket marcaba como **nuevo** (no estaba
en `FIND-755`): no es un crate distinto de lo esperado — es el binding PyO3, y
rompía por **otra** razón, no por links a ítems privados: sus docstrings son
contratos de Python (`list[SearchHit]`, `list[int]`) que rustdoc intenta leer
como links. Aplanar ahí **no** pierde nada: nunca fueron enlaces.

### (d) Otros lints (no de links) — 10

| Lint | Sitio(s) | Qué estaba mal | Fix |
|---|---|---|---|
| `redundant_explicit_link_target` | `src/sdk/api/sharing.rs:2` | `[`PermissionChecker`](crate::entity::checker::PermissionChecker)` donde el label **ya resuelve al mismo destino** (`use crate::entity::checker::PermissionChecker;`, `:43`); ayuda del compilador: *"explicit target is redundant … because label contains path that resolves to same destination"* | se quitó el `(…)`, queda `[`PermissionChecker`]` |
| `invalid_html_tags` ×3 | `vantadb-python/src/types.rs:21,35,36` | `Vec<f32>` se parsea como tag `<f32>` abierto | `` `Vec<f32>` `` (el `help: try marking as source code` del compilador) |
| `invalid_html_tags` ×3 | `vantadb-mcp/src/temporal.rs:375,376` | `<unit>` en la prosa de spans relativos (3 tags en 2 líneas) | `` `<unit>` `` |
| `bare_urls` | `vantadb-mcp/src/handlers/initialize.rs:14` | URL desnuda, no es hipervínculo | `<https://…>` |
| `invalid_rust_codeblocks` | `vantadb-wasm/src/worker.rs:7` | un fence `ignore` con un diagrama de caja (caracteres `│ ├ ─`): *"could not parse code block as Rust code"* | fence `text` |
| `broken_intra_doc_links` (falso positivo de Markdown) | `src/cache_warmer.rs:35` | `co_access[A][B]` es **reference link** de Markdown: el segundo `[B]` se leyó como target | backticks en el índice |

Recuento: (a) 14 + (b) 1 + (c) 11 + (d) 10 = **36**.

## Por qué el repo necesita **dos** steps y no uno

La tentación es unificar `FIND-755` y `FIND-761` en un solo comando. Falla por
las dos puntas:

1. **Son dos superficies distintas, y la de la flag es un superconjunto.** El
   gate público (`cargo doc --workspace`, sin flags) **no procesa** los módulos
   privados ni los wrappers internos: por eso 21 de los 43 de `FIND-755` eran
   links a privados que ahí no resuelven, y por eso `lsm.rs:5` (módulo
   `pub(crate)`) **no aparece** en esa vista. A la inversa, la vista privada
   documenta ítems que el público descarta y **genera lints propios**.
2. **Si se unifica, una de las dos queda sin gate.** Elegir
   `--document-private-items` para todo obliga al gate público a cargar con 33
   lints internos que el lector de docs.rs nunca verá; elegir el público deja el
   artefacto publicado sin juzgar (lo que pasa hoy). Un gate que siempre está
   rojo se ignora — el argumento entero de `FIND-740`. La prueba de que la clase
   es distinta: **uno** de estos 36 es `private_intra_doc_links`
   (`src/ingestion.rs:46`), una clase que sólo existe cuando documentás lo
   privado; el resto de los "links a privado" del diff viven en módulos privados
   (`pub(crate) mod lsm`, `mod report_store;`), donde rustdoc ni procesa el doc
   y el lint no aplica.

Conclusión operativa: **el step público es el `required`** (es la superficie que
consume el usuario y la que `FIND-755` dejó en verde), y el privado entra como
**step nuevo del mismo job**, sin tocar el step del artefacto: comparte el árbol
caliente, así que el coste marginal es una segunda invocación de `cargo doc`.
Y si el step privado también se niega (recomendado ahora que los 36 están
cerrados), el artefacto publicado deja de ser "un tarball con warnings".

## Impacto mapeado (Regla 0)

- **Leídos completos:** cada región de doc con error (34 líneas `-->`) más el
  contexto necesario para decidir resolución vs aplanado: `src/executor.rs`
  (definición de `Executor::execute_plan`), `src/index/mod.rs` +
  `src/index_port.rs` (re-export de `IndexType`), `src/lsm.rs` (`SegmentRegistry`),
  `src/shred/mod.rs:263-360` (el `impl` y la cfg de `put`),
  `src/storage/engine/insert.rs:775-836` (par `rayon` / `not(rayon)`),
  `src/config.rs:2278-2345` (que el canal es `mpsc`, no `watch`),
  `vanta-memory/src/services/pipeline_worker.rs:213,380,795` (a qué tipo
  pertenece `run_context_assembly`), `vantadb-python/src/types.rs:19-46,365-372`
  (nombre Rust vs nombre Python), `vantadb-mcp/src/handlers/tools.rs:4520-4531`
  (la const local), `.github/workflows/docs-gates.yml:527-582` y
  `.github/workflows/ci-gate.yml:36-55`.
- **Referencias salientes:** ninguna. Son comentarios de documentación.
- **Referencias entrantes:** ninguna ejecutable. La única superficie que cambia
  es el HTML de rustdoc y el texto de los docstrings que PyO3 expone a Python.
- **Veredicto:** cero símbolos públicos nuevos, cero cambios de firma, cero
  cambios de comportamiento. Ningún ítem cambió de visibilidad. El binario no
  cambia: `cargo fmt --check` exit 0 y clippy del `pre-commit` exit 0.

## Alcance

24 ficheros de código + este task file, Backlog (1 fila) y `avance/activo/docs.md`.
**Sólo líneas de comentario** (`///`, `//!`); comprobado mecánicamente:

```
git diff -U0 -- src/ vantadb-python/ vantadb-mcp/ vanta-memory/ vantadb-wasm/ \
  | rg '^[+-][^+-]' | rg -v '^[+-]\s*(///|//!)'      # → vacío
```

| Fichero | Lints |
|---|---|
| `src/cache_warmer.rs` · `src/config.rs` · `src/governor.rs` · `src/index/diskann.rs` · `src/ingestion.rs` | 1 c/u |
| `src/llm.rs` | 4 |
| `src/lsm.rs` · `src/physical_plan/mod.rs` · `src/scalar_index.rs` · `src/sdk/api/sharing.rs` · `src/sdk/search/lexical.rs` | 1 c/u |
| `src/shred/mod.rs` | 2 |
| `src/storage/engine/{cache,coordinators,insert,mod}.rs` | 1 c/u |
| `vanta-memory/src/{services/pipeline_worker,context_engine/report_store}.rs` | 1 c/u |
| `vantadb-python/src/{lib,types}.rs` | 5 + 3 |
| `vantadb-mcp/src/{handlers/tools,handlers/initialize,temporal}.rs` | 1 + 1 + 3 |
| `vantadb-wasm/src/worker.rs` | 1 |

**No tocados (respetado):** `.github/**`, `.githooks/**`, `dev-tools/**`,
`docs/dev/workflow/**`, `docs/dev/plans/**`, `.opencode/**`,
`docs/dev/extraccion/**`, `src/bin/vanta-cli.rs`, `tests/cli_tests.rs`,
`src/wal.rs`, `src/wal_sharded.rs`, `src/schema.rs`,
`src/index/serialize/file.rs`.

**Regresión de `FIND-755` no reintroducida:** `vanta-memory/src/ingest/observability.rs`
(`note_backfill` → `derived_health`) no se tocó; `git diff` no lo incluye.

## Desviación declarada (1 fichero fuera del scope autorizado)

El encargo listaba como superficie escribible `src/**`, `vantadb-python/**`,
`vantadb-mcp/**` y `vanta-memory/**` — **`vantadb-wasm/**` no figuraba** (el
ticket contaba 33 lints en 4 crates y no lo incluía). La medición real tiene
**1 lint ahí**: `vantadb-wasm/src/worker.rs:7`, un fence `ignore` sobre un
diagrama de caja que rustc no puede parsear como Rust
(`invalid_rust_codeblocks`). Sin tocar esa línea el contrato (exit 0) es
**inalcanzable**, igual que en `FIND-755` con `vanta-memory`.

Se tocó **1 línea de comentario** (fence `ignore` → `text`),
declarada acá en vez de silenciada. Es el mismo patrón: la lista de ficheros del
ticket se derivó de una proyección, y una proyección no puede acotar la
superficie real.

## Verificación

```powershell
# 1. el contrato de esta fila (superficie privada / artefacto publicado)
$env:RUSTDOCFLAGS="-D warnings"
cargo doc --no-deps --workspace --all-features --document-private-items --keep-going

# 2. el gate de FIND-755 (superficie pública) — no debe haber regresión
Remove-Item Env:RUSTDOCFLAGS
$env:RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --workspace --keep-going

# 3. sólo doc tocada (debe imprimir nada)
git diff -U0 | rg '^[+-][^+-]' | rg -v '^[+-]\s*(///|//!)'

# 4. no se debilitó el bar (debe ser 0)
rg -n 'allow\(rustdoc' src/ vantadb-python/ vantadb-mcp/ vanta-memory/ vantadb-wasm/

# 5. fmt / clippy
cargo fmt --all -- --check
```

| Comando | Resultado |
|---|---|
| (1) contrato, `--all-features --document-private-items`, `-D warnings` | **ANTES exit 101 / 41 `^error`** (36 lints + 5 `could not document`) → **DESPUÉS exit 0 / 0** |
| (2) gate de `FIND-755`, sin `--document-private-items` | **exit 0 / 0** (sin regresión) |
| (3) `git diff -U0` filtrado a líneas no-doc | **vacío** |
| (4) `rg 'allow\(rustdoc'` en los 5 crates | **0** |
| (5) `cargo fmt --all -- --check` | **exit 0** |
| clippy del `pre-commit` (`-p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring --tests -j 2 -- -D warnings`) | **exit 0** (1m48s) |
| `pwsh dev-tools/ocr-review.ps1` (input del reviewer) | **exit 0**; el diff es 24 ficheros × (+46/−35) y el task file va excluido por `unsupported_ext` |

No se corrió la suite de tests: **cero líneas ejecutables** en el diff (es la
propiedad que verifica (3)). Sí se corrió el gate de docs dos veces, que es el
contrato real de la fila.

## Review (P2-01)

Tier **fast** por glob: todo el diff cae en "cualquier otro path" (doc comments
de `src/**`, `vantadb-python/**`, `vantadb-mcp/**`, `vanta-memory/**`,
`vantadb-wasm/**`, `docs/dev/**`); ningún path adversarial. Gate: verify mecánico
+ veredicto registrado.

- **Revisor:** `vanta-review` — contexto fresco, no implementó nada de esto, no
  editó ningún fichero (report-only).
- **Enfoque:** re-corrió **los dos** gates por su cuenta (privado y público);
  re-derivó por visibilidad/cfg los **11** aplanados de la clase (b)/(c) en vez
  de aceptar la tabla del autor; verificó contra el cuerpo que las 14
  resoluciones apunten al ítem que la prosa describe; comprobó que los 10 de (d)
  sean fixes y no aplanados; recompuso la aritmética por clase y por crate; y
  auditó el borrado del Backlog **por contenido**.
- **Cómo se probó (suyo):** privado `EXIT=0` / 0 `^error` / 0 `^warning` (1m33s)
  · público `EXIT=0/0/0` · `git diff` filtrado a no-doc **vacío** ·
  `allow(rustdoc` = **0** · `cargo fmt --all -- --check` = **0** · Backlog:
  `FIND-759` (`:828`) y `FIND-762` (`:834`) presentes, `FIND-761` **0 hits**,
  **0 inserciones**.
- **Ronda 1 → `changes-required`**: **0 Blocking en los 24 ficheros de código**
  (*"la superficie de rustdoc está cerrada de verdad y sin debilitar el bar"*),
  **1 Blocking de bookkeeping**, 3 Medium, 4 Low. Todos cerrados en el commit de
  cierre:

| # | Sev | Hallazgo | Cómo se cerró |
|---|---|---|---|
| B-1 | **Blocking** | se borraba la fila cuyo contrato tenía **tres** partes (step + `REQUIRED` + cerrar los lints) cumpliendo sólo la tercera: el cableado quedaba sin fila viva — el mismo fallo que la ronda 1 de `FIND-755` (M-2) | fila nueva **`FIND-763`** en el Backlog con el YAML exacto y la entrada en `REQUIRED`, citada desde §Incógnitas 1 |
| M-1 | Medium | §"el número del ticket" atribuía el 33 a una *proyección* y la referencia citada lo desmiente (`FIND-755.md:436-437` y `:309` lo dan por medido) | reescrito: **33 = medido sin `--all-features`**, 36 = con ella. El término sobrevive una sola vez (§Desviación, donde sí es defendible: el desglose por crate del ticket está incompleto) y se corrigió también en §Incógnitas 3 (R-1) |
| M-2 | Medium | se nombraban +5 causas para un delta de +3, y los tres −1 por crate quedaban sin explicar | eliminada la reconciliación crate a crate; se pegan las dos mediciones y **sólo** los dos sumandos probados (`llm`+`config` gateados, `vantadb-wasm` fuera de la lista) |
| M-3 | Medium | "(a) 14 lints / 13 líneas" contradecía su tabla, y atribuir 2 lints a `physical_plan/mod.rs:94` forzaba 37 | corregido a **14/14**; la aritmética real es **34 líneas `-->` + 2 diagnósticos sin `-->` = 36**, con el doble en `temporal.rs:376` |
| L-1 | Low | "20 ficheros" en prosa contra 24 reales | 24 |
| L-2 | Low | se nombraba una feature inexistente (`llm`) | `remote-inference`/`embed-local`, con `Cargo.toml:156` |
| L-3 | Low | "dos de estos 36 son `private_intra_doc_links`": sólo se sostiene **uno** | corregido a uno en §Por qué dos steps (el párrafo viejo duplicado se borró: R-2), con la razón de por qué los demás no aplican (módulos privados) |
| L-4 | Low | el contrato del ticket usa `rg`/`wc`, que no existen en pwsh | el bloque de medición muestra la forma `Select-String` que se corrió |
| Nit | — | los backticks nuevos **sí** aparecen en `help()` de Python | declarado en la fila de `vantadb-python` |
| Opt-1 | — | poner el step nuevo **antes** del artefacto (no comparten fingerprint) | incorporado al YAML de `FIND-763` |

- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas: los 8 comandos de §Verificación salieron de este
        árbol; el revisor re-corrió los 5 principales.
  - [x] No declarar done sin el contrato: exit 101/41 → 0/0, medido dos veces.
  - [x] No aplanar por comodidad: los 11 de (c) traen su prueba de
        inexistencia (§Clasificación) y el revisor los re-derivó uno por uno.
  - [x] No debilitar el bar: 0 `#[allow(rustdoc::*)]`, 0 `continue-on-error`.
  - [x] No cerrar la fila dejando trabajo huérfano: B-1 → `FIND-763`.
  - [x] Cobertura SDP: `SKILLS_CARGADAS` incluye `documentation-skill` (edita
        `.md` bajo `docs/`); el SDP no devolvió `pinned`.
- **Veredicto:** ronda 1 `changes-required` (bookkeeping, **0 en código**) →
  **corregido**; el contrato mecánico y la ausencia de debilitamiento quedaron
  verificados por el revisor **antes** de estas correcciones.
- **Ronda 1-bis — verificación de cierre** (misma sesión de review, sobre el
  commit `d9bb8f5f9`, por contenido con `git show HEAD:<path>`): **0 Blocking,
  0 Medium, 2 Low**. B-1/M-2/M-3/L-1/L-2/L-4/Nit confirmados cerrados; M-2
  además validado con un sumando que el autor no había identificado
  (`watch_config` tras `#[cfg(feature = "hot-reload")]`, `Cargo.toml:180`, fuera
  de `default` ⇒ explica el +1 de `vantadb`). Quedaban **R-1** (una segunda
  ocurrencia de "proyección" en §Incógnitas 3) y **R-2** (el párrafo viejo de
  L-3 convivía con el corregido), ambas **Low de texto**. Aplicadas en este
  commit de cierre, junto con la corrección de las dos celdas de la tabla que
  las daban por cerradas antes de estarlo. El revisor dejó el pre-compromiso
  explícito: *"Con R-1 y R-2 aplicados, la ronda 2 es `approve` sin más."*

## Incógnitas

1. **Cableado en CI: no aplicado, `.github/**` es del carril de CI.** Queda
   **registrado como fila `FIND-763`** en el Backlog (una incógnita no es un ítem
   de backlog: es el B-1 de la review). El step nuevo, exacto (mismo job
   `rustdoc`, `docs-gates.yml`, **sin tocar** el step del artefacto, que debe
   seguir subiendo el tarball), y **antes** de él: el árbol caliente de esta
   invocación es el del workspace **default**, que es el que reusa; el del
   artefacto es `--all-features` y no comparte fingerprint:

   ```yaml
   # docs-gates.yml, job `rustdoc`, step NUEVO (ANTES de "Generate rustdoc")
   - name: Deny rustdoc warnings (doc gate)
     env:
       RUSTDOCFLAGS: "-D warnings"
     run: cargo doc --no-deps --workspace --keep-going
   ```

   `env:` va **en el step**, no en el job: en el job contaminaría también el step
   del artefacto, que hoy corre sin negar (y ahí, con `-D warnings`, hoy también
   pasaría — ver 2).
   Segunda pieza, en `ci-gate.yml`: agregar `"Generate API reference (rustdoc)"`
   a `REQUIRED` (`:36-55`).
   **`FIND-719` sigue abierto:** *ningún job de `docs-gates` es required en
   `develop`* (los contexts requeridos son de Rust), así que el gate sólo muerde
   en el PR de release hasta que se toque esa lista — decisión del owner.
2. **El step del artefacto ya podría negar también.** Con los 36 cerrados,
   `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --all-features
   --document-private-items` sale **exit 0**. No se propone como `required`
   porque es la superficie *interna* y su coste (rocksdb/bindgen) ya lo paga el
   job; sí se propone que **no vuelva a quedar sin juzgar**: una de las dos
   líneas de arriba tiene que entrar.
3. **`FIND-755` y `FIND-761` miden distinto por diseño.** Cualquier intento
   futuro de "unificar" los dos comandos deja una superficie sin gate (§Por qué
   dos steps). El 33/4 del ticket se midió **sin** `--all-features` y **sin**
   `vantadb-wasm`; medido completo es **36/5**.
4. **`--all-features` sigue siendo el comando caro** (rocksdb + bindgen). El
   step público propuesto (sin `--all-features`) no lo paga; el del artefacto sí,
   y ya lo venía pagando.
5. **Es el único gate de rustdoc del repo.** `rg -n 'cargo doc' .github/workflows/`
   devuelve **3 líneas, todas del job `rustdoc`** de `docs-gates.yml` (`:558` el
   comando, `:581` el summary, `:585` un comentario del job hermano). No hay otro
   workflow que construya docs: si este job no mide, **nadie mide**.
6. **El aplanado de la clase (c) no pierde enlaces públicos** porque esos
   targets no existen en un build de doc; el de la clase (b) sí pierde **un**
   hipervínculo, y sólo en el artefacto privado. Declarado, no silenciado.
