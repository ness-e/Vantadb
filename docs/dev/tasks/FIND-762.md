---
title: "FIND-762 - el `pre-push` medía fmt con otro rustfmt que el job `fmt` de CI: medido 0 de 739 ficheros difieren hoy"
kind: task
status: completed
description: "El `pre-push` medía fmt con el rustfmt del PATH (pin 1.95.0) y el job `fmt` de CI con nightly: dos instrumentos, un verde. Medido hoy: 0 de 739 `.rs` difieren entre stable, el pin 1.95.0 y nightly. Fix: el gate lee el toolchain del job `fmt` de `ci-rust.yml` y mide con él; si falta, mide igual y lo declara ADVISORY. La versión de rustfmt se imprime siempre."
tags: [vantadb, ci, pre-push, rustfmt, toolchain]
---

# FIND-762 - el `pre-push` medía fmt con otro rustfmt que el job `fmt` de CI

## Contexto

`FIND-760` (`FIND-753-760`) hizo que el `pre-push` juzgue **el rango que viaja**
en vez del árbol de trabajo, y dejó el `fmt` como el único gate acotable a
fichero (`rustfmt --check` por `.rs` del rango, en `dev-tools/fmt-range.ps1`).
Lo que **no** quedó alineado es el **instrumento**: el script invocaba el
`rustfmt` que resolvía el PATH —el de la toolchain pineada, `1.95.0`— mientras
el job `fmt` de CI mide con la toolchain que declara su step de
`rust-setup`. Es la clase de `FIND-736`/`FIND-731` (medir con un instrumento
que no es el del contrato) reapareciendo en el último gate antes de la rama
compartida: un push verde en `fmt` podía fallar en CI y el carril no tenía
forma de saberlo sin esperar al run.

