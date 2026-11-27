---
title: "FIND-789: el gate de artefactos no fallaba por los artefactos, sino por cómo los pedía"
kind: task
status: completed
description: "Los dos jobs rojos fallaban por su invocación, no por artefactos ausentes: `@('-Smoke')` se enlazaba posicionalmente a `[string]$Tag` (`FAIL invalid tag '-Smoke'`) y el job del tarball llamaba `gh release list` sin checkout (`fatal: not a git repository`). Arreglado con splatting por nombre y `-R`; sobre v0.9.0 el gate da ALL GREEN."
---

# FIND-789: el gate de artefactos no fallaba por los artefactos

## Metadata

- **Fuente:** `docs/dev/Backlog.md` §`FIND-789` (fila retirada al cerrar)
- **Esfuerzo:** 🟠 2h
- **Prioridad:** 🔴 Alta
- **Tipo:** GitHub Actions (`.github/workflows/release-verify.yml`)
- **Creado:** 2026-11-26
- **Estado:** 🟡 IMPLEMENTADO, VERIFICADO Y COMMITEADO — **no cerrado**. La revisión vigente es `changes-required` (rondas 1 y 2), así que no se marca `completed`: espera **ronda 3** o **aceptación explícita del owner**. Implementación en `6e4119c8d` + `d15564b06`; la fila `FIND-788` que mi commit arrastró fue restaurada en `43b8a7006` (por el orquestador). Ver §Review.
- **Archivo tocado en producción:** `.github/workflows/release-verify.yml` (exclusivo de este carril)

## Impacto mapeado (Regla 0)

- **Leído completo:** `.github/workflows/release-verify.yml` (78 líneas, antes del cambio), `scripts/verify-release.ps1` (`param()` `:27-31` y la resolución de tag `:48-59`), `.opencode/rules/release-ci.md`, `AGENTS.md` §"Release: 7 trampas que tumbaron el 0.9.0".
- **Referencias hacia dentro:** `release-verify.yml` no es invocado por ningún otro workflow (verificado: 0 apariciones de `release-verify` bajo `.github/`). Su consumidor es el humano vía `workflow_dispatch` y el cron de `main`.
- **Referencias hacia fuera:** `scripts/verify-release.ps1` (param `-Tag`, `-Smoke`, `-Repo`) + `release-manifest.json` (lista de assets/registros). **Ninguno se modificó** — el contrato del script se respeta.
- **Veredicto:** cambio contenido al fichero YAML; el script y el manifiesto quedan intactos, así que ningún otro consumidor cambia de comportamiento.

## Diagnóstico: los dos jobs, con su línea de log

Run `45253495978` (`2026-11-23T19:52:50Z`, `event=schedule`), dos jobs, ambos `failure`:

```
gh run view 45253495978 --json jobs --jq '.jobs[] | {name, conclusion}'
→ Verify release artifacts      failure   (id 137969075007)
→ Tarball builds from crates.io failure   (id 137969075379)
```

### Job `Verify release artifacts` → **bug de invocación (YAML)**

```
$ gh run view --job 137969075007 --log-failed
2026-11-23T19:53:02.4432211Z FAIL invalid tag '-Smoke' (expected vX.Y.Z)
2026-11-23T19:53:02.6803484Z ##[error]Process completed with exit code 1.
```

Ese mensaje lo emite `scripts/verify-release.ps1:57`, cuya firma es `param([string]$Tag, [switch]$Smoke, [string]$Repo)`. El workflow le pasaba los argumentos con **splatting de array**:

```powershell
$scriptArgs = @("-Smoke")
if ($env:TARGET_TAG) { $scriptArgs += @("-Tag", $env:TARGET_TAG) }
./scripts/verify-release.ps1 @scriptArgs
```

El splatting de **array** enlaza **posicionalmente**, no por nombre: el elemento `"-Smoke"` cae en el primer parámetro posicional, que es `[string]$Tag`. Reproducido en local con un calco del `param()`:

```
$a=@("-Smoke"); & { param([string]$Tag,[switch]$Smoke) "Tag=[$Tag] Smoke=[$Smoke]" } @a
→ Tag=[-Smoke] Smoke=[False]
```

El script recibe el tag `-Smoke`, no casa con `^v(\d+\.\d+\.\d+)$` y sale con `FAIL invalid tag`. **Nunca llegó a mirar un solo artefacto.**

### Job `Tarball builds from crates.io` → **bug de invocación (YAML)**

```
$ gh run view --job 137969075379 --log-failed
failed to run git: fatal: not a git repository (or any of the parent directories): .git
##[error]Process completed with exit code 1.
```

