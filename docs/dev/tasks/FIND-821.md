---
title: "FIND-821 - `.opencode` es un repo git separado sin hooks: sus commits saltaban el gate entero"
kind: task
status: completed
description: "El repo del harness (working tree `.opencode/`, gitdir `.git/modules/.opencode`) no tenía pre-commit: el helper avisaba y commiteaba igual. Ahora tiene `.githooks/pre-commit` con DOC-04 y el harness contract, instalado con `core.hooksPath`, y por tanto el aviso desaparece."
tags: [vantadb, harness, git-hooks, opencode, gates, find-821]
---

# FIND-821 - el repositorio que define los gates no los aplicaba a sí mismo

## Contexto

`.opencode/` es un repositorio git **separado** (remoto privado `configOpencode`)
cuya working tree es ese directorio:

```text
$ cat .opencode/.git
gitdir: ../.git/modules/.opencode
$ git ls-files .opencode | wc -l          # en el padre
0
```

El padre lo ignora entero (`.gitignore:208`), así que **los hooks del padre no lo
alcanzan**: el padre instala los suyos con `git config core.hooksPath .githooks`
(`.githooks/pre-commit:3`), y eso es config del repo del padre.

Medido dos veces hoy al commitear `AGENTS.md`:

```text
WARNING: no pre-commit hook under .../.git/modules/.opencode/hooks - committing WITHOUT the gate
```

y el commit se creaba. O sea: **el repositorio que define los gates del proyecto
—prompts del pipeline, agentes, skills, el helper de commit, los chequeos de
contrato— se commiteaba sin ninguno**.

## Contrato — y por qué se eligió (a)

La fila ofrecía dos caminos:

- **(a)** instalar los mismos hooks en el submódulo;
- **(b)** que el commit del padre **verifique el SHA del submódulo** con
  `check-harness-contract.mjs` antes de aceptarlo.

**Elegido: (a).** (b) **no es aplicable tal como está escrito**: el padre no tiene
gitlink del submódulo (`git ls-files .opencode` → 0, no hay `.gitmodules`), así que no
hay SHA que verificar en el índice del padre. Construir esa vigilancia exigiría
convertir `.opencode` en submódulo real (decisión de arquitectura del owner, y
`.opencode/` está deliberadamente fuera del repo por política). (a) es el mismo
mecanismo que usa el padre, con su misma instalación por clon.

## Impacto mapeado (Regla 0)

**Leídos completos:** `.opencode/.githooks/pre-commit` (nuevo, 91 líneas),
`.githooks/pre-commit` del padre (194 líneas, de donde salen DOC-04 y la convención
`SKIP=<hook-id>`), `.githooks/pre-commit-config.yaml` (para saber qué hooks del padre
matchean paths de `.opencode` — **ninguno** de markdownlint: `^docs/.*\.mdx?$` no
alcanza `.opencode/**`), `dev-tools/check-harness-contract.mjs` (firma `--root`,
`--self-test`, 17 s de corrida) y `dev-tools/git-commit-isolated.ps1:482-499` (la
detección del hook y el WARNING).

**Referencias hacia dentro:** `git-commit-isolated.ps1` resuelve el directorio de
hooks con `git rev-parse --git-path hooks` — que **respeta `core.hooksPath`** — y avisa
si no hay `pre-commit` ahí. Con `core.hooksPath=.githooks` en el repo del harness, el
predicado pasa a verdadero y el aviso desaparece **sin tocar el helper**.

**Referencias hacia fuera:** `check-harness-contract.mjs` se invoca con `--root <padre>`
porque escanea `.opencode/` desde el árbol del padre (el hook vive dentro de
`.opencode` y sube un nivel). Si alguien mueve `.opencode/` fuera del árbol, el hook lo
dice en voz alta (`not found`) en vez de fingir que verificó.

**Veredicto:** aditivo y aislado dentro de un repo que no tenía gate. El riesgo real no
es romper el padre (no lo toca): es **dejar el gate inservible por falso rojo** y
empujar a `--no-verify`. Se mide abajo.

## Spec - decisiones

| # | Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|---|
| D1 | ¿Qué gates corre? | **DOC-04** (junk/binarios) + **harness contract** | replicar los 8 hooks del padre | `cargo fmt`/`clippy` no aplican (este repo no tiene Rust); `ruff`/`prettier` no matchean nada de `.opencode/**`; `markdownlint` del padre sólo matchea `^docs/.*\.mdx?$`, así que **no cubriría `.opencode/**` ni llamándolo**. Lo que de verdad guarda el texto del harness es el contrato. Gatear lo que no aplica es teatro. |
| D2 | ¿Se corre el contrato siempre? | **sí**, ~17 s | sólo si el staged toca rutas que escanea | leer el Backlog y los planes del padre también forma parte del contrato (`claimViolations`/`reverifyViolations`): "si toca" no es una condición que se pueda enumerar sin duplicar la lista que el propio script mantiene. 17 s sobre un repo que se commitea 3 veces al día es barato. |
| D3 | ¿Y si el rojo es de otro carril? | `SKIP=harness-contract` **declarado y ruidoso** | fail-closed puro | el contrato lee el árbol del padre, que otros carriles editan en vivo (medido hoy: una corrida dio 13 violaciones y la siguiente 9, con el árbol cambiando en medio). Un gate que se pone rojo por el WIP ajeno empuja a `--no-verify`, que es peor. El skip es el mecanismo que el padre ya usa y **el hook lo dice en voz alta**. |
| D4 | ¿Cómo se instala? | `git config core.hooksPath .githooks` en el repo del harness, documentado en `.opencode/AGENTS.md` | copiar el hook a `.git/modules/.opencode/hooks` | `core.hooksPath` es **exactamente** el mecanismo del padre, es versionable como ruta y no depende de rutas absolutas ni de un `install` script. Coste declarado: es config local, como en el padre — un clon nuevo corre un comando, y está escrito dónde. |
| D5 | ¿Se toca el helper? | **no** | añadir una rama que instale hooks | el helper ya detecta y avisa; lo que faltaba era que el predicado fuera verdadero. Cero cambios en `dev-tools/**` (además, zona prohibida de este lote). |

