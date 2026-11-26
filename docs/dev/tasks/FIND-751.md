---
title: "FIND-751 - un HOME vacío no es un home (4 sitios, rutas relativas al CWD)"
kind: task
status: in-progress
description: "Con `HOME=` vacío, `var_os` devuelve `Some(\"\")` y cuatro sitios construían rutas relativas al CWD; uno de ellos (`mcp init`) escribía la config del cliente y reportaba éxito. Los cuatro pasan a `utils::env_value`; sin home, `mcp init` falla con error accionable (`--config`) en vez de plantar la config en el repo. Test de los tres casos por sitio, sin `set_var`."
---

# FIND-751 — un `HOME` vacío no es un home

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`
> **Ruta:** `vanta-worker` · **Carril:** R (1 slot) · **Appetite:** 2-3 h
> **Dependencias:** `FIND-738` ✅ (`85d8423bd`, el helper `env_value`) · `FIND-697` (adyacente, **no** se resuelve acá).
> **Prohibido en este ticket:** `src/bin/vanta-cli.rs` y `tests/cli_tests.rs` (`FIND-98`), `src/wal*.rs`, `src/schema.rs`, `src/index/serialize/file.rs`, `src/hardware/mod.rs`.

## Contexto

`std::env::var_os` devuelve `Some("")` con la variable **definida y vacía**.
`PathBuf::from("")` es la ruta vacía y `.join("x")` sobre ella da `x`: una ruta
**relativa al CWD**. `HOME=` es la forma que deja un shell al exportar sin valor,
y también `docker -e HOME=` / algunas units de systemd. No es hipotético.

`FIND-738` cerró la misma clase para **flags** y para **variables que
transportan un valor** con dos predicados en `src/utils/mod.rs`
(`env_present`, `env_value`). Este ticket reusa `env_value` en los cuatro sitios
medidos; `src/hardware/mod.rs:120` **no entra** (ya comprueba `!home.is_empty()`).

## Impacto mapeado (Regla 0)

**Archivos leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `src/utils/mod.rs:1-38` | `env_present` / `env_value` y sus docstrings | el predicado a reusar; no se inventa otro |
| `src/cli_handlers/mcp_init.rs:1-673` | `resolve_default_config_path`, `cmd_mcp_init`, `cmd_mcp_init_with_binary`, todo el `mod tests` | sitio 1: es el único que **escribe**; había que ver el orden resolución → lectura → escritura |
| `src/cli_handlers/config.rs:1-80`, `:140-175`, `:385-410` | `user_config_path` y sus **dos** consumidores (`cmd_config_show:150`, `cmd_config_set:396`) | sitio 2: `?` sobre `None` ya es el fallback de ambos consumidores |
| `src/cli_handlers/server.rs:300-420` | `cmd_server_mcp`, la lista de `candidates` y el `spawn` | sitio 3: el candidato relativo entra **antes** del `exe_name` pelado |
| `src/config.rs:1-40`, `:575-645`, `:3020-3055`, `:3225-3260` | `local_model_bases`, `resolve_local_model_path`, `EnvGuard` y los tests de `FIND-697`/`FIND-703` | sitio 4: es resolución de rutas de **modelo**, no escritura de config; los tests vecinos son el patrón |
| `tests/api/public_api.rs:40-84` | el predicado que recibe el `Option` (`FIND-731`) | patrón de test sin `set_var` |
| `docs/dev/tasks/FIND-738.md` §Residual `HOME`/`USERPROFILE` | la tabla de los cuatro sitios, severidad y contrato de cierre | origen del ticket |

**Referencias hacia dentro (quién consume cada función)**

- `resolve_default_config_path` es **API pública** (`tests/api/public-api.txt:1320` y su re-export en `:1620`). La firma **no cambia** → el golden no cambia.
- `user_config_path`: 2 call sites internos (`config.rs:150`, `config.rs:396`); ambos ya tratan `None` como "sin capa de usuario" / error accionable.
- `local_model_bases`: 1 call site (`resolve_local_model_path:631`), API pública con firma intacta.
- Los candidatos de `server.rs` son locales a `cmd_server_mcp`.

**Referencias hacia afuera**

- `crate::utils::env_value` es `pub(crate)` → sin superficie pública nueva.
- Ningún script, workflow ni test del repo depende del fallback relativo al CWD (`rg 'mcp init|resolve_default_config_path|claude\.json'` sobre `tests/`, `scripts/`, `dev-tools/`, `.github/`: sólo el golden de public-api y `scripts/setup-embeddings.ps1`, que genera sus propios bloques).

**Veredicto:** 4 archivos de `src/` tocados (3 de ellos sólo en el predicado) + tests en los mismos archivos. Nada en paths prohibidos.

## Spec — decisiones

| # | Decisión | Alternativas descartadas |
|---|---|---|
| 1 | Los cuatro sitios usan `crate::utils::env_value(var_os(..))` en la **misma** cadena `HOME → USERPROFILE` que ya tenían (sitio 3 conserva `USERPROFILE → HOME`) | Un predicado nuevo por sitio: `FIND-738` ya fijó la semántica ("existe y no está vacío" ⇒ `Some(valor)`/`None`) y dos definiciones de lo mismo es la clase que este ticket cierra |
| 2 | Cada sitio extrae una función **pura** que recibe `Option<OsString>` y aplica `env_value` adentro; el call site productivo sólo lee el entorno | `set_var` en el test: **unsafe** en edición 2024 y racy bajo nextest (pre-mortem b). Testear sólo el predicado (como `FIND-738` sitios 1-2) deja el call-site sin cubrir: revertirlo a `var_os` no rompe nada |
| 3 | `mcp init` **sin home utilizable** devuelve `Error::Cli` accionable nombrando `--config`, en vez de resolver `.claude.json` / `.cursor/mcp.json` **relativos al CWD** | (a) Dejar el `else` relativo: con `HOME=""` produce **el mismo path** que hoy (`"" .join(".claude.json") == ".claude.json"`), así que el ticket no cambiaría **nada** observable y seguiría escribiendo en el repo reportando éxito. (b) Escribir en el home "real" vía API de plataforma: dependencia nueva para un caso que ya tiene flag (`--config`). El ticket §6 autoriza explícitamente "fallar o escribir en el sitio correcto" y pide declararlo |
| 4 | La rama de proyecto de `claude-code` (`.mcp.json` local, si existe) **se conserva** | Es config project-scoped documentada (`README.md:53`), no un fallback de home |
| 5 | `opencode` no se toca: `.opencode/opencode.json` es project-relative **por diseño** y no tiene rama de home | — |
| 6 | Lugar del `avance`: `docs/dev/avance/activo/bindings.md` | Es donde vive `FIND-738`, la misma clase; partirla en `operaciones.md` rompe la lectura |
| 7 | Tras el review: la cadena `env_value(..).or_else(..)` se factoriza en `utils::env_chain(first, second)` (una implementación para los 4 sitios) y `resolve_default_config_path_from` recibe `local_mcp_exists: bool` | Cuatro copias de la cadena no se pueden testear sin `set_var` y el bug real (un `Some("")` que corta el `or_else`) quedó sin guardia; el probe de `.mcp.json` hacía que el test dependiera del CWD |

## El cambio observable (leer antes de aprobar)

**`vanta-cli mcp init` con `HOME`/`USERPROFILE` ausentes o vacíos pasa de "escribe
la config en el CWD y reporta éxito" a "error accionable"**. Con `HOME=""` y
`USERPROFILE` seteado (Windows normal) sigue funcionando igual que siempre: el
vacío cae al segundo. Sólo falla cuando **ninguno** de los dos transporta un
valor. Mensaje: `Cannot determine the cursor config directory: HOME and
USERPROFILE are both unset or empty. Pass --config <path> ...`. Es un cambio de
comportamiento **a mejor**: el estado anterior dejaba al cliente sin configurar y
un fichero plantado en el repo.

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | `mcp_init`: `home_env()` + `resolve_default_config_path_from(client, home, local_mcp_exists)` + error accionable sin home; 3 casos | ✅ |
| 2 | `cli_handlers/config.rs`: `user_config_path_from(home)`; 3 casos | ✅ |
| 3 | `cli_handlers/server.rs`: `home_bin_dir(home)`; 3 casos | ✅ |
| 4 | `config.rs`: `home_data_dir(home)`; 3 casos | ✅ |
| 5 | Verify (fmt/clippy/nextest/contrato `rg`) + review P2-01 + fixes del review (`env_chain`, `local_mcp_exists`, doc) + cierre | ✅ |

## Contrato mecánico — resultado

```
rg -n 'var_os\("(HOME|USERPROFILE)"\)' src/
src/config.rs:610-611                      env_chain(..)   (sitio 4)
src/cli_handlers/config.rs:41-42            env_chain(..)   (sitio 2)
src/cli_handlers/mcp_init.rs:149            env_chain(..)   (sitio 1)
src/cli_handlers/server.rs:382-383          env_chain(..)   (sitio 3)  [USERPROFILE primero]
src/hardware/mod.rs:123-124, 752-753        env_value / env_present (fuera del ticket, ya correctas)
src/utils/mod.rs:58                        (comentario del test, no es código)
```

**10/10 ocurrencias de código envueltas, 0 sin justificar.** `env_chain(first,
second)` es literalmente `env_value(first).or_else(|| env_value(second))`
(`src/utils/mod.rs:47`) — factorizado tras el review porque el filtrado tiene
que ir en **cada** variable **antes** del `or_else`: con
`var_os("HOME").or_else(..)` a secas, un `HOME=""` es `Some("")` y **corta la
cadena**, así que `USERPROFILE` no se consulta nunca. Ese error se cometió en la
primera versión y lo cazó la prueba E2E; ahora lo fija
`env_chain_falls_through_an_empty_first_value`.

## Verificación

| Comando | Resultado |
|---|---|
| `cargo fmt -p vantadb -- --check` | ✅ |
| `cargo clippy -p vantadb --all-targets -- -D warnings` | ✅ |
| `cargo nextest run --profile audit -p vantadb -E 'test(mcp_init) or test(config) or test(server) or test(local_model)' --build-jobs 2` | ✅ **90/90** |
| `cargo nextest run --profile audit -p vantadb -E 'test(f751) or test(empty_home) or test(home_bin_dir) or test(home_value_resolves) or test(opencode_config_is_project_scoped) or test(user_config_path_needs) or test(env_chain)'` | ✅ **7/7** |
| `cargo check -p vantadb --all-targets` | ✅ |
| `node scripts/docs/check-docs.mjs` / `check-links.mjs` | ✅ gating clear / 0 enlaces markdown roto |
| E2E `vanta-cli mcp init --client cursor` con `HOME=""` y `USERPROFILE` sin setear | ✅ exit **1**, mensaje accionable, **0 ficheros** en el CWD |
| E2E con `HOME=""` y `USERPROFILE=<fakehome>` | ✅ resuelve `<fakehome>\.cursor\mcp.json` (cae al segundo, no al CWD) |

## Review (GATE P2-01 — agente distinto, contexto fresco)

- **Revisor:** `vanta-review` (sub-agente; sin participación en la implementación → P2-01 satisfecho).
- **Enfoque:** lectura del diff de los 4 archivos + task file; `rg` del contrato por su cuenta; re-corrida de la suite filtrada (`90/90`), `fmt` y `clippy`; comprobación de la firma pública en `tests/api/public-api.txt:1320`; `git diff --name-only` contra los paths prohibidos. Brainstorm de alternativas (dejar el `else` relativo / home por API de plataforma / `set_current_dir`).
- **Veredicto:** **approve** — 0 critical / 0 high; 2 medium (uno de proceso, uno de test-gap ya declarado), 2 low, 1 nit.
- **Requeridos, todos aplicados:**
  1. *(medium, proceso)* Faltaba esta sección `## Review` → añadida.
  2. *(medium, test-gap)* El fall-through `HOME="" → USERPROFILE` no tenía test y **ya se había shipeado mal una vez**: revertir `home_env()` a `var_os(..).or_else(..)` dejaba 90/90 verdes. → Extraído `utils::env_chain` (una sola implementación para los 4 sitios) + `env_chain_falls_through_an_empty_first_value`. El **orden** por sitio (`HOME` primero en 3, `USERPROFILE` primero en `server.rs`) sigue sin test — es el único hueco que queda y no es observable sin leer el entorno.
  3. *(low)* Doc block de `local_model_bases` mal adosado a `home_data_dir` → movido; `local_model_bases` recupera su doc con la referencia a `FIND-697`.
