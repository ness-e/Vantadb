---
title: "FIND-788 — el contador medía la presentación de `nextest list`, no la población"
kind: task
status: completed
description: "Contaba con una regex sobre la presentación de `nextest list` (ANSI + `package test`), medía 0 y apagaba las 5 fases. Se reusa el patrón de `FIND-785`: población desde `--message-format json` (`filter-match == matches`), cuatro estados (counted/suspect/unmeasured) y aborto sólo si `list` falla. Harness de 30 casos: 30/30; sobre `HEAD`, 6/30."
tags: [vantadb, ci, nextest, certification, actionlint, ci-gates]
---

# FIND-788 — la misma bomba, en el otro contrato

> **Dependencias:** `FIND-785` (el fix que se reusa, `dfa4af7d7`), `FIND-768` (la aserción
> `listed == executed` y su §Regresión), `FIND-766` (el techo de `actionlint` y las fases de
> este job). **Carril:** CI, sin compilar Rust. **Escritura:** `.github/workflows/certification.yml`
> (exclusivo), este task file, la fila del Backlog y `docs/dev/avance/activo/ci-cd.md`.

## Contexto

La fila decía: *"`certification.yml:434` tiene la **misma regex sobre la presentación** de
`nextest list` y **sin `--message-format`**: la bomba que tumbó `CERT` (`FIND-785`) sigue armada
en otro contrato"*. Se re-midió antes de tocar nada, y es literal: el bloque de `HEAD` tenía

```bash
listed=$(grep -oE '^[^[:space:]]+::[^[:space:]]+ ' "$out/$phase.listed.txt" | wc -l || true)
if [[ "$list_status" -ne 0 || "$listed" -eq 0 ]]; then
  echo "$phase: nextest list exited $list_status selecting $listed test(s) with a full selector." >&2
  return 1
fi
```

o sea los mismos tres defectos que `FIND-785` midió en el gemelo:

1. `nextest list` imprime `vantadb accumulator::tests::test_x` — **package, espacio, test**, no
   `pkg::mod::…` —, así que el `^` ancla contra un `::` que nunca sigue al package;
2. `CARGO_TERM_COLOR: always` (`:76`, el mismo `env` de workflow) envuelve cada token en ANSI y
   `[^[:space:]]+` lo absorbe;
3. `-T auto` elige `oneline` bajo pipe y `human` en TTY, así que **el conteo depende del entorno**:
   no se arregla con otro regex.

Y el cuarto, que es el que mata la lane: el `|| "$listed" -eq 0` **aborta la fase**. Con un
contador que da 0, esa rama apaga las cinco fases antes de ejecutar un test — el modo de fallo que
`FIND-785` produjo de verdad en `CERT` (7 fases muertas, `01-unit-plain: nextest list exited 0
selecting 0 test(s)` sobre un artifact de 42 líneas).

**Confirmación local del defecto, con un `nextest` real (0.9.133):** sobre un crate sonda, la
misma población se lista como `probe tests::beta_snapshot` en la presentación y como
`filter-match.status == "matches"` en el JSON; un `#[ignore]` llega como `mismatch` con
`reason: "ignored"`, y con `-E` los no seleccionados llegan como `mismatch` con
`reason: "expression"`. `test-count` los cuenta a todos (3 con 1 ignored), así que **no** es la
población.

## Contrato — lo que pedía la fila y lo que se hizo

