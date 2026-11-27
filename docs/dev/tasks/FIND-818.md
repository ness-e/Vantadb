---
title: "FIND-818: segundo lote del barrido — las 10 marcas de 2026-10-18 medidas contra HEAD"
kind: task
status: in-progress
description: "Segundo lote del barrido de marcas `verificado <fecha>`: las 10 filas de 2026-10-18 medidas contra HEAD — 5 VIVA (4 con cita corregida) y 5 CAMBIADA (3 citan una spec borrada del árbol, 1 un `FIND-296` que nunca fue fila, 1 un contrato a medias). 0 retiradas. Quedan 18 marcas para el lote 3."
---

# FIND-818: segundo lote del barrido de marcas

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-818`, línea 986)
- **Origen:** cierre de `FIND-811` (2026-11-27) · **Ver:** `FIND-811`, `FIND-296` (fantasma), `EGO-15`, `FIND-290`
- **Esfuerzo:** 4-8h (un lote) · **Prioridad:** 🟡 Media · **Tipo:** docs/gobernanza del backlog (medición pura, sin Rust)
- **Creado / cerrado:** 2026-11-27
- **Estado:** ⏳ IN PROGRESS
- **Lote:** 2 de 4 cerrado; quedan 18 marcas fechadas

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `docs/dev/Backlog.md` (10 filas re-escritas + la fila de `FIND-818`) · este task file |
| Inmutable | `src/**` (`FIND-798` compilando) · `tests/**` · `vanta-memory/**` · `vantadb-mcp/**` · `.github/**` · `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md` · `docs/dev/feature-matrix.md` (`FIND-819` en vuelo) · `docs/dev/architecture/FEATURES.md` |
| Fuera de alcance | las 18 marcas restantes (`11-22`:11 · `10-20`:5 · `10-05`:1 · `11-27`:1) |

## Premisa (medida, no re-derivada)

La fila afirma que **quedan 29 marcas `verificado <fecha>` sin barrer**. Medido contra `HEAD`, con el regex del gate aplicado **por celda** (que es como lo aplica `check-harness-contract.mjs:314`, no por línea):

```
$ node -e '<censo: splitCells por fila de tabla + REVERIFY_CLAIM por celda>'
  → HEAD: 28 filas de tabla con marca fechada (+1 línea que matchea pero NO es fila:
    el blockquote Backlog.md:85, "verificado contra HEAD el 2026-10-01")
  → de esas 28, con fecha 2026-10-18 → 10 (el lote de esta pasada)
```

La premisa se corrige en la forma, no en el fondo: **29 líneas = 28 filas de tabla + 1 blockquote**, y de las 28 filas, 10 son de `2026-10-18`. Contar líneas (29) en vez de filas (28) fue exactamente el error que la review P2-01 cazó en la primera redacción de este archivo, y de ahí salía el "quedan 19" cuando son **18**.

## Cómo se filtró el incidente del 2026-11-26

`72643d9df` (borra 2365 paths) y `19f759980` (los restaura) aparecen en **cualquier** `git log -S` sobre cualquier path, y con ellos los dos de arreglo `290df23d0` y `db6ad7f79`. Sin filtrarlos, todo se fecha el 2026-11-26. Filtro aplicado a cada fecha de este lote:

```powershell
$inc = @('72643d9df','19f759980','290df23d0','db6ad7f79')
git log --format='%h %ad %s' --date=short --diff-filter=A -- <path> |
  Where-Object { $inc -notcontains ($_ -split ' ')[0] }
```

Medido, y es el hallazgo del lote: `git log -1 -- <ADR-0066.md>` devuelve **`db6ad7f79` (2026-11-26)** — el commit de arreglo del incidente, no el autor del ADR. Con filtro, el ADR lo añadió **`a67a39650`**, también del 2026-11-26 pero un commit real de contenido. Igual con la spec de MGR-23/24: `git log -1 -- <spec>` devuelve uno de los 4; con filtro, el alta es **`542b352a0` (2026-10-05)** y la baja **`7114ec20f` (2026-10-24)**, 24 días antes de la marca que la sigue citando como vigente.

## Tabla de las 10 verificadas (todas de 2026-10-18)

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
- `git diff --numstat -- docs/dev/Backlog.md` → **12 inserciones / 11 borrados**: 11 filas re-escritas (las 10 del lote + la fila de `FIND-818`) y 1 fila nueva (`FIND-822`). Ninguna fila retirada; el total de filas de datos no baja.
- `pwsh dev-tools/check-closure.ps1 -Diff` → **rojo declarado y por causa medida**: 1 por `FIND-798.md` (otro carril, sin `**Estado:**`) y 5 por el agujero que registra `FIND-822` (rewrite leído como retirada). Ninguno es una retirada real de este lote.
- Sin compilar Rust (medición pura): `rg`, `git cat-file`, `git log`, `Select-String`, `node -e`.
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
- **Lo que la review reprodujo sin cambios:** los 10 veredictos uno por uno, 0 pipes sueltos (las 12 filas del diff con 11 pipes), `FIND-822` correcto (`Get-RemovedBacklogIds` `:148-159`), aislamiento del carril (`12/11` en el Backlog, nada ajeno).



## Deuda

- **18 marcas fechadas quedan sin barrer**, medidas por celda con el regex del gate sobre filas de tabla: `11-22`:11 · `10-20`:5 · `10-05`:1 · `11-27`:1 (`FIND-215`, ya con veredicto propio). `EGO-15` y `FIND-290` siguen entrando ahí: `EGO-15` ya se sabe que citaba una línea muerta (`seed.rs:25` → `:31` desde `8f3b7039d`).
- **Evidencia extra del fantasma (N4 de la review):** `ADR-0066:447` documenta `FIND-296` como "`docs/dev/Backlog.md:801`", y esa línea **nunca** fue `FIND-296`: en `HEAD` es `## Media 2026-11-26 …` y en `29e9c12f` es un `> CIERRE 2026-08-31 …`. El puntero está roto también en el ADR que lo cita: el lote 3 debería corregirlo ahí.
- El lote 3 debe ser el de `2026-11-22` (11 filas), el más grande y el más reciente: la mitad se escribió después de que `docs/dev/research/` se vaciara (`7114ec20f`, 2026-10-24), así que la apuesta es que ahí hay más citas podridas que falsas.
- **Gate hole registrado:** `check-closure -Diff` lee todo rewrite como retirada (`Get-RemovedBacklogIds`, `check-closure.ps1:148-159` no resta los IDs que sobreviven) y pide entrada en `docs/dev/avance/` por filas que siguen en el Backlog. Registrado como `FIND-822`; el archivo es de otro dueño y este carril no puede tocarlo.
