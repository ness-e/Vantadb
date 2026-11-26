---
title: "FIND-730: la aserción del corpus TSan es inerte — da success con el summary vacío"
kind: task
status: active
description: "El step `Assert the TSan partition was actually exercised` imprime `partition 1/4:  (selected == executed)` sin haber parseado nada: el log de `tee` trae ANSI y el step corre sin pipefail, así que las dos guardas (aborto X/Y y selección vacía) nunca se evalúan. Fix: sed ANSI + `shell: bash` con pipefail explícito + fallo propio cuando el summary queda vacío"
tags: [ci, tsan, gates]
---

# FIND-730: la aserción del corpus TSan es inerte — da `success` con el summary vacío

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b
- **Fuente:** fila `FIND-730` de `docs/dev/Backlog.md` (L771), derivada del monitor de CI sobre `06485fe8e`
- **Esfuerzo:** 🟢 1 h · **Prioridad:** 🔴 Alta (el verificador de la lane TSan no verificaba nada)
- **Tipo:** CI/CD — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-714` ✅ (el step y el contrato que esta tarea arregla)
- **Creado:** 2026-11-26

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Nadie: el step vive dentro del job `sanitizer-tsan` (matrix de 4 particiones). No es check requerido (el job lleva `continue-on-error`), así que un rojo aquí no bloquea merges |
| Callees | Solo `sed`, `grep`, `tail`, `awk` sobre `tsan-partition-<n>.log` (el fichero que escribe el `tee` del step anterior) |
| Implicaciones | Ninguna sobre código ni sobre la lane de test. El riesgo es **de señal**: un verificador que pasa sin medir enseña a leer verde donde no hay veredicto — el patrón que este step existía para eliminar |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la extensión necesaria):** `.github/workflows/ci-rust.yml` L1000-1238 (job `sanitizer-tsan` entero: header, `continue-on-error` con su `# CATEGORY:`, los 4 steps, y el step de aserción completo), `docs/dev/tasks/FIND-717.md` (formato canónico de task file), `docs/dev/Backlog.md` L771 (contrato de la fila).
- **Referencias hacia dentro:** el step anterior (`Test with ThreadSanitizer …`) produce el fichero que este step consume; el bloque de comentario L1169-1178 declara el alcance del chequeo y ahora debe declarar también por qué el parseo puede fallar.
- **Referencias hacia fuera:** `dev-tools/tsan-suppressions.txt`, `TSAN_OPTIONS`, `.config/nextest.toml` (fuera de alcance: no se tocan).
- **Veredicto de impacto:** **bajo en corrección, alto en señal.** El cambio vive en un step de 20 líneas. No toca el job, ni `TSAN_OPTIONS`, ni las suppressions, ni `continue-on-error`.

## Diagnóstico — medido, no derivado

Run `45564374043` (attempt 1, sha `06485fe8e`, job de la partición 1/4 = `139036370042`).
El step salió **`success`** imprimiendo:

```
partition 1/4:  (selected == executed)
```

Es decir `SUMMARY=""`: nunca parseó un número y las dos guardas de abajo (aborto `X/Y`, selección `0`)
no se evaluaron. Tres defectos encadenados:

1. **ANSI.** El log viene de un `tee` con `CARGO_TERM_COLOR: always`. La línea real del resumen es

   ```
   <ESC>[31;1m     Summary<ESC>[0m [ 137.485s] <ESC>[1m821<ESC>[0m tests run: <ESC>[1m567<ESC>[0m <ESC>[32;1mpassed<ESC>[0m, …
   ```

   El `\x1b[0m` queda **entre** el número y ` tests run`, así que
   `grep -Eo '([0-9]+/)?[0-9]+ tests? run'` no matchea nunca. Verificado: sobre el log real del step (24.060 líneas, 5.447.588 bytes, reconstruido desde el job log) ese `grep` devuelve **0 líneas**.
2. **Sin `pipefail`.** El comentario del YAML afirmaba *"GitHub runs `run:` as `bash -eo pipefail`"*. Es falso: el step loguea `shell: /usr/bin/bash -e {0}` (07:10:39.175), mientras los steps que sí declaran `shell: bash` loguean `shell: /usr/bin/bash --noprofile --norc -e -o pipefail {0}`. Sin `pipefail`, `grep | tail` sale 0 aunque `grep` no encuentre nada → el `if ! SUMMARY=…` no dispara.
3. **Summary vacío no era un fallo.** `[ "" -eq 0 ]` es un error aritmético **dentro de un `if`**, o sea exento de `-e`: cae al `else`, no falla, y llega al `echo` verde. El único camino que quedaba era el silencio.

**Lo que NO estaba roto** (no se toca): la lane en sí mide. Ese mismo intento corrió 3138 tests en las 4 particiones, **0 abortos**, 939 fallos que son 100 % exit 66 del sanitizer, y **0 races del motor** — el único frame en `src/` es el falso positivo de `OnceLock`/futex en `kernels.rs:275`.

