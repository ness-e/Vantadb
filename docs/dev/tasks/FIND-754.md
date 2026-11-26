---
title: "FIND-754 - el check de frescura del CLI corre en el pre-push y avisa, sin cambiar el veredicto"
kind: task
status: completed
description: "`scripts/check-cli-fresh.ps1` detectaba un `vanta-cli` stale y no lo invocaba nada. Ahora el `pre-push` lo corre en modo `-Advisory`: nombra los dos commits si el binario instalado está stale, calla si no hay binario o si `CI` está definido, y nunca cambia el veredicto del push. El job de nightly queda propuesto (§Incógnitas): `nightly.yml` es de otro carril."
tags: [vantadb, cli, githooks, verification]
---

# FIND-754 - el check de frescura del CLI corre en el `pre-push`

## Contexto

La fila del Backlog decía: «`scripts/check-cli-fresh.ps1` detecta un `vanta-cli`
stale, pero no está cableado en CI ni en ningún hook: nadie lo corre». El script
es correcto y tiene matriz medida de seis casos (`FIND-98`), pero un instrumento
que nadie ejecuta es indistinguible de no tenerlo: el `vanta-cli` instalado
seguía respondiendo `vanta-cli 0.9.0 (commit 210bbfaa5)` con `HEAD` en
`2f8ea4d76`, y nada se lo decía a nadie.

## Spec / Decisión

**Elegido: (a) `pre-push` advisory.** **(b) job de `nightly` queda propuesto**, no
hecho: `.github/workflows/**` es de otro carril (`FIND-769` en vuelo en
`docs-gates.yml`). El YAML propuesto está en §Incógnitas.

| Opción | Coste | Detecta el caso | Veredicto |
|---|---|---|---|
| Gate por PR (`ci-*.yml`) | un job más | ❌ el runner **no tiene** `~/.cargo/bin`: compara contra un binario que no existe | ❌ descartado por diseño |
| `pre-push` bloqueante | ~1 s por push | ✅ | ❌ frena pushes legítimos de quien nunca instaló el CLI: el stale lo sufre **quien lo usa**, no quien pushea, y el push no es el momento de arreglarlo |
| `pre-push` **advisory** | ~1 s por push | ✅ (avisa y nombra el fix) | ✅ **elegido** |
| Job de `nightly` con `cargo install` | un job programado | ✅ con el binario correcto | ⏸ propuesto (§Incógnitas) |

Por qué advisory y no bloqueante: el artefacto stale es un problema **local de
consumo** (alguien corre un `vanta-cli` de hace dos meses y sus `--help`/flags no
son los de `HEAD`), no un defecto del código que viaja. Un rojo por «no lo
instalé» es peor que no tener el check: bloquearía a la mayoría de los carriles,
que nunca instalan el CLI. El aviso, en cambio, llega justo a quien tiene el
binario — con los dos commits y el comando de arreglo.

**Distinción fina del modo `-Advisory`** (la razón de tocar el script):

| Estado | Modo default (`FIND-98`) | `-Advisory` (hook) | Salida |
|---|---|---|---|
| binario ausente | exit 2 | exit 0 | **nada** (ni una línea) |
| binario fresco | exit 0 (`OK: …` + aviso de árbol sucio) | exit 0 | **nada** |
| binario stale / sin commit / sha ajeno | exit 1 | **exit 0** | `STALE: …` + los dos commits + `cargo install --path … --force` |
| binario rotundo (crash, sin salida) | exit 2 | exit 0 | `CANNOT CHECK: … exited N without output` |
| `CI` definido | — | — | el hook **no lo llama**: un runner no tiene el binario |

La matriz 0/1/2 queda **intacta** para quien quiera el veredicto (`pwsh
scripts/check-cli-fresh.ps1` sin switch): el switch solo neutraliza el *código* y
calla los dos casos sin hallazgo (ausente y fresco). El hook llama el script
**tal cual** — no reimplementa la comparación de commits.

