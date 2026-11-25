---
title: "FIND-741 - el binario declara su commit, para que la verificación contra un artefacto stale no pueda mentir"
kind: task
status: completed
description: "Un binario local anterior a HEAD responde con la forma anterior de la respuesta y el DoD pasa: `vantadb-server --version` debe imprimir versión + commit + sucio/límpio, con un test que falle si el commit declarado no es el de `git rev-parse HEAD`"
tags: [vantadb, cli, build, verification, mcp]
---

# FIND-741 - el binario declara su commit

## Contexto

Al cerrar `MCP-47` (para verificar el L0 de `MCP-44`) se usó el binario de
`target/release` (16:13), **anterior** al commit `2fb20c728`. Respondió
`{"ok":true}` **sin** el bloque `memory:{captured,queued}`: la feature no estaba
y la respuesta tenía la forma anterior. `target/debug` (17:53) sí contenía el
fuente, pero `thread_send` tardó **>15 s** (timeout medido).

Los DoD de `MCP-43`, `MCP-44`, `MCP-47` y `MEM-73` se verifican contra un
binario. Un binario que no puede decir de qué commit es **hace pasar por buena
una feature que no está**, y eso puede dar por buena media ola.

Hoy los tres estados se ven idénticos:

| estado | lo que dice el binario hoy |
|---|---|
| (a) binario == HEAD | `vantadb-server 0.9.0` |
| (b) binario de un commit anterior | `vantadb-server 0.9.0` |
| (c) binario de un árbol sucio | `vantadb-server 0.9.0` |

## Spec

### Decisión: `build.rs` en `vantadb-server` que embebe commit + estado del árbol

| Criterio | `build.rs` embebe SHA | `--version` corre `git rev-parse` en runtime | `vergen` |
|---|---|---|---|
| Distingue (b) binario de un commit anterior | ✅ el SHA viaja **con el artefacto** | ❌ lee el `HEAD` del cwd → siempre coincide con HEAD, incluso stale | ✅ |
| Distingue (c) árbol sucio | ✅ `git status --porcelain` en build time | ❌ reporta el árbol actual, no el que produjo el binario | ✅ |
| Reproducibilidad / `Cargo.lock` | ✅ sin dependencias nuevas; mismo commit → mismo binario | ✅ | ✅ |
| Coste de rebuild al commitear | ✅ solo `vantadb-server` (crate pequeño) | ✅ ninguno | ✅ |
| Dependencia nueva | ✅ ninguna (solo `std::process`) | ✅ | ❌ ~10 crates transitorios |
| Regla "no agregar dependencia para lo que 3 líneas resuelven" | ✅ | ✅ | ❌ |

**Elegido: `build.rs` en `vantadb-server`.** Razón corta: la pregunta que el
DoD necesita responder es "¿lo que ejecuté salió de este commit?", y eso solo
lo puede contestar algo grabado **en el artefacto**. Un `git rev-parse` en
runtime contesta "¿en qué commit estoy parado ahora?" — que es justo el dato
que ya tengo y nunca el que me falta: colapsa (a) y (b) en la misma salida, así
que el test que exige esta tarea no podría fallar nunca. `vergen` haría lo
mismo con ~10 crates transitorios de dependencia (Regla: no).

**Ubicación:** `vantadb-server/build.rs`, no el `build.rs` del crate raíz. El
crate raíz (`vantadb`) lo linkean todos los bindings y el propio server: meterle
un `rustc-env` que cambia por commit invalida la compilación de **todo** el
workspace en cada commit y tira a la basura la caché de sccache para el crate
más grande del repo. `vantadb-server` son ~1.4k líneas: rebuild de segundos.
Medición: ver §Verificación.

**Por qué `vantadb-server` y no `vanta-cli`:** el binario que verifican los DoD
de MCP es `vantadb-server` (`vanta-cli mcp-call` y `vanta-cli server mcp` lo
spawnean, `src/cli_handlers/mcp_call.rs:204-232`). `vanta-cli --version` es
superficie pública (lo lee `scripts/verify-release.ps1:161-164`) y no se toca
(pre-mortem (c): sin ADR no se cambia la forma de un output parseado).

### Salida (contrato)

```
vantadb-server 0.9.0 (commit 7818ecd99, worktree dirty)
vantadb-server 0.9.0 (commit 7818ecd99, worktree clean)
vantadb-server 0.9.0 (commit unknown, not a git checkout)
```

- `commit` = `git rev-parse --short HEAD` del worktree donde se compiló.
- `worktree` = `dirty` si `git status --porcelain --untracked-files=no` no está
  vacío en build time: un binario de un árbol con cambios sin commitear tampoco
  representa `HEAD`.