## Contrato — qué hace el step a partir de ahora

| Eje | Antes | Ahora |
|---|---|---|
| Parseo | `grep` directo sobre el log con ANSI | `sed 's/\x1b\[[0-9;]*m//g'`, luego **selección de la línea de `Summary`** y solo entonces extracción del conteo |
| `pipefail` | heredado de un comentario falso | `shell: bash` (documentado como `bash --noprofile --norc -eo pipefail {0}`) **y** `set -euo pipefail` como primera línea del `run:` — duplicado a propósito para no depender de la herencia |
| Summary vacío | caía al verde | `[ -z "$SUMMARY" ]` → `::error::` + `exit 1`, independiente del `if !` |
| Guardas `X/Y` y `0` | inertes (nunca se alcanzaban con datos) | evalúan por primera vez |

El `if !` se queda: es gate **y** diagnóstico (con `pipefail`, un `grep` sin match da el mensaje en vez de un `exit 1` pelado). Sin `|| true` (RULES.md regla 6). `continue-on-error` del job **no se toca**: retirarlo exige un run verde real de las 4 particiones y es decisión del owner.

## Evidencia — el script extraído del YAML, contra 6 logs

El script de cada caso es el bloque `run:` **extraído del workflow** (no una copia: se re-extrae con `awk`/`sed` en cada corrida, sustituyendo `${{ matrix.partition }}` por `PARTITION`), ejecutado con `bash` desde un directorio que contiene el `tsan-partition-1.log` del caso.

### 1. Log real (run `45564374043` attempt 1, partición 1/4) → parsea el número

```
=== case1 === (5504039 bytes)
partition 1/4: 821 tests run (selected == executed)
race reports: 254
suppression hits: 0
EXIT=0
```

Contraste con el parseo viejo, sobre el mismo fichero:

```
--- old parse on the tee file (no ANSI strip)
(no match)
--- new parse on the tee file (ANSI stripped)
821 tests run
```

### 2. Log vacío → exit 1

```
=== case2 === (0 bytes)
::error::partition 1/4 produced no TSan log — no verdict at all (FIND-714).
EXIT=1
```

### 3. Log sin línea de conteo → exit 1

```
=== case3 === (5503886 bytes)
::error::partition 1/4 has no nextest summary — it never reported how many tests it ran (FIND-714).
EXIT=1
```

### 4. Selección 0 (`0 tests run`) → exit 1

```
=== case4 === (88 bytes)
::error::partition 1/4 selected and ran 0 tests — check the -E filterset and .config/nextest.toml default-filter (FIND-714).
EXIT=1
```

### 5. Extra — el `-z` es independiente del `pipefail`

Mismo log del caso 3 con el `set -euo pipefail` degradado a `set -eu` (simulando la regresión de la causa 2): el `if !` vuelve a ser inerte y **solo** la guarda `-z` salva el paso.

```
=== case3 log + script without pipefail (only the -z guard can catch it) ===
::error::partition 1/4 parsed an empty summary — the assertion cannot verify the corpus (FIND-730).
EXIT=1
```

### 6. Extra — un `N tests run` impreso *después* del summary no gana (review O1)

nextest imprime las FAILURES **después** del `Summary`, así que un test que escriba "N tests run" en su stdout queda más abajo y un `tail -1` sin anclar lo tomaría como resumen — el decoy desactivaría las dos guardas. El fixture es el log real con una línea `9999 tests run` añadida al final:

```
--- un-anchored (one-stage) parse on the decoy log:
9999 tests run
--- anchored (two-stage) parse on the same log:
821 tests run
=== case6 === (5504174 bytes)
partition 1/4: 821 tests run (selected == executed)
race reports: 254
suppression hits: 0
EXIT=0
```

### Trampa encontrada al montar el test (documentada)

Probar el parser contra el **job log completo** da un falso positivo: el log incluye el eco del propio script del step, que contiene el texto `` `0 tests run` `` del comentario, y `tail -1` lo toma como resumen. En CI el fichero que lee el step es solo la salida del `tee`, así que el fixture correcto es esa sección, no el job log. El caso 1 de arriba usa la reconstrucción fiel (líneas entre el header del step de test y el `##[group]Run LOG=` de la aserción, prefijo de timestamp eliminado, ANSI preservado).

## Verify