| Pedido | Hecho |
|---|---|
| Mismo patrón que `FIND-785` | `--message-format json` → `$phase.listed.json` como fuente de verdad; `$phase.listed.txt` derivado con `jq` (una línea `suite test`, sin ANSI) |
| Población = `filter-match == matches` | `jq '[ ."rust-suites"[] \| .testcases[] \| select(."filter-match".status == "matches") ] \| length'`; `test-count` **no** se usa |
| Cuatro estados | `counted` · `suspect` (0 contados con `list` exit 0) · `unmeasured` (JSON ilegible) · aborto sólo si `list` exit ≠ 0 |
| No abortar por un contador roto | `suspect`/`unmeasured` → `::warning` + la fase sigue; `jq` ausente aborta **antes** de medir (contar 0 en cinco fases no es una medición) |
| La aserción original se conserva | `measurement=counted` y `executed != listed` → **rojo**; `suspect`/`unmeasured` con `executed == 0` → **rojo** |
| Harness de 30 casos → 30/30 | 30/30 sobre el bloque nuevo; **6/30** sobre el de `HEAD` (las 24 que fallan son las del contador y las de la aserción que dependía de él) |
| `actionlint` exit 0 con y sin shellcheck | exit 0 en `1.7.12`, 3 corridas por modo (0,8-1,1 s con shellcheck; 0,1-0,5 s con `-shellcheck=`) |

## Impacto mapeado (Regla 0)

- **Leído completo:** `.github/workflows/certification.yml` (1.269 líneas en `HEAD` → 1.323, `+76/−22`),
  `snapshot-certification.yml` (§el bloque gemelo que `FIND-785` arregló), `docs/dev/tasks/FIND-768.md`
  §Regresión, el diff `dfa4af7d7`.
- **Referencias hacia dentro:** el job `other-heavy` (`:316`) y sus 5 llamadas `run_phase`; el step
  `Upload execution logs, JUnit reports and the counts` (`:535`) sube `target/nextest/other-heavy/`;
  `listed-vs-executed.txt` es citado como fuente autoritativa en ese mismo comentario
  (`:540`) y en `docs/dev/operations/CI_POLICY.md` (por nombre, sin formato de columnas).
- **Referencias hacia fuera:** `./.github/actions/rust-setup` (instala `nextest` **sin pin** vía
  `taiki-e/install-action`), `jq` (preinstalado en `ubuntu-latest`; el gemelo ya lo exige desde
  `dfa4af7d7`), `actions/upload-artifact` (path sin cambios).
- **Veredicto:** el único consumidor del formato de `listed-vs-executed.txt` es un humano leyendo
  el artifact; se le **añade** un quinto campo (`measurement=`), no se le quita ninguno. Ningún otro
  step parsea `.listed.txt` (verificado con grep: sólo el comentario del upload lo nombra).

## El bloque: antes → después

Antes (`HEAD`, 58 sentencias / 101 líneas de script):

```bash
set +e
cargo nextest list ... "${expect[@]}" > "$out/$phase.listed.txt" 2> "$out/$phase.list.stderr.txt"
list_status=$?
set -e
listed=$(grep -oE '^[^[:space:]]+::[^[:space:]]+ ' "$out/$phase.listed.txt" | wc -l || true)
if [[ "$list_status" -ne 0 || "$listed" -eq 0 ]]; then
  echo "$phase: nextest list exited $list_status selecting $listed test(s) with a full selector." >&2
  return 1
fi
...
set +e
cargo nextest run ... | tee "$out/$phase.txt"
pipeline_status=("${PIPESTATUS[@]}")
set -e
status="${pipeline_status[0]}"
if [[ "${pipeline_status[1]}" -ne 0 ]]; then status=1; fi
...
if [[ "$executed" -ne "$listed" ]]; then ... status=1; fi
```

Después (63 sentencias / 107 líneas; el bloque completo vive entre `certification.yml:432` — el
`- name:` — y `:544`, donde termina la quinta llamada; `run_phase()` ocupa `:454-522`):

