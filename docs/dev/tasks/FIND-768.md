---
title: "FIND-768 - la lane de snapshot elegía 42 de 56 tests; ahora ejecuta el binario y compara listed contra executed"
kind: task
status: completed
description: "Las fases 03/04 de snapshot-certification.yml estaban verdes eligiendo 42 de los 56 tests del binario: 14 nunca se ejecutaban, incluido find319_child_flag_ignores_empty_values. Ahora seleccionan binary(snapshot_certification) y run_phase compara nextest list del target contra el JUnit de la fase (listed == executed) en las siete fases."
tags: [vantadb, ci, nextest, snapshot, coverage]
---

# FIND-768 — un verde que elige 42 de 56 no mide

> **Dependencias:** `FIND-752` ✅ (la auditoría que midió el hueco) · `FIND-738` ✅ (el caso
> concreto que se topó con uno de los 14) · `FIND-766` (vecino: allí la lane no arrancaba).
> **Carril:** CI (sólo YAML + docs; ningún `.rs` tocado).

## Contexto

`snapshot-certification.yml` corre un binario excluido del `default-filter` de nextest.
Las fases 03/04 lo seleccionaban **por nombre de test**:

```
-E 'test(snapshot) or test(pending_restore) or test(schedule)'
```

Con `--features failpoints` el binario tiene **56 tests** y ese selector elige **42**.
Los 14 restantes existen, compilan, pasan — y no los ejecuta nadie:

| Test | Por qué queda fuera del selector |
|---|---|
| `export_format_*` (4) | el nombre no contiene `snapshot`/`pending_restore`/`schedule` |
| `find319_child_flag_ignores_empty_values` | idem — es el caso que `FIND-738` tuvo que correr a mano |
| `vantafile_*` (5) | idem |
| `wal_*` (4) | idem |

La lane **estaba verde** (39/48 runs en 30 días), así que nadie miró el selector. Ese es
el defecto de fondo: la lane no sabía cuántos tests había elegido, así que "42 de 56
pasan" y "56 de 56 pasan" producían el mismo verde.

## Contrato

1. La lane cubre **los 56** (las dos vías válidas eran cubrirlos o justificar cada
   exclusión; se elige cubrirlos porque `find319_child_flag_ignores_empty_values` **es**
   comportamiento de snapshot — `FIND-319` es el espejado recursivo de `keyspaces/` — y
   justificar su exclusión habría sido escribir una razón falsa).
2. La lane compara `nextest list` contra lo ejecutado: **`listed == executed`**.
3. Mecánico: `actionlint` 0 · YAML parseado · los dos números publicados.

## Impacto mapeado (Regla 0)

| Archivo | Leído | Referencias hacia dentro | Referencias entrantes | Impacto |
|---|---|---|---|---|
| `.github/workflows/snapshot-certification.yml` | completo (176 L) | `.config/nextest.toml:67`, `Cargo.toml` (target `snapshot_certification`) | `docs/dev/operations/CI_POLICY.md` §2a/§2c, `docs/dev/avance/activo/ci-cd.md` | **se edita**: fases 03/04 (`-E`) + `run_phase` (aserción). Sin cambios en triggers, permisos, timeouts ni steps |
| `docs/dev/operations/CI_POLICY.md` | §2c completo (642-690 L) | `AGENTS.md`, `dev-tools/verify.ps1` | 25 ficheros | **sólo la fila de `snapshot_certification` en §2c** (el ticket prohíbe el resto del fichero) |
| `docs/dev/Backlog.md` | fila 832 | — | — | **se retira la fila `FIND-768`** (sólo esa) |
| `docs/dev/avance/activo/ci-cd.md` | entradas de la ola (FIND-752/760/767) | — | — | **se añade una entrada** al final |
| `target/find752-snap-fp-all.txt`, `target/find752-snap-fp-filter.txt` | completos | los generó `FIND-752` | — | **sólo lectura** — son la medición de los dos números |

Fuera de alcance (prohibido por el ticket y respetado): `.github/workflows/ci-rust.yml`,
`certification.yml`, `docs-gates.yml`, `.config/nextest.toml`, `src/**`,
`vantadb-mcp/**`, `.opencode/**`, `dev-tools/**`, `docs/dev/plans/**`.