- `actionlint .github/workflows/ci-rust.yml` → **exit 0**
- Los 6 casos de arriba → el positivo y el decoy imprimen `821 tests run`; los 4 negativos `exit 1`
- `pwsh dev-tools/ocr-review.ps1` → preview del diff + `ocr delegate rule .github/workflows/ci-rust.yml`; reglas de `.opencode/rules/release-ci.md` revisadas: sin `continue-on-error` nuevo, sin `|| true`, sin secrets, `shell` explícito (mejora), sin inyección (`${{ matrix.partition }}` es un int de la matrix, no texto de evento). Sin Critical/High
- `node scripts/docs/gen-index.mjs --check` → **exit 0** (759 documentos, `Task files — 222`)
- `node scripts/docs/check-links.mjs` → exit 0 · `node scripts/docs/check-docs.mjs` → exit 0 (`GATING: all clear`, orphans 0)

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Leer el step completo y confirmar las 3 causas contra el run `45564374043` | ✅ |
| 2 | Fix: `shell: bash` + `set -euo pipefail` + `sed` ANSI + parseo anclado al summary + guarda `-z` | ✅ |
| 3 | Probar con el log real (1 positivo + 3 negativos + 2 controles) y pegar las salidas | ✅ |
| 4 | `actionlint` + OCR review + review P2-01 + commit local | ✅ |

## Lo que esta tarea NO hace

- **No** retira el `continue-on-error` del job: exige un run verde de las 4 particiones y es decisión del owner (`FIND-714` step 7).
- **No** toca `dev-tools/tsan-suppressions.txt`, `TSAN_OPTIONS`, el `-E` filterset, `.config/nextest.toml` ni el step de test.
- **No** toca el job entero: solo el bloque `run:` de la aserción y su comentario.
- **No** edita `docs/index.md` ni `llms.txt` a mano: los regenera `gen-index.mjs --write` y el commit los lleva como salida generada (220 → 222 task files, `FIND-730` incluida).
- **No** hace push (política de git: el push es acto explícito del owner).

## Deuda — lo que queda abierto

- **El caso `X/Y` (aborto) sigue sin fixture real.** Ningún run del repo ha abortado una partición desde `FIND-714`, así que la rama `[[ "$SUMMARY" == */* ]]` está probada por construcción del regex, no por un log medido. El primer run que aborte la ejercita; pegarlo aquí entonces.
- **`suppression hits: 0` en la corrida real.** El step lo imprime como señal, y con 0 el propio comentario dice que las dos entradas de suppressions podrían estar muertas. No es asunto de esta tarea (la suppression class de `FIND-714` se evalúa con el run verde completo); queda anotado para no perderlo.
- **La lane sigue siendo no bloqueante** (`continue-on-error`), así que este verificador, ahora que mide, todavía no gatea merges. Es el estado declarado y del owner.
- **`sed` es GNU** (`\x1b` es extensión GNU). Correcto en `ubuntu-latest` y silenciosamente inerte en BSD/macOS: si algún día esta lane corre en macOS, el strip no aplica y el parseo vuelve a fallar — fallaría **cerrado** (exit 1), que es la dirección correcta, pero conviene saberlo.

## Review (P2-01)

- **Reviewer:** `vanta-review`, contexto fresco (sesión `ses_dee3009dfffeXP2qf0GImFeuh4`), sin acceso a la sesión que implementó. Tier Fast (`.github/workflows/**` + `docs/dev/**`), review de agente distinto igualmente.
- **Veredicto inicial:** 🔴 `changes-required` — 1 Critical (indexación) + 2 Optional + 1 Nit. **El contrato de CI lo aprobó explícitamente**: 3 causas cubiertas, `pipefail` real (no heredado), 5/5 casos reproducidos por el reviewer con el script extraído del YAML, `actionlint` exit 0, diff de 1 hunk, `if: always()` intacto, sin `continue-on-error` nuevo, sin `|| true`, `TSAN_OPTIONS`/suppressions/`-E`/step de test sin tocar, frontmatter válido.

### Lo corregido (una ronda)

- **C1 — `docs/index.md` y `llms.txt` no estaban regenerados.** Cierto: la copia que había generado en un worktree la pisó el commit de otro carril (`ecc2df930`, FIND-729) que aterrizó mientras corría el review. Regenerado de nuevo, ya con `FIND-729` commiteado, en el árbol real: `Task files — 222`, `759 documents`, una sola fila nueva (`FIND-730`). Los tres gates de docs dan exit 0.
- **O1 — `grep` sin anclar + `tail -1`.** Aceptado: el conteo ahora se extrae **solo** de la línea de `Summary` (dos etapas). Con el decoy del caso 6, el parseo de una etapa devuelve `9999 tests run` y el de dos etapas devuelve `821 tests run`.
- **O2 — `sed` GNU.** No se cambia el código (la lane corre en `ubuntu-latest`); queda declarado en §Deuda y en el comentario del step.
- **Nit — la guarda `-z` es redundante con el script actual.** Cierto y deliberado: es la defensa contra la causa 2 si alguien vuelve a tocar el `shell:`/`set`, y el caso 5 demuestra que se alcanza sola. Se queda.