- Sin `.git` (tarball, Dockerfile) → `unknown`, salvo que CI pase
  `VANTADB_BUILD_COMMIT` (fallback explícito, no adivinado).

### Uso desde un DoD (el comando exacto)

```powershell
$bin = "./target/debug/vantadb-server.exe"   # o el path EXACTO del artefacto verificado
& $bin --version                              # → vantadb-server 0.9.0 (commit <sha>, worktree <estado>)
git rev-parse --short HEAD                    # → <sha>
git status --porcelain --untracked-files=no   # → vacío
```

Verificar **solo** si el `<sha>` de las dos primeras líneas coincide **y** el
`worktree` es `clean` y la tercera línea está vacía. Cualquier otra cosa →
`cargo build -p vantadb-server --bin vantadb-server` y repetir.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-server/src/main.rs` (181 L),
  `vantadb-server/tests/cli_args.rs` (88 L), `vantadb-server/Cargo.toml`,
  `build.rs` (raíz, no-op), `src/cli.rs` (bloques `Cli`/`McpCommand`),
  `src/cli_handlers/mcp_call.rs` (resolución del binario),
  `.opencode/rules/release-ci.md`.
- **Referencias hacia dentro de `--version` de `vantadb-server`:** `main.rs:46-49`
  (`-V`/`--version`) → `print_version()` (`:132-134`); `is_known_flag` (`:114`);
  `validate_args` + tests de `main.rs:140-180`; `tests/cli_args.rs` (no cubre
  `--version`); `docs/dev/avance/activo/operaciones.md:825-828` (registro
  histórico de FIND-695, solo lectura).
- **Referencias hacia fuera:** `print_version` no tiene callers fuera de
  `main`. `env!("CARGO_PKG_VERSION")` se mantiene como prefijo.
- **Verificaciones de superficie:** `tests/api/public-api.txt` no contiene nada
  de CLI (`grep -i cli|version` solo devuelve campos `version` de tipos);
  `vantadb-server` es `publish = false`, así que no hay tarball de crates.io
  cuyo build cambie.
- **Veredicto:** impacto contenido a `vantadb-server` (crate hoja, `publish =
  false`). `vanta-cli` y el crate raíz **no** se tocan: sin rebuild del engine
  por commit y sin cambio de superficie pública.

## Steps

1. ✅ RED: `vantadb-server/tests/git_commit_declared.rs` — el binario debe
   imprimir el commit de `git rev-parse --short HEAD`.
2. ✅ `vantadb-server/build.rs` — embebe commit + worktree (skip-honesto sin git).
3. ✅ `vantadb-server/src/main.rs` — `version_line()` + unit test de las tres formas.
4. ✅ Verificación mecánica del contrato (dos salidas pegadas + test) y coste de rebuild medido.
5. ✅ Cierre: `ocr-review`, review P2-01, commit local.

## Skills (SDP)

`campaign_discover_skills_v2` (phase=BUILD, taskType="Rust core", keywords
`version`, `git sha`, `build script`, `reproducible build`) → base-only
(campaign-executor, progreso, ponytail, source-driven-development,
doubt-driven-development) + lifecycle BUILD (incremental-implementation,
test-driven-development, context-engineering). Sin `pinned`.

## Verification

- `cargo build -p vantadb-server --bin vantadb-server` → `--version` imprime el
  SHA de `git rev-parse --short HEAD` (salidas pegadas en §Resultado).
- `cargo nextest run -p vantadb-server --test git_commit_declared` → pasa; con
  el `print_version()` anterior **falla** (RED registrado).
- `cargo nextest run -p vantadb-server` (suite del crate) sin regresiones.
- `cargo clippy -p vantadb-server --all-targets -- -D warnings`, `cargo fmt --check`.

## Resultado

### RED (registrado, no recreado)

```
FAIL [0.963s] vantadb-server::git_commit_declared version_declares_the_head_commit
panicked at vantadb-server\tests\git_commit_declared.rs:58:5:
`--version` must declare the commit it was built from (HEAD = 03dfece67),
got: vantadb-server 0.9.0
```

### Contrato (salidas literales)

```
> cargo build -p vantadb-server --bin vantadb-server
    Finished `dev` profile [optimized] target(s) in 1m 10s