Ese job **no tiene `actions/checkout`** (a diferencia del primero) y su primer step resuelve la versión con `gh release list`, que deduce el repositorio del directorio git — que ahí no existe. Es exactamente la trampa nº 6 de `AGENTS.md`: *"`gh workflow run` sin checkout → `-R "${{ github.repository }}"`"*. Reproducido desde un directorio no-git:

```
$ cd $env:TEMP/notagit; gh release list --limit 1 --json tagName --jq '.[0].tagName'
→ failed to run git: fatal: not a git repository (or any of the parent directories): .git
$ gh release list -R ness-e/Vantadb --limit 1 --json tagName --jq '.[0].tagName'
→ v0.9.0
```

### Lo que NO era (descartado con evidencia, no por descarte)

| Hipótesis | Veredicto | Evidencia |
|---|---|---|
| Artefacto borrado | ❌ | Los 14 assets del release v0.9.0 responden (`OK` ×14 en la corrida local) |
| Nombre de artefacto cambiado | ❌ | El job 1 ni siquiera resolvió el tag; el manifiesto se renderizó OK al arreglar la invocación |
| Paquete sin publicar en el registro | ❌ | 15/15 registros `is live` 0.9.0 (crates.io / PyPI ×11 / npm ×3) |
| Tarball que no compila desde crates.io | ❌ | El job moría **antes** de `cargo install`, en la resolución del tag |
| Token / credencial | ❌ | No hay ningún 401/403 en ninguno de los dos logs |

**Y lo decisivo:** el release que el gate dice verificar **existe y está completo**. Corriendo el mismo script que corre el job, con la invocación arreglada:

```
Verifying release v0.9.0 (version 0.9.0) - ness-e/Vantadb
  OK   … 14 assets (5 binarios + sus 5 .sha256 + 4 wheels) …
  OK   crates.io vantadb 0.9.0 is live · PyPI vantadb-py 0.9.0 is live · npm vantadb … → 15/15
Smoke: OK sha256 verified · OK vanta-cli --version -> vanta-cli 0.9.0
ALL GREEN - release v0.9.0 verified (version 0.9.0).
```

O sea: **no es un gate adelantado a un publish que falta — es un gate que miente desde hace cinco semanas.** Por eso se arregla el código; relajarlo habría escondido un gate que ya estaba en verde.

## La causa raíz de que nadie lo viera

`gh run list --workflow=release-verify.yml --limit 5` devuelve **5/5 `failure`** (26-oct, 2-nov, 9-nov, 16-nov, 23-nov — todos lunes). No es una regresión del 23-nov: es el mismo rojo semanal desde al menos el 26-oct. El 29-oct se **quitó el cron** en `develop` (`bbd380ea0`, "PRE-PUB dispatch-only") para dejar de verlo rojo — sin leer el log, que decía `invalid tag '-Smoke'`.

**Y ese arreglo del trigger nunca llegó a `main`:** GitHub dispara `schedule` desde la rama por defecto, y `origin/main` **todavía tiene** `cron: '0 8 * * 1'`. Por eso los lunes siguieron rojos después del 29-oct. `git diff origin/main -- .github/workflows/release-verify.yml` contiene exactamente el cambio de trigger **más** los dos arreglos de este ticket: el merge `develop → main` es lo que los activa.

## El arreglo

Dos líneas de lógica, ninguna relajación:

```yaml
# job 1 — splatting por nombre (hashtable), no por posición (array)
$params = @{ Smoke = $true }
if ($env:TARGET_TAG) { $params['Tag'] = $env:TARGET_TAG }
./scripts/verify-release.ps1 @params

# job 2 — gh no puede inferir el repo sin checkout
TAG="$(gh release list -R "$GITHUB_REPOSITORY" --limit 1 --json tagName --jq '.[0].tagName')"
```

Sin `continue-on-error`, sin `|| true`, sin borrar ningún job, sin tocar el umbral del script ni `release-manifest.json`. Los dos jobs siguen teniendo que pasar para que el workflow esté verde.

## Verificación