**Veredicto:** un fichero de workflow y tres de documentación. El riesgo no es de código:
es que la lane siga midiendo poco o que la aserción dé un falso rojo. Ambas cosas se
probaron con el propio bloque `run:` extraído del YAML (§Verificación).

## Spec — decisiones

| # | Decisión | Alternativas y por qué se descartan |
|---|---|---|
| 1 | Cubrir los 56 con `-E 'binary(snapshot_certification)'` en 03/04 | **(a)** Justificar por escrito los 14: imposible sin mentir en `find319_*`, que es snapshot puro. **(b)** Borrar el `-E`: funciona (ya hay `--test snapshot_certification`), pero deja la intención invisible y una futura reintroducción de un `-E` estrecho no tendría dónde chocar |
| 2 | La población se cuenta **del target, no del `-E` de la fase** (`coverage=full`) | Esta es la decisión que hace que la aserción sirva. Si `listed` se calculara con el mismo `-E` que el run, un selector estrecho encogería **los dos** lados y la comparación seguiría verde — el mismo agujero, ahora con una aserción decorativa. En `full`, `listed` es el binario entero y `executed` lo que corrió: un `-E` estrecho se ve como `executed < listed` |
| 3 | `coverage=scoped` para 01/02/05/06/07 | Esas fases **sí** quieren un subconjunto (lib con nombre, el roundtrip de rocksdb, los consumidores de memory/MCP). Ahí la población correcta es la que elige su filtro; comparar contra el target entero daría cinco rojos permanentes |
| 4 | La aserción vive en `run_phase`, no en un step aparte | Cubre las siete fases con ~10 líneas en vez de siete steps. Una fase nueva hereda el gate sin que nadie se acuerde de copiarlo |
| 5 | Se conserva `--no-tests fail` (ya estaba) | Cubre el caso "selección vacía". No cubre el caso de este ticket —una selección **parcial**— y por eso no bastaba: es exactamente lo que `FIND-738` documentó (0 tests, exit 0) |
| 6 | El conteo de lo ejecutado sale del JUnit de la fase, no del stdout de nextest | El JUnit ya se escribe y se versiona como artefacto; parsear `Summary [...] 42 tests run` del log implica ANSI y formato de consola. `grep -o '<testcase'` es mecánico y estable |
| 7 | Sin `--run-ignored` | El binario no tiene ningún `#[ignore]` (verificado: `rg 'ignore'` sólo devuelve nombres de test), así que la población es 56 corra o no los ignorados. Si mañana alguien agrega uno, `listed != executed` y la fase se pone roja — que es lo correcto: un ignorado en este binario es otro hueco |
| 8 | No se toca `.config/nextest.toml` | Path prohibido por el ticket, y no hace falta: la exclusión del fast filter es correcta, lo que estaba mal era la lane |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Medir: 56 del binario vs 42 del selector, y los 14 nombres del set-diferencia | ✅ (artefactos de `FIND-752`, re-diffeados acá) |
| 2 | Fases 03/04 → `binary(snapshot_certification)` | ✅ |
| 3 | `run_phase` cuenta la población antes de correr y la compara con el JUnit | ✅ |
| 4 | Verificar el bloque `run:` real (no una copia): sintaxis + 3 escenarios con `cargo` falso | ✅ |
| 5 | `actionlint` + YAML parseado + los dos números | ✅ |
| 6 | Docs: `CI_POLICY.md` §2c/§2a · `Backlog.md` · `ci-cd.md` | ✅ |
| 7 | Review P2-01 (una ronda) + `ocr-review.ps1` | ✅ |
| 8 | Commit local (sin push, sin `--amend`, sin `gen-index`) | ✅ |

## Verificación

**El workflow no se puede disparar desde acá** (`workflow_dispatch` requiere la rama
por defecto y no hay push): los dos números salen de los listados ya medidos y la
aserción se prueba ejecutando el bloque `run:` real con un `cargo` falso.

