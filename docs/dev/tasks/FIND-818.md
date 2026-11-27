---
title: "FIND-818: barrido de marcas `verificado <fecha>` — cierre (4 lotes, censo a 0)"
kind: task
status: completed
description: "Barrido completo de las marcas `verificado <fecha>` del Backlog: 4 lotes, 37 filas medidas contra HEAD (27 VIVA, 7 CAMBIADA, 3 retiradas), censo final a 0 celdas con marca fechada sin evidencia in-cell sobre 391 filas de tabla. Los tres modos de fallo del catálogo, con un ejemplo medido de cada uno."
---

# FIND-818: el barrido de marcas y su cierre

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-818`, línea 986)
- **Origen:** cierre de `FIND-811` (2026-11-27) · **Ver:** `FIND-811`, `FIND-296` (fantasma), `EGO-15`, `FIND-290`
- **Esfuerzo:** 4-8h (un lote) · **Prioridad:** 🟡 Media · **Tipo:** docs/gobernanza del backlog (medición pura, sin Rust)
- **Creado / cerrado:** 2026-11-27 (abierto y cerrado el mismo día, 4 lotes)
- **Estado:** ✅ COMPLETED
- **Lote:** 4 de 4 cerrado; **censo a 0** (0 celdas con marca fechada sin evidencia in-cell)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `docs/dev/Backlog.md` (27 filas re-escritas: 10 del lote 2 + 11 del lote 3 + 5 del lote 4, más la fila de `FIND-818` y la de `FIND-477` retiradas) · este task file · `docs/dev/avance/historial/backlog-history.md` (cierre + retirada) |
| Inmutable | `src/**` (`FIND-798` compilando) · `tests/**` · `vanta-memory/**` · `vantadb-mcp/**` · `.github/**` · `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md` · `docs/dev/feature-matrix.md` (`FIND-819` en vuelo) · `docs/dev/architecture/FEATURES.md` |
| Fuera de alcance | nada: el censo queda a 0 |

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

## Lote 4 — las 6 restantes (`2026-10-20`:5 · `2026-10-05`:1) (verificadas 2026-11-27)

| Fila | Veredicto | Evidencia (re-ejecutable) |
|------|-----------|---------------------------|
| `FIND-475` | **VIVA** (cita movida) | `Select-String .github/workflows/release-wheels.yml -Pattern "if: startsWith"` → `:181`, `:269`, ninguno con rama `workflow_dispatch`; los hermanos sí (`release-adapters.yml:4`, `:118`). La cita era `:179` |
| `FIND-476` | **VIVA** (y el doc existió) | `DIST-01` referenciado en `release-plz.toml:60,67` y `vanta-memory/Cargo.toml:12,15`; el doc **existió** (alta `f79c50551`, 2026-10-04) y lo borró `2cbe9fe9f` (2026-10-10) → referencias huérfanas, no rotas de nacimiento |
| `FIND-477` | **REFUTADA → retirada** | La fila afirmaba que las rutas citadas por el comentario "ya no existen". **Las dos existen hoy**: `crates/release_plz_core/src/version.rs` tiene el mapping literal (`matches!(diff.semver_check, SemverCheck::Incompatible(_))` → `VersionIncrement::breaking(self)`) y `crates/next_version/src/version_increment.rs` también, con `pub fn breaking` → `Minor` para 0.x (leídos 2026-11-27 de `raw.githubusercontent.com/release-plz/release-plz/main`). El comentario de `release-plz.toml:30-33` es correcto: no hay nada que arreglar |
| `FIND-478` | **VIVA** (cita movida) | `rg -n "git add -A" .github/workflows/release.yml` → `:144` (era `:131`) |
| `FIND-480` | **VIVA** (citas movidas) | `documented_only` en `release-manifest.json:84`, único `crates.io` = `vanta-memory` (`:86`), sin `vantadb-wasm`; `release-plz.toml:56` = `release = false` para wasm (era `:57`) |
| `FIND-284` | **CAMBIADA** (contrato) | `_batch_guard = self.purge_lock.write()` en `memory.rs:1542` dentro de `put_batch_inner` (`:1486`): el lock **sigue ancho y por decisión** (DUR-03, `:1533-1541`); angostarlo debe preservar el invariante anti-deadlock y medirse con `canonical_p99` |

**Retiradas: 1** (`FIND-477`), y el motivo es el modo de fallo nº 1: la fila daba por muertas dos rutas del upstream que siguen vivas. Las otras 5 no tenían nada roto salvo la cita.

## Cierre del barrido (4 lotes, 2026-11-27)

### Los números

| Lote | Fecha de las marcas | Filas | VIVA | CAMBIADA | Retiradas |
|------|--------------------|-------|------|----------|-----------|
| 1 (`FIND-811`) | `09-25`, `10-04`, `10-05` | 10 | 8 | 0 | **2** (`FIND-271`, `FIND-282`) |
| 2 | `2026-10-18` | 10 | 5 | 5 | 0 |
| 3 | `2026-11-22` | 11 | 10 | 1 | 0 |
| 4 | `2026-10-20` / `10-05` | 6 | 4 | 1 | **1** (`FIND-477`) |
| **Total** | | **37** | **27** | **7** | **3** |

(Los VIVA de los lotes 2-4 incluyen 8 "VIVA con cita corregida": la fila estaba viva y lo que estaba muerto era la línea o el path que citaba.)

### El censo final, medido con los predicados del gate

`REVERIFY_CLAIM` + `hasReverifyEvidence` (inlined de `check-harness-contract.mjs:314-325`) + el splitter de celdas (`:183`, `(?<!\\)\|`), aplicados **por celda sobre todas las filas de tabla** del Backlog (no solo las añadidas), sobre el estado **final** del archivo:

```
piped lines: 490 | separators: 100 | filed rows: 390
dated cells (REVERIFY_CLAIM): 27 | cells WITHOUT in-cell evidence: 0
```

**Censo a 0 para el predicado `verificado <fecha>`.** Las 27 celdas con marca fechada llevan comando o commit en la misma celda.

Dos precisiones que el número solo no dice:

- **Predicado acotado.** El gate juzga `verificad[oa]s?` + fecha; hay **6 celdas de otros carriles** con la forma `**Veredicto HEAD 2026-11-27: …**` que no dicen "verificado" y por tanto no entran en el censo (`DEAD-01`, `FIND-159`, `FIND-198`, `FIND-598`, `GTM-05`, `GTM-09`). Quedan anotadas como deuda, no como barridas.
- **El corte anterior era pre-retiro.** Durante el lote 4 el censo dio `492 piped lines / 29 dated cells`; al retirar `FIND-818` y `FIND-477` y limpiar la sección vacía, el estado final es `490 piped lines / 390 filed rows / 27 dated cells`. Contar líneas con pipe (que incluyen los 100 separadores `|---|`) como "filas" fue el error que la review marcó dos veces: la primera me hizo pasar de 19 a 18, la segunda de 492 a 390.

### Los tres modos de fallo, con un ejemplo medido de cada uno

1. **Falso** — nunca fue cierto, ni el día que se escribió. `FIND-271` citaba `docs/dev/architecture/CRASH_MODEL.md:75` y ese path **no existió en ningún commit**: `git cat-file -e` → exit 128 también en `b7b2b4c04`, el commit del día en que se escribió la fila. La marca midió la relectura de la fila anterior, no el repo.
2. **Podrido** — fue cierto y envejeció. `EGO-15` citaba `seed.rs:25`; `8f3b7039d` movió el `embedded.search` a `:31` **dos días antes** de que se escribiera la marca "re-verificado 2026-11-22". Lo mismo con las 8 citas desplazadas de los lotes 2-4 y con las 3 filas que aún citaban la spec `mgr-23-24-memoria-proyecto.md`, borrada el 2026-10-24.
3. **Ausente ≠ inexistente** — el error que casi cometí. `EGO-08` afirmaba una ausencia en `docs/dev/plans/`; mi primera redacción concluyó "el plan citado no existe" porque no estaba en `HEAD`, y la review lo refutó: `docs/dev/plans/` **como directorio** no existía en git el 2026-11-22 (`git ls-tree 4b8f72cd -- docs/dev/plans/` → vacío), pero el plan **sí existió** (`2026-10-08-ego-feedback-0.9.0.md`, alta `16fe0c0b0`, baja `4e831d3e1`, dos días de vida). Regla que queda: **ausencia en `HEAD` no es inexistencia; hay que preguntarle a `git ls-tree`/`git log --all` por el snapshot de la fecha.**

### La regla que sale del barrido

> Una marca `verificado <fecha>` sólo vale si **la misma celda** cita el comando o el commit con el que se comprobó, y si el comando se puede re-correr hoy. La fecha sin comando es una relectura; el `archivo:línea` sin comando es una cita que envejece sola; y "no está en `HEAD`" no es "no existió".

### Deuda residual (no bloquea el cierre)

- `ADR-0066:447` sigue citando `Backlog.md:801` como `FIND-296`, línea que nunca lo fue (`docs/dev/architecture/**` no era escribible en este carril).
- La segunda ruta de `FIND-477` (`next_version version_increment.rs`) quedó sin verificar: el crate no está en `release-plz/next_version` ni en `MarcoIeni/next_version`.
- El censo file-wide es un one-liner fuera del repo (`.opencode/` y `dev-tools/**` no eran escribibles aquí): si se quiere permanente, el vehículo es un check más en `check-harness-contract.mjs` (`reverify` ya cubre las filas **añadidas**; falta el barrido completo como comando).

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

## Verificación (cierre)

- `node dev-tools/check-harness-contract.mjs` → **harness contract holds (9 checks)**.
- `node dev-tools/check-harness-contract.mjs --self-test` → **89/89** (gate `reverify` sin tocar).
- **Censo file-wide** con los predicados del gate (inlined) y separadores excluidos: `490 piped lines / 100 separators / 390 filed rows / 27 dated cells / 0 without in-cell evidence`.
- `git diff --numstat -- docs/dev/Backlog.md` → **6 inserciones / 8 borrados**: 5 filas re-escritas + el retítulo de la sección, y 2 filas retiradas (`FIND-818`, `FIND-477`). El Backlog pasa de 1003 a 1001 líneas (−2 exacto).
- **Aislamiento del WIP ajeno:** el Backlog tenía 21 filas más re-escritas por otro carril (`Estado`). El commit se construyó con `HEAD + solo mis hunks` (reconstrucción línea a línea: 8 operaciones verificadas, 0 hunks ajenos), y los 21 pares del otro carril se devolvieron al worktree después con `git show HEAD:…` + reemplazo exacto. Verificado tras el restore: sus 21 ediciones presentes, las mías ya en el commit.
- `pwsh dev-tools/check-closure.ps1 -Diff` → 1 sola violación, **ajena**: `docs/dev/tasks/FIND-800.md` (otro carril, sin `**Estado:**`). La retirada de `FIND-818` y `FIND-477` **no** dispara la regla (b): ambas tienen entrada en `docs/dev/avance/historial/backlog-history.md`.
- Sin compilar Rust (medición pura): `rg`, `git cat-file`, `git log`, `git ls-tree`, `Get-ChildItem`, `Select-String`, `node`.
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
### Review ronda 3 — lote 4 y cierre (`vanta-review`, `ses_de1a395a9ffe342zCXiuS6FDAr`)

- **Veredicto ronda 3:** 🔴 **changes-required** (2 bloqueantes) → **ambos corregidos** antes del commit.
  - **B1 (bloqueante, corregido):** `FIND-477` no era CAMBIADA sino **REFUTADA → retirada**. La fila decía que las rutas citadas por `release-plz.toml` "ya no existen" y **las dos existen**: `version.rs` (con el mapping literal) y `crates/next_version/src/version_increment.rs` (con `pub fn breaking` → `Minor` para 0.x), ambas leídas del upstream `main` el 2026-11-27. Mi error fue declarar la segunda "sin verificar" tras dos 404 en el path raíz equivocado: el crate vive **dentro** del monorepo. Retirada con entrada en `backlog-history.md`; el lote 4 pasa a 4 VIVA / 1 CAMBIADA / 1 retirada y los totales a **27 / 7 / 3**.
  - **B2 (bloqueante, corregido):** el censo publicaba `table rows: 492` contando **líneas con pipe** (392 filas + 100 separadores) y en el estado **pre-retiro**. Es la misma confusión línea-vs-fila que la ronda 1 bloqueó, esta vez dentro del artefacto que declara cerrado el barrido. Re-medido con separadores excluidos y estado final: `490 piped lines / 100 separators / 390 filed rows / 27 dated cells / 0 without evidence`.
  - **N1–N5 (corregidos):** `description` alineada con la tabla (27/7/3) · `title` y H1 ya no dicen "lotes 2 y 3 / segundo lote" · Blast Radius con los 3 lotes y las 2 retiradas · §Deuda sin "quedan 6 marcas" · §Verificación reescrita con el estado final (y `check-closure` declarado **rojo por `FIND-800.md`, ajeno**, no "CLOSURE OK" como decía).
  - **N6 (corregido):** la sección vacía del Backlog se retitula a `Barrido de marcas verificado: cerrado (4 lotes, censo a 0; el gate queda en check-harness-contract)`, porque su título anterior ("Quedan 29 marcas…") era falso dentro del archivo que se declara barrido.
  - **N7 (anotado, no corregido):** el censo es del predicado `verificado <fecha>`; hay **6 celdas de otros carriles** con `**Veredicto HEAD 2026-11-27: …**` sin evidencia in-cell (`DEAD-01`, `FIND-159`, `FIND-198`, `FIND-598`, `GTM-05`, `GTM-09`). No entran en el censo y quedan como deuda declarada.
  - **Nits (corregidos los del carril):** las columnas `Archivo:línea` de las 5 VIVA se alinean con la prosa (`release-wheels.yml:181`, `release-plz.toml:67` + `vanta-memory/Cargo.toml:15`, `release.yml:144`, `release-manifest.json:84`). El defecto de `cells=10` con header de 9 en `Backlog.md:598-622` es pre-existente y ajeno: no se toca.
- **WIP ajeno declarado:** durante el lote 4 otro carril reescribía la columna `Estado` de 21 filas del Backlog (`⏸️ Diferida a v0.11.0` → `🟡 Pendiente (2026-11-27: sacada de v0.11.0…)`). El commit se construyó **sin** sus hunks (ver §Verificación) y se le devolvieron al worktree después.



## Deuda

- **Ninguna marca fechada queda sin barrer:** censo a 0 (28 celdas con marca, 0 sin evidencia in-cell). El lote 4 midió las últimas 6 (`10-20`:5 + `10-05`:1).
- **`EGO-15` no necesitó re-medición, sí verificación:** sus 6 citas (`pipeline.rs:71/:96/:101`, `expand.rs:13`, `public-api.txt:3151,3153`, `seed.rs:31`) resuelven exactas; la que estaba muerta (`seed.rs:25`) ya la había corregido el otro carril (`8f3b7039d`). `FIND-290` ídem: `lib.rs:45/:1164` y `reflection/mod.rs:245/:6/:351` exactas.
- **Punteros fantasma encontrados:** `FIND-296` (lote 2, citado por `MEMG-03` y por `ADR-0066:447` como `Backlog.md:801`, línea que nunca lo fue). El `Ver: plan EGO 2026-10-08` de `EGO-08` **no** era fantasma: existió (`16fe0c0b0`, borrado `4e831d3e1`), y la primera redacción de este archivo lo dio por inexistente — corregido por la review. Ninguno de los dos justificó retirar la fila que lo cita.
- **El lote 4 se hizo con delta mínimo** (recomendación de la review del lote 3): solo se reemplazó la cola `Verificado 2026-10-20: …` de cada celda, sin re-emitir la prosa. Fue la decisión correcta: 6 filas, 0 pipes sueltos, 0 retrabajo.
- **Tres cosas quedan fuera de este carril** (no bloquean el cierre): `ADR-0066:447` sigue citando `Backlog.md:801` como `FIND-296`; la segunda ruta de `FIND-477` (`next_version version_increment.rs`) sin verificar; y el censo file-wide como one-liner, que para ser permanente necesita un check en `dev-tools/check-harness-contract.mjs` (no escribible aquí).