| Comprobación | Comando | Resultado |
|---|---|---|
| Job 1 (la causa) | calco del `param()` con splatting de array | `Tag=[-Smoke] Smoke=[False]` — reproduce el rojo |
| Job 1 (el arreglo) | `./scripts/verify-release.ps1 @params` con `@{Smoke=$true}` | **ALL GREEN** v0.9.0 (14 assets + 15 registros + sha256 + `vanta-cli 0.9.0`) |
| Job 2 (la causa) | `gh release list` desde `%TEMP%\notagit` | `failed to run git: fatal: not a git repository` — reproduce el rojo |
| Job 2 (el arreglo) | `gh release list -R ness-e/Vantadb …` | `v0.9.0` |
| Job 2 (el build) | `cargo install vantadb --vers 0.9.0 --locked --root …` + `vanta-cli --version` | `Finished release profile in 8m 40s`, `Installed package vantadb v0.9.0`, `vanta-cli 0.9.0` (Windows/MSVC local — el job corre en `ubuntu-latest`) |
| YAML | `actionlint 1.7.12` (con y sin `-shellcheck=`) | `exit=0` en ambos modos |
| Delta vs `main` | `git diff origin/main -- …/release-verify.yml` | trigger + comentarios + los 2 arreglos, nada más |

**No verificado, declarado:** el workflow **no se ejecutó en CI**. El bloqueo real no es la rama: el cambio **ya está commiteado** (`6e4119c8d` + `d15564b06`) y sólo falta el **push**, que es del owner (Regla 7). **Prueba de campo asignada:** tras el merge `develop → main`, `gh workflow run release-verify.yml -f tag=v0.9.0` (o el step post-release de `docs/dev/workflow/PUBLISH.md`) debe salir verde con los dos jobs; lo dispara quien haga el merge, no este carril. Hasta entonces el rojo de los lunes seguirá viniendo de la copia de `main`, idéntica salvo estos dos arreglos.

## Review

**P2-01 · `vanta-review`, contexto fresco — cuatro rondas (2026-11-26).** El revisor no participó en el cambio. **Ronda 1: 🔴 `changes-required`** (el cambio técnico aprueba, el cierre no) · **Ronda 2: 🔴 `changes-required`** (8 de 9 cerrados, H1 abierto por un commit que retiró la fila de otro carril) · **Ronda 3: 🔴 `changes-required`** (5 de 6; el bloqueante era una afirmación falsa heredada del orquestador: "fila restaurada verbatim") · **Ronda 4: ✅ `approve`** (6/6 en el árbol sobre `8dc536de7`).

**Veredicto de la ronda 1: 🔴 `changes-required`** — el cambio **técnico** aprueba, el **cierre** no. **Veredicto de la ronda 2: 🔴 `changes-required`** — 8 de 9 hallazgos cerrados, **H1 abierto por culpa de mi commit** (abajo). Lo que sigue es del revisor, no del autor; este fichero **no se marca `completed`**: espera **ronda 3** o **aceptación explícita del owner**.

**Enfoque:** correcto. Los dos rojos son de invocación (splatting posicional y `gh` sin repositorio), no de artefactos; el arreglo es código y no relaja el gate.

**Cómo se probó (re-ejecutado por el revisor, no auto-reporte):**

- `param([string]$Tag,[switch]$Smoke)` con `@("-Smoke")` → `Tag=[-Smoke] Smoke=[False]`; con `@{Smoke=$true}` → `Tag=[] Smoke=[True]`.
- `gh run view --job 137969075007 --log-failed` → `2026-11-23T19:53:02.4432211Z FAIL invalid tag '-Smoke' (expected vX.Y.Z)`, verbatim.
- `gh run view --job 137969075379 --log-failed` → `failed to run git: fatal: not a git repository (or any of the parent directories): .git`, verbatim.
- Desde `%TEMP%` no-git: `gh release list` → el mismo fatal (exit 1); con `-R` y `GITHUB_REPOSITORY=ness-e/Vantadb` → `v0.9.0`.
- `./scripts/verify-release.ps1 @{Smoke=$true}` (sin `-Tag`, el camino del dispatch vacío) → `ALL GREEN - release v0.9.0 verified`, 37,5 s, exit 0: **14 assets** `OK`, 15 registros `is live`, `sha256 verified`, `vanta-cli 0.9.0`.
- `git diff origin/main -- .github/workflows/release-verify.yml` = trigger + comentarios + los 2 arreglos, nada más.
- `actionlint 1.7.12` y `actionlint -shellcheck=` → exit 0 en ambos.
- `gh run list --workflow=release-verify.yml --limit 5` → 5/5 `failure` (`schedule`, 26-oct → 23-nov); `git show bbd380ea0` quita `schedule:` y `origin/main` todavía lo tiene.
- **Sin relajación, verificado:** sin `continue-on-error`, sin `|| true`, sin jobs borrados, `scripts/verify-release.ps1` y `release-manifest.json` idénticos a `HEAD`.

**Hallazgos de la ronda 1 y qué se hizo:**

