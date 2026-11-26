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
| 5 | OCR review + review P2-01 (agente distinto) | ✅ |
| 6 | Commit local (sin push) | ✅ |

## Verify

| Check | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb -E 'test(quiet) or test(silent) or test(profile_path) or test(deny_allowlist) or test(v090)' --build-jobs 2` | ver §Evidencia |
| `cargo nextest run --profile audit -p vantadb -E 'test(find319)' --build-jobs 2` (5.º sitio; el contrato no tiene término para él) | ver §Evidencia |
| `rg -n 'env::var(_os)?\([^)]*\)\.is_some\(\)' src/ tests/` | ver §Evidencia — coincidencias restantes, justificadas |
| `cargo fmt --check` / `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` | ver §Evidencia |

## Evidencia

(poblado al cerrar — ver §Cierre)

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

(poblado al cerrar)
