---
title: "FIND-818: barrido de marcas — lotes 2 y 3 (21 filas medidas contra HEAD)"
kind: task
status: in-progress
description: "Lotes 2-3 del barrido de marcas `verificado <fecha>`: 21 filas medidas contra HEAD (15 VIVA, 6 CAMBIADA, 0 retiradas). El modo de fallo no es 'falso' sino podrido: citas desplazadas, punteros fantasma y referencias a ficheros que se borraron. Quedan 6 marcas para el lote 4."
---

# FIND-818: segundo lote del barrido de marcas

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-818`, línea 986)
- **Origen:** cierre de `FIND-811` (2026-11-27) · **Ver:** `FIND-811`, `FIND-296` (fantasma), `EGO-15`, `FIND-290`
- **Esfuerzo:** 4-8h (un lote) · **Prioridad:** 🟡 Media · **Tipo:** docs/gobernanza del backlog (medición pura, sin Rust)
- **Creado / cerrado:** 2026-11-27
- **Estado:** ⏳ IN PROGRESS
- **Lote:** 3 de 4 cerrado; quedan 6 marcas fechadas

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `docs/dev/Backlog.md` (21 filas re-escritas: 10 del lote 2 + 11 del lote 3, más la fila de `FIND-818`) · este task file |
| Inmutable | `src/**` (`FIND-798` compilando) · `tests/**` · `vanta-memory/**` · `vantadb-mcp/**` · `.github/**` · `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md` · `docs/dev/feature-matrix.md` (`FIND-819` en vuelo) · `docs/dev/architecture/FEATURES.md` |
| Fuera de alcance | las 6 marcas restantes (`10-20`:5 · `10-05`:1 = `FIND-284`) |

## Premisa (medida, no re-derivada)

La fila afirma que **quedan 29 marcas `verificado <fecha>` sin barrer**. La premisa se mide con el regex del gate aplicado **por celda** (que es como lo aplica `check-harness-contract.mjs:314`, no por línea). Dos cortes, cada uno con su fecha:

```
# corte del lote 2 (HEAD de entonces, antes de reescribir nada)
HEAD: 28 filas de tabla con marca fechada (+1 línea que matchea pero NO es fila:
      el blockquote Backlog.md:85, "verificado contra HEAD el 2026-10-01")
      de esas 28, con fecha 2026-10-18 → 10
# corte del lote 3 (HEAD al empezar, antes de reescribir nada)
HEAD: 29 filas de tabla con marca fechada, 14 con veredicto 2026-11-27 previo,
      15 pendientes; de las pendientes, 11 son de 2026-11-22
# corte de hoy (worktree, tras los lotes 2 y 3)
29 filas con marca fechada · 23 con veredicto del 2026-11-27 · 6 pendientes
```

Los tres números conviven porque son tres cortes distintos: **29 líneas del lote 2 = 28 filas + 1 blockquote**, y las 29 filas de hoy incluyen las dos filas de `FIND-818`/`FIND-822` creadas después. Contar líneas en vez de filas fue el error que la review P2-01 cazó en el lote 2 (de ahí el "quedan 19" que eran 18).

## Cómo se filtró el incidente del 2026-11-26

`72643d9df` (borra 2365 paths) y `19f759980` (los restaura) aparecen en **cualquier** `git log -S` sobre cualquier path, y con ellos los dos de arreglo `290df23d0` y `db6ad7f79`. Sin filtrarlos, todo se fecha el 2026-11-26. Filtro aplicado a cada fecha de este lote:

```powershell
$inc = @('72643d9df','19f759980','290df23d0','db6ad7f79')
git log --format='%h %ad %s' --date=short --diff-filter=A -- <path> |
  Where-Object { $inc -notcontains ($_ -split ' ')[0] }
```

Medido, y es el hallazgo del lote: `git log -1 -- <ADR-0066.md>` devuelve **`db6ad7f79` (2026-11-26)** — el commit de arreglo del incidente, no el autor del ADR. Con filtro, el ADR lo añadió **`a67a39650`**, también del 2026-11-26 pero un commit real de contenido. Igual con la spec de MGR-23/24: `git log -1 -- <spec>` devuelve uno de los 4; con filtro, el alta es **`542b352a0` (2026-10-05)** y la baja **`7114ec20f` (2026-10-24)**, 24 días antes de la marca que la sigue citando como vigente.

## Lote 3 — las 11 de `2026-11-22` (verificadas 2026-11-27)

| Fila | Veredicto | Evidencia (re-ejecutable) |
|------|-----------|---------------------------|
| `MEMG-22` | **VIVA** | `rg -c "track_property_source" .` → 0 fuera de esta fila; `rg -c "property_source" src vanta-memory` → 0; `src/sdk/types/record.rs` existe (47 865 B) |
| `DIST-21` | **VIVA** | 20 workflows en `.github/workflows`, `rg -il "electron"` → 0; `rg -il "asar" vantadb-node/tests` → 0 (12 tests, ninguno Electron) |
| `EGO-07` | **CAMBIADA** | `embed_on_put` → 0 en `src/config.rs`; `embed-local` ausente en `vantadb-node/Cargo.toml` (la feature existe en `Cargo.toml:166`). Pero la cita `memory.rs:733` **nunca fue `put_one`**: `put_one` está en `:971` también en `4b8f72cd` (2026-11-22) y `:733` pertenece a `purge_expired_record_locked` (`:720`), mientras `get` está en `:1819`; el filtro real es `input.vector.as_ref().filter(Self::usable_vector)` (`:1062`, `:1218`) |
| `MEMG-25` | **VIVA** | `rg -c "MemoryScope" src/sdk/types vanta-memory` → 0; lo más cercano es `RecallScope` (`auto_recall.rs:288`) y `ScopedPersonaRecord` (`profile_sync.rs:75`): filtros, no el modelo multi-nivel |
| `MEMG-26` | **VIVA** | `MemoryCaptured` → 0, `event_id` → 0; 15 módulos en `vanta-memory/src/core`, ninguno es event log |
| `EGO-08` | **VIVA** (corregida en review) | `git ls-tree --name-only 4b8f72cd -- docs/dev/plans/` → 0 entradas: el directorio no existía en git el 2026-11-22, así que la ausencia de plan EGO es verificable (y vacua). `git ls-tree --name-only 4b8f72cd -- docs/` → 9 entradas: `docs/` sí estaba trackeado desde `354db146d` (2026-04-03). Hoy hay 3 planes, ninguno baseline EGO. El `Ver:` citaba un plan que **sí existió**: `docs/dev/plans/2026-10-08-ego-feedback-0.9.0.md`, alta `16fe0c0b0` (2026-10-08) y baja `4e831d3e1` (2026-10-10) |
| `EGO-10` | **VIVA** | `pub enum Value` en `types.rs:124` con 11 variantes, sin `Map`/`Array`/`Json` recursiva (`rg -c "Array(" ` → 0) |
| `EGO-15` | **VIVA** (verificada, no re-derivada) | default 128 (`expand.rs:13`, aplicado en `pipeline.rs:71`); setters `:96`/`:101`; `public-api.txt:3151,3153`; `seed.rs:31` = `embedded.search(request)`; `max_degree_per_node` en python/node/wasm → 0; `graphrag_search` solo namespace/query/query_vector/abort |
| `CORE-03` | **VIVA** | 69 ADRs; `rg -il "partici"` → 2 (ADR-0055:61, ADR-0057:233, prosa ajena); `CORE-03` en `docs/dev/architecture` → 0 |
| `FIND-165` | **VIVA** | `problem\+json` en `src` → 0; `src/server/errors.rs` (14 835 B) y `handlers.rs` (60 494 B) existen |
| `FIND-290` | **VIVA** (verificada, no re-derivada) | `perform_auto_recall_scored` importado en `lib.rs:45` e invocado en `:1164`; `reflect_session` `reflection/mod.rs:351`, llamadores solo en tests; `:245` y `:6` declaran la promotion fuera de scope |

**Retiradas: 0.** Otra vez el fallo es podrido y no falso: la única cita que **nunca** resolvió es `EGO-07` `memory.rs:733` (el mismo patrón que `FIND-271` en el lote 1). El caso instructivo del lote es `EGO-08`: su `Ver:` citaba un plan que **sí existió** (alta `16fe0c0b0`, baja `4e831d3e1`, dos días de vida) — la primera redacción de este archivo lo dio por inexistente y la review lo refutó con `git ls-tree`. Ausencia en `HEAD` ≠ nunca existió: es la lección del barrido aplicada al barrido.

## Lote 2 — las 10 de `2026-10-18` (verificadas 2026-11-27)

| Fila | Veredicto HEAD 2026-11-27 | Evidencia (re-ejecutable) |
|------|---------------------------|---------------------------|
| `MEMG-03` | **CAMBIADA** | L1 VIVO (`git cat-file -t e41fef7d` → commit; `reconcile_lineage_edges` `memory.rs:187`). El residual "L2/L3 sin linkage (FIND-296)" **quedó cerrado**: ADR-0066 aceptado (`a67a39650`) + `ensure_lineage_edge` `ontology/mod.rs:262`. `FIND-296` **nunca fue una fila** (`git show 29e9c12f:docs/dev/Backlog.md` → solo la cita dentro de MEMG-03): puntero fantasma desde su creación. Citas movidas: `mod.rs:77`→`:118`, `record.rs:213-215`→`:341` |
| `MEMG-05` | **VIVA** | `git cat-file -t 12ed4000` → commit (2026-10-05); ADR-0055 `status: accepted`; `merge_record` `src/sdk/api/memory.rs:2319`; `rg -c "fn merge_record" tests/memory_multi_writer.rs` → 11 tests de convergencia |
| `MEMG-09` | **CAMBIADA** | Slice v0 VIVO (`0905e3c5`, `vantadb-mcp/src/code_index.rs` 60 KB, tool `code_index:69`); `code_files` sigue stub (`code.rs:327`). La **spec citada ya no está en el árbol**: la borró `7114ec20f` (2026-10-24) — se lee con `git show 542b352a0:docs/dev/research/mgr-23-24-memoria-proyecto.md` |
| `FIND-320` | **VIVA** (citas corregidas) | `perform_auto_recall` `:391`, `_governed` `:402`, `_scored` `:424`; `rg -n "fn retrieve" vanta-memory/src` → 0; `trust_aware` invocado en `vantadb-node/src/lib.rs:1163`. `MGR-06` `:329` (decía 331), `MGR-18` `:357` (decía 359) |
| `EGO-13` | **VIVA** | `rg -c "Float32Array" vantadb-node/src` → 0; `get_f32_vec` `vantadb-node/src/lib.rs:1929`; `PyBuffer` `vantadb-python/src/lib.rs:988`, `as_slice(py)` `:1008`; `pyo3` en `Cargo.toml:15` sin feature `numpy`; `git cat-file -e HEAD:docs/dev/tasks/EGO-03.md` → exit 128 |
| `MGR-05` | **VIVA** (nombre corregido) | `Fellegi-Sunter` `src/entity/linking.rs:1,6`; el `mark_duplicate` del texto es `ManualLink` (`pub struct` `:322`, doc `:316`); `rg -c "auto_resolve_entities" src` → 0 |
| `MGR-23` | **CAMBIADA** | La spec existió: `git show 542b352a0:docs/dev/research/mgr-23-24-memoria-proyecto.md` → §2 en `:38`; fuera del árbol por `7114ec20f` (2026-10-24). `git cat-file -e HEAD:<spec>` → exit 128 |
| `MGR-24` | **CAMBIADA** | Ídem: §3 en `:64` del mismo blob histórico; borrada por `7114ec20f` (2026-10-24) |
| `H3-05` | **VIVA** (cita corregida) | `rg -c "campaign_diagnose_pipeline" .opencode/task-system/mcp/campaign-server.mjs` → 1, el comentario de retiro en `:1838`; `Get-ChildItem .opencode/task-system -Recurse -Filter campaign-server.mjs` → solo `task-system/mcp/`, así que la ruta citada (`:1729`) no resuelve; `validate`/`sandboxed`/`workflow` → 5 coincidencias, siguen registrados |
| `H3-08` | **CAMBIADA** | `validateReviewAccept` sigue exacto en `config/state-tools.mjs:124`; apareció el **verificador** de cierre (`dev-tools/check-closure.ps1`, H-04 del 2026-11-26), pero **no** el comando único que corra verify + review + progreso + learnings |

**Retiradas: 0.** Ninguna de las 10 describía código inexistente *al escribirse*: las tres que hoy citan una spec borrada eran ciertas el 2026-10-18 (la spec murió el 2026-10-24). El modo de fallo de este lote no es "falso", es **podrido**: 4 citas desplazadas, 1 puntero fantasma y 3 referencias a un archivo que ya no está.

## El detalle que importa

`MEMG-03` citaba `FIND-296` como dueño del residual. `FIND-296` **nunca existió como fila**: 0 coincidencias en HEAD y 0 en el snapshot `29e9c12f` (2026-10-18), donde la única aparición es la propia fila de `MEMG-03` que lo invoca; y ningún commit de la historia cambió jamás el conteo de esa forma de fila (`git log --all -S` de la fila `FIND-296` → 0 commits). Es el mismo patrón que `FIND-271` en el lote 1 (`CRASH_MODEL.md` inexistente), pero invertido: aquí el puntero fantasma no invalida la fila entera, invalida el **residual** — y el residual real lo dejó escrito `ADR-0066` §Relation (L1 batch-flushed sin linkage = D1b, procedencia a nivel mensaje diferida, query de linaje con scope para persona).

## Verificación

- `node dev-tools/check-harness-contract.mjs` → **harness contract holds (9 checks)**.
- `node dev-tools/check-harness-contract.mjs --self-test` → **89/89** (gate `reverify` sin tocar).
- `git diff --numstat -- docs/dev/Backlog.md` → **12 inserciones / 12 borrados** en el lote 3: las 11 filas de `2026-11-22` + la fila de `FIND-818`. Ninguna fila retirada.
- `pwsh dev-tools/check-closure.ps1 -Diff` → **CLOSURE OK**, pero por una razón que hay que declarar: el `-Diff` pasa porque **otro carril está implementando `FIND-822` en este mismo worktree** (`M dev-tools/check-closure.ps1`, 31/4, + `docs/dev/tasks/FIND-822.md` sin trackear). Contra `HEAD` esa regla sigue roja (lee un rewrite como retirada); con su fix en vuelo, ya no. No es mío y no lo toco.
- Sin compilar Rust (medición pura): `rg`, `git cat-file`, `git log`, `Get-ChildItem`, `Select-String`, `node`.
- Rama `develop`, sin `checkout`/`switch`/`worktree`.

## Review (P2-01, una ronda)

- **Tier:** **fast** (globs del diff: `docs/dev/**`; ningún path de la tabla adversarial).
- **Reviewer:** `vanta-review`, contexto fresco (`ses_de1d46b07ffeF3zJ3AiTKEMj39`), autor distinto.
- **Veredicto ronda 1:** 🔴 **changes-required** → **corregido y aceptado** en esta misma ronda.
  - **B1 (bloqueante, corregido):** la cuenta que titulaba la tarea decía **19**; la real es **18**. Causa medida: `29` contaba **líneas** que matchean el regex, y la línea 29 es el blockquote `Backlog.md:85` (`> … verificado contra HEAD el 2026-10-01`), no una fila. Con la semántica real del gate (por celda, `check-harness-contract.mjs:314`): **28 filas en `HEAD`**, 10 de ellas de `2026-10-18` → quedan **18**. Además mi desglose contaba dos veces `FIND-320` (una de las 10 barridas, que conserva su `Verificado 2026-10-21` histórico). Corregido en `Backlog.md:986` (título, Estado y cuerpo) y en este archivo (frontmatter, Metadata, Blast Radius, Premisa, Deuda).
  - **N1 (corregido):** la "Verificación" heredada de la fila (`rg -c 'verificado [0-9]{4}-'` baja a 0 filas) **no podía fallar**: cuenta líneas. Reemplazada por el contrato real y falsable, `node dev-tools/check-harness-contract.mjs` (check `reverify (FIND-811)`).
  - **N2 (corregido):** en `H3-05`, "el archivo se movió" era aserción (`.opencode/` no está trackeado: `git ls-files .opencode` → 0). Reemplazado por la medición `Get-ChildItem .opencode/task-system -Recurse -Filter campaign-server.mjs` → solo `task-system/mcp/`.
  - **N3 (aceptado sin cambio):** `MEMG-03` conserva en la misma celda las citas viejas `mod.rs:77` / `record.rs:213-215` porque están etiquetadas como la verificación del 2026-10-01; son históricas, no claims vigentes.
  - **N4 (registrado):** `ADR-0066:447` cita `Backlog.md:801` como `FIND-296`, y esa línea nunca lo fue; anotado en §Deuda para el lote 3.
- **Lo que la review reprodujo sin cambios:** los 10 veredictos del lote 2 uno por uno, 0 pipes sueltos (las 12 filas del diff con 11 pipes), `FIND-822` correcto (`Get-RemovedBacklogIds` `:148-159`), aislamiento del carril (`12/11` en el Backlog, nada ajeno).

### Review ronda 2 — lote 3 (`vanta-review`, `ses_de1b5af4effeYTw64ECivCYZzz`)

- **Veredicto ronda 2:** 🔴 **changes-required** (3 bloqueantes) → **los tres corregidos** antes del commit.
  - **B1 (bloqueante, corregido):** `EGO-08` estaba clasificada **CAMBIADA** con la premisa *"`docs/` no está trackeado antes de `b0d740e32`"*, que es **falsa**: `git ls-tree 4b8f72cd -- docs/` → 9 entradas, `docs/` trackeado desde `354db146d` (2026-04-03). Lo que no existía el 2026-11-22 es `docs/dev/plans/` como directorio (`git ls-tree 4b8f72cd -- docs/dev/plans/` → vacío), así que la ausencia de plan EGO ese día es **verificable y vacua**. Veredicto corregido a **VIVA**; el lote 3 pasa a **10 VIVA + 1 CAMBIADA** y los totales a **15 / 6**.
  - **B2 (bloqueante, corregido):** mi "corrección" del `Ver:` de `EGO-08` cambiaba un puntero real por un fantasma. El plan **existió**: `docs/dev/plans/2026-10-08-ego-feedback-0.9.0.md`, alta `16fe0c0b0` (2026-10-08), baja `4e831d3e1` (2026-10-10). Ahora la columna cita el archivo y sus dos commits.
  - **B3 (bloqueante, corregido):** la evidencia de `EGO-07` tenía dos cláusulas falsas — `rg -l "ort::" src` da **33** archivos por `IndexPort::`/`Sort::` (la buena es `rg -l '\bort::' src` → 2: `llm.rs`, `embedding_health.rs`), y `:733` **no** es `get` (es `purge_expired_record_locked`, `:720`; `get` está en `:1819`). Corregidas ambas; el veredicto CAMBIADA se sostiene por la cita muerta de `put_one`.
  - **N1–N7 (corregidos salvo N7):** bullet duplicado del gate hole borrado · Blast Radius a 21 filas · `title` alineado a lotes 2-3 · Premisa reconciliada en tres cortes fechados (28 filas del lote 2 / 29-14-15 del lote 3 / 29-23-6 de hoy) · medición `'verificado 2026-11-22'` anotada como **pre-rewrite** · `MEMG-25` con la ruta completa (`vanta-memory/src/core/profile/profile_sync.rs:75`). **N7 no se corrige porque no es mío:** `M dev-tools/check-closure.ps1` es el fix de `FIND-822` que otro carril tiene en vuelo en este mismo worktree; este carril no lo tocó (ver §Verificación).
- **Lo que la review confirmó:** el censo 29/23/6 exacto y con los 6 pendientes nominales, 8 de las 9 VIVA sólidas una por una, `EGO-15` bien declarada como verificada-sin-re-derivar, y el gate verde (9 checks / 89 fixtures) re-corrido por el reviewer.



## Deuda

- **6 marcas fechadas quedan sin barrer** (lote 4): `10-20`:5 (`FIND-475`, `FIND-476`, `FIND-477`, `FIND-478`, `FIND-480`) + `10-05`:1 (`FIND-284`). Censo por celda con el regex del gate: 29 filas de tabla con marca fechada, 23 con veredicto del 2026-11-27, 6 pendientes. Ojo con el criterio: `FIND-320` y las 8 del lote 1 no dicen literalmente `verificado 2026-11-27` (dicen `Veredicto HEAD 2026-11-27` / `VIVA HEAD 2026-11-27`), así que un conteo por la cadena literal las cuenta como pendientes y **no lo están**.
- **`EGO-15` no necesitó re-medición, sí verificación:** sus 6 citas (`pipeline.rs:71/:96/:101`, `expand.rs:13`, `public-api.txt:3151,3153`, `seed.rs:31`) resuelven exactas; la que estaba muerta (`seed.rs:25`) ya la había corregido el otro carril (`8f3b7039d`). `FIND-290` ídem: `lib.rs:45/:1164` y `reflection/mod.rs:245/:6/:351` exactas.
- **Punteros fantasma encontrados hasta ahora:** `FIND-296` (lote 2, citado por `MEMG-03` y por `ADR-0066:447` como `Backlog.md:801`, línea que nunca lo fue). El `Ver: plan EGO 2026-10-08` de `EGO-08` **no** era fantasma: existió (`16fe0c0b0`, borrado `4e831d3e1`), y la primera redacción de este archivo lo dio por inexistente — corregido por la review. Ninguno de los dos justifica retirar la fila que lo cita.
- **`docs/` sí está trackeado desde 2026-04-03** (`354db146d`); lo que faltaba el 2026-11-22 era `docs/dev/plans/` como directorio (`git ls-tree 4b8f72cd -- docs/dev/plans/` → vacío). Cualquier "no existe" del barrido debe distinguir *no está en `HEAD`* de *no existió nunca*: es el tercer modo de fallo, después de falso y podrido.
- **Para el lote 4 (6 filas):** la review recomienda delta mínimo en vez de re-emitir la celda (`**Veredicto HEAD …: VIVA**` + `Medición:` al frente, sin tocar la prosa): re-emitir fue lo que arrastró los N del lote 3. Y el censo 29/23/6 depende de un one-liner que no vive en el repo (`dev-tools/**` es inmutable en este carril): si se quiere reproducible, va como `FIND` propio.
