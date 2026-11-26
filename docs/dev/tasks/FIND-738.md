---
title: "FIND-738 - un valor de entorno vacío no debe activar un flag (5 sitios, 2 predicados)"
kind: task
status: completed
description: "Cinco sitios más decidían un flag o una ruta con `var_os(..).is_some()` / `.is_none()`: definir la variable vacía activaba el modo (o lo desactivaba). Se unifican en dos predicados con nombre (`env_present`, `env_value`) compartidos en `src/utils` y en `tests/common`, con un test de los tres casos por sitio."
---

# FIND-738 - un valor vacío no es un flag, y una ruta vacía no es una ruta

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`
> **Ruta:** `vanta-worker` · **Carril:** R (1 slot libre; `FIND-750` ocupa el otro) · **Appetite:** 1-1,5 h
> **Dependencias:** `FIND-731` ✅ (`1add025e5`) — el predicado de referencia y el patrón de test sin `set_var`.
> **Origen:** review de `FIND-731` §"Clase completa" — la misma forma en 5 sitios, ninguno arreglado allí.

## Contexto

`std::env::var_os` devuelve `Some("")` cuando la variable **existe vacía**.
Con `VAR= cargo nextest run …` — la forma shell de "definir y olvidar", o un
`env:` mal escrito en un workflow — un predicado de *presencia* dice "sí" y el
modo se activa (o, con `.is_none()`, se desactiva). `FIND-731` fue grave porque
auto-aceptaba el contrato público; estos cinco son menores en consecuencia pero
idénticos en forma.

La semántica correcta **no es una sola**:

| Forma de la variable | Predicado | Por qué |
|---|---|---|
| flag booleano (`VANTADB_QUIET`, `FIND632_DENY_ALLOWLIST`, `FIND319_*_CHILD`) y chequeo de presencia (`HOME`) | **existe y no está vacía** | un flag vacío no es un flag; un `HOME` vacío no es un hogar |
| variable que **transporta un valor** (`VANTADB_V090_CLI`, `VANTADB_PROFILE_PATH`) | **existe, no está vacía** → `Some(valor)` / `None` | `Some("")` como ruta produce una ruta vacía: `Command::new("")` o `PathBuf::from("")` |

Son **dos** predicados, no uno. Aplicar el mismo fix mecánico a los cinco
habría dejado `VANTADB_V090_CLI=` intentando lanzar un binario con nombre vacío.

## Los cinco sitios (líneas verificadas en `develop` @ `0094f0f6a`)

| # | Sitio | Forma previa | Riesgo real |
|---|---|---|---|
| 1 | `src/hardware/mod.rs:320-321` | `var_os("VANTADB_QUIET").is_some() \|\| var_os("VANTADB_SILENT").is_some()` | **el único con consecuencia para un humano**: un `VANTADB_QUIET=` heredado del shell oculta la salida de diagnóstico justo cuando se depura |
| 2 | `src/hardware/mod.rs:749-750` (guard del test) | `var_os("VANTADB_PROFILE_PATH").is_none() && (var_os("HOME").is_some() \|\| var_os("USERPROFILE").is_some())` | `VANTADB_PROFILE_PATH=` **salteaba en silencio** las aserciones de home-dir: producción ignora el vacío y usa `HOME`, el guard creía que había ruta custom |
| 3 | `src/storage/engine/tests/read_only_files.rs:101` | `var_os("FIND632_DENY_ALLOWLIST").is_some()` | `FIND632_DENY_ALLOWLIST=` denegaba toda la allowlist: los tests de `read_only` fallaban ruidosamente (no falso verde) |
| 4 | `tests/core/snapshot_certification.rs:2800` | `var_os("FIND319_WINDOWS_NAME_TEST_CHILD").is_none()` | `FIND319_…=` corría la rama *hija* en el proceso padre: se perdía el aislamiento que da sentido al test |
| 5 | `tests/compat_v090.rs:424` | `let Some(cli) = var_os("VANTADB_V090_CLI") else { skip }` | `VANTADB_V090_CLI=` intentaba `Command::new("")` → panic "spawn v0.9.0 vanta-cli" en vez del skip que el propio docstring promete |

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `src/hardware/mod.rs:95-130` | `default_profile_path` y su orden de resolución documentado (`VANTADB_PROFILE_PATH` → `~/.vanta` → CWD) | el sitio 2 es el *guard* de un test sobre esta función; había que ver si el productor ya trataba el vacío (sí: `if !custom.is_empty()`, línea 117) |
| `src/hardware/mod.rs:290-324` | `log_adaptive_status` y su `#[cfg(feature = "cli")]` | sitio 1 |
| `src/hardware/mod.rs:414-431, 700-781` | `mod tests`, el guard del sitio 2 y `test_log_adaptive_status_respects_quiet_env` (que usa `set_var`) | decidir dónde viven los tests nuevos y qué estilo NO repetir |
| `src/storage/engine/tests/read_only_files.rs:1-124` | `fjall_open_allowlist` completa y doc de `FIND632_DENY_ALLOWLIST` | sitio 3: el flag es una sonda RED documentada, no un flag de producto |
| `tests/core/snapshot_certification.rs:1-20, 2796-2839` | cabecera, `#[path = "../common/mod.rs"]`, el test y su spawn hijo | sitio 4 |
| `tests/compat_v090.rs:1-30, 405-466` | cabecera, imports, el test y su mensaje de SKIP | sitio 5 |
| `tests/api/public_api.rs:40-84` | el predicado de `FIND-731` y **por qué** recibe el `Option` | referencia de semántica y de estilo de test |
| `tests/common/mod.rs:1-3` | `#![allow(dead_code)]` y el harness de 22 KB | decidir si se reusa el harness (no) o se agrega un fichero hermano (sí) |
| `src/utils/mod.rs:1-10` | módulo de utilidades stateless, `pub use confidence_metrics::...` | hogar del helper del crate lib |