| Check | Resultado |
|---|---|
| `actionlint .github/workflows/snapshot-certification.yml` (v1.7.12) | ✅ **exit 0** |
| YAML parseado (`yaml.safe_load`) | ✅ triggers `push, pull_request, workflow_dispatch` · jobs `snapshot-linux` · 5 steps · bloque `run:` 3612 B extraído |
| `bash -n` sobre el bloque `run:` extraído del YAML | ✅ `SYNTAX_OK` |
| Escenario A — población 56, ejecutados 56 | ✅ las **siete** fases escriben `listed=56 executed=56` y el driver sale **0** |
| Escenario B — el hueco de este ticket (población 56, selector elige 42) | ✅ fase 03 **roja**: `03-certification-plain: population is 56 test(s) but 42 executed.` driver exit **1**. Las fases `scoped` (01/02) siguen verdes: la aserción no da falsos positivos |
| Escenario C — selección vacía (las tres knobs del harness a `0`) | ✅ `01-unit-plain: nextest list exited 0 selecting 0 test(s).` driver exit **1** |
| `cargo nextest list` real (re-medición) | ⏸ **abortado y declarado**: quedó **10 min** bloqueado en `artifact directory lock` con dos carriles Rust vivos, sin una línea de salida. Los números son los medidos en `FIND-752` con el mismo comando y las mismas features (`target/find752-snap-fp-all.txt` = 56; `find752-snap-fp-filter.txt` = 42) |

### Los dos números

| Medición | Comando | Resultado |
|---|---|---|
| Binario completo (`--features failpoints`) | `cargo nextest list -p vantadb --ignore-default-filter --features failpoints --test snapshot_certification` | **56** (medido en Windows por `FIND-752`) |
| Selector **viejo** de 03/04 | mismo comando `+ -E 'test(snapshot) or test(pending_restore) or test(schedule)'` | **42** (14 fuera) |
| Selector **nuevo** de 03/04 | `-E 'binary(snapshot_certification)'` | el target entero, por construcción — ver abajo |

Sobre el tercer número, con precisión porque es el que la review tumbó: `binary(<name>)` no
puede elegir menos que el target que ya fija `--test snapshot_certification`, pero **no es
una medición mía**: es una derivación, y la fase 04 **no** lista 56. `encryption` no está en
los defaults (`Cargo.toml:181`) y **tres** tests están gateados por ella
(`tests/core/snapshot_certification.rs:2915`, `:3297`, `:3327`), así que la fase 04
(`failpoints,encryption`) lista **59** si la 03 lista 56. La plataforma de CI además cambia
un test `cfg(windows)` por uno `cfg(unix)`, de modo que el total de Linux puede diferir en
±1 del medido en Windows. Por eso el número autoritativo **no se publica acá**: lo emite la
propia lane, una línea por fase, en `artifacts/snapshot-certification/listed-vs-executed.txt`.
Intenté cerrar el conteo estáticamente (parseo de los `#[cfg]` de los 62 `#[test]`) y el
resultado (53/54/57) **no coincide** con el medido: el heurístico no modela bien
`all(fjall, not(fs2))` ni los `cfg` anidados en módulos. Se descarta en vez de publicarlo.

Los 14 del set-diferencia, uno a uno: `export_format_empty_lines_skipped`,
`export_format_preserves_all_record_fields`, `export_format_schema_version_roundtrip`,
`export_format_updates_existing_records_version_tracking`,
`find319_child_flag_ignores_empty_values`, `vantafile_export_golden_file`,
`vantafile_mixed_representations_roundtrip`, `vantafile_multi_write_rebuild`,
`vantafile_serialization_determinism`, `vantafile_tombstone_persistence`,
`wal_crc32c_deterministic_and_detects_corruption`,
`wal_postcard_serialization_deterministic`, `wal_record_all_variants_roundtrip`,
`wal_replay_recovers_from_known_wal`.

Con el cambio, la lane publica en cada run `artifacts/snapshot-certification/listed-vs-executed.txt`
una línea por fase (`03-certification-plain listed=56 executed=56 coverage=full`). Ese
fichero es la prueba en CI de lo que acá se deriva del listado — y es la respuesta a "¿cuántos
eligió?" que antes no existía en ningún sitio. **Con una fase roja el step aborta (`set -e`),
así que el fichero sólo tiene las siete líneas en un run verde**; con 03 roja quedan 3.

