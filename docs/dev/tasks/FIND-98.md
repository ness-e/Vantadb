---
title: "FIND-98 - el vanta-cli instalado declara de que commit salio, y un check falla si no es el de HEAD"
kind: task
status: completed
description: "El binario instalado en ~/.cargo/bin no puede distinguirse de uno fresco (--version solo dice 0.9.0). Se le embebe el commit via build.rs escribiendo un fichero OUT_DIR que solo incluye el target bin (no el engine), y scripts/check-cli-fresh.ps1 falla si el commit declarado no es git rev-parse HEAD."
tags: [vantadb, cli, build, verification, install]
---

# FIND-98 - el `vanta-cli` instalado declara su commit

## Contexto

La fila del Backlog decía: «Binario `vanta-cli` instalado stale (79 vs 82 tools)».
El contrato de `FIND-741` (`e8df96232`) dejó montado el mecanismo (`build.rs` +
`rerun-if-changed` sobre `.git/{HEAD,index,logs/HEAD}`) y dejó escrito en
§Incógnitas que replicarlo en el paquete raíz detectaría este caso **a cambio
de un rebuild del engine por commit**, y que esa decisión era de esta tarea.

## Estado real medido (no el reportado)

| Artefacto | Fecha | `--version` | `tools/list` (perfil `full`) |
|---|---|---|---|
| `~/.cargo/bin/vanta-cli.exe` (instalado) | 2026-09-11 12:12 | `vanta-cli 0.9.0` | 84 |
| `target/debug/vanta-cli.exe` (fuente) | 2026-11-26 01:21 | `vanta-cli 0.9.0` | 84 |
| `~/.cargo/bin/vantadb-server.exe` | 2026-11-24 16:31 | `vantadb-server 0.9.0 (...)` | 84 |

Comandos: `vanta-cli server --mcp` + JSON-RPC `initialize`/`tools/list`
(`%TEMP%\opencode\count_tools.py`), y `vanta-cli --help` diff.

- **El «79 vs 87 tools» ya no mide lo que la fila creía**: el registro de tools
  vive en `vantadb-mcp`, que linkea `vantadb-server`, no `vanta-cli`
  (`src/cli_handlers/server.rs:314` spawnea el binario hermano/PATH). Con el
  server instalado al día, instalado y fuente dan **84 == 84** aunque el CLI
  tenga 2,5 meses. El criterio del plan (`tools/list` diff 0) **pasa con un CLI
  stale**: no sirve como detector.
- Lo que sí está stale es el **binario del CLI**: 2026-09-11 vs `HEAD` de hoy.
  El diff de superficie es menor (solo cambia la descripción de `encryption`),
  por eso nadie lo notó: el binario no puede decir de qué commit salió.

## Diagnóstico (por qué está stale)

- El install es una **copia manual** de un artefacto a `~/.cargo/bin`; no hay
  entrada en `~/.cargo/.crates2.json` (no lo instaló `cargo install`), no hay
  script de instalación del CLI (`skills/vantadb/scripts/install-vantadb.sh`
  instala el SDK de Python), y nada reconstruye ni valida el binario después.
- `vanta-cli --version` imprime solo `CARGO_PKG_VERSION`, que no cambia entre
  commits de desarrollo (`0.9.0` todo el ciclo). **Ningún dato del artefacto
  permite responder «¿esto salió de HEAD?»** — el mismo agujero que cerró
  `FIND-741` para `vantadb-server`.

## Spec / Decisión

**Elegido: (a)** — `vanta-cli --version` (y `-V`) declara el commit, y un check
mecánico falla si no coincide con `HEAD`. **(b) descartada por medición**: el
conteo de tools es del server, no del CLI (84 == 84 con un CLI stale), así que
no detecta el caso. (c) no aplica: (a) sola lo detecta.