```bash
cargo nextest list \
  ... --ignore-default-filter \
  --message-format json \
  "${cfg[@]}" \
  "${expect[@]}" > "$out/$phase.listed.json" 2> "$out/$phase.list.stderr.txt" || list_status=$?
listed=$(jq -r '[ ."rust-suites"[] | .testcases[] | select(."filter-match".status == "matches") ] | length' "$out/$phase.listed.json" 2>/dev/null || true)
jq -r '."rust-suites" | to_entries[] as $s | $s.value.testcases | to_entries[] | select(.value."filter-match".status == "matches") | "\($s.key) \(.key)"' "$out/$phase.listed.json" > "$out/$phase.listed.txt" 2>/dev/null || true
case "$listed" in
  '' | *[!0-9]*) measurement=unmeasured; listed=0 ;;
  0) measurement=suspect ;;
  *) measurement=counted ;;
esac
if [[ "$list_status" -ne 0 ]]; then
  echo "$phase: nextest list exited $list_status; the population could not be measured." >&2
  return 1
fi
if [[ "$measurement" != "counted" ]]; then
  echo "::warning title=Population not measured::$phase measurement=$measurement; continuing without the listed/executed check."
fi
...
cargo nextest run ... | tee "$out/$phase.txt" || status=$?
...
if [[ "$measurement" != "counted" ]]; then
  if [[ "$executed" -eq 0 ]]; then echo "..."; status=1; fi
elif [[ "$executed" -ne "$listed" ]]; then
  echo "$phase: population is $listed test(s) but $executed executed." >&2
  status=1
fi
```

Las dos decisiones de forma, medidas y no argumentadas:

- **`set +e` / `PIPESTATUS` → `|| status=$?`.** Con `set -o pipefail` el pipeline falla si falla el
  `nextest` **o** el `tee`, que es exactamente lo que las cuatro sentencias viejas calculaban. Pagó
  8 de las 13 sentencias que pide el contador de cuatro estados (neto **+5**: 58 → 63).
- **Prosa fuera del `run:`.** Todos los porqués van en comentarios YAML encima del step; dentro del
  script no hay un solo `#`. Es la regla que `FIND-766`/`FIND-785` dejaron por el techo de
  `actionlint`, y se respetó: el `run:` creció 6 líneas y no una sentencia de prosa.

## Verificación

### 1. El harness de 30 casos, contra el `run_phase` **real extraído del YAML**

`FIND-785` dejó su harness en `%TEMP%` (no hay fichero en el repo: `dev-tools/**` está prohibido por
el ticket), así que se reconstruyó desde su §Regresión conservando los casos que allí se nombran
(42→42 verde en compacto y `json-pretty`; 0 contados/7 ejecutados verde con `suspect`; JSON ilegible
/7 verde con `unmeasured`; 42 listados/41 ejecutados rojo; lista fallida aborta antes del run; run
con tests rojos rojo; sin `jq` aborta) y se extendió a 30. El sujeto de prueba es el `run:` sacado
del YAML con `awk` (indent 10 quitado, sin heurísticas) y ejecutado bajo `bash` 5.3 con un `cargo`
falso que **elige su salida como el real**: JSON sólo si `--message-format json` está en `argv`, y
la presentación (`package<space>mod::test`, envuelta en ANSI) si no.