### Coste declarado

14 tests extra × 2 fases (03/04). Son tests de formato/roundtrip (`export_format_*`,
`vantafile_*`, `wal_*`) más un spawn hijo (`find319_*`): ninguno está en la clase de
`test_snapshot_consistent_under_concurrent_writes`, que ya corría. El job tiene 90 min y
las fases 03/04 son una fracción; el techo real es el `slow-timeout` por test (180 s), que
no cambia.

## Review (P2-01 — agente distinto, una ronda)

**Revisor:** `vanta-review`, sesión independiente (`ses_de6e91584ffei6z9qsARl1r8fa`), sin
participación en la implementación. Tier **fast** por glob (`.github/**` + `docs/dev/**`).
**Veredicto de la ronda: `changes-required`** — 1 High + 2 Medium + 4 Low; todos cerrados
abajo salvo dos que se declaran como techo.

Re-ejecutado **por el revisor**, no heredado: `actionlint` 0 · YAML `safe_load` (3 triggers,
1 job, 5 steps, bloque `run:` **3612 B / 92 líneas**) · `diff` normalizado del bloque-YAML
contra el `run_phase.sh` del harness → **0 líneas** (es el bloque real, no una copia) ·
escenarios A/B/C del harness con `bash -eo pipefail` reproducidos (A exit 0 con 7 fases
`listed=56 executed=56`; B `03: population is 56 test(s) but 42 executed.` exit 1 con 01/02
verdes; C exit 1) · `grep -c` de los artefactos de `FIND-752` 56 y 42 con `comm` del
set-diferencia = los 14 nombres exactos · `tests/core/snapshot_certification.rs` 62 `#[test]`
con la aritmética de `cfg` que da 56 en la plataforma de CI · `check-links` 0 roto ·
`check-docs` GATING all clear · Backlog `0 6` con `FIND-757/764/766` intactas · `CI_POLICY`
sólo 2 hunks, ambos en §2c · `description` 337 chars · `bash` sin fugas de `set -e`/`pipefail`.

| # | Sev | Hallazgo | Disposición |
|---|---|---|---|
| 1 | **High** | El gate P2-01 se declaraba cumplido antes de existir: `status: completed`, Step 7 ✅ y la entrada de `ci-cd.md` decían «Dictamen y cierres en §Review» mientras §Review decía «Ver §Cierre» y §Cierre no tenía dictamen | ✅ **corregido**: esta sección es el dictamen, y el estado pasa a `completed` **después** de escribirla |
| 2 | Medium | «runs **56 of 56**» en `CI_POLICY` §2c: la fase 04 corre `failpoints,encryption`, `encryption` no es default y **3** tests están gateados por ella → 04 lista **59**, no 56 | ✅ **corregido**: §2c dice 56 (03) / 59 (04); el task file explica que el número de 04 es derivado y que la plataforma puede diferir ±1, y que el número autoritativo lo emite la lane |
| 3 | Medium | `ci-cd.md` arrastraba la entrada de `FIND-764` (~24 líneas) sin commitear de otro carril: commitear el fichero entero la habría enviado dentro de `FIND-768` | ✅ **corregido**: se reconstruyó `ci-cd.md` = `HEAD` + mi entrada para el commit y se restauró el working copy completo después (patrón `RULES.md` §11) |
| 4 | Low | El escenario C estaba citado con los knobs ambiguos: con `FAKE_LISTED=0 FAKE_EXECUTED=56` el rojo es 03, no 01 | ✅ **corregido**: la cita dice «las tres knobs del harness a `0`» |
| 5 | Low | Con `set -e`, el primer rojo aborta el step y `listed-vs-executed.txt` queda con <7 líneas; «una por fase, las siete» sólo vale en verde | ✅ **corregido**: declarado en el task file. No se cambia el comportamiento — «a failure stops subsequent phases» es el contrato de la lane desde su origen (§2a) |
| 6 | Low | El invariante depende del literal `full`/`scoped` en el call site; un futuro `run_phase 03 scoped '<estrecho>'` reabriría el hueco, y el harness que lo caza vive en `%TEMP%` | 📌 **techo aceptado**: commitear el harness pediría `dev-tools/**`, que el ticket prohíbe. Mitigación parcial: el call site de 03/04 lleva el comentario `# full: every test in the binary, whatever its name (FIND-768)` |
| 7 | Low | `cancel-in-progress: true` con `push` en esta lane (`:68`): misma clase que `FIND-764`, y ahora la lane es la única que carga 56 tests | 📌 **NOTICED BUT NOT TOUCHING**: el trigger no está en el contrato; va como fila aparte |