**Referencias hacia dentro (quién consume cada flag)**

- `src/bin/vanta-cli.rs:43` setea `VANTADB_QUIET=1` (no vacío) → sigue silenciando. Sin cambio.
- `src/hardware/mod.rs:777-779` (`test_log_adaptive_status_respects_quiet_env`) setea `=1` → sin cambio.
- `docs/dev/avance/activo/bindings.md` §`FIND-731` documenta la forma con valor (`=1`) como camino válido; se mantiene.
- No hay ningún workflow ni script del repo que exporte alguno de los cinco en forma vacía (`rg` sobre `.github/`, `dev-tools/`, `scripts/` = 0).

**Referencias hacia afuera**

- `crate::utils::env_present` / `env_value`: `pub(crate)`, **no** agregan superficie pública → el golden `tests/api/public-api.txt` no cambia (verificado: `cargo nextest -p vantadb --test public_api` no corre acá por costo nightly; en su lugar se razona sobre la regla — `pub(crate)` no aparece en la superficie `pub`, y el CI `public-api-snapshot` lo confirma).
- `tests/common/env_var.rs`: crate de test, `pub` interno sin efecto fuera de él.

**Veredicto:** 3 archivos de `src/`/`tests/` existentes tocados + 1 fichero nuevo de test compartido + 1 helper en `src/utils/mod.rs` (sin `pub`). Nada en paths prohibidos (`tests/api/public_api.rs` **no** se toca: se importa la semántica, no el símbolo).

## Spec — decisiones

