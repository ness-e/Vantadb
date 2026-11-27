---
title: "FIND-797: el `paths:` de la lane CERT se deriva de los crates que sus fases ejecutan"
kind: task
status: in-progress
description: "`snapshot-certification.yml` no disparó en el push de FIND-783/FIND-782 porque su `paths:` enumeraba a mano rutas que no incluyen los crates que sus 7 fases corren. Se deriva la lista de los `-p` del YAML, se añade un chequeo que falla si una fase gana un crate sin su filtro, y se mide el coste: 4/20 pushes disparaban antes, 8/20 después."
---

# FIND-797: la lane certifica un árbol que su filtro no mira

## Metadata
- **Fuente:** `docs/dev/Backlog.md` — `Alta 2026-11-27` (fila `FIND-797`)
- **Origen:** monitor del push `66404bedc` → **Ver:** `FIND-752`, `FIND-791`, `FIND-783`, `FIND-782`, `FIND-794`
- **Esfuerzo:** 1-2h · **Prioridad:** Alta · **Tipo:** CI/tooling (sin Rust)
- **Creado:** 2026-11-27 · **Estado:** ⏳ IN PROGRESS

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `.github/workflows/snapshot-certification.yml` (los dos `paths:` + un step nuevo) · `dev-tools/check-cert-lane-paths.mjs` (nuevo) · `docs/dev/tasks/FIND-797.md` · `docs/dev/Backlog.md` (sólo la fila) · `docs/dev/avance/activo/ci-cd.md` |
| Inmutable | `src/**`, `vanta-memory/**`, `vantadb-mcp/**` (sólo se **nombran** en `paths:`, no se escriben) · `dev-tools/git-commit-isolated.ps1` · `dev-tools/check-harness-contract.mjs` (cerrado en `FIND-794`, `81/81`) · `ci-rust.yml` (de `FIND-807`) · `certification.yml` · `release-verify.yml` · `.opencode/**` · `docs/dev/plans/**` |
| Fuera de alcance | Que `CERT` sea **required** (`FIND-791`), que las 214 ejecuciones se ejecuten siempre, y `vantadb-server` (no lo corre ninguna fase) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/snapshot-certification.yml` (259 L — `on.push.paths`, `on.pull_request.paths`, la función `run_phase`, las 7 invocaciones) · `dev-tools/check-harness-contract.mjs` (§1-70: cabecera, contrato de exit, forma de los checks; **no** se toca) · `.githooks/pre-commit-config.yaml` (85 L — gates que corren en el commit) · `.githooks/pre-commit` (§1-80) · fila `FIND-797` del Backlog · `docs/dev/tasks/FIND-794.md` (precedente y formato) · `tests/core/snapshot_certification.rs` (imports) · `vanta-memory/tests/{backup_snapshot,rollback}.rs` (imports) · `vantadb-mcp/tests/mcp_tests.rs` (imports) · `docs/dev/avance/activo/ci-cd.md` (§ formato de sección).
- **Referencias hacia dentro del workflow:** lo dispara GitHub por `paths:` (push a `main`/`develop` y PR contra ellas) y a mano por `workflow_dispatch`. No lo importa ningún script; ningún otro workflow lo referencia por nombre. Los `run_phase` son locales al step.
- **Referencias hacia dentro del step nuevo:** sólo `node` del runner (presente en `ubuntu-latest`) y el checkout ya hecho. No añade acciones, no añade secrets, no añade permisos.
- **Referencias salientes del filtro nuevo:** los globs apuntan a `src/**`, `tests/**`, `vanta-memory/src/**`, `vantadb-mcp/src/**` — árboles propiedad de otros roles; aquí sólo se declaran como **entradas de filtro**, no se editan.
- **Veredicto de impacto:** bajo y confinado a `CI + dev-tools`. El riesgo real no es el diff: es (a) encender la lane en pushes donde hoy no corre, y (b) que el chequeo nuevo se compare contra una lista escrita a mano en otro sitio. (a) se mide abajo con los 20 pushes reales; (b) se evita derivando **los dos lados** del mismo fichero (fases y filtro viven en el mismo YAML; el script no lleva ninguna lista de crates).

## Premisa (medida, no re-derivada)

La fila afirma: el push `66404bedc` **no** produjo run de `CERT` y su diff tocaba `vanta-memory/src/ingest/runner_config.rs`, cuatro `vantadb-mcp/src/*.rs` y `vantadb-server/src/scheduler.rs`. Re-medido con el rango real del push (`gh api repos/ness-e/Vantadb/activity` → evento `push` a `refs/heads/develop` con `before=dca7d5818`, `after=66404bedc`) y salida literal:

```
$ (git diff --name-only dca7d5818..66404bedc).Count      -> 31
$ git diff --name-only dca7d5818..66404bedc | rg '^(src|vanta-memory|vantadb-mcp|tests)/'
vanta-memory/src/ingest/runner_config.rs
vantadb-mcp/src/memory_scheduler.rs
vantadb-mcp/src/proxy.rs
vantadb-mcp/src/threads.rs
vantadb-mcp/src/wiki.rs
vantadb-mcp/tests/find782_probe_deadline.rs
vantadb-mcp/tests/mcp_proxy_reconnect.rs
$ gh api ".../snapshot-certification.yml/runs?head_sha=66404bedc" --jq '.total_count'
0
```

✅ **La premisa se sostiene** en lo que importa: 0 runs para ese SHA, y las dos crates que ese push tocó (`vanta-memory`, `vantadb-mcp`) son las que corren las fases 06 y 07. ⚠️ **Corregido por el review (R3):** la primera redacción de este bloque citaba `997c41c62..66404bedc` (que da **2** ficheros, no 31) y listaba `vanta-memory/src/services/scheduler.rs` / `vantadb-server/src/scheduler.rs`, que **no** están en este rango, omitiendo los dos `vantadb-mcp/tests/*.rs` que sí están. El rango correcto es el de arriba; el `vantadb-server/src/scheduler.rs` pertenece al push **posterior** (`66404bedc..997c41c62`), y `vantadb-server` no lo corre ninguna fase.

## Lo que las 7 fases ejecutan (leído del YAML, no de una enumeración)

| Fase | Línea | Comando (tal cual) | Crate | Target |
|---|---|---|---|---|
| `01-unit-plain` | `:256-257` | `run_phase 01-unit-plain scoped 'test(snapshot) or test(pending_restore)' -p vantadb --lib --features failpoints` | `vantadb` | `--lib` |
| `02-unit-encrypted` | `:258-259` | idem `--features failpoints,encryption` | `vantadb` | `--lib` |
| `03-certification-plain` | `:261-262` | `coverage=full`, `-p vantadb --test snapshot_certification --features failpoints` | `vantadb` | `--test snapshot_certification` |
| `04-certification-encrypted` | `:263-264` | idem `+encryption` | `vantadb` | `--test snapshot_certification` |
| `05-rocksdb-native` | `:265-266` | `-p vantadb --test snapshot_certification --features failpoints,encryption,rocksdb` | `vantadb` | `--test snapshot_certification` |
| `06-memory-snapshots` | `:267-268` | `-p vanta-memory --test backup_snapshot --test rollback` | `vanta-memory` | `--test backup_snapshot`, `--test rollback` |
| `07-mcp-snapshots` | `:269-270` | `-p vantadb-mcp --test mcp_tests` | `vantadb-mcp` | `--test mcp_tests` |

(Líneas del fichero **entregado**; en `HEAD` eran `:236-250`.)

**Crates: 3** — `vantadb` (raíz, 5 fases), `vanta-memory` (1), `vantadb-mcp` (1). `vantadb-server` **no** lo corre ninguna fase: sus cambios no entran en la derivación (coste sin certificación).

**El subconjunto "que la fase toca" no se puede justificar con el selector, y se midió antes de escribirlo:**

- `--lib 'test(snapshot) or test(pending_restore)'` selecciona por **nombre de test**: los 11 `fn` que matchean viven en `backend.rs`, `backends/*`, `migration.rs`, `metrics/core/mod.rs`, `sdk/version_history.rs`, `error.rs`, `accumulator.rs` — pero el test ejercita el camino de snapshot completo (`storage/engine/**`, `wal*`, `config`), así que "sólo los módulos con test en su nombre" es la población equivocada.
- `--test snapshot_certification` es `coverage: full` (todo el binario) y sus imports son `vantadb::{index,node,sdk,storage,wal,config}` (`tests/core/snapshot_certification.rs:9-16`): la clausura transitiva es el crate entero.
- `--test mcp_tests` importa `vantadb_mcp::*` (`vantadb-mcp/tests/mcp_tests.rs:9`): el crate entero.
- `--test backup_snapshot|rollback` importan `vanta_memory::core::{abstractions,record}` + `vanta_memory::utils::backup`: la clausura excede esa lista (error, storage, ingest → `runner_config.rs`, que es exactamente el fichero que no disparó).

⇒ **Cada crate aparece ⇒ su `Cargo.toml` + todo su `src/**`.** Sin excepciones justificadamente recortadas.

## Decisión

1. **`paths:` derivado** en `on.push` y `on.pull_request` (las dos listas, idénticas): raíz `src/**`, `tests/**`, `build.rs`; `vanta-memory/{Cargo.toml,src/**,tests/backup_snapshot.rs,tests/rollback.rs}`; `vantadb-mcp/{Cargo.toml,src/**,tests/mcp_tests.rs}`; más las entradas de build que ya estaban (workflow, `rust-setup/**`, `.cargo/**`, `.config/nextest.toml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`) y el propio chequeo.
2. **Chequeo de divergencia `dev-tools/check-cert-lane-paths.mjs`**, externo (no se toca `check-harness-contract.mjs`, cerrado en `FIND-794`). Deriva **los dos lados del mismo fichero** y aplica **cinco reglas**, todas con su mutación medida:
   - **R-crates:** por cada `-p`/`--package` de cada `run_phase` ⇒ su `Cargo.toml`, su `build.rs` **si existe** (compila para todos los targets) y todo su `src/**`, en **cada** bloque `paths:`.
   - **R-targets:** el fichero real de cada `--test <name>` (o `--test=<name>`) debe estar cubierto por algún glob.
   - **R-simetría:** ningún bloque puede dejar de ver una entrada que otro bloque ve (un push que sólo toque esa entrada dejaría de disparar sin aviso).
   - **R-autorreferencia:** cada bloque debe listar **este** workflow y **este** script, derivados de sus rutas reales (si `--workflow` apunta a una copia fuera del repo, se usa la ruta canónica de la lane). Sin esto, borrar la entrada auto-listada apaga el gate y nada falla — era el agujero del primer intento (review R1).
   - **R-forma:** un `paths:` que el lector no puede convertir en bloque (flow style, reindentado) se cuenta y se reporta; antes desaparecía del audit en silencio.
   **Cero listas escritas a mano en el script** (pre-mortem b): no hay ningún crate ni ruta literal más allá del nombre del fichero de la lane.
3. **El chequeo corre como primer `run:` step de la lane**, justo después del checkout y antes de `rust-setup`. No es circular: editar este YAML **siempre** dispara la lane (el fichero está en su propio `paths:`, y **eso** lo verifica R-autorreferencia), así que el chequeo se ejecuta exactamente sobre la mutación que abre el hueco y muere en segundos, antes de las fases.
4. **Coste medido, no estimado** — §Medición. No se enciende nada más: `4/20 → 8/20` (×2, no ×5), así que **no** se separa en dos lanes.

## Medición del coste (los 20 últimos pushes, rangos reales)

Rangos tomados de `gh api repos/ness-e/Vantadb/activity` (eventos `push` a `refs/heads/develop`), diff con `git diff --name-only <before>..<after>`, y semántica de glob de GitHub (`*` no cruza `/`, `**` sí) aplicada a ambos filtros.

| | Dispara | No dispara |
|---|---|---|
| `paths:` actual | **4 / 20** | 16 |
| `paths:` derivado | **8 / 20** | 12 |

`both=4`, `onlyNew=4`, `onlyOld=0`. **×2**, no ×5 ⇒ no hace falta separar la lane. Los 4 que se suman:

| Push | Fichero que enciende | ¿Debe certificarse? |
|---|---|---|
| `0ef9160de` | `vanta-memory/src/services/scheduler.rs` | sí — crate de la fase 06 |
| `997c41c62` | `vantadb-mcp/src/proxy.rs` | sí — crate de la fase 07 |
| `66404bedc` | `vanta-memory/src/ingest/runner_config.rs`, `vantadb-mcp/src/{proxy,memory_scheduler}.rs` | sí — **es el push que la fila mide** |
| `2f8ea4d76` | `src/cli_handlers/*.rs` | sí — crate raíz de las fases 01-05 |

`onlyOld=0` ⇒ ningún push que hoy dispara deja de disparar (la lista nueva es **superconjunto** de la vieja).

## Steps

1. ✅ Medir la premisa y los crates por fase (§Premisa, §Lo que las 7 fases ejecutan).
2. ✅ Medir el coste antes/después sobre 20 pushes reales (§Medición).
3. ✅ Escribir `dev-tools/check-cert-lane-paths.mjs` con `--self-test` en las dos direcciones.
4. ✅ Reescribir los dos `paths:` por la lista derivada y añadir el step del chequeo.
5. ✅ Verificar: chequeo rojo contra `HEAD` y verde contra el árbol; self-test; `actionlint` con y sin shellcheck; prettier; docs gates.
6. ✅ Backlog (borrar la fila) + `avance/activo/ci-cd.md` + commit con `-ExpectAbsent`.

## Resultados (ejecutados)

**Lista nueva de `paths:`** (idéntica en `on.push` y en `on.pull_request`, **18** entradas; la de `HEAD` tenía 24):

```
.github/workflows/snapshot-certification.yml   .github/actions/rust-setup/**   .cargo/**
.config/nextest.toml   Cargo.toml   Cargo.lock   rust-toolchain.toml   build.rs
dev-tools/check-cert-lane-paths.mjs
src/**        tests/**
vanta-memory/Cargo.toml   vanta-memory/src/**   vanta-memory/tests/backup_snapshot.rs
                                                vanta-memory/tests/rollback.rs
vantadb-mcp/Cargo.toml    vantadb-mcp/src/**    vantadb-mcp/tests/mcp_tests.rs
```

**1. El chequeo, contra el árbol y contra `HEAD` (salida pegada).**

| Entrada | Resultado | Exit |
|---|---|---|
| árbol de trabajo (lista nueva) | `ok cert-lane-paths (FIND-797): 7 phase(s), 2 paths block(s), crates: vanta-memory, vantadb, vantadb-mcp` | **0** |
| `git show HEAD:…` → `--workflow` (lista vieja, sin tocar nada más) | **26 violaciones**: `src/**` ×5 fases ×2 bloques (**10**) · `build.rs` ×5 ×2 (**10**) · `vanta-memory/src/**` ×2 · `vantadb-mcp/src/**` ×2 · el script sin listar ×2 | **1** |

**2. `--self-test`: 4/4**, fixtures en el propio script: `green` 0 · `crate añadido a la fase` **6** (3 entradas × 2 bloques) · `src/**` borrado de un bloque **2** (1 derivada + 1 asimetría) · `la lane no se vigila a sí misma` **2**.

**3. Mutaciones del review, cada una con su rojo** (sobre copias, `--workflow`):

| Mutación | Violaciones | Qué la caza |
|---|---|---|
| (R1) se borra `- ".github/workflows/snapshot-certification.yml"` de los dos bloques | **2** | R-autorreferencia |
| (R5) se borra `- "Cargo.lock"` del bloque `push` | **1** | R-simetría |
| (R2) `-p vantadb -p vantadb-server` en una fase existente | **6** | R-crates (`Cargo.toml`, `src/**`, `build.rs` de `vantadb-server` ×2) |
| (O1) `run_phase 01-unit-plain` con los argumentos en la línea siguiente | **1** (`phase …: no -p`) | R-crates, en vez de desaparecer la fase |
| (O2) `paths:` en flow style en `pull_request` | **1** (`2 paths: key(s) but 1 readable block(s)`) | R-forma |
| (O3) `--test=backup_snapshot` con su entrada de `paths:` borrada | **2** | R-targets |

**4. Coste, con las dos listas parseadas del YAML real** (`pathBlocks` del propio chequeo, rangos de `gh api …/activity`): **`old=4/20` → `new=8/20`**, `both=4`, `onlyNew=4`, `onlyOld=0` ⇒ **×2, no ×5** ⇒ **no se separa en dos lanes**. Los 4 nuevos: `0ef9160de` (`vanta-memory/src/services/scheduler.rs`), `997c41c62` (`vantadb-mcp/src/proxy.rs`), `66404bedc` (el push de la fila), `2f8ea4d76` (`src/cli_handlers/*.rs`). Ninguno de los 20 toca `build.rs`, así que la entrada derivada de O4 no mueve el número.

**5. `actionlint` 1.7.12: exit 0 con shellcheck (0,22 s) y exit 0 con `-shellcheck=` (0,06 s).** El step nuevo es **una** sentencia (`run: node …`), no una sentencia añadida a un bloque existente: el cuelgue de `FIND-785`/`FIND-766` no aplica. Primer intento (`name: Verify paths: covers …`) fue **error de YAML** (`mapping values are not allowed in this context`, exit 1) y se corrigió quitando el `:` del nombre.

**6. `npx prettier --check dev-tools/check-cert-lane-paths.mjs` exit 0** (el hook `prettier` del pre-commit corre sobre `.mjs`).

**7. `node dev-tools/check-harness-contract.mjs` → `harness contract holds (8 checks)`, exit 0; `--self-test` → 81/81, exit 0.** No se tocó el fichero (prohibido: cerrado en `FIND-794`); su escaneo de `dev-tools/**` no encuentra ningún `nextest run` ni átomo `test()` nuevo en el script.

**8. `node scripts/docs/check-links.mjs` exit 0 · `node scripts/docs/check-docs.mjs` exit 0.**

**9. `node dev-tools/check-cert-lane-paths.mjs` es importable sin ejecutarse** (guarda `invokedDirectly`): lo usa la medición y evita que un `import` dispare `process.exit`. El `process.exit` top-level original mataba la medición antes de imprimir una línea — defecto encontrado y cerrado antes del cierre.

**10. La cifra "214 ejecuciones" del encargo no se re-midió acá** (no tiene fuente en el repo; el review la marcó como no reproducible, O5): el costo se expresa en **pushes que disparan** (§4), no en tests ejecutados.

## Verificación (contrato del cierre)

- `node dev-tools/check-cert-lane-paths.mjs --self-test` → **4/4**, exit 0. ✅
- `node dev-tools/check-cert-lane-paths.mjs` → **verde** en el árbol; **rojo** sobre `HEAD` (26), sobre las 6 mutaciones de la tabla §3 (2 / 1 / 6 / 1 / 1 / 2). ✅
- `actionlint …` → **exit 0** con y sin shellcheck. ✅
- `npx prettier --check` → exit 0. ✅
- `node scripts/docs/check-links.mjs` + `check-docs.mjs` → exit 0. ✅
- **No se compiló Rust** (disco 19,5 GB libres; `FIND-799` compilando). Ninguno de los dos artefactos lo necesita: el chequeo es estático y `actionlint`/`prettier`/docs son texto.
- **`gen-index` no se corrió** (instrucción del carril): `docs/index.md` queda desactualizado respecto a este fichero nuevo y lo regenera el carril de índice, igual que con `FIND-794`.

## Review

**P2-01, `vanta-review`, contexto fresco, una ronda → `changes-required`: 0 Critical + 5 Required + 6 Optional + 4 Nits. Los 5 Required y los Optionals baratos, cerrados; los nits corregidos salvo el `id:` del frontmatter (no es convención de estos task files: `FIND-794` tampoco lo lleva).**

- **R1 — el gate no vigilaba la entrada que lo hace ejecutable.** Borrar `- ".github/workflows/snapshot-certification.yml"` de los dos bloques dejaba el chequeo **verde** y la lane muda para siempre: el argumento de no-circularidad descansaba en una entrada que nada verificaba. **Cerrado:** R-autorreferencia exige, en cada bloque, la ruta real del workflow que se audita y la del propio script (derivadas con `relative()`, sin literales en el cuerpo; si `--workflow` apunta a una copia fuera del repo se usa la ruta canónica de la lane). Mutación: **2** violaciones.
- **R2 — sólo se derivaba el primer `-p`.** `-p vantadb -p vantadb-server` en una fase existente pasaba verde: contraejemplo literal del contrato. **Cerrado:** `matchAll` de `-p`/`--package` (con `=` también) y todas las crates de la fase se auditan. Mutación: **6** violaciones (`vantadb-server` × `Cargo.toml`/`src/**`/`build.rs` × 2 bloques).
- **R3 — el rango del §Premisa estaba mal transcrito.** Decía `997c41c62..66404bedc` (2 ficheros) para un push de 31, listaba `vanta-memory/src/services/scheduler.rs` y `vantadb-server/src/scheduler.rs` (que son de otro push) y omitía los dos `vantadb-mcp/tests/*.rs` que sí estaban. **Cerrado:** el bloque es ahora la salida literal de `dca7d5818..66404bedc` (el `before` real del evento) + el `.total_count` de la API, con la corrección declarada al lado. La conclusión no cambia (0 runs para ese SHA).
- **R4 — cifras autocontradictorias en §Verificación** (decía 8 y 2 donde lo medido era 6 y 1). **Cerrado:** 4/4, 0/6/2/2, y la tabla de mutaciones con el número de cada una.
- **R5 — la promesa de simetría no estaba implementada.** Quitar `- "Cargo.lock"` del bloque `push` dejaba 1 ocurrencia y el gate verde: un push de sólo `Cargo.lock` dejaba de disparar en silencio. **Cerrado:** R-simetría compara la unión de entradas contra cada bloque. Mutación: **1** violación.
- **Optionals aplicados:** O1 (`run_phase` partido en dos líneas ya no desaparece: se deriva y se reporta `no -p`) · O2 (un `paths:` en flow style se cuenta y se reporta: `2 paths: key(s) but 1 readable block(s)`; el conteo ahora ancla en `^\s{2,}paths:` sin exigir línea vacía tras los dos puntos) · O3 (`--test=<name>`) · O4 (`build.rs` de cada crate derivado, no añadido a mano: la raíz tiene uno de 141 L que compila para todas las fases) · O5 (la cifra "214" no se cita: el coste se mide en pushes) · O6 (`mod`/`include!`/`#[path]` declarados en el techo, con la razón de que hoy el hueco está vacío).
- **Nits corregidos:** N1 (prosa cortada en el comentario del workflow) · N2 (las líneas de la tabla de fases son las del fichero entregado, `:256-270`) · N3 ("primer `run:` step", no "primer step": el primero es el checkout). **No aplicado:** N4 (`id:` en frontmatter), porque ningún task file del repo lo lleva.
- **Resistió sin cambios (re-ejecutado por el revisor):** la derivación completa (3 crates, 7 fases, sin `-p` escondido ni `--package`/`--workspace` fuera de `run_phase`), el coste `4/20 → 8/20` con `both=4`/`onlyNew=4`/`onlyOld=0`, la ausencia de listas a mano, `actionlint` 0 con y sin shellcheck, el harness en 8 checks / 81-81, y la posición del step antes de `rust-setup` (no rompe nada: el shell es de job, `ubuntu-latest` trae node y el script no necesita toolchain ni historia de git).
- **NOTICED BUT NOT TOUCHING (review):** el árbol arrastra WIP ajeno (`docs/dev/plans/2026-11-23-release-v0.10.0-gates.budget.json`, `docs/dev/tasks/H-02-H-07.md`, `web/` sin trackear) — el commit va con `git-commit-isolated.ps1` y `-Paths` explícitos para no llevárselos.