| # | Caso | `HEAD` | nuevo |
|---|---|---|---|
| 1 | 42 matches → `listed=42 executed=42` verde | ✗ | ✓ |
| 2 | el conteo ignora ANSI + `package test` | ✗ | ✓ |
| 3 | dos suites suman 42 | ✗ | ✓ |
| 4 | `ignored` fuera de la población (`test-count=45`) | ✗ | ✓ |
| 5 | JSON `json-pretty` cuenta igual | ✗ | ✓ |
| 6 | `.listed.txt` derivado: 42 líneas | ✓ (crudo) | ✓ (derivado) |
| 7 | `.listed.txt` = `suite test`, sin ANSI | ✗ | ✓ |
| 8 | `.listed.json` es el documento crudo | ✗ | ✓ |
| 9 | stderr del `list` en su fichero | ✓ | ✓ |
| 10 | 0 contados / 7 ejecutados → `suspect` + `::warning` + verde | ✗ (aborta) | ✓ |
| 11 | todo `mismatch` → `suspect` | ✗ | ✓ |
| 12 | JSON truncado → `unmeasured` + `::warning` + verde | ✗ | ✓ |
| 13 | JSON sin `rust-suites` → `unmeasured` | ✗ | ✓ |
| 14 | lista de 0 bytes → `unmeasured` | ✗ | ✓ |
| 15 | sin `jq` → aborta **antes** del run | ✗ | ✓ |
| 16 | `list` exit 3 → aborta, `run` nunca invocado | ✓ | ✓ |
| 17 | `list` exit 101 con JSON válido → aborta | ✓ | ✓ |
| 18 | **42 listados / 41 ejecutados → ROJO** | ✗ | ✓ |
| 19 | 42 listados / 43 ejecutados → ROJO | ✗ | ✓ |
| 20 | 42 listados / 1 ejecutado → ROJO | ✗ | ✓ |
| 21 | `suspect` + 0 ejecutados → ROJO | ✗ | ✓ |
| 22 | `unmeasured` + 0 ejecutados → ROJO | ✗ | ✓ |
| 23 | run con tests rojos → ROJO | ✓ | ✓ |
| 24 | rojo sin JUnit → ROJO | ✓ | ✓ |
| 25 | verde sin JUnit fresco → ROJO | ✗ | ✓ |
| 26 | JUnit viejo movido a `prior-report.txt` | ✗ | ✓ |
| 27 | `scoped`: el `-E` llega al `list` **y** al `run` | ✗ | ✓ |
| 28 | `full`: el `list` **no** se estrecha, el `run` sí | ✗ | ✓ |
| 29 | el `list` pide JSON; el `run` conserva `--no-tests fail` | ✗ | ✓ |
| 30 | `listed-vs-executed.txt` con 5 campos y `measurement=counted` | ✗ | ✓ |

**30/30 y 6/30.** El 6/30 de `HEAD` no es "el harness está mal": son los casos que el bloque viejo
acierta por casualidad o porque el defecto no los toca (9: separa stderr; 16/17: aborta por
`list_status`; 23/24: el `status` del pipeline; 6: escribe un `.listed.txt` crudo que hoy coincide
en número). Las 24 restantes son el defecto, incluida la que importa: **la aserción de ejecución
parcial no se perdió**.

**La premisa, medida sobre la presentación real (no heredada de la fila):** con 42 tests con la forma
real (`vantadb accumulator::tests::test_N`), el regex viejo ve **0** de **42** — y **también 0**
después de pelar el ANSI (`sed 's/\x1b\[[0-9;]*m//g'`): el ancla `^[^[:space:]]+::` no falla sólo por
el color, falla porque la primera columna es el **package** y no lleva `::`. Los dos defectos son
independientes, que es exactamente lo que `FIND-785` midió en `CERT` (42 líneas, 0 contadas). El JSON
de la misma población dice **42**.

### 2. `actionlint`: el binario **pineado** del gate, con y sin shellcheck

El gate real no es "un actionlint": `docs-gates.yml:725-733` instala **v1.7.7** con su `sha256`
(`023070a287cd…0757`) y corre `/tmp/actionlint -color`. Se ejecutó **ese** tarball (mismo sha,
verificado) desde WSL, que es Linux x86_64 como el runner:

```
actionlint 1.7.7 (pineado, sha256 OK) .github/workflows/certification.yml → exit 0 · 1939 ms   (con shellcheck 0.11.0)
actionlint 1.7.7 (pineado) .github/workflows/certification.yml            → exit 0 ·  728 ms   (sin shellcheck en el PATH)
actionlint 1.7.7 sobre el fichero de HEAD (referencia)                    → exit 0 · 1624 ms
actionlint 1.7.12 (binario local)                                         → exit 0 · 849/818/1063 ms · y 131/154/466 ms con -shellcheck=
```