| # | Decisión | Alternativas y por qué se descartan |
|---|---|---|
| 1 | **Dos** predicados con nombre: `env_present(Option<OsString>) -> bool` y `env_value(Option<OsString>) -> Option<OsString>` | Un solo predicado booleano obligaba a cada sitio con ruta a re-parsear `var_os` (dos lecturas del entorno) o a `PathBuf::from("")`. La distinción flag/valor es la del ticket §8. |
| 2 | Helper en `src/utils/mod.rs` (`pub(crate)`, sin fichero nuevo) para los sitios 1-2-3 | **(a)** `src/hardware/` — crea dependencia `storage → hardware`, dirección equivocada. **(b)** `src/config.rs` — **prohibido** (`FIND-750` en vuelo). **(c)** `src/lib.rs` raíz — funciona pero mete plumbing de test en la raíz del crate. `utils` ya es "utilidades stateless" y `pub(crate)` no toca la API pública. |
| 3 | Helper **copiado** en `tests/common/env_var.rs` para los sitios 4-5 — misma semántica, mismos nombres | **No se pueden compartir los cinco**, y la razón es de crate, no de pereza: `tests/*.rs` son **crates distintas** entre sí y del crate lib; un `pub(crate)` de `src/` no es alcanzable desde ellas, y promoverlo a `pub` **cambia la superficie pública** → obligaría a regenerar el golden de `tests/api/public_api.rs`, que es el path prohibido del ticket y cuyo gate cuesta ~13 min de rustdoc nightly. Se prefiere 2 definiciones (lib + tests) que 5 copias, con nombres y semántica idénticos y el comentario de cabecera que explica por qué son dos. |
| 4 | `VANTADB_PROFILE_PATH` vacío = **ignorar, no error** | El productor (`default_profile_path:110-120`) ya documenta y aplica "if set and non-empty" y cae al orden de resolución (home → CWD). Fallar con un error accionable rompería `VANTADB_PROFILE_PATH=` heredado en CI y **no** protege nada: un perfil es caché regenerable, no un dato. Lo que estaba mal no era el productor sino el guard del test (`.is_none()` decía "hay ruta custom" cuando el productor decía "no hay"). |
| 5 | `VANTADB_V090_CLI=` = **ignorar y saltar con el mensaje de SKIP** (que ahora nombra el caso vacío) | Fallar sería peor: el propio test declara que sin binario se saltea, y un env heredado vacío no es una petición de correr el check cross-version. El mensaje se amplía para que quien lo seteó y se equivocó lo vea, en vez de creer que corrió. |
| 6 | Tests sobre el **predicado**, no sobre el efecto, en los sitios 1-2 | Capturar la salida de `log_adaptive_status` (consola/ANSI) requiere subproceso y es más frágil que el predicado que cubriría; `default_profile_path` lee el entorno adentro, y `set_var` es racy con nextest. Se testean los tres casos (ausente / vacía / con valor) con los nombres de variable de cada sitio, y se dice acá. Sitios 3-4-5 sí testean su propio punto de decisión porque son puros. |
| 7 | No se agrega variante de `default_profile_path` que reciba el entorno | Sería la forma testeable, pero es refactor de un productor fuera del hallazgo (el productor ya es correcto). Queda como `ponytail:` en el test. |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | RED: predicados con la semántica vieja → los 5 tests nuevos fallan | ✅ |
| 2 | GREEN: `env_present` / `env_value` en `src/utils/mod.rs` + los 3 sitios del crate lib migrados | ✅ |
| 3 | GREEN: `tests/common/env_var.rs` + los 2 sitios de test migrados | ✅ |
| 4 | Verify del contrato (`-E` de los 5 términos + `rg` de cierre) | ✅ |
| 5 | OCR review + review P2-01 (agente distinto) → **approve** | ✅ |
| 6 | Commit local del código (sin push, sin `--amend`) | ✅ `85d8423bd` |
| 7 | Hallazgo medio del review aplicado: `HOME` vacío cae a `USERPROFILE` en el productor | ✅ commit propio |
| 8 | Cierre de fila: Backlog + `avance/activo/bindings.md` + este task file | ✅ commit `docs(avance,backlog)` |

## Verify