**Dato nuevo (no se citan líneas del Backlog).** `ci-rust.yml` se movió hoy con
`FIND-770`; verificado con `grep -n "toolchain: nightly"` + lectura del bloque,
no por línea citada: el job `fmt` empieza en **`:192`** y su step de
`rust-setup` declara **`toolchain: nightly` (`:208`)**, con el gate en
**`cargo fmt --check` (`:214`)** sobre el árbol entero. La línea `:117` **existe
hoy, pero ya no es el gate** (es un `fi` dentro de `rust-scope`): lo que se movió
fue el `fmt`, no el fichero (precisión de la review, L2). `on:` es 3-61 y el job
nuevo `rust-scope` 64-160.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.githooks/pre-push` (369 líneas),
  `dev-tools/fmt-range.ps1` (54 líneas, versión HEAD),
  `dev-tools/git-commit-isolated.ps1` (155), `docs/dev/workflow/RULES.md` §10
  (`:213-234`), `rust-toolchain.toml`, bloque `fmt` de
  `.github/workflows/ci-rust.yml` (`:192-215`), `docs/dev/avance/activo/ci-cd.md`
  §FIND-760/§FIND-770.
- **Referencias hacia dentro:** el hook es la única entrada de
  `fmt-range.ps1` (`-ListFile`); `fmt-range.ps1` es la única consumidora del
  `rustfmt` local. Nadie más depende del shape del script.
- **Referencias entrantes:** `RULES.md` §10 describe el comando del gate y
  citaba `ci-rust.yml:123` (stale desde `FIND-770`); `ci-cd.md` §FIND-760
  describe el gate por rango.
- **Veredicto impacto:** **bajo en código, medio en semántica**. El script es de
  un solo consumidor y su contrato de salida es un exit code; el riesgo real es
  que el gate pase a **bloquear** por un motivo que no es del commit (toolchain
  ausente) o que **mienta** sobre qué midió. Ambos se tratan abajo.

## Contrato

(a) el gate local mide con **el mismo toolchain que el job `fmt` de CI**, leído
del workflow — nunca un literal hardcodeado; (b) si ese toolchain no está
instalado, el gate **igual mide** con el del PATH y lo **declara** ADVISORY
(nunca bloquea por "no tengo nightly"); (c) la **versión de rustfmt se imprime
siempre**, así el verde tiene nombre y es comparable con el run; (d) prueba de
dos caras o declaración medida de por qué no se puede fabricar; (e)
`bash -n .githooks/pre-push` → 0.

## Spec (tabla de decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|---|---|---|
| 1 | ¿Unificar o declarar? | (a) unificar: medir siempre con nightly · (b) declarar: avisar que local es stable · (c) **leer el toolchain de CI y medir con él cuando está, declararlo cuando no** | **(c)**. (a) rompe a todo carril sin nightly instalado —el pre-mortem del ticket, y "no lo instalé" no es una propiedad del commit—. (b) deja el verde mintiendo cuando nightly **sí** está (que es el caso aquí, medido). (c) es (a) cuando se puede y (b) cuando no. |
| 2 | ¿De dónde sale el toolchain? | (a) literal `nightly` en el script · (b) leerlo del job `fmt` de `ci-rust.yml` | **(b)**. (a) es exactamente el defecto que esta fila cierra: un segundo sitio donde el pin puede derivar. Coste: 12 líneas de regex sobre el bloque del job; si no lo encuentra, lo **dice** en vez de asumir. |
| 3 | ¿Bloquea si falta el toolchain? | bloquear · advisory | **advisory**. Bloquear convertiría un gate de formato en un gate de inventario de toolchains, y el fallo es del entorno, no del diff. Es el mismo criterio que el advisory del CLI (`FIND-754`). |
| 4 | ¿Se toca `ci-rust.yml`? | (a) pinear `nightly` en CI · (b) no tocarlo | **(b)** este carril. El fichero lo acaba de dejar `FIND-770` y el contrato lo prohíbe. El residual va **declarado** en el output (`nightly` no está pineado, CI instala el más nuevo por run) y como propuesta en §Deuda. |
| 5 | `-Toolchain` param | sin param (solo lectura del workflow) · con override | **con override**. Sin él la prueba de paridad stable-vs-nightly no se puede ejecutar (sería un test que no prueba nada). 3 líneas; el hook no lo pasa. |

## Mediciones (el dato que decide la fila)

**1. ¿Cuántos ficheros difieren entre instrumentos, hoy?** Formateando los
**739** `.rs` tracked con cada rustfmt (`--emit stdout`, `--edition 2021`,
`--config skip_children=true`, comparando la salida completa byte a byte):

| Par de instrumentos | Difieren | Parse-fail |
|---|---:|---:|
| `rustfmt +stable` `1.9.0-stable (8bab26f4f6 2026-07-14)` vs `+nightly` `1.11.0-nightly (c36f145719 2026-10-01)` | **0 / 739** | 0 / 0 |
| pin `1.95.0` `1.9.0-stable (59807616e1 2026-04-14)` vs `+nightly` `1.11.0-nightly` | **0 / 739** | 0 / 0 |

Es decir: **hoy los tres instrumentos coinciden en todo el árbol**. La fila no
tenía un defecto *materializado*, tenía un defecto **latente y no observable**:
el gate no decía con qué medía, así que el día que `rustfmt` cambiara de
formato el verde local seguiría pareciendo el mismo verde.

**2. La paridad también se midió en el lado del fallo (el que importa).** Dos
`cargo fmt --all -- --check` seguidos, con minutos de diferencia (2026-11-26,
~12:0x), devolvieron **el mismo fichero y el mismo diff** con las dos toolchains
— primero `vantadb-mcp/tests/mcp_proxy_reconnect.rs`, después
`src/storage/engine/delete.rs`, ambos WIP **de otro carril** (el árbol cambió
entre las dos corridas; ningún fichero es mío). El instrumento no cambia el
veredicto; lo que faltaba era poder **atribuirlo**. **No es reproducible hoy**
(review L3): esos dos WIP ya no están sucios en el árbol, así que
`rustup run nightly cargo fmt --all -- --check` y
`rustup run 1.95.0 cargo fmt --all -- --check` devuelven **exit 0 los dos** — que
es la conclusión opuesta leída al revés: sobre un árbol limpio los instrumentos
coinciden, y la medición que importa es la de la tabla 1 (0/739).

**3. Coste de unificar en local.** `nightly` **ya está instalado** aquí
(`rustfmt 1.11.0-nightly c36f145719`), así que medir con el instrumento de CI
no cuesta nada en esta máquina. Para un colaborador sin nightly el coste es
`rustup toolchain install nightly` (descarga de toolchain, ~minutos) — por eso
es **advisory** y no requisito.

## Prueba de dos caras

- **Cara (a) — "fichero formateado con stable y no con nightly → el gate local
  falla igual que CI": NO FABRICABLE, y se declara.** Con 0/739 diferencias no
  existe en este árbol un fichero que stable acepte y nightly rechace; inventarlo
  exigiría un `rustfmt` viejo con reglas que hoy no difieren. Lo que **sí** se
  probó es la propiedad equivalente y medible: **un fichero mal formateado hace
  fallar a los dos instrumentos con el mismo veredicto y el mismo diff** (sondas
  3a/3b abajo), y el gate ahora **mide con el de CI**, así que si la divergencia
  existiera, el juez sería el de CI.
- **Cara (b) — "formateado con nightly → pasa en los dos": ✅** (sondas 4a/4b).

Sondas ejecutadas en `%TEMP%\opencode\fmt762\probe` (no quedan en el árbol):

| Sonda | Entrada | `-Toolchain` | Instrumento impreso | Exit |
|---|---|---|---|---|
| 3a | `bad.rs` (sin formatear) | `stable` | `1.9.0-stable (8bab26f4f6)` | **1** |
| 3b | `bad.rs` (sin formatear) | `nightly` | `1.11.0-nightly (c36f145719)` | **1** |
| 4a | `good.rs` (formateado) | `stable` | `1.9.0-stable (8bab26f4f6)` | **0** |
| 4b | `good.rs` (formateado) | `nightly` | `1.11.0-nightly (c36f145719)` | **0** |
| 5 | `good.rs` | `1.50.0` (ausente) | ADVISORY: CI usa `1.50.0`, mide con el del PATH | **0** |
| 6 | `src/lib.rs` (real, del árbol) | *(default)* | `1.11.0-nightly` (leído del workflow) | **0** |

Las seis bajo `pwsh` 7 **y** `powershell.exe` 5.1 (mismo veredicto en ambos
motores); la columna "Instrumento impreso" y el exit se leyeron de la salida,
no de la intención.

**Un defecto propio, encontrado al sondear y corregido (por eso la sonda 5 se
probó con un toolchain realmente ausente):** la primera implementación resolvía
el binario con `rustup which --toolchain <want> rustfmt` sin comprobar antes si
existía. Medido: rustup **lo instala** ("syncing channel updates for
1.50.0… downloading 6 components", y el propio rustup avisa *"the missing
active toolchain has been auto-installed"*), así que la sonda colgó el shell
más de 3 minutos. En el gate, eso convierte cada push de quien no tenga el
toolchain de CI en una **descarga**, justo lo contrario del advisory. El script
ahora consulta `rustup toolchain list` (local, offline, 200 ms) y **sólo
después** resuelve el binario. Verificado: tras las seis sondas, `rustup
toolchain list` sigue con las mismas 4 toolchains que antes (y las dos que mis
probes instalaron de más se desinstalaron).

La sonda 6 es la que prueba el contrato (a): **sin pasar `-Toolchain`**, el
instrumento que imprime es nightly — el que `Get-CiFmtToolchain` leyó de
`ci-rust.yml` — no el del PATH.

**Cómo se detectaría cuando deje de ser 0.** El gate imprime
`instrument = rustfmt <versión> (<toolchain>)` en **cada** corrida y lo repite
en el `ok` final. Un rojo de CI que el local no reproduce se atribuye leyendo
esa línea contra el log del run. Residual declarado en el propio output: CI's
`nightly` **no está pineado** (instala el más nuevo por run), así que "mismo
canal" no es "mismo binario"; el mensaje lo dice y nombra el remedio
(`rustup update nightly`).

## Steps

1. **Medir** los tres instrumentos sobre los 739 `.rs` tracked → 0 diferencias
   en ambos pares (tabla §Mediciones). Sin esta medición la fila habría elegido
   "unificar" a ciegas.
2. **`dev-tools/fmt-range.ps1`:** `Get-CiFmtToolchain` (lee el job `fmt` del
   workflow), **chequeo previo de `rustup toolchain list`** y sólo después
   `rustup which --toolchain <instalado> rustfmt` (una vez, no por fichero — el
   shim por fichero habría costado un proceso extra por `.rs` del rango), línea
   de instrumento **siempre**, ADVISORY cuando el toolchain no está, nota del
   `nightly` sin pinear, y `-Toolchain` para sondas. Guarda explícita para un
   `rustfmt` sin `rustup` (instalación standalone): sin ella el gate nuevo
   **introducía** una dependencia que antes no tenía, y un `command-not-found`
   no capturado bloquearía el push con un mensaje ajeno. El loop de `--check` no
   cambia (`skip_children=true` sigue por lo medido en `FIND-760`).
3. **`.githooks/pre-push`:** comentario del bloque de gates — el hook delega el
   instrumento en el script y ahora lo **dice**. Cero cambios funcionales; el
   hook sólo pasa `-ListFile`.
4. **Encoding:** el fichero es **ASCII puro**. No es cosmético: `—` dentro de un
   string, leído como ANSI/CP1252 por `powershell.exe` 5.1 (el fallback de
   `find_pwsh`), decodifica el byte `0x94` como `”`, que PowerShell interpreta
   como **delimitador de string**. Lo que pasa después depende de dónde caiga
   (review M1, ambas formas reproducidas): (i) **error de parseo** ruidoso — el
   fixture mínimo de 3 líneas
   (`Write-Host "ok (range) — measured with x"` + `Fix: cargo fmt -p <crate>`) da
   **2 errores** (*"El operador '<' está reservado"*, *"Falta la cadena en el
   terminador"*) y `powershell.exe -File` sale **1**; mi primera versión del
   script dio **4** por esta vía, y HEAD daba **0**; (ii) **verde falso
   silencioso** — cuando el mojibake no rompe el parse, solo altera el string:
   la review midió salida destrozada con **exit 0**. Las dos son peores que un
   rojo, y las dos desaparecen con ASCII puro: medido **0** errores en ambos
   motores y `non-ascii bytes: 0`.
5. **`RULES.md` §10:** el instrumento en la línea `Must`, y las **dos** citas
   stale a `ci-rust.yml:123` reemplazadas — la del `Must` y la del `Por qué`
   (`:218`). La primera pasada arregló solo una y la review lo cazó (B1): el
   `diff` no respaldaba la afirmación. Hoy:
   `git diff docs/dev/workflow/RULES.md` → `1 1`, y
   `Select-String 'ci-rust.yml:1' RULES.md` → **0**.
6. **Cierre:** fila fuera del Backlog, esta entrada y la de `ci-cd.md`, commit
   con `git-commit-isolated.ps1` (con la excepción declarada de `ci-cd.md`, ver
   §Review M2).

## Verificación

- `grep -n "toolchain: nightly" .github/workflows/ci-rust.yml` → primera
  ocurrencia **`:208`**, dentro del job `fmt` (`:192`), gate `cargo fmt --check`
  (`:214`). Leído, no citado del Backlog.
- Medición 739 ficheros × 2 pares → **0** diffs, **0** parse-fail.
- Sondas 3a/3b/4a/4b/5/6 → exit `1,1,0,0,0,0` y el instrumento esperado en cada
  una (tabla §Prueba de dos caras), bajo `pwsh` 7 **y** `powershell.exe` 5.1.
- `bash -n .githooks/pre-push` → **0**.
- Parse del script: `[Parser]::ParseFile` → **0 errores** en pwsh 7 y en
  PowerShell 5.1; `non-ascii bytes: 0`.
- `node scripts/docs/check-links.mjs` → **0** (`broken markdown links: 0 (budget 0)`,
  3593 internos; el único wikilink es un snapshot histórico presupuestado) ·
  `check-docs.mjs` → **0** (mi task file aparece como *orphan*, igual que los de
  `FIND-757/767/770/771` en vuelo: informativo, no gate) · `gen-index` **no**
  ejecutado (prohibido por el contrato).

## Deuda declarada / no tocado

- **`ci-rust.yml` no se toca** (contrato; acaba de salir de `FIND-770`).
  Propuesta para quien lo posea, sin registrar fila: **pinear el rustfmt del
  job `fmt`** (p.ej. una fecha de nightly o el canal del repo + una versión) para
  que "mismo canal" sea "mismo binario". Hoy `toolchain: nightly` significa
  *el nightly que haya ese día*, así que el residual no se puede cerrar del todo
  desde local: sólo se puede **nombrar**, que es lo que hace el gate ahora.
- **`fmt-range.ps1` mide el rango, CI mide el árbol**: diferencia intencional de
  `FIND-760`, no se toca acá.
- `RULES.md` §10 cita además `verify.ps1` como gate de árbol del push; fuera de
  alcance.
- **Ancla del toolchain (review L1, medido):** `Get-CiFmtToolchain` toma la
  **primera** línea `toolchain:` dentro del job `fmt`, no específicamente la del
  step `uses: rust-setup`. Sondas con fixtures: no sangra a otro job (`B`/`D` →
  `$null`) pero un `run: |` que imprimiera `toolchain:` antes del real ganaría
  (`C` → `1.95.0`). **No alcanzable hoy**: el job `fmt` es un checkout + un
  rust-setup + un run. Queda como techo escrito en el script, no como olvido.
- **Instrumento del PATH con shim (review L4):** si el `rustfmt` del PATH es un
  shim de rustup, su `--version` puede arrastrar logs de sincronización en el
  mismo stream. Ahora se toma **solo** la línea `^rustfmt ` (o
  `(version unavailable)`), para que la línea del instrumento siga siendo legible.

## Review

**P2-01, contexto fresco (`vanta-review`), una ronda → `changes-required`, 2
Blocking + 2 Medium + 4 Low. Todos cerrados en este mismo cierre.**

Lo que **resistió** el ataque (vale registrarlo, no solo los hallazgos): la
medición 0/739 se **reprodujo completa** —739 ficheros × 3 instrumentos,
343 s— sin discrepancias en versiones ni en conteos; intentó fabricar la
divergencia de la cara (a) con dos ficheros sintéticos y 38 constructos y **no
pudo** (`nightly==pin`, `nightly==stable`, `pin==stable`), confirmando que el
"no fabricable" es un hecho medido; reprodujo las sondas 3a/3b/4a/4b exactas;
verificó con un stub de `rustup` que el gate **no instala** nada (solo
`toolchain list`, ningún `which` ni descarga) y que el guard de `rustup`
ausente es *load-bearing* (`& rustup` sin rustup lanza
`CommandNotFoundException` bajo `ErrorActionPreference=Stop`); y confirmó
`bash -n` 0, hook `5 +0` solo comentarios, Backlog `0 6`, y las tres citas del
workflow (`:192`, `:208`, `:214`).

- **B1 — corregido.** Afirmaba haber reemplazado la cita `ci-rust.yml:123` y el
  diff **no la respaldaba**: solo había tocado el bullet `Must`; la cita seguía
  viva en el `Por qué` (`RULES.md:218`). Arreglada ahora; la afirmación y el
  `diff` coinciden (`1 1` en `RULES.md`, `0` ocurrencias de `ci-rust.yml:1`).
- **B2 — corregido.** `status: completed` sin §Review poblada y con la sección
  apuntando a una Review inexistente. Esta sección es ese registro.
- **M1 — corregido (atribución).** Mi texto decía que el fallo era "4 errores de
  parseo"; la review objetó que `ParseFile` decodifica UTF-8 y no puede verlo.
  Reproducido a mano: **sí puede** cuando el `.ps1` va sin BOM — el fixture
  mínimo de 3 líneas da **2 errores** por `ParseFile` y exit **1** por
  `-File`. Pero la review tiene razón en lo que importa: hay una **segunda**
  forma, el verde falso silencioso (string alterado, **exit 0**), que midió
  ella. Las dos quedan escritas en §Steps 4.
- **M2 — asumido y declarado.** La entrada de `ci-cd.md` de este carril ya viajó
  en `4fa4d9fc0` (commit de otro carril, `FIND-767`): su snapshot se llevó el
  working copy completo, así que el plan de `RULES.md` §11 (blob desde `HEAD` +
  mi sección) quedó **sin efecto** — el fichero ya estaba en HEAD y lo que
  commiteo ahora es solo la corrección de la review. No se revierte (sería
  reescribir un commit ajeno); se declara.
- **L1 / L4 — declarados** (ver §Deuda: ancla del toolchain, línea de versión).
- **L2 — corregido.** `:117` **existe** (es un `fi` de `rust-scope`): lo que se
  movió fue el gate. Precisión arreglada en §Contexto.
- **L3 — corregido.** La medición 2 va fechada y marcada como **no
  reproducible** hoy (el WIP ajeno se limpió; los dos instrumentos dan exit 0
  sobre árbol limpio, que es el mismo resultado leído al derecho).