Sin cuelgues y sin regresión de orden de magnitud (HEAD 1,6 s → 1,9 s con shellcheck). Y el
`run:` extraído del YAML es **byte-idéntico** al que pasó 30/30 (`cmp`). El bloque creció a 63
sentencias / 107 líneas (antes 58 / 101): el techo de `FIND-766` no se supuso, se volvió a medir
con el binario que decide, no con el que había a mano.

### 3. El conteo real de las fases afectadas

Medido con el binario local (`cargo-nextest 0.9.133`) y **el comando exacto de cada `run_phase`**
(paquete, `--test`/`--lib`, features) más `--ignore-default-filter --message-format json`, contando
`filter-match == matches`:

| Fase | Población real (perfil dev) | Suites | `test-count` |
|---|---|---|---|
| `01-vantadb-cli-tests` (`-p vantadb --features cli,arrow --test cli_tests`) | **96** | 1 | 96 |
| `02-vantadb-dataset-free` (11 binarios de `vantadb`) | **12** | 11 | **13** |
| `03-vantadb-lib-hnsw` (`--lib -E 'test(index::core::tests::concurrent_insert_preserves_hnsw_invariants)'`) | **1** por construcción | 1 | — |
| `04-server` (`-p vantadb-server --features prometheus --test mcp_integration --test e2e --test benchmarks --test server`) | **45** (1+19+5+20) | 4 | 45 |
| `05-mcp` (`-p vantadb-mcp --test mcp_tests`) | **123** | 1 | 123 |

**276 tests medidos** en cuatro fases; la quinta es un selector de nombre exacto.

Tres notas que la medición deja y que valen para el que lea el artifact:

- **La fase `02` es la prueba en vivo de que `test-count` no es la población:** mide **13** y hay
  **1 `#[ignore]`** (`structured_api_v2` conserva su `ollama_integration`), así que la población es
  **12**. Un contador basado en `test-count` habría exigido 13 ejecutados y puesto la fase en rojo
  por un test que decide no correr. El conteo por `filter-match` da 12. (`FIND-766` ya lo había
  razonado de forma analítica; acá queda medido.)
- **El `0` de las fases `04`/`05` sin `--ignore-default-filter` es un artefacto de la medición
  local, no del step.** El `list` de CI pasa `--config-file "$out/nextest.toml"`
  (`certification.yml:452`, `cfg=(--config-file "$out/nextest.toml")`), y `--config-file` **reemplaza**
  la ubicación de `.config/nextest.toml`
  (doc de nextest, *configuration*), así que en CI ese `default-filter` ni se carga y la flag es
  no-op; mi medición local no pasaba `--config-file`, de ahí el 0. Los números de la tabla no
  cambian (el `list` de CI no filtra por default-filter). Lo que sí importa y sigue en pie: el
  conteo se lee del **JSON** con el `-E` ya aplicado en `filter-match`, así que la población no
  depende de cómo se imprimió ni de qué filtro por defecto hubiera.
- **La fase `03` no se pudo medir acá, y no por este cambio:** `cargo nextest list -p vantadb --lib`
  murió **linkeando** el binario de test (`error: linking with link.exe failed: exit code 1120`,
  `LNK2019` sobre símbolos `anon.*.llvm.*` de `tracing_subscriber`/`dashmap`) — la clase de fallo de
  un `target/` incremental compartido por varios carriles. La lane de CI fija
  `CARGO_INCREMENTAL: 0` (`:77`) justamente por eso. El `1` sale del selector: hay exactamente un
  `fn concurrent_insert_preserves_hnsw_invariants` (`src/index/core.rs:152`) y `-E test(<nombre
  exacto>)` selecciona ese y sólo ese.

### 4. Higiene

`git diff --numstat` del workflow: un fichero, 12 hunks, todos dentro del job `other-heavy`
(`387-554`), YAML válido (lo parsea el `actionlint` pineado del gate), sin cambios en ningún otro
workflow. El hook de pre-commit corre sus gates de YAML.