| Check | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb -E 'test(quiet) or test(silent) or test(profile_path) or test(deny_allowlist) or test(v090)' --build-jobs 2` | ✅ **21 tests run: 21 passed** (2971 skipped; los otros 16 que matchean los términos son preexistentes y siguen verdes) |
| `cargo nextest run --profile audit -p vantadb --test snapshot_certification -E 'test(find319_child_flag)' --ignore-default-filter --build-jobs 2` | ✅ **1 test run: 1 passed** (el binario está en la exclusión de `profile.default.default-filter`, por eso el flag) |
| `rg -n 'env::var(_os)?\([^)]*\)\.is_some\(\)' src/ tests/` | ✅ **0 coincidencias** (`exit=1`) → no queda nada que justificar |
| `rg -n 'env::var(_os)?\([^)]*\)\.is_none\(\)' src/ tests/` | ✅ 0 coincidencias (`exit=1`) |
| `cargo fmt --check` | ✅ exit 0 |
| `cargo clippy -p vantadb --all-features --lib -- -D warnings` | ✅ exit 0 |
| `cargo clippy -p vantadb --all-features --test compat_v090 -- -D warnings` | ✅ exit 0 |
| `cargo clippy -p vantadb --all-features --test snapshot_certification -- -D warnings` | ✅ exit 0 |
| `cargo check --no-default-features --features fjall` | ✅ exit 0 (valida que los `#[allow(dead_code)]` cubren la config sin `cli`/`sysinfo`) |
| Suite workspace / `deny` | ⏸ no corridos acá por diseño: el árbol tiene WIP de otra sesión (`FIND-750`, `src/sdk/version_history.rs`, `src/node/vector_data.rs`, `tests/wal_rollback.rs`) — se corren en el batch de la wave |

## Evidencia

**RED (Prove-It).** El helper se escribió primero con la semántica vieja
(`value.is_some()` / `value`), que es exactamente el comportamiento previo de
los cinco sitios, y los tests nuevos fallaron:

```
Summary [2.056s] 21 tests run: 18 passed, 3 failed
FAIL hardware::tests::profile_path_env_ignores_empty_value
     panicked: an empty value must count as unset, never as PathBuf::from("")
FAIL storage::engine::tests::read_only_files::deny_allowlist_flag_ignores_empty_values
     panicked: defined-but-empty = probe off
FAIL hardware::tests::quiet_and_silent_flags_ignore_empty_values
     panicked: VANTADB_QUIET: defined-but-empty must not silence
```

Los dos sitios de integración se probaron por separado (su helper vive en
`tests/common/env_var.rs`), revirtiéndolo a passthrough temporalmente:

```
FAIL vantadb::snapshot_certification find319_child_flag_ignores_empty_values
     panicked at tests\core\snapshot_certification.rs:3380: defined-but-empty = parent
FAIL vantadb::compat_v090 v090_cli_path_ignores_empty_value
     panicked at tests\compat_v090.rs:482: defined-but-empty = skip, never Command::new("")
```

**GREEN.** Restaurada la semántica (`is_some_and(|v| !v.is_empty())` /
`.filter(|v| !v.is_empty())`), mismas corridas: 21/21 y 1/1.

**Sitios verificados** (las líneas se habían movido respecto del ticket; estas
son las medidas en `develop` antes de editar):

| Sitio | Línea medida | Predicado aplicado |
|---|---|---|
| `VANTADB_QUIET` / `VANTADB_SILENT` | `src/hardware/mod.rs:319-321` | `env_present` |
| `VANTADB_PROFILE_PATH` (guard de test) | `src/hardware/mod.rs:748-750` | `env_value` (path) + `env_present` (HOME/USERPROFILE) |
| `FIND632_DENY_ALLOWLIST` | `src/storage/engine/tests/read_only_files.rs:101` | `env_present` |
| `FIND319_WINDOWS_NAME_TEST_CHILD` | `tests/core/snapshot_certification.rs:2803` | `env_present` (negado) |
| `VANTADB_V090_CLI` | `tests/compat_v090.rs:429` | `env_value` (path) |

**Nota de desvío (commit):** el commit `85d8423bd` lo materializó el árbol
compartido — otra sesión trabaja en `develop` al mismo tiempo (`FIND-750`) — y
de paso normalizó a ASCII los guiones largos y puntos suspensivos de los
comentarios (`…`→`.`, `—`→`-`). No cambia una línea de código ni la semántica.