**Por qué advisory calla también el caso fresco** (hallazgo medium de la ronda
P2-01): el único camino que llegaba al aviso de árbol sucio era el fresco, así
que quien hacía lo que el mensaje mandaba (`cargo install --force`) quedaba con
un `ADVERTENCIA: the repository has uncommitted changes` en **cada** push — un
falso positivo permanente (4 carriles dejan el árbol sucio siempre) bajo una
etiqueta localizada. Un advisory que habla cuando no hay nada que decir es un
advisory que se ignora, que es justo la falla que esta tarea corrige.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.githooks/pre-push` (343 líneas: `SKIP`
  granular, guarda `CI`, bloque advisory de `temp-collector`, `winpath`,
  `find_pwsh`), `scripts/check-cli-fresh.ps1` (113 líneas, seis salidas),
  `docs/dev/tasks/FIND-98.md`, `docs/dev/avance/activo/operaciones.md`
  §`FIND-98`, `docs/dev/workflow/RULES.md` §10, `dev-tools/verify.ps1`
  (`param([switch]$IncludeCoverage, [string[]]$Skip)` + `Skip-Step`).
- **Referencias hacia dentro:** nadie invocaba `check-cli-fresh.ps1`
  (`rg -n 'check-cli-fresh' .github/ .githooks/ dev-tools/ justfile` → solo el
  script mismo). `.githooks/pre-push` lo usan **todos los carriles** vía
  `core.hooksPath`.
- **Referencias hacia fuera del hook:** `dev-tools/{verify,fmt-range,check-unix,
  temp-collector}.ps1`, `node dev-tools/build-wasm-types.mjs`, `npx
  markdownlint-cli2`, `actionlint`, `cargo fmt` por manifest.
- **Veredicto:** el cambio del hook es **puramente aditivo** (`git diff -U0` no
  tiene ni una línea `-`), así que ningún camino de bloqueo existente se toca; el
  del script está confinado al `param` y a `exit` → `Exit-Check`. No se toca
  `src/**`, `.github/**`, `vanta-memory/**`, `vantadb-mcp/**`.

## Steps

1. ✅ `scripts/check-cli-fresh.ps1`: `-Advisory` + `Exit-Check` (un solo punto de
   salida); el caso «binario ausente» calla en advisory.
2. ✅ `.githooks/pre-push`: bloque advisory tras el recolector de temporales,
   guarda `CI`, llama el script tal cual y prefija su salida.
3. ✅ Prueba de tres caras sobre el hook real (§Resultado) + `bash -n`.
4. ✅ Docs: `RULES.md` §10, esta ficha, `avance`.

## Resultado

### Prueba de tres caras (hook real, no un extracto)

Corrida del hook de verdad (`.githooks/pre-push` completo, con `stdin` de git y
el rango `HEAD~1..HEAD`, 3 ficheros de docs) bajo **Git Bash**, controlando el
`PATH` para que la resolución de `vanta-cli` sea la de cada cara. Los gates
pesados van con `SKIP=` para que la corrida no compile (carril C: otro carril
compila); el veredicto del hook es el que se mide.

```
# 1) binario fresco (declara HEAD 2f8ea4d76) -> sin línea del check, push NO cambia
probe: /c/Users/Eros/.cargo/bin/vanta-cli
pre-push: 3 archivos en el rango (lo que viaja).
pre-push: sin Rust en el rango — gates de Rust no corren (CI los cubre en develop).
pre-push: barrier passed. Push allowed.
exit=0

# 2) binario stale (el instalado de verdad: declara 210bbfaa5) -> AVISO con los
#    dos commits, push NO se bloquea
probe: /c/Users/Eros/.cargo/bin/vanta-cli
pre-push: CLI freshness ADVISORY (does not block this push):
  STALE: the installed vanta-cli is behind HEAD.
    binary declares: 210bbfaa5 (a different commit)
    repo HEAD:       2f8ea4d76
    binary:          C:\Users\Eros\.cargo\bin\vanta-cli.exe
    fix:             cargo install --path "C:/Users/Eros/VantaDB Proyect/VantaDB" --bin vanta-cli --force
pre-push: barrier passed. Push allowed.
exit=0

# 3) sin binario (PATH sin ~/.cargo/bin) -> skip limpio, cero ruido
probe: (no vanta-cli on PATH)
pre-push: 3 archivos en el rango (lo que viaja).
pre-push: sin Rust en el rango — gates de Rust no corren (CI los cubre en develop).
pre-push: barrier passed. Push allowed.
exit=0

# 4) CI=1 con el binario stale presente -> también se calla
probe: /c/Users/Eros/.cargo/bin/vanta-cli
pre-push: 3 archivos en el rango (lo que viaja).
pre-push: barrier passed. Push allowed.
exit=0
```

### Matriz del script, modo default vs `-Advisory` (medida, binarios falsos y reales)

| Caso | default | `-Advisory` | Salida en advisory |
|---|---|---|---|
| fresco (`.cmd` que declara `HEAD`) | 0 | 0 | *(nada: 0 caracteres)* |
| stale (real, `210bbfaa5`) | **1** | **0** | `STALE: … behind HEAD` + ambos commits |
| crash (`exit /b 7`, sin salida) | **2** | **0** | `CANNOT CHECK: … exited 7 without output` |
| ausente (PATH sin `~/.cargo/bin`) | **2** | **0** | *(nada)* |

Los cuatro códigos del modo default son los de la matriz de `FIND-98`
(`1`/`2`/`0`): el switch **no** cambia el contrato existente, solo su consumo.

### Verificación

- `bash -n .githooks/pre-push` → 0.
- `git diff -U0 .githooks/pre-push` → **solo líneas `+`**: ningún camino de
  bloqueo existente se modificó (el repo entero depende de ese hook).
- Las cuatro corridas de arriba: el hook llega a `barrier passed` y sale 0 con el
  binario fresco, stale, ausente y con `CI=1`.
- `pwsh scripts/check-cli-fresh.ps1` (default) → exit **1** con el binario stale
  instalado: el contrato `FIND-98` sigue midiendo.
- `node scripts/docs/check-links.mjs` → **0** · `node scripts/docs/check-docs.mjs` → **0** (gating all clear).
- `node scripts/docs/gen-index.mjs --check` → **1 (ROJO, esperado y no
  regenerado aquí)**: `docs/index.md` cuenta 713/261 documentos contra 715/263
  esperados — los dos task files nuevos son éste y `FIND-769.md`. **No se corre
  `--write` en esta tarea** porque `docs/index.md` es **compartido con el carril
  de `FIND-769`** y el que regenere primero se lleva el diff del otro; lo hace el
  orquestador al agrupar. El `push` a `develop` **no** puede aterrizar sin ese
  paso: `docs-gates.yml` tiene `docs-index` como check *required*.

## Incógnitas / deuda

- **Job de `nightly` (b) propuesto, no aplicado**: `.github/workflows/**` es de
  otro carril (`FIND-769` toca `docs-gates.yml`). El YAML listo para pegar en
  `nightly.yml` (instala el binario y corre el check en modo default, ese sí
  falla):

  ```yaml
  # FIND-754: el binario instalado declara su commit (FIND-98); el check lo
  # exige == HEAD. Acá SÍ se puede: el job instala el CLI antes de medirlo.
  cli-freshness:
    runs-on: ubuntu-latest
    timeout-minutes: 25
    steps:
      - uses: actions/checkout@<SHA fijado> # v4.x — pin SHA, RULES.md §3
        with: { fetch-depth: 0 }
      - run: cargo install --path . --bin vanta-cli
      - run: pwsh -NoProfile -File scripts/check-cli-fresh.ps1
  ```

  Coste estimado: un build del binario (`cargo install`), no del workspace.
  Mientras no exista, el único call site es el `pre-push` advisory, que es lo que
  pedía el contrato de la fila (`rg 'check-cli-fresh' .github/ .githooks/` → 1).
- **El aviso depende de que el carril tenga el binario**: quien nunca instaló
  `vanta-cli` no verá nada, y eso es deliberado (ver §Spec). La señal universal
  sería el job de nightly.
- **`-Advisory` neutraliza también el exit 2 de «no es un checkout git»**: en el
  hook el cwd siempre es el repo, así que ese caso no aporta veredicto. Si algún
  día el script se llama desde fuera, el mensaje `CANNOT CHECK` sigue saliendo
  con la razón.

- **`-Advisory`, error terminante y `|| true`:** `-Advisory` garantiza exit 0 en
  **todos los caminos que el script maneja** (los `Exit-Check`). Un error que
  lanza **antes** de llegar a uno —p. ej. `git` ausente del `PATH`— sigue saliendo
  ≠0 (`$ErrorActionPreference = "Stop"`); en el hook es inocuo porque la llamada
  va con `|| true`, y por eso el comentario del `param` lo dice explícitamente en
  vez de prometer «siempre 0». El job de nightly propuesto usa el modo **default**
  (donde ≠0 es el veredicto deseado), así que no hereda la ambigüedad.
- **Job de nightly sin fila propia todavía:** la ronda P2-01 sugirió registrarlo
  como fila `FIND-*` en vez de dejarlo sólo como YAML aquí, para que no envejezca
  como deuda invisible. No se creó: el Backlog es de este carril **sólo para su
  propia fila** (`-1`, `FIND-754`). Queda como `queda_pendiente` para el
  orquestador.

## Skills (SDP)

`campaign_discover_skills_v2` (phase=BUILD, archivosClave
`.githooks/pre-push, scripts/check-cli-fresh.ps1`, keywords `advisory gate`,
`pre-push`, `stale binary`, `skip clean`) → base (`campaign-executor`,
`progreso`) + `incremental-implementation`, `test-driven-development`,
`context-engineering`, `source-driven-development`, `doubt-driven-development`
(descartada `frontend-ui-engineering`: no hay `web/` en este carril) +
`documentation-skill` (obligatoria por tocar `docs/**`).

## Review (P2-01)

- **Revisor:** `vanta-review` (sub-agente, sesión propia, sin haber implementado).
- **Enfoque:** contrato declarado verificado **en vivo** por el revisor, no por
  auto-reporte: `bash -n`; aditividad con `git diff -U0 --numstat` (0 líneas `-`,
  +23); los seis casos de la matriz × 2 modos (fresco, stale real, `commit
  unknown`, sha ajeno, crash, ausente); las 4 caras del hook real (`core.hooksPath`
  → `.githooks`); `git grep check-cli-fresh` → 1 call site; auditoría de los
  caminos de bloqueo uno por uno (`$PS_EXE` resuelto y abortado en `:91-95`, el
  `|| true`, `winpath` entre comillas con la ruta que **tiene un espacio**).
- **Veredicto ronda 1: 🔴 CHANGES-REQUIRED** (0 critical · 1 high · 2 medium ·
  3 low · 1 nit). El código pasó; lo que no cerró fue el borde del changeset.
- **Qué se aplicó:**

| Finding | Acción |
|---|---|
| [high] `gen-index --check` → **1** (required `docs-index`): el DoD decía «0» con un 1 real | ✅ **medido y nombrado como rojo esperado** en §Verificación, con el motivo (2 task files nuevos, `docs/index.md` compartido con `FIND-769`) y el paso que falta (orquestador). `--write` **no** se corre acá: es compartido y el que regenere primero se lleva el diff del otro. |
| [medium] `Backlog.md` no era `-1` sino `-7`: 6 líneas de `FIND-769` ya borradas en el working tree por su carril | ✅ el commit **no** lleva el working copy: se reconstruyó el blob como `HEAD` − sólo mi fila (`git show HEAD:…` + `hash-object` + `update-index --cacheinfo`), y la verificación de contenido confirma 0/1. La supresión ajena queda **sin stagear**, que es donde pertenece. |
| [medium] el advisory hablaba justo cuando no había nada que decir (banner `ADVERTENCIA` de árbol sucio en cada push, único camino: el fresco) | ✅ el camino fresco es **silencio** en `-Advisory` (y con él el aviso de árbol sucio, que es tree-wide y en este repo es un falso positivo permanente). |
| [low] «ALWAYS exit 0» era falso ante un error terminante (`git` fuera del `PATH` → 1) | ✅ el comentario del `param` ahora distingue «todos los caminos con `Exit-Check`» de «error terminante: lo absorbe el `|| true` del hook». |
| [low] la transcripción de la cara fresca omitía una línea real (`ADVERTENCIA`) | ✅ transcripciones **literales** de la corrida post-fix (la cara fresca ya no imprime nada). |
| [low] `status: completed` con §Review vacío y puntero a una sección inexistente | ✅ este bloque + el bullet `Review P2-01` en `avance/activo/operaciones.md`. |
| [nit] `exit 0` final no pasaba por `Exit-Check` (invariante del comentario) | ✅ `Exit-Check 0`. |
| [nit] `printf … \| sed` sin `|| true` dentro de `set -e` (EPIPE abortaría el push) | ✅ `|| true` en la tubería: «advisory nunca bloquea» queda estructural, no accidental. |
| [nit] el bullet advisory de `RULES.md` estaba después de *Por qué* | ✅ movido entre *Must not* y *Por qué*. |

- **Ronda 2:** no hubo, sobre el mismo diff; el revisor dejó escrito que con los
  dos cambios requeridos (registro del rojo de `gen-index` y staging por hunk de
  `Backlog.md`) el cambio pasa a APPROVE **sin re-revisión de código**. Ambos
  están aplicados y re-medidos por el autor (`bash -n` 0; matriz completa; 4 caras
  0; contenido del commit verificado).

## Commit

`<pendiente>` — commit local, sin push (política del owner, Regla 7).