- **C1 (crítico) — el veredicto estaba escrito antes de que hubiera revisor.** Esta sección afirmaba `approve` y el fichero ya decía `status: completed`; el revisor es el primero de este cambio y no existía registro que lo sostuviera. Es el modo de fallo de `FIND-118`: un gate pre-satisfecho por aserción. **Aplicado:** reemplazada por este dictamen real; metadata corregida.
- **H1 (alto) — la fila del Backlog resucitó.** El carril de `FIND-782` reescribió `Backlog.md` completo a las 23:10:35 con una copia previa y revirtió el retiro de mi fila (el mismo choque de `RULES.md` §11 / `FIND-753`). **Ronda 1: aplicado** (retiro re-hecho y verificado con `-ExpectAbsent`). **Ronda 2: sigue abierto, y la causa es mi commit `d15564b06`** — ver abajo.
- **M1 (medio) — el doc decía un arreglo que el código no usa:** `-R "${{ github.repository }}"` vs el real `-R "$GITHUB_REPOSITORY"`. **Aplicado** (aquí y en `ci-cd.md`).
- **M2 (medio) — "12 assets" era falso: son 14.** La corrida imprime 14 `OK`; `release-manifest.json` da 5 binarios (4 `tar.gz` + 1 `zip`) + 5 `.sha256` + 4 wheels = 14. El desglose anterior además sumaba 18. **Aplicado** en los dos ficheros.
- **M3 (medio) — la prueba de campo quedaba declarada sin dueño.** **Aplicado:** asignada al merge (arriba, §Verificación).
- **M4 (medio) — la deuda del `case` era un falso defecto.** `case` usa globs: los puntos ya son literales y `v1x2x3` se rechaza (medido). La holgura real es `[0-9]*`. **Aplicado:** deuda reescrita.
- **L1 — motivo impreciso del "no se pudo disparar"** (el bloqueo es push owner-gated, no la rama). **Aplicado.**
- **L2 — rangos de línea del script corridos:** `param()` es `:27-31` y la resolución de tag empieza en `:48`. **Aplicado.**
- **L3 — `docs/dev/workflow/PUBLISH.md` queda contradiciendo este cambio.** Fuera de mi lista de escritura → declarado en §Deuda.
- **Nits:** "trigger + comentarios + arreglos" (aplicado); `FIND-789.md` queda huérfano de inbound link (reporting-only en `check-docs`, 2/811) — aceptado.

**No verificado, declarado:** el workflow **no se ejecutó en CI** (push owner-gated) — no se infiere verde; y el tarball de 0.9.0 se compiló en **Windows/MSVC local** con el comando exacto del job, no en el runner `ubuntu-latest`.

### Ronda 2 — 🔴 `changes-required` (8 de 9 cerrados, H1 abierto)

Corrida por el orquestador sobre `6e4119c8d` + `d15564b06`. **Cerrados y verificados por el revisor:** C1, M1, M2, M3, M4, L1, L2, L3. C1 explícitamente: el doc ya registra `changes-required` sin auto-aprobación. M4 re-medido en `bash`: `v1x2x3` reject, `v1a2a3` reject, `v1.2.3.4` accept — el doc dice la verdad.

**H1 sigue abierto, y la causa es mía — `d15564b06`:**

```
git show d15564b06 -- docs/dev/Backlog.md   →  -| `FIND-788` | …   ← arrastró DOS filas
git grep '| `FIND-788` |' HEAD              →  0
```

Mi commit de retiro de fila es **por path**, así que horneó también la eliminación de la fila `FIND-788` que otro carril tenía sin commitear. `FIND-788` está **vivo** (task file `completed`, `certification.yml` en `M`, listado en W10): retirarle la fila fue una reversión silenciosa de trabajo ajeno, la clase que `RULES.md` §11 y `FIND-753` existen para evitar — y la puerta que el helper **no** cierra, porque `-ExpectAbsent` sólo prueba lo que yo declaré ausente. **Reparación, con su cadena real (medida por bytes, no por PowerShell):** `43b8a7006` restauró la fila **doble-codificada** —el texto pasó por PowerShell, que leyó UTF-8 como CP437 y lo re-codificó: el emoji de severidad quedó como una secuencia de cuatro caracteres CP437 y cada vocal acentuada como dos, con el prefijo de dibujo de caja delante—, así que la fila midió **1428 chars / 1519 bytes** frente a **1396 chars / 1428 bytes** de la genuina (`d15564b06^`): el "1428" que se citó como *chars* era la **longitud en bytes del mojibake**. Luego `560ff7e8a` cerró `FIND-788` de verdad y `932f8137d` retiró la fila **legítimamente**. `HEAD` queda con **0 filas `FIND-788`, 0 mojibake y 2 menciones textuales** (en las filas `FIND-792` `:893` y `FIND-794` `:905`). La trampa ya la había declarado el carril de `FIND-788` en `bf3671d81` ("record the three hops of the backlog row and the encoding trap"). Este carril no la vuelve a tocar.