> ./target/debug/vantadb-server.exe --version
vantadb-server 0.9.0 (commit d246ba9a6, worktree dirty)
> git rev-parse --short HEAD
d246ba9a6
```

`worktree dirty` es correcto y es la dirección segura: el árbol tenía
modificaciones trackeadas (este task file + cambios en vuelo de otro carril).
La bandera se calcula con `git status --porcelain --untracked-files=no`; un
falso positivo (`dirty` cuando lo único sucio son docs) solo obliga a
recompilar/commitear, un falso negativo dejaría pasar el binario.

**Repro en vivo de los tres estados** (un carril paralelo commiteó entre el
build y la lectura; no hizo falta fabricar el escenario):

```
# (a) binario == HEAD
> ./target/debug/vantadb-server.exe --version   → commit d246ba9a6
> git rev-parse --short HEAD                    → d246ba9a6            ✅ coincide

# (b) HEAD avanzó después del build  → el mismo binario, ahora stale
> ./target/debug/vantadb-server.exe --version   → commit d246ba9a6
> git rev-parse --short HEAD                    → 3bfd2f14a            ❌ difiere → rebuild

# (b') artefacto de un día antes, el que originó el hallazgo
> ./target/release/vantadb-server.exe --version → vantadb-server 0.9.0  ❌ no declara commit
```

El caso (c) `worktree dirty` quedó observado en el build real (árbol con
modificaciones trackeadas) y su forma exacta está cubierta por el unit test.

### GREEN

```
PASS [1.240s] vantadb-server::git_commit_declared version_declares_the_head_commit
Summary 1 test run: 1 passed, 0 skipped