## Verificación — las dos direcciones, ejecutadas

### Instalación

```text
$ git -C .opencode config core.hooksPath .githooks
$ git -C .opencode rev-parse --git-path hooks
.githooks
$ git -C .opencode ls-files -s .githooks/pre-commit
100755 7dc5d2b6... 0    .githooks/pre-commit      # ejecutable en el índice
```

### (1) Un commit con un fichero inválido **falla**

```text
$ git -C .opencode add .DS_Store                      # junk de DOC-04
$ git -C .opencode commit -m "test: must be blocked by the new hook"
pre-commit: running the harness contract (check-harness-contract.mjs)...
harness contract holds (9 checks)
pre-commit: harness contract passed.
ERROR [DOC-04]: Prohibited junk/binary files staged for commit:
  .DS_Store
pre-commit: gates FAILED — commit blocked (FIND-821: this repo has a gate now).
exit=1                                                 # bloqueado
```

### (2) El gate también caza una violación **del propio texto del harness**

El primer intento de commit de este cambio **lo bloqueó él mismo**, con un hallazgo
real en la documentación que yo acababa de escribir:

```text
FAIL commit-path (H-03)
  - .opencode/AGENTS.md:44: **Skips:** `SKIP=harness-contract git commit ...` si el rojo ...
pre-commit: harness contract FAILED — see the violations above.
pre-commit: gates FAILED — commit blocked
```

Reescrita la línea (ahora dice "anteponer `SKIP=harness-contract` al comando de
commit"), el commit pasó. No es una anécdota: es la prueba de que el gate **lee** lo que
se commitea y puede fallar.

### (3) El WARNING del helper desaparece

El helper imprime el WARNING cuando
`Test-Path (git rev-parse --git-path hooks)/pre-commit` es falso. Medido ahora, dentro
del repo del harness:

```text
hookDir = .githooks
predicado del helper (hookPresent) = True   -> el WARNING no se imprime
```

### (4) El skip declarado no es silencioso

```text
$ SKIP=harness-contract git commit ...
pre-commit: SKIP includes harness-contract — the harness contract was NOT verified.
[main ...] (el commit pasa)
```

### (5) El gate que corre es falsable

```text
$ node dev-tools/check-harness-contract.mjs --self-test
89/89 self-test fixtures passed          # cada regla tiene un fixture que la rompe
```

## Deuda

- **La instalación es config local, como en el padre.** Un clon nuevo de `configOpencode`
  corre un comando (`git config core.hooksPath .githooks`), documentado en
  `.opencode/AGENTS.md` § "Gate propio de este repo". Un `install-hooks` común a los dos
  repos sería la mejora, y **hoy no existe para el padre tampoco** (su hook lo documenta
  en el encabezado): crearlo para uno solo sería una asimetría nueva.
- **`markdownlint` sigue sin cubrir `.opencode/**`.** Lo sirve el entorno de `pre-commit`
  del padre, cuyo hook matchea `^docs/.*\.mdx?$`. Agregarlo acá exige resolver
  `markdownlint-cli2` (no está en PATH; `npx` implica red en un hook) — declarado, no
  olvidado. El contrato cubre las **reglas** del harness; el estilo de sus `.md` no.
- **`actionlint`**: no hay `.github/workflows` en este repo (los workflows son del padre
  y los lintea su hook). Nada que lintar.
- **La verificación por contenido** (`-ExpectPresent`/`-ExpectAbsent`) sigue siendo del
  helper: un hook no sabe qué esperaba el autor.

## Review

**P2-01 - `vanta-review`, contexto fresco, una ronda. Veredicto sobre esta fila: `cumple el
mecanismo`, con 1 Low que se cerró.**

| # | Hallazgo | Estado |
|---|---|---|
| L-3 | El hook era **fail-open** si faltaba `node`: imprimía un WARNING y dejaba pasar el commit. El caso "falta el script del padre" sí estaba declarado; el de `node` no. | **cerrado**: ahora es **fail-closed** (`FAILED=1`), con el escape declarado `SKIP=harness-contract`. `node` es prerequisito duro de este repo (`package.json`, `node_modules/`) |
| — | DOC-04 byte-idéntico al del padre, `--root "$PARENT"` correcto, `SKIP=` ruidoso, config local declarada como coste, `--self-test` 89/89 | confirmado por el revisor |

**La razón del cambio, en la frase del propio reviewer:** *"un gate que avisa y pasa es el
patrón que este lote combate"*. El hook ya bloqueaba lo que decía bloquear; lo que faltaba
era que **no poder verificar** contara como rojo, no como aviso.

## NOTICED BUT NOT TOUCHING

- `dev-tools/git-commit-isolated.ps1` no se toca (zona prohibida del lote): no hacía
  falta, su detección ya era correcta y el WARNING era **verdadero** hasta hoy.
- `.opencode/task-system/memory/{decisions,lessons}.md` están modificados en el árbol por
  los otros carriles: no entran en el commit del harness.
- El padre sigue sin `.gitmodules` y con `.opencode/` ignorado: convertirlo en submódulo
  real es una decisión de arquitectura del owner, no de esta fila.