- **Aplicado además:** el `*.mcp.json` de `claude-code` dejó de sondearse desde el CWD del test — `resolve_default_config_path_from` recibe `local_mcp_exists: bool`, así que las dos ramas son deterministas y no dependen del directorio de ejecución.
- **Nit aceptado sin cambio (follow-up):** `HOME=" "` o un `HOME` **relativo** siguen produciendo rutas CWD-relativas en los cuatro sitios (`env_value` sólo filtra `is_empty()`). Fuera del alcance declarado del ticket; el upgrade path es exigir `Path::new(h).is_absolute()` en los cuatro helpers puros.
- **Límite del review:** el E2E del binario no fue reproducible por el revisor (disco lleno a mitad de build); lo sustituyó por el orden estructural `?`→I/O y la suite. Los E2E los corrió el implementador y están registrados arriba con su salida.

## Techo declarado

- Los tests del sitio 1 prueban **la resolución**, no el efecto "no escribió en el
  CWD": la resolución es el **primer** paso del handler (`target_path`,
  `mcp_init.rs:280`) y su `?` precede a **toda** lectura/escritura (`:285`, `:339`),
  así que un `Err` ahí implica cero I/O. No se usa `set_current_dir` (no hay
  precedente en el repo y `cargo test` comparte proceso); se cumple la cláusula del
  ticket "testeá la resolución del path y decilo".
- La **cadena** `HOME → USERPROFILE` ya está cubierta (`env_chain` + su test); lo
  único sin cubrir es el **orden de argumentos** por sitio, que sólo se puede
  observar leyendo el entorno. `env_value`/`env_chain` filtran `is_empty()`: un
  `HOME=" "` o un `HOME` relativo siguen siendo un home para este código
  (follow-up propuesto en `## Review`).

## Cierre de la fila

_(se completa al cerrar)_