**Mensaje de commit:** el contrato pedía `fix(core): an empty env var must not
flip a flag (FIND-738)`; el commit quedó con `fix(env): un valor vacio no es un
flag ni una ruta — env_present/env_value en 5 sitios (FIND-738)`. **No se
reescribe historia para corregirlo**: el índice y el HEAD los comparte el carril
`FIND-750`, y un `--amend` se llevaría el commit de la otra sesión si commitea
en el medio. La redacción exacta del contrato es la de esta línea, que es la
fuente de verdad para el `progreso`. El `§Review` y los ajustes de este archivo
van en un commit **nuevo** encima (`docs(tasks): ...`), con pathspec explícito.

## Coincidencias restantes del `rg` de cierre

Ninguna. El contrato da **0** y la clase ampliada también:

- `.is_none()` sobre `env::var`/`var_os`: 0.
- Las 16 apariciones restantes de `var_os(` son `if let Some(..)` / `let Some(..)`
  donde el **valor se usa** (no se decide un booleano por presencia): `PATH`
  (`src/cli_handlers/mcp_call.rs:212`, `diagnostics.rs:158`), `SystemRoot`
  (`diagnostics.rs:162`), `HOME`/`USERPROFILE` en la resolución de rutas
  (`src/cli_handlers/config.rs:40`, `mcp_init.rs:132`, `server.rs:369`,
  `hardware/mod.rs:120` — **este último sí comprueba `!home.is_empty()`**),
  guardado/restaurado de valores en tests
  (`tests/prefetch_benchmark.rs:101,102,132`), y `update_requested(...)` de
  `FIND-731` (`tests/api/public_api.rs:136,170`).
- **Residual de la misma familia, fuera de los cinco sitios medidos y no tocado**
  (`NOTICED BUT NOT TOUCHING`) — ver §Residual `HOME`/`USERPROFILE`.
  **Requiere fila `FIND-*` propia**; no se arregla acá.

## Residual `HOME`/`USERPROFILE` (fila propuesta, la registra el orquestador)

**Cuatro** sitios leen `HOME`/`USERPROFILE` y aceptan el vacío. Con `HOME=`
(`var_os` → `Some("")`), `PathBuf::from("")` es la ruta vacía, y `.join(..)`
produce una ruta **relativa al CWD**. `src/hardware/mod.rs:120` **no** es parte
del residual: ya comprueba `!home.is_empty()`.

| Sitio | Con `HOME=` produce | Severidad del síntoma |
|---|---|---|
| `src/cli_handlers/mcp_init.rs:132` (`resolve_default_config_path`) | `.claude.json` / `.cursor/mcp.json` **en el CWD**; `home` es `Some("")`, así que no cae en el `else` | 🔴 **la peor: escribe**. `vanta-cli mcp init` reporta éxito, deja un fichero de config en el repo en vez del home, y el cliente nunca queda configurado |
| `src/cli_handlers/config.rs:40` (`user_config_path`) | `.vanta/config.toml` relativo → normalmente no existe → capa de config de usuario **vacía** | 🟠 **válido pero equivocado**: el CLI arranca con defaults e ignora en silencio lo que el usuario configuró (storage path, límites, proveedor). Si existe `./.vanta/config.toml` en el proyecto, se lee como si fuera del usuario |
| `src/cli_handlers/server.rs:369` (candidatos de spawn) | `.vanta/bin/<exe>` **relativo**, insertado **antes** del `exe_name` pelado | 🟠 ensancha la búsqueda del binario del server al CWD (forma de *path hijack*: `./.vanta/bin/<exe>` se ejecuta si existe) |
| `src/config.rs:609` (`local_model_bases`) | `.vantadb` relativo como base de búsqueda del modelo | 🟢 **molesto, casi limpio**: si no hay `./.vantadb` el modelo resuelve como ausente (clase `FIND-697`); si lo hay, acierta por accidente |

**Veredicto de severidad: 🟡 Media**, no Baja. Tres de los cuatro son
"válido pero equivocado" (no fallan, hacen otra cosa), y el de `mcp_init`
además **escribe** en el lugar equivocado reportando éxito. El de `config.rs:609`
por sí solo sería 🟢.

**Path bloqueado:** `src/config.rs` está en la lista de paths **prohibidos** de
este ticket (`FIND-750` en vuelo) — ese sitio no se puede arreglar hasta que
`FIND-750` cierre. Los otros tres están libres.