**Sin segunda ronda del reviewer:** los dos Blocking/High/Medium materiales (1, 2) se cerraron
con los números y el estado corregidos; el revisor no re-emite sobre el follow-up (misma
doctrina que `FIND-752`/`FIND-738`). Lo que **no** cambió: el diseño de la aserción, que el
revisor confirmó por reproducción y no por reporte.

## Cierre de la fila

- Fila `FIND-768` retirada de `docs/dev/Backlog.md` (sólo esa fila; el diff del fichero es
  `0 6` y las filas de `FIND-757`/`FIND-764`/`FIND-766` siguen ahí — el primer corte se hizo
  sobre un working copy que ya venía sin las dos primeras, se detectó con `numstat` y se
  rehízo desde `git show HEAD:`).
- Entrada añadida a `docs/dev/avance/activo/ci-cd.md`.
- `gen-index.mjs` **no** se corrió: lo prohíbe el contrato de la tarea (la ola lo
  regenera al cerrar, misma decisión que `FIND-752`/`FIND-760`).
- **Verificación no corrida, declarada:** `dev-tools/verify_changed.ps1` / `verify.ps1` no
  se ejecutaron — el carril es CI y no toca ni un `.rs`; además el árbol es compartido con
  dos carriles Rust vivos y el artefacto estaba bloqueado (los 10 min de espera del
  `nextest list`). Los gates que sí aplican a la superficie tocada están todos en verde
  (`actionlint`, YAML, `bash -n`, harness A/B/C, `check-links`, `check-docs`).
- **Incidente del primer commit (declarado, reparado):** `3554aa3db` se llevó por delante la
  sección de `FIND-757` en `docs/dev/Backlog.md` — la expectativa del helper lo detectó
  (`REVERTED: '| FIND-768 |' is STILL in HEAD`), pero el commit ya estaba hecho: lo que había
  commiteado era un snapshot del working tree donde el fichero se había reordenado mientras
  yo reconstruía desde `HEAD`. Reparado en `a187d034d` (+6/−6: restaura `FIND-757`, retira
  `FIND-768`) y verificado **por contenido** (`ExpectPresent FIND-757`, `ExpectPresent
  FIND-758`, `ExpectAbsent FIND-768`). Lección: con dos carriles escribiendo el mismo fichero,
  reconstruir desde `HEAD` y commitear no son atómicos; la expectativa del helper detecta el
  daño pero no lo previene.
- **NOTICED BUT NOT TOUCHING:** (1) `cancel-in-progress: true` en `push` dentro de esta
  lane (`:68`) es la clase de `FIND-764` y esta lane es la única que carga los 56 —
  merece fila propia; (2) el harness A/B/C vive en `%TEMP%` y no en el repo, porque
  `dev-tools/**` está prohibido por el ticket.

## Regresión (FIND-785 — 2026-11-26)

El guard que esta fila añadió **apagó la lane entera** en el push `43e41f055`: siete fases
sin ejecutar, y ninguna por un test rojo.

- **La medición que lo mató:** `01-unit-plain: nextest list exited 0 selecting 0 test(s).`
  El artifact del mismo run lo desmiente: `01-unit-plain.listed.txt` = **5440 bytes / 42
  líneas**. La lane no midió cero tests: midió cero con el patrón.