| Opción | Coste por commit | Detecta el caso | Veredicto |
|---|---|---|---|
| `cargo:rustc-env` en `build.rs` raíz (patrón FIND-741 literal) | **rebuild del engine de 200k LOC** + dependientes (tira sccache) | ✅ | ❌ demasiado caro: 3 carriles commitean en este árbol |
| `build.rs` raíz escribe `OUT_DIR/build_version.txt`; **solo el target bin** lo `include_str!` | recompilar+relink **solo `vanta-cli`** | ✅ | ✅ elegido |
| `tools --count` (b) | 0 | ❌ (medido) | ❌ |

Por qué el fichero OUT_DIR y no `rustc-env`: `rustc-env` es del *paquete*, así
que invalida lib+bins. Un `include_str!` de un fichero de `OUT_DIR` entra
únicamente en el dep-info del target que lo incluye (`src/bin/vanta-cli.rs`);
la lib no lo ve. Coste medido en §Resultado.

**Sin etiqueta `worktree clean|dirty`** (a diferencia de FIND-741): esa etiqueta
solo es cierta si el script se re-ejecuta en cada cambio del árbol de trabajo,
o sea en cada edición de `src/`, y entonces el label cambia → se recompila.
`vantadb-server` puede permitírselo porque son ~1,4k líneas; el engine de 200k
no. Con solo metadatos de git (reflog + ref) el **commit** es exacto y el
binario no afirma nada que no pueda observar. Un build con cambios sin commitear
declara el commit base; el check lo avisa (el artefacto no puede saberlo).

**Rerun set:** `.git/HEAD` (detached), `.git/logs/HEAD` (todo commit), la ref
simbólica (`refs/heads/<rama>`), `Cargo.toml`, `build.rs`,
`VANTADB_BUILD_COMMIT`. **`.git/index` fuera**: cambia en cada `git add`, solo
alimenta el label de árbol que ya no se declara, y tenerlo en el dep-info hacía
que cada `git add` de cualquier carril relinkeara el CLI (medido: 8-12 s → 0,5 s
al quitarlo).

Contrato mecánico:

```
pwsh scripts/check-cli-fresh.ps1   # FAIL si el commit declarado por el instalado != git rev-parse --short HEAD
```

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `build.rs` (raíz), `src/bin/vanta-cli.rs`,
  `src/cli.rs` (bloque `Cli`), `vantadb-server/build.rs`,
  `vantadb-server/tests/git_commit_declared.rs`,
  `src/cli_handlers/server.rs` (`cmd_server_mcp`), `scripts/verify-release.ps1`
  (smoke de `--version`), `docs/dev/tasks/FIND-741.md`.
- **Referencias hacia dentro de `--version` de `vanta-cli`:**
  `scripts/verify-release.ps1:163-164` (regex `vanta-cli --version` contiene la
  versión → **el prefijo se conserva**, el sufijo `(commit <sha>)` es
  aditivo); `tests/cli_tests.rs` (no cubría versión).
  No hay golden de superficie (el bin no está en `tests/api/public-api.txt`).
- **Referencias hacia fuera:** `build.rs` raíz es no-op sin directivas (nadie
  depende de él); el bin solo usa `Cli`/`cli_handlers`. Añadir
  `rerun-if-changed` desactiva el default «cualquier fichero del paquete»: el
  script solo depende de git y de `Cargo.toml`, ambos listados.
- **Veredicto:** impacto contenido a `build.rs`, `src/bin/vanta-cli.rs`,
  `tests/cli_tests.rs`, `scripts/check-cli-fresh.ps1`. **No se toca**
  `src/cli.rs` (la lib), el engine, ni `vantadb-mcp`.

## Steps

1. ✅ RED: test de proceso real (`CARGO_BIN_EXE_vanta-cli`) que exige commit
   declarado + prefijo `vanta-cli <version>`; falló antes del fix.
2. ✅ `build.rs` raíz: commit → `OUT_DIR/build_version.txt` (rerun sobre
   reflog + ref; sin `index`, sin label de árbol).