**Y una advertencia de árbol compartido, porque el alcance real del commit no es el que se planeó.**
Cuatro carriles estaban escribiendo `docs/dev/Backlog.md` y `docs/dev/avance/activo/ci-cd.md`
mientras esto se hacía, y la fila del Backlog de esta tarea hizo **tres viajes**:

| Commit | Qué le pasó a la fila `FIND-788` |
|---|---|
| `d15564b06` | salió del Backlog dentro del commit del carril de `FIND-789`, que arrastró una escritura mía en su snapshot |
| `43b8a7006` | **volvió** — el carril de `FIND-792` la restauró al detectar la desaparición (`FIND-792` es justo esa clase: un commit path-scoped retira la fila de otro carril) |
| `932f8137d` | **retirada por su dueño**, en un commit que toca **sólo** `docs/dev/Backlog.md`, con `-ExpectAbsent` de esta fila y `-ExpectPresent` de las siete vecinas (`781/783/787/790/791/792/793`) |

**Y la sección de este ticket en `ci-cd.md` ya estaba commiteada** en `6e4119c8d` (mismo accidente,
mensaje que no la nombra); el commit `560ff7e8a` lleva sólo las cuatro líneas que corregí después.
Resultado neto: **nadie perdió una fila** (`FIND-792` la recuperó a tiempo) y la retirada final es
atribuible.

**Un hallazgo que este carril se hizo a sí mismo, y que se declara en vez de esconderse:** el primer
intento de retirar la fila se construyó con `(git show HEAD:…)` **dentro de PowerShell**; el pipe
re-codifica el texto con la página de códigos de la consola y convirtió el fichero entero en mojibake
(`514/516` líneas cambiadas, `🟠` → `≡ƒƒá`). Se detectó con `git diff --numstat` **antes de
commitear**, se revirtió con `git checkout --` y se rehizo con `[IO.File]::ReadAllText(…, UTF8)` +
`WriteAllText(…, UTF8-sin-BOM)` sobre el fichero en disco: `0 2`. El `≡ƒƒá` de la fila restaurada por
`43b8a7006` es la misma trampa del otro lado — **la restauración de `FIND-792` metió el emoji
mangleado**. Es materia de `FIND-793` ("el helper da falso negativo con expectativas acentuadas"),
que ya está registrada: no se abre fila nueva.

## Lo que NO se puede verificar (y por qué)

1. **Un run real de este `run:`.** El job `other-heavy` está gateado (`:317`) a `workflow_dispatch`
   **o** al `schedule` del domingo 03:00. Un push a `develop` —el que llevará este commit— **no**
   ejercita este step. Hasta el próximo dispatch o cron, la prueba es el harness sobre el bloque
   real + `actionlint`: no hay run que lo respalde, y decirlo es parte del cierre.
2. **La fase `03` medida.** `cargo nextest list -p vantadb --lib` murió en el **link** del binario
   de test (`link.exe` exit 1120; `LNK2019` sobre `anon.*.llvm.*` de `tracing_subscriber`/`dashmap`),
   el modo de fallo de un `target/` incremental compartido entre carriles. No lo causa este cambio
   (el diff es YAML) y la lane de CI fija `CARGO_INCREMENTAL: 0` (`:77`). El `1` de esa fase se
   sostiene en el selector, no en una corrida: un solo `fn concurrent_insert_preserves_hnsw_invariants`
   (`src/index/core.rs:152`).
3. **El `--cargo-profile release` de CI.** Los conteos de arriba son de perfil **dev**: la población
   por nombre y por feature es la misma, el binario que corre no.
