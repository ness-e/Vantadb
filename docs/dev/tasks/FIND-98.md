---
title: "FIND-98 - el vanta-cli instalado declara de que commit salio, y un check falla si no es el de HEAD"
kind: task
status: in-progress
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
la lib no lo ve. El build script no emite directivas que cambien, así que su
re-ejecución no ensucia a los dependientes. Se re-escribe el fichero **solo si
el contenido cambia** (evita relinks por `git add`). Coste medido en §Resultado.

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
  versión → **el prefijo se conserva**, el sufijo `(commit …, worktree …)` es
  aditivo, igual que en FIND-741); `tests/cli_tests.rs` (no cubría versión).
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
2. ✅ `build.rs` raíz: commit + worktree → `OUT_DIR/build_version.txt`
   (solo escribe si cambió).
3. ✅ `src/bin/vanta-cli.rs`: `const BUILD_VERSION = include_str!(...)` y
   `Cli::command().version(BUILD_VERSION).get_matches()`.
4. ✅ `scripts/check-cli-fresh.ps1`: compara instalado vs `HEAD`, falla nombrando ambos.
5. ✅ Coste medido + demo stale→falla / fresco→pasa. Verify + cierre.

## Resultado

### RED (registrado, no recreado)

```
test find98_build_identity::version_declares_the_head_commit ... FAILED
panicked at tests\cli_tests.rs:2077:9:
`--version` must declare the commit it was built from (HEAD = 0303219f3),
got: vanta-cli 0.9.0
```

### GREEN

```
test find98_build_identity::version_declares_a_commit_from_this_history ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out
```

El test afirma que el commit declarado **existe en este repo y es ancestro de
HEAD**, no que sea igual a HEAD: en este árbol multi-carril HEAD avanza entre
compilar el binario y correr el test (me pasó en la primera corrida:
`HEAD = 1de8d9896` vs binario `f097c5f78`). La igualdad con HEAD la exige el
script, que se corre a demanda — mismo reparto que FIND-741 documentó.

### Contrato mecánico (dos corridas literales)

```
# 1) binario instalado el 2026-09-11 (stale: no declara commit)
> pwsh -NoProfile -File scripts/check-cli-fresh.ps1
FAIL: installed vanta-cli does not declare a commit - it predates FIND-98 and cannot be verified.
  binary:  C:\Users\Eros\.cargo\bin\vanta-cli.exe
  reports: vanta-cli 0.9.0
  HEAD:    f097c5f78
  fix:     cargo install --path "C:/Users/Eros/VantaDB Proyect/VantaDB" --bin vanta-cli --force
exit=1

# 2) tras reconstruir e instalar (copia de target/debug/vanta-cli.exe)
> pwsh -NoProfile -File scripts/check-cli-fresh.ps1
OK: installed vanta-cli matches HEAD (f097c5f78) - vanta-cli 0.9.0 (commit f097c5f78, worktree dirty)
exit=0
```

`worktree dirty` es lo correcto: el árbol tiene los cambios de esta tarea (y de
los otros carriles) sin commitear. El script avisa pero no falla por eso: lo que
gatea es la identidad, no la limpieza.

### Coste medido (la decisión de la tarea)

Cronometrado con `cargo build -p vantadb --bin vanta-cli -j 2 -v`, descartando
las corridas con `Blocking waiting for file lock` (otro carril compilando en el
mismo `target/`):

| Escenario | Wall clock | `rustc src/lib.rs` | `rustc vanta_cli` | build script |
|---|---|---|---|---|
| Build script nuevo (una vez) | 105.4 s | sí | sí | sí |
| Simular commit (`touch .git/logs/HEAD`) | **10.8 s** | **no** | sí | sí |
| `git add`/index touch | 8.3 s | no | sí | sí |
| No-op (sin tocar git) | 0.6 s | no | no | no |

- **El engine no se recompila por commit**: `cargo:rustc-env` habría invalidado
  lib+bins+dependientes; el fichero de `OUT_DIR` solo entra en el dep-info del
  bin. Coste real por commit: **~11 s de recompilar+relinkear el CLI**, no el
  rebuild del engine de 200k LOC ni un miss de sccache workspace-wide.
- Nota honesta: cargo **sí** reconstruye el bin cuando el build script se
  re-ejecuta, aunque el contenido del fichero no cambie (medido: tocar
  `.git/index` → 8.3 s y un `rustc vanta_cli`). Por eso el coste se paga al
  commitear (`logs/HEAD`) y al hacer `git add` (index), no en cada edición.

### Incógnitas / deuda

- **Reinstalación**: se copió `target/debug/vanta-cli.exe` (perfil dev,
  `opt-level=1`) sobre `~/.cargo/bin/vanta-cli.exe`, con respaldo
  `vanta-cli.exe.bak-find98` — el fix que la propia fila del Backlog sancionaba.
  Un `cargo install --path . --bin vanta-cli --force` (release) dejaría el
  binario más rápido; no se corrió por tiempo/espacio y porque el pin
  `binarySha256` del binario certificado apunta a `vantadb-server`, no al CLI.
- **`git status` desde el build script** refresca el índice; si eso moviera su
  mtime, el siguiente build re-ejecutaría el script (8 s). No se observó el
  bucle en la práctica (no-op 0.6 s), pero es el modo de fallo a vigilar.
- **Rama `unknown`** (sin git y sin `VANTADB_BUILD_COMMIT`) no se ejecutó
  aquí; el script la trata como "no declara commit" → falla cerrado.
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

- `cargo nextest run -p vantadb --test cli_tests -- find98` → RED antes / GREEN después.
- `pwsh scripts/check-cli-fresh.ps1` → falla con el binario del 2026-09-11; pasa
  tras reconstruir e instalar.
- `cargo clippy -p vantadb --all-targets -- -D warnings`, `cargo fmt --check`.
- Coste medido del rebuild por commit (no inventado).