PASS vantadb-server::bin/vantadb-server tests::version_line_names_commit_and_worktree
PASS tests::accepts_known_flags · rejects_unknown_args · first_unknown_arg_wins
Summary 4 tests run: 4 passed
```

### Coste medido (blast radius §7)

- **Alcance de la invalidación:** solo `vantadb-server` (build script + crate).
  Evidencia: tras tocar su fuente o tras un commit, `cargo build -p
  vantadb-server` imprime `Compiling vantadb-server` y **nunca** `Compiling
  vantadb v0.9.0` (el engine de 200k LOC no se recompila, ni siquiera al cambiar
  el SHA, porque `cargo:rustc-env` pertenece a este paquete). Revisión P2-01
  independiente lo midió igual (19.2 s, engine intacto).
- **Wall clock:** un commit cuesta **~19 s de rebuild de `vantadb-server`** y
  cero del engine (medición del reviewer, sin la contención del carril
  paralelo); el no-op cuando el árbol está quieto costó **1.55 s**. Las
  mediciones tomadas con otro carril compilando en el mismo `target/` salieron
  contaminadas (`Compiling vantadb-mcp` sin tocar ese crate, `Blocking waiting
  for file lock`) y se descartaron.
- **Reproducibilidad / `Cargo.lock`:** sin dependencias nuevas (`std::process`
  solamente), ningún cambio de lockfile. Mismo commit + mismo árbol → mismo
  binario. Un commit distinto → binario distinto, que es el punto.
- **`cargo package`:** `vantadb-server` es `publish = false`, así que no hay
  tarball de crates.io afectado. Sin `.git` (Docker/vendored) el binario dice
  `commit unknown, not a git checkout`, salvo que CI pase
  `VANTADB_BUILD_COMMIT=<sha>` (fallback explícito; si git responde y el
  override discrepa, el build script emite `cargo:warning` y gana git).
- **Inputs vigilados por el build script:** `.git/{HEAD,index,logs/HEAD}`,
  `src`, `Cargo.toml`, `build.rs`, `../Cargo.{toml,lock}`, `../build.rs`,
  `../rust-toolchain.toml`, `../.cargo/config.toml` y `src`+`Cargo.toml` de
  `vantadb-mcp`/`vanta-memory`/`vantadb-ffi-core`.

### Cierre

Commit local `0256b79aa` (sin push). Artifacto verificado **después** del commit:

```
> cargo build -p vantadb-server --bin vantadb-server
> ./target/debug/vantadb-server.exe --version
vantadb-server 0.9.0 (commit 0256b79aa, worktree dirty)
> git rev-parse --short HEAD
0256b79aa
> cargo nextest run -p vantadb-server --test git_commit_declared
PASS [0.731s] version_declares_the_head_commit
```

Nota del hook pre-commit: el primer intento lo bloqueó `cargo-fmt` por los
diff **del otro carril** (`vantadb-mcp/src/memory_scheduler.rs:188`,
`proxy.rs:630`; `cargo fmt --check` → exit 1 con esos dos únicos diffs), archivos
que esta tarea tiene prohibido tocar. Se usó el escape nativo documentado en
`.githooks/pre-commit-config.yaml` (`SKIP=cargo-fmt`); los `.rs` de esta tarea
pasan el mismo gate por separado: `cargo fmt -p vantadb-server -- --check` →
exit 0. El resto de los gates (clippy `-p vantadb-server`, yaml, markdownlint,
verify_staged, actionlint) corrió y pasó.

### Fuera de la superficie pública

`vanta-cli --version` **no** se tocó: lo parsea `scripts/verify-release.ps1:161-164`
y cambiarlo sin ADR es exactamente el pre-mortem (c). `tests/api/public-api.txt`
no contiene superficie de CLI, así que no hay golden que actualizar.

## Incógnitas / deuda

- **FIND-98 (misma familia):** con este mecanismo, `vantadb-server --version`
  distingue el binario instalado del recién compilado — pero para `vanta-cli`
  haría falta replicar el `build.rs` en el paquete raíz (`vantadb`), y eso
  invalida el crate más grande del repo en cada commit (sccache miss
  workspace-wide). No se hace aquí; queda para FIND-98 con el coste sobre la mesa.
- **`vanta-cli mcp-call` / `vanta-cli server mcp` resuelven `vantadb-server`
  por PATH antes que por path hermano** (`src/cli_handlers/mcp_call.rs:204-232`).
  Un `vantadb-server` instalado viejo puede ganarle al de `target/`; ahora es
  detectable (`--version` lo delata), pero el DoD debe invocar el binario por
  **path explícito**.
- **Pin `binarySha256` del binario certificado** (`.tasksystem`, ver
  `docs/dev/avance/activo/operaciones.md:886,932`): cualquier build posterior a
  este cambio tiene otro sha256. No se cambia acá (el pin apunta al binario
  instalado, no se rebuildó); al reinstalar, el pin debe re-derivarse del
  artefacto nuevo.
- **Rama `unknown` del `build.rs`** (sin git y sin override) no se ejecutó en
  este entorno: la cubre el unit test en su forma de salida, no en su camino de
  build. Deuda menor.
- **"git ausente con `.git` presente"** (Docker con `.git` copiado y sin `git`
  en PATH, o `safe.directory`/dubious-ownership como root en un contenedor):
  el test falla duro en vez de skipear. Es deliberado (un `.git` que existe y
  no se puede leer es un repo roto, no una excusa) pero es un modo de fallo
  conocido: el escape es `VANTADB_BUILD_COMMIT` + PATH con git.
- **El label `worktree` no tiene igualdad testeada** contra `git status` del
  momento del test: en este repo multi-carril (otro agente commitea y edita en
  el mismo árbol) esa igualdad sería flaky. El test afirma la **forma**
  (`clean|dirty`); el estado exacto lo cierra el tercer comando del DoD
  (`git status --porcelain --untracked-files=no`).
- **Docs con la salida vieja:** `docs/dev/tasks/F0-2-binary-repin.md:189` y
  `docs/dev/avance/activo/operaciones.md:828,886,932` dicen `--version` →
  `vantadb-server 0.9.0`. Son registros históricos (superficie: el prefijo
  `vantadb-server <version>` sigue siendo el mismo) y están fuera del scope de
  esta tarea; solo hay que releerlos si F0-2 se toma como DoD vigente.

## Review (P2-01)

Tier **Fast**: paths tocados = `vantadb-server/{build.rs,src/main.rs,tests/}` +
`docs/dev/tasks/FIND-741.md` — ninguno matchea los globs adversariales
(`docs/api/**`, `src/sdk/**`, `src/wal*.rs`, `src/node/**`, …).

**Veredicto: APPROVE** (contexto fresco, `vanta-review`, no participó de la
implementación; 7/7 puntos del contrato verificados con comando + salida).
Findings opcionales y qué se hizo:

| Finding | Acción |
|---|---|
| [medium] `rerun-if-changed` no cubría `Cargo.lock`, pins de toolchain/config, `build.rs` raíz ni manifests de sub-crates → label `clean` stale tras `cargo update` | ✅ agregados al array (`build.rs:74-95`) |
| [low] el comentario subestimaba el coste ("cheap `git status` per edit") | ✅ corregido: ~19 s de rebuild de este crate, cero del engine |
| [low] override `VANTADB_BUILD_COMMIT` sin validación y silenciosamente ignorado | ✅ `cargo:warning` cuando git responde y el override discrepa |
| [low] el test acepta `clean|dirty` (un build script que siempre diga `clean` pasa) | ⚠️ aceptado y documentado en §Incógnitas (igualdad con `git status` = flaky multi-carril; lo cierra el 3.er comando del DoD) |
| [low] "git ausente con `.git` presente" falla duro | ⚠️ aceptado y documentado en §Incógnitas |
| [nit] docs con la salida vieja | ⚠️ registros históricos, fuera de scope; documentado |