**Texto de la fila (para `docs/dev/Backlog.md`):**

> | `FIND-XXX` | 🟡 Media | **`HOME`/`USERPROFILE` definidos vacíos dan rutas relativas al CWD (4 sitios)** | `src/cli_handlers/mcp_init.rs:132` · `src/cli_handlers/config.rs:40` · `src/cli_handlers/server.rs:369` · `src/config.rs:609` | 🟢 1-2h | 🟡 Media | 🆕 | Misma clase que `FIND-738` (`var_os` devuelve `Some("")` con la variable vacía) pero con síntoma distinto: no invierte un flag, produce `PathBuf::from("")` → `.join(..)` = ruta **relativa al CWD**. `HOME=` es la forma que deja un shell al exportar sin valor. Peor caso: `mcp_init.rs:132` **escribe** la config del cliente en el CWD y reporta éxito (el cliente queda sin configurar); `config.rs:40` ignora en silencio la capa de config del usuario; `server.rs:369` añade `.vanta/bin/<exe>` relativo **antes** del `exe_name` pelado (ensancha la búsqueda del binario al CWD). `src/config.rs:609` solo hace que el modelo resuelva como ausente. `src/hardware/mod.rs:120` ya comprueba `!home.is_empty()` y no entra. | Origen: review de `FIND-738` (2026-11-26) | Ver: `FIND-738` |
>
> **Contrato de cierre:** los cuatro usan el predicado existente
> `crate::utils::env_value` (`FIND-738`, `src/utils/mod.rs`) —
> `env_value(var_os("HOME")).or_else(|| env_value(var_os("USERPROFILE")))` — de
> modo que vacío cae al fallback que cada sitio ya tiene (`else` del `match`,
> `None` de la cláusula `?`, o el candidato pelado). Un test por sitio con los
> tres casos (ausente / vacía / con valor) sin `set_var`. `rg -n
> 'var_os\("(HOME|USERPROFILE)"\)' src/` sin una ocurrencia envuelta en
> `env_value(..)` o guardada por `if !x.is_empty()` = 0, o cada resto
> justificado por escrito. **Dependencia:** `FIND-750` antes de tocar
> `src/config.rs`.

## Cómo correr los tests de esta tarea (importa para el sitio 4)

El sitio 4 (`FIND319_WINDOWS_NAME_TEST_CHILD`) vive en el binario
`snapshot_certification`, que **está excluido por `profile.default.default-filter`**
(`.config/nextest.toml:67`) por ser *heavy*. Sin `--ignore-default-filter` el
filtro por nombre devuelve **0 tests y exit 0**: un gate que no mide. Comando
correcto:

```
cargo nextest run --profile audit -p vantadb --test snapshot_certification \
  -E 'test(find319_child_flag)' --ignore-default-filter --build-jobs 2
```

Test: `vantadb::snapshot_certification find319_child_flag_ignores_empty_values`.
El resto de los términos del contrato (`quiet`, `silent`, `profile_path`,
`deny_allowlist`, `v090`) sí corren con el filtro por defecto porque su lib/bin
no está excluido.


## Cambio de comportamiento observable (blast radius)

`VANTADB_QUIET=` y `VANTADB_SILENT=` **dejan de silenciar**. Quien dependía de
esa forma — probablemente por accidente, porque el predicado solo miraba
presencia — verá salida nueva en stderr al arrancar la CLI. Eso **es** el punto
del ticket: un instrumento de diagnóstico no debe callarse por un valor vacío,
y menos justo cuando alguien heredó la variable de un shell. `VANTADB_QUIET=1`
(la forma documentada y la que usa `src/bin/vanta-cli.rs:43`) no cambia.

`VANTADB_V090_CLI=` deja de intentar lanzar un binario sin nombre y pasa a
saltar el test con el mensaje de SKIP (ampliado para nombrar el caso vacío).

`VANTADB_PROFILE_PATH=` sigue significando "sin ruta custom" (ya lo era en el
productor); cambia únicamente el guard de test, que ahora ejerce las
aserciones de home-directory en vez de saltearlas.