3. ✅ `src/bin/vanta-cli.rs`: `const BUILD_VERSION = include_str!(...)` y
   `Cli::command().version(BUILD_VERSION).get_matches()`.
4. ✅ `scripts/check-cli-fresh.ps1`: compara instalado vs `HEAD`, falla nombrando ambos.

## Resultado

### RED (registrado antes del rename del test)

La primera versión del test se llamaba `version_declares_the_head_commit` y
exigía igualdad con HEAD; falló así:

```
test find98_build_identity::version_declares_the_head_commit ... FAILED
panicked at tests\cli_tests.rs:2077:9:
`--version` must declare the commit it was built from (HEAD = 0303219f3),
got: vanta-cli 0.9.0
```

Ese test se **renombró y se relajó** a
`version_declares_a_commit_from_this_history` (ancestro, no igualdad) al
comprobar que en este árbol multi-carril HEAD avanza entre compilar y correr:
la primera corrida GREEN falló con `HEAD = 1de8d9896` vs binario `f097c5f78`.

### GREEN

```
test find98_build_identity::version_declares_a_commit_from_this_history ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out
```

El test afirma que el commit declarado **existe en este repo y es ancestro de
HEAD**; `-V` además debe imprimir exactamente lo mismo que `--version`. La
igualdad con HEAD la exige el script, que se corre a demanda — el reparto que
FIND-741 documentó. **Consecuencia declarada:** el test no puede fallar por un
binario stale; el único gate de frescura es el script (y hoy nada lo cablea en
CI: `.github/**` está fuera del alcance de esta tarea).

### Contrato mecánico (dos corridas literales)

```
# 1) binario instalado el 2026-09-11, puesto de vuelta en su sitio (stale: no declara commit)
> pwsh -NoProfile -File scripts/check-cli-fresh.ps1
STALE: the installed vanta-cli does not declare a commit - it predates FIND-98 and cannot be verified.
  binary:  C:\Users\Eros\.cargo\bin\vanta-cli.exe
  reports: vanta-cli 0.9.0
  HEAD:    e1b917fc8
  fix:     cargo install --path "C:/Users/Eros/VantaDB Proyect/VantaDB" --bin vanta-cli --force
exit=1

# 2) tras reconstruir e instalar (copia de target/debug/vanta-cli.exe)
> pwsh -NoProfile -File scripts/check-cli-fresh.ps1
ADVERTENCIA: the repository has uncommitted changes: an artifact built now would carry e1b917fc8 plus local edits.
OK: installed vanta-cli matches HEAD (e1b917fc8) - vanta-cli 0.9.0 (commit e1b917fc8)
exit=0
```

El aviso de árbol sucio es en vivo (`git status` del script), no una etiqueta
del artefacto: el binario declara el commit base y el check dice que ahora
mismo un build llevaría además ediciones locales.

El caso `commit unknown` (artefacto de tarball, sin `.git`) tiene su propia
rama y también sale 1, pero el mensaje dice "unverifiable", no "predates
FIND-98"; un binario que no arranca o sale ≠0 sale **2** ("cannot check"), no 1.

### Coste medido (la decisión de la tarea)

Cronometrado con `cargo build -p vantadb --bin vanta-cli -j 2 -v`, descartando
las corridas con `Blocking waiting for file lock` (otro carril compilando en el
mismo `target/`):

| Escenario | Wall clock | `rustc src/lib.rs` | `rustc vanta_cli` | build script |
|---|---|---|---|---|
| Build tras cambiar el script (una vez) | 10.4 s | no | sí | sí |
| Simular commit (`touch .git/logs/HEAD`) | **9.5 s** | **no** | sí | sí |
| `git add`/index touch (no se vigila) | 0.5 s | no | no | no |
| No-op (sin tocar git) | 0.5 s | no | no | no |