4. **El `jq` del runner.** No se ejecutó nada en `ubuntu-latest`; que `jq` esté preinstalado es
   propiedad de la imagen y el gemelo (`snapshot-certification.yml`, `dfa4af7d7`) ya depende de
   ella. Si faltara, el step ahora **aborta con mensaje** en vez de contar 0 en cinco fases.

## Review P2-01

**Revisor:** agente distinto (sesión sin participación en la implementación). **Enfoque:**
approach + premisa + evidencia re-ejecutada. **Veredicto inicial:** 🔴 `changes-required` — el cambio
de código es correcto y lo reproduje; lo que fallaba eran dos registros (un número falso y
`actionlint` medido en una versión que no es la que gatea). **Tras el cierre de los dos Required:
aprobado** (§Hallazgos y qué pasó con cada uno). Ningún defecto funcional.

**Premisa (H-07), re-ejecutada:** ✅ confirmada. `git show HEAD:.github/workflows/certification.yml`
líneas 425-441: `grep -oE '^[^[:space:]]+::[^[:space:]]+ '` sobre `.listed.txt`, sin
`--message-format`, y `if [[ "$list_status" -ne 0 || "$listed" -eq 0 ]]` → `return 1`. Con una
población de 42 con la forma real (`package<space>mod::test`, ANSI): el regex viejo cuenta **0**
con ANSI y **0** después de pelarlo (`vantadb accumulator::tests::test_0` — la primera columna no
lleva `::`); el `jq` de la misma población da **42**.

**Cómo se probó (lo que re-ejecuté):**

- **Harness:** `BLOCK=…/block-new.sh bash harness.sh` → **30/30**; `block-head.sh` → **6/30**.
  El bloque extraído con `awk`+`sed` del YAML de trabajo es **byte-idéntico** a `block-new.sh`
  (`cmp`, 3894 B), así que el sujeto del harness es el fichero que se commitea. `bash -n` OK en los dos.
- **Datos reales del autor, recontados:** `c01=96`, `c02=12` (`test-count=13`, 1 `ignored`, 11
  suites), `c04=45` (1+19+5+20), `c05=123` → 276. Coincide con las tablas. `c03.json` sí contiene
  el fallo de link (`link.exe` exit 1120 / `LNK2019`).
- **Aserción con datos reales:** `c02` con 12 ejecutados → verde `listed=12 executed=12`; con 13 →
  rojo `population is 12 test(s) but 13 executed.`; `c01` 96/96 verde; 96/0 rojo.
- **`tee` roto** (`$out/p01.txt` como directorio): `rc=1` — el `|| status=$?` con `pipefail` sí
  registra el fallo del `tee`, como afirmaba el comentario.
- **`filter-match` sin `-E`:** el `probe-plain.json` real (0.9.133) trae `status:"matches"` para los
  no ignorados y `mismatch/ignored` para el `#[ignore]`; la doc de nextest muestra lo mismo sin
  filtro. La duda "las fases `full` contarían 0" queda cerrada.
- **`case`:** orden correcto (`'' | *[!0-9]*` → `0` → `*`); `unmeasured` fuerza `listed=0`, así que
  el `[[ -ne ]]` de la rama `counted` nunca ve un no numérico.
- **Alcance:** los hunks de `certification.yml` son sólo el bloque + los comentarios de encima + el
  comentario del upload (`git diff -U0`, 12 hunks, todos 387-554). Ningún otro workflow tocado.

**Hallazgos y qué pasó con cada uno** (una ronda; el veredicto se emitió sobre el change set, no
sobre su propio follow-up):