**Lección de mecanismo (para el próximo carril con fichero caliente):** un commit por path en un fichero que otros carriles editan **arrastra sus hunks**. No se detecta con `-ExpectPresent`/`-ExpectAbsent` (miran sólo lo declarado): exige inspeccionar `git show <commit> -- <path>` **después** del commit y confirmar que cada `-`/`+` es propio. Es exactamente lo que se hizo acá para encontrar H1.

**Estado del cierre:** implementación y verificación completas, revisión **no aprobada**. `status: active`, no `completed` — espera **ronda 3** o **aceptación explícita del owner**.

## Deuda y NOTICED BUT NOT TOUCHING

- **`case "$TAG" in v[0-9]*.[0-9]*.[0-9]*)`** (job 2): `case` usa **globs, no regex**, así que los puntos ya son literales — `v1x2x3`, `v...` y `v1.2.` se rechazan (medido con `bash`: sólo `v0.9.0`, `v1x.2.3` y `v1.2.3.4` casan). La holgura real es `[0-9]*` = "un dígito y luego cualquier cosa", así que `v1x.2.3` pasa; como el probe seguiría 404 → `::warning` → build salteado, el job puede ir **verde sin haber compilado nada**. Es entrada del owner y preexiste a este ticket: se declara, no se toca.
- **`scripts/verify-release.ps1` defaultea `-Repo ness-e/Vantadb`** hardcodeado en vez de recibir `$GITHUB_REPOSITORY`. Fuera del contrato de esta fila (el script no es mío) y hoy correcto porque es el único repo que lo usa.
- **`docs/dev/workflow/PUBLISH.md`** sigue anunciando que el workflow "también corre semanalmente contra `releases/latest`" y "los 2 assets binarios": las dos cosas son falsas con este cambio (14 assets, dispatch-only desde `bbd380ea0`). Fuera de mi lista de escritura → queda como fila `FIND-*` para quien tenga ese fichero.
- **El cron sigue activo en `main`** y sólo se desactiva con el merge `develop → main` (owner, Regla 7). Hasta entonces el lunes rojo es esperado y ya no es un misterio: es la copia vieja del fichero.
- **`Ver: … FIND-789` colgante en la fila `FIND-790` (`Backlog.md`) — decisión: se queda, y se declara acá, no sólo en `ci-cd.md`.** Los dos caminos eran quitarlo o justificarlo; se justifica porque (a) `Ver:` es una relación informativa hacia un ID **sin fila en el Backlog**, no un enlace que resuelva a un fichero, así que no se rompe nada, y (b) el `Backlog.md` es justo el fichero caliente que en esta misma tarea ya horneó hunks ajenos dos veces (`6e4119c8d` arrastró el retiro de `FIND-775`, `d15564b06` el de `FIND-788`, reparado en `43b8a7006`): editarlo por un `Ver:` cosmético es exactamente el riesgo que no se justifica. **Quien cierre `FIND-790` lo actualiza en su propio commit**, que es donde la fila se toca de todos modos.

### Ronda 4 — ✅ `approve` (cierre)

Revisor con contexto fresco, verificando **por bytes** y no por la salida de PowerShell. 6/6 puntos: la frase falsa desapareció y la cadena `43b8a7006` → `560ff7e8a` → `932f8137d` quedó auditable salto por salto; `ci-cd.md` corregido en una línea (numstat `1/1`); el `Ver:` justificado como "ID **sin fila en el Backlog**"; el commit `8dc536de7` con 3 hunks, todos propios; **0 mojibake nuevo** (la única secuencia en `ci-cd.md:481` es de `db6ad7f79a`, 8 h antes); `status: active` honesto.

**Nits anotados, no bloquean:** (a) las menciones están en `:892`/`:904`, no `:893`/`:905` — off-by-one de cita, no de contenido; (b) `1396 chars` son **unidades UTF-16** (en code points, 1392 — que es justamente lo que reporta PowerShell, la herramienta que la frase dice no haber usado); el número que sostiene el argumento son los **bytes: 1428 vs 1519**. **No "arreglar" 1396 a 1392**: rompería la simetría con la fila de 1428 bytes.

**Veredicto final: `approve`.** Con esto la fila se cierra. Queda del lado del owner: el **push** (Regla 7) y, tras el merge a `main`, la prueba de campo `gh workflow run release-verify.yml -f tag=v0.9.0`.