- **Dos errores deterministas en una línea** — `grep -oE '^[^[:space:]]+::[^[:space:]]+ '`.
  (1) `nextest list` imprime `vantadb accumulator::tests::test_accumulator_snapshot` —
  **package, espacio, test** —, así que `^` ancla contra un `::` que en la primera columna
  no existe. (2) `CARGO_TERM_COLOR: always` (`:71`) envuelve cada token en escapes ANSI, que
  `[^[:space:]]+` absorbe. Medido sobre el artifact real: **42** líneas, **0** casan con el
  patrón, **42** con `^\S+ \S+$` una vez pelado el ANSI.
- **Por qué la verificación de `FIND-766` no lo cazó:** `-T auto` imprime `human` en un
  terminal y `oneline` cuando stdout es un pipe (lo dice `cargo nextest list --help`), así
  que el formato **depende del entorno**. Un conteo sobre la presentación no mide la
  población: mide cómo se imprimió.
- **El arreglo:** `--message-format json` y el conteo sobre
  `."rust-suites"[].testcases[] | select(."filter-match".status == "matches")` — sólo los
  tests que el `run` va a ejecutar (el `-E` de la fase ya está aplicado ahí, y un `ignored`
  aparece como `mismatch`). `.listed.json` guarda la salida cruda; `.listed.txt` se deriva
  del JSON (una línea `suite test`, sin ANSI) para que el artifact siga siendo legible.
  `test-count` **no** se usa: es «tests, incluidos skipped e ignored», no la población.
- **El fallo de diseño, que importaba más que el bug:** un gate cuya **medición** falla
  apagaba la certificación — el modo de fallo que esta fila existía para evitar, al revés.
  Ahora sólo aborta el caso en que no hay nada que medir (`nextest list` exit ≠ 0). Los otros
  dos se **distinguen** y ninguno apaga la fase: `measurement=suspect` es una medición
  **válida de cero** (exit 0, 0 tests) y `measurement=unmeasured` es el **contador roto**
  (el JSON no se pudo leer). `listed > 0` mantiene `listed == executed` como estaba. La
  aserción original **se conserva**: un `run` que ejecuta de menos sigue rojo, y cualquiera
  de los dos estados no medidos es rojo si el `run` ejecuta 0.
- **Verificación:** el `run_phase` **real, extraído del YAML** (86 líneas / 3091 B), con un
  `cargo` falso, sobre el artifact real y fixtures con la forma real
  (`package<space>mod::test`, con y sin ANSI): 42 → 42 verde (compacto y `json-pretty`), 0
  contados / 7 ejecutados → **verde** con `suspect`, JSON ilegible / 7 ejecutados → **verde**
  con `unmeasured`, 42 listados / 41 ejecutados → rojo (`population is 42 test(s) but 41
  executed.`), lista fallida → aborta antes del `run`, run con tests rojos → rojo, y sin
  `jq` en el `PATH` el step aborta en vez de contar 0 en las siete fases. **30/30**; el mismo
  harness contra el `run_phase` de `HEAD` da **18 fallos / 12 aciertos**.
- **Un segundo límite, medido acá:** `actionlint` 1.7.12 con su shellcheck embebido cuelga
  con este `run:` en cuanto **crece una sentencia** — no es cuestión de prosa: `final.yml`
  (bloque de 6377 B) linta en 0 s y aguanta +148 B de líneas en blanco, pero **+1 sentencia
  (113 B) lo deja sin terminar en más de 100 s**, y quitar una línea no lo recupera si
  quedan más. Por eso la prosa de `FIND-768`/`FIND-785` vive en comentarios YAML encima del
  step (como en `certification.yml`, `FIND-766`), y el cuarto estado se escribió como un
  `case` que sustituye **dos** compuestos por **uno** (6773 B de bloque) para no gastar
  presupuesto de sentencias. Regla para el próximo que toque este `run:`: añadir código sólo
  si se retira otro tanto, y volver a correr `actionlint` **sin** `-shellcheck=`.
- **Lo que cambia para quien lea `listed-vs-executed.txt`:** la línea gana un quinto campo,
  `measurement=counted|suspect|unmeasured`. Y una selección vacía deja de detectarse en el
  `list` (`--no-tests fail` la ve en el `run`, exit ≠ 0): sigue roja, por la otra puerta.