| # | Severidad | Hallazgo | Resolución |
|---|---|---|---|
| R1 | Required | el task file decía que el fichero pasaba de 1.269 a **1.284** líneas | **corregido**: 1.269 → 1.323 (`+76/−22`, medido con `Get-Content … .Count` y `git diff --numstat`) |
| R2 | Required | `actionlint` medido en **1.7.12**, pero el gate pinea **1.7.7** (`docs-gates.yml:725`) | **corregido con evidencia, no con una nota**: se descargó el tarball pineado, se verificó su `sha256` y se corrió desde WSL (Linux x86_64 como el runner) → **exit 0** con y sin shellcheck (1,9 s / 0,7 s) |
| M1 | Medium | la mitad del change set ya estaba commiteada por otros carriles, con mensajes que no la nombran | **declarado** en §Higiene, con los SHAs (`d15564b06`, `6e4119c8d`) y la verificación de que las filas vecinas sobreviven |
| M2 | Medium | "keeps the FIND-768 assertion untouched" sólo vale para `counted` | **corregido** en la prosa del YAML: *"untouched while measurement=counted"*, explicitando qué se cede y por qué (y que el `::warning` es anotación de run summary, no una línea de log) |
| M3 | Medium | tres documentos con dos rangos de tiempo distintos para `actionlint` | **corregido**: una sola tabla de tiempos, con la versión pineada primero |
| L1 | Low | el rango `:427-534` se cortaba 5 líneas antes del final | **corregido**: `:432-544`, `run_phase()` en `:454-522` |
| L2 | Low | el `0` sin `--ignore-default-filter` no aplica a CI: `--config-file` reemplaza la ubicación del config | **corregido**: el `0` es artefacto de la medición local; los números no cambian y la razón está escrita |
| L3 | Low | `status: completed` con placeholders | **corregido**: las dos secciones se escriben antes del commit |
| L4 | Low | `.listed.txt` queda en 0 bytes si `jq` falla (el `>` trunca antes) | **aceptado sin cambio**: el `measurement=unmeasured` + `::warning` lo declaran, y un artifact vacío con la fase verde y su estado escrito es más honesto que un fichero con datos viejos |

**No re-emitió ronda** (misma doctrina que `FIND-752`/`FIND-738`/`FIND-786`): los dos Required se
cerraron con medición nueva, no con una promesa, y los Medium/Low son de registro.

## Cierre

- **Commits (locales, sin `--amend` y sin push — Regla 7), por `dev-tools/git-commit-isolated.ps1`
  (índice privado + verificación por contenido):**
  - `560ff7e8a` — `fix(ci): count certification tests from JSON, not from the presentation (FIND-788)`:
    `.github/workflows/certification.yml`, `docs/dev/tasks/FIND-788.md`, `docs/dev/avance/activo/ci-cd.md`
    (4 líneas de corrección; la sección grande ya viajaba en `6e4119c8d`, ajeno).
  - `932f8137d` — `docs(backlog): retire the FIND-788 row (closed by 560ff7e8a)`: **sólo**
    `docs/dev/Backlog.md`, con `-ExpectAbsent` de esta fila y `-ExpectPresent` de las siete vecinas
    (`781/783/787/790/791/792/793`) → 8/8 verificadas. Se hizo en un commit aparte porque la fila
    había vuelto en `43b8a7006` después de que `560ff7e8a` se creara.
- **`gen-index.mjs` no se corre**: lo prohíbe el contrato de la tarea; lo regenera la ola.
- **Fila del Backlog:** retirada en `932f8137d` (§Higiene cuenta los tres viajes). **`ci-cd.md`:**
  sección escrita, parcialmente en `6e4119c8d` y parcialmente en `560ff7e8a`.
- **Learnings:** contador de población en CI = leer el formato máquina, nunca la presentación; y un
  gate cuya **medición** falla no puede apagar la lane — se distingue el estado y se sigue. Tercero,
  de este árbol: **nunca** construir contenido de `docs/` desde `(git show …)` en PowerShell — se
  re-codifica y manglea el fichero entero.
- **Deuda / siguiente:** fijar la versión de `nextest` en `rust-setup` (hoy sin pin, y
  `--message-format` depende de ella); los otros usos de `test-count` en el repo como si fuera
  población; y `CARGO_TERM_COLOR: always` tiñendo todo el log de la lane.