- **El engine no se recompila por commit**: `cargo:rustc-env` habría invalidado
  lib+bins+dependientes; el fichero de `OUT_DIR` solo entra en el dep-info del
  bin. Coste real por commit: **~9,5 s de recompilar+relinkear el CLI**, no el
  rebuild del engine de 200k LOC ni un miss de sccache workspace-wide.
- **Medición de por qué se quitó `index`:** con `.git/index` en el rerun set,
  tocar el índice (lo hace cada `git add` de cualquier carril) re-ejecutaba el
  script y **relinkeaba el bin** (8-12 s) aunque el fichero de identidad no
  cambiara — comprobado con mtime idéntico antes/después. Sin `index`: 0,5 s y
  cero rustc. `write_if_changed` no evitaba ese relink (cargo reconstruye los
  dependientes cuando el script se re-ejecuta), así que se eliminó.
- Nota de honestidad: una corrida de 105 s con `rustc src/lib.rs` se observó al
  introducir el script por primera vez; no se reprodujo al reescribirlo (10,4 s,
  lib intacta), así que se reporta como "una vez" y no como coste del diseño.

### Incógnitas / deuda

- **Un build sucio declara el commit base.** El artefacto no puede saber si el
  árbol tenía cambios (ver §Spec); el script avisa con `git status` en vivo,
  pero **no falla**: en este árbol multi-carril el worktree casi nunca está
  limpio y el gate quedaría siempre rojo. Techo declarado.
- **Nada corre el script en CI/hook**: `.github/**` y `.githooks/**` están
  fuera del alcance de esta tarea. `dev-tools/e2e-memory-cycle.ps1:25-31` ya
  tiene el mismo gate para `vantadb-server` y sí está cableado en
  `nightly.yml:276-284`; lo correcto sería factorizar ambos y cablearlo (no se
  puede tocar `dev-tools/**` aquí).
- **Artefacto de release (tarball sin `.git`)** declara `commit unknown` y el
  script no puede verificarlo; el release debería pasar
  `VANTADB_BUILD_COMMIT` (soportado por el build script) — `.github/**` fuera
  de alcance.
- **Reinstalación**: se copió `target/debug/vanta-cli.exe` (perfil dev,
  `opt-level=1`) sobre `~/.cargo/bin/vanta-cli.exe`, con respaldo
  `vanta-cli.exe.bak-find98` — el fix que la propia fila del Backlog sancionaba.
  Un `cargo install --path . --bin vanta-cli --force` (release) dejaría el
  binario más rápido; no se corrió por tiempo/espacio y porque el pin
  `binarySha256` del binario certificado apunta a `vantadb-server`, no al CLI.
- **`tools/list` sigue siendo del server**: quien verifique `FIND-98` con el
  criterio viejo del plan (`tools/list` diff 0) obtendrá 84 == 84 aunque el CLI
  esté stale. El criterio vigente es este script.

## Skills (SDP)

`campaign_discover_skills_v2` (phase=BUILD, archivosClave `build.rs, src/bin/vanta-cli.rs,
src/cli.rs, tests/cli_tests.rs, scripts/`, keywords `stale binary`, `version sha`,
`install check`, `tool count`) → base (campaign-executor, progreso, ponytail) +
pinned (test-driven-development, systematic-debugging) + source-driven-development,
doubt-driven-development, incremental-implementation.

## Verification

- `cargo test -p vantadb --test cli_tests find98` (nextest excluye `cli_tests` del perfil default) → RED antes / GREEN después.
- `pwsh scripts/check-cli-fresh.ps1` → falla con el binario del 2026-09-11; pasa
  tras reconstruir e instalar. Matriz de códigos con binarios falsos en
  `%TEMP%`: sin commit → 1; `commit unknown` → 1; sha ajeno → 1; crash (exit 7)
  → 2; ruta inexistente → 2.