## Review (GATE P2-01 — agente distinto)

**Revisor:** `vanta-review`, sesión independiente (`ses_de9ded888ffegV0i44vC6vfEJS`), sin participación en la implementación. **Veredicto: ✅ approve** (0 críticos, 0 bloqueantes).

Verificado **por el revisor**, no heredado de mi reporte:
`nextest audit` con el filtro de los 5 nombres → 21/21 · `--test snapshot_certification -E 'test(find319_child_flag)' --ignore-default-filter` → 1/1 · `fmt --check` 0 · `clippy --all-features --lib -D warnings` limpio · **`--test public_api --ignore-default-filter` 4/4 con `git diff -- tests/api/` vacío ⇒ superficie pública intacta** (`src/lib.rs:168` es `pub mod`, pero las `fn` son `pub(crate)`) · `rg` de cierre 0 y `.is_none()` sobre env 0 · `git show --name-only` → 7 paths, **0 prohibidos**. Sin `unsafe`, sin `unwrap`/`expect` nuevos, sin `set_var` nuevo.

**Hallazgos y disposición**

| # | Sev. | Hallazgo | Disposición |
|---|---|---|---|
| 1 | media | `default_profile_path:120` y el guard `:748-750` **discrepaban** con `HOME=""` + `USERPROFILE` seteado: el productor no caía al segundo (`or_else` no corre sobre `Some("")`) y el guard, ya más estricto, entraba al assert de home-dir → **fallo espurio**, en la clase del ticket | ✅ **aplicado** — el productor usa ahora `env_value` en ambos lados (`src/hardware/mod.rs:120-121`), misma semántica que el guard. El defecto preexistente lo volvió visible mi cambio del guard: arreglar uno solo de los dos lados lo empeoraba |
| 2 | media | Los 5 tests son del **helper**, no del call-site: revertir `:319` a `.is_some()` deja los 21 verdes | 📌 **techo aceptado y documentado** (§Techo declarado). El tradeoff es `set_var` racy bajo nextest; el `ponytail:` del test nombra la extracción (`default_profile_path_from(env)`) si alguna vez hace falta cobertura de efecto. No se bloquea |
| 3 | baja | Residuos de la misma clase fuera del blast radius | 📌 §Residual `HOME`/`USERPROFILE`: 4 sitios con severidad razonada, row propuesta y contrato de cierre. No se tocan acá |
| 4 | nit | `for name in ["VANTADB_QUIET","VANTADB_SILENT"]` (`:794`): `name` solo aparece en mensajes; `#[allow(dead_code)]` en `utils/mod.rs:26,36` silencia permanentemente | 📌 Aceptados. El loop afirma que **las dos** variables reciben el mismo tratamiento (es el punto); los `allow` cubren las configs sin `cli`/`sysinfo` y están acotados con razón |

**Sin segunda ronda:** el veredicto fue `approve` a la primera; el hallazgo 1 se aplicó después y lo re-verifiqué con `-E 'test(quiet) or test(silent) or test(profile_path) or test(hardware)'` → **34/34** y `fmt --check` 0.

## Techo declarado

Los tests de los sitios 1-2 ejercitan el **predicado**, no el efecto: revertir
un call-site a `is_some()` los deja verdes (lo señaló el reviewer). Es
deliberado — `set_var` es racy bajo nextest y capturar la salida de
`log_adaptive_status` pide subproceso. Los sitios 3-4-5 sí testean su punto de
decisión completo porque son puros. Upgrade path si hace falta cobertura de
efecto: extraer `default_profile_path_from(env)` / `log_adaptive_status_with(env, ..)`.

## Cierre de la fila

- Fila `FIND-738` retirada de `docs/dev/Backlog.md` junto con su sección
  (`## Media 2026-11-25 - Flags de entorno vacío en 5 sitios más`), que quedaba
  con una sola fila.
- Entrada agregada al final de `docs/dev/avance/activo/bindings.md` (misma
  sección que `FIND-731`, para no partir la clase).
- `gen-index.mjs` **no** se corrió: se regenera en el cierre de wave.