- `cargo clippy -p vantadb --bin vanta-cli --test cli_tests -- -D warnings` y el
  comando exacto del hook (`--no-default-features --features
  cli,fjall,memmap2,fs2,roaring --tests`) → exit 0; `rustfmt --check` de los
  tres archivos → exit 0.
- `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps.
- Coste medido del rebuild por commit (no inventado).

## Review (P2-01)

Tier **Fast**: `build.rs` (raíz), `src/bin/vanta-cli.rs`, `tests/cli_tests.rs`,
`scripts/check-cli-fresh.ps1`, docs — ningún glob adversarial.

**Ronda 1 (`vanta-review`, contexto fresco, sin participar de la
implementación): 🔴 CHANGES-REQUIRED** — 2 high, 4 medium, 4 low, 0 critical.
Todo verificado en vivo por el reviewer (binarios falsos incluidos). Qué se hizo:

| Finding | Acción |
|---|---|
| [high] F1: el label `worktree clean\|dirty` puede quedar stale para siempre (el script no se re-ejecuta al editar el árbol) | ✅ se **eliminó el label**. Declararlo exacto exigiría vigilar `src/` → recompilar el engine cuando el label cambia. Ahora el artefacto afirma solo el commit (exacto: commit mueve reflog/ref). |
| [high] F2: el test aceptaba `clean` o `dirty` (no verificaba nada) | ✅ eliminada la aserción junto con el label; se añadió que `-V` imprima **exactamente** lo mismo que `--version`. |
| [medium] F3: exit 2 no se respetaba (binario inexistente → excepción → 1; binario que crashea → "predates FIND-98" → 1); nada corre el script | ✅ `Test-Path` + `try/catch` + `$LASTEXITCODE` → 2 "CANNOT CHECK"; matriz de 5 casos verificada. El cableado en CI/hook queda como deuda (§Incógnitas): `.github/**` y `dev-tools/**` están fuera del alcance. |
| [medium] F4: `write_if_changed` no evitaba el relink (contradecía la medición) y `.git/index` vigilado relinkeaba en cada `git add` de cualquier carril | ✅ medido con mtime idéntico: el relink ocurría igual → se borró `write_if_changed` y se quitó `index` del rerun set. `git add`: 8-12 s → **0,5 s**; commit: 9,5 s. |
| [medium] F5: un build sucio pasa el gate declarando el commit base | ⚠️ techo declarado: el artefacto no puede saberlo sin el label (F1); el script lo avisa en vivo con `git status` pero no falla, porque en este árbol multi-carril el worktree casi nunca está limpio. |
| [medium] F6: "sin git" y "binario roto" se confundían; receta de dev para un consumidor de tarball | ✅ rama `commit unknown` propia ("UNVERIFIABLE", exit 1) separada de "predates FIND-98"; el mensaje de release apunta a `VANTADB_BUILD_COMMIT`. |
| [low] F7: el commit dependía solo del reflog (`HEAD` no cambia al commitear) | ✅ se vigila también la ref simbólica resuelta vía `--git-path` (correcto en worktrees). |
| [low] F8: `-V` sin cubrir | ✅ aserción de igualdad `-V` == `--version`. |
| [low] F9: el RED citaba un test renombrado | ✅ documentado en §Resultado, con el rename y el motivo. |
| [low] F10: `status: in-progress` sin §Review | ✅ este bloque; status `completed`. |
| [nit] derive `version` en `src/cli.rs` queda como señuelo | ⚠️ no se toca (comentario en la lib = recompilar el engine); el comentario del bin ya explica que el override vive ahí. |

Estado tras aplicar la ronda: los **2 high** y los medium accionables están
cerrados y re-verificados por el autor (mediciones de rerun set + matriz de
códigos con binarios falsos); **no hubo segunda ronda** (la tarea pide una).
Residual declarado: F5 (un build sucio declara el commit base — techo del
artefacto) y el cableado del script en CI/hook (`.github/**` y `dev-tools/**`
fuera de alcance).
