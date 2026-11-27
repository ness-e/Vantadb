---
title: "FIND-191: el cold-cache del snapshot public-api no puede envenenar el exit code"
kind: task
status: completed
description: "Residuo (a) de FIND-191: el exit 253 es STATUS_STACK_OVERFLOW del rustc hijo (no del parser ni del hilo de test); el test ahora le fija el presupuesto de pila por contrato en vez de heredar el del shell, con rail de compilacion y 4 tests unitarios."
tags: [tests, public-api, windows, stack-overflow]
---

# FIND-191: cold-cache del snapshot `public-api` — exit 253

## Metadata
- **Plan file:** [2026-11-26-batch-parciales-y-nuevas.md](../plans/2026-11-26-batch-parciales-y-nuevas.md) W4 (carril R, fila 11)
- **Fuente:** Backlog `FIND-191` (origen: review VER-01, FIND 2)
- **Esfuerzo:** 🟢 2h (real: ~2h)
- **Prioridad:** 🟢 Baja
- **Tipo:** Test harness (Rust, `tests/**`) — bug (`fix:`)
- **Estado:** ✅ COMPLETED

## Alcance declarado

**Residuo único:** el ítem `(a)`. El `(b)` está cerrado en `2510c16e` (el test es `#[ignore]`
y el job dedicado lo corre con `--run-ignored ignored-only`) — **no se reabre**.

## Contrato

```
cargo nextest list --profile audit -p vantadb -E 'test(public_api)'                                # 0 tests (ignored)
cargo nextest list --profile audit -p vantadb -E 'test(public_api)' --run-ignored ignored-only     # 1 test
cargo nextest run  --profile audit -p vantadb --test public_api                                    # 8/8 unidad
cargo nextest run  --profile audit -p vantadb --test public_api --run-ignored ignored-only         # 1/1 PASS
```

El `-E` del contrato devuelve **0 tests** y eso **no es verde**: es `FIND-794` (el filtro `test(...)`
matchea por nombre de función y el del snapshot está `#[ignore]`). La población se declaró
antes de correr el run real, como pide el contrato mecánico.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | job `public-api-snapshot` (`.github/workflows/ci-rust.yml:333-365`, **no editado**: carril de `FIND-219`); el humano que regenera con `VANTADB_PUBLIC_API_UPDATE=1`. `dev-tools/verify*.ps1` no lo invocan (`#[ignore]`) |
| Callees | `rustdoc_json::Builder` 0.9.10 → `rustup run nightly cargo rustdoc --lib -Z unstable-options --output-format json` → **`rustc` nightly**; `public_api::Builder` 0.52.2; `std::fs` |
| Implicaciones | Cero cambio de superficie pública y cero cambio del golden (`tests/api/public-api.txt` intacto: el run compara y pasa). El único contrato modificado es **qué presupuesto de pila recibe el compilador hijo** |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `tests/api/public_api.rs` (197L antes), `.config/nextest.toml` (108L),
  `rustdoc-json-0.9.10/src/builder.rs` (1-218), `public-api-0.52.2/src/lib.rs` (60-129),
  `rustc_driver_impl/src/lib.rs` (`pub fn main`, 1681-1702, upstream `master`),
  `docs/dev/Backlog.md` (fila `FIND-191`), `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md` (1-60),
  `.github/workflows/ci-rust.yml:293-365` (sólo lectura).
- **Referencias hacia dentro:** el golden `tests/api/public-api.txt` (828 808 B) tiene un único
  consumidor: este test. El job CI invoca por *binary id* (`public_api`), no por ruta.
  `rg "public_api" .github/**` → 1 job; `rg "public_api" dev-tools/**` → 0.
- **Referencias salientes:** `rustdoc_json` + `public_api` (`Cargo.toml:263-264`), `std::process`
  (vía `rustdoc_json`), y el toolchain `nightly` de `rustup`.
- **Veredicto impacto:** **bajo**. Un test `#[ignore]`, un consumidor, cero superficie pública.
  Lo que se preserva es el golden, no el harness.

## Fase 1 — Evidencia de Debugging

### (a) Reproducido: el 253 es `STATUS_STACK_OVERFLOW` del **compilador hijo**

`0xC00000FD` = 3221225725; `3221225725 % 256 = 253` (verificado con PowerShell). Un gate que lee
el byte bajo de un proceso que abortó reporta **253**.

Estado frío de verdad = `target/` vacío (sin `.rustc_info.json`, sin fingerprints, sin `vantadb.json`).
Con el binario de test directo y `CARGO_TARGET_DIR` a un directorio nuevo (0.20 GB: `cargo rustdoc`
pide sólo *metadata* de las dependencias), `RUST_MIN_STACK=2097152`:

```text
thread 'rustc' (28776) has overflowed its stack
Caused by:
  process didn't exit successfully: `...\nightly-x86_64-pc-windows-msvc\bin\rustc.exe
  --crate-name typenum ... --emit=dep-info,metadata ...` (exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)
thread 'public_api_snapshot_matches_committed_file' (9396) panicked at tests\api\public_api.rs:134:42
===== EXIT=101  8.21s
```

Dos hechos que fijan la causa:

1. **El proceso que aborta es `rustc`, y aborta en `typenum`** — una *dependencia*, 8 s después de
   arrancar, antes de que exista el JSON de `vantadb`. No es nuestro parser ni nuestra API.
2. **Es cold-only por construcción:** el único paso que compila es el primer run. Caliente, `cargo`
   no compila (`Finished target in 0.64s`): el binario hijo ni se ejecuta, y el run pasa. De ahí
   "el archivo sale bien y el 2º run pasa": el golden **nunca se tocó** porque el fallo ocurre antes
   de generar. No era benigno, era *invisible*.

### (b) Descartado, con evidencia

- **(i) recursión del parser `public-api`/`serde`:** descartada. El JSON es el mismo objeto de
  10 199 915 B en frío y en caliente (mismo `public-api` 0.52.2 sobre `rustdoc_types` con mapa plano
  de `Id`); una recursión del parser fallaría en **los dos** runs, y no falla. Además el aborto de
  arriba ocurre compilando `typenum`, sin haber leído el JSON.
- **(ii) stack chico del hilo de test:** descartada y **medida**. El hilo del test sobrevive con
  **512 KiB** (`RUST_MIN_STACK=524288` → el test corre y sólo falla por el hijo) y sólo revienta a
  **256 KiB** (`thread 'public_api_snapshot_matches_committed_file' has overflowed its stack`).
  Con el default de std (2 MiB) tiene **≥4× de margen**: no es el candidato.
- **(iii) proceso hijo:** **confirmada**. Es el único paso frío-dependiente y el único que aborta.

### (c) Por qué el disparador era el ambiente y no el disco

`verify.ps1:63` exporta `RUST_MIN_STACK=33554432` y `verify_changed.ps1:5` 16 MiB: el repo ya
depende de este lever (`docs/user/operations/BENCHMARKS.md:926,995` lo documenta: *"sin esto rustc
crashea al compilar"*). Pero el test **no lo fijaba**: heredaba lo que el shell del llamador tuviera.
Correr el snapshot desde un shell limpio, o con el `4194304` que recomienda
`docs/user/troubleshooting.md:53`, cambiaba el resultado del gate sin que nadie tocara el test.
Medición: 2 MiB ambient → aborto del hijo; 32 MiB → build frío completo OK (53.7 s).

## Fix aplicado

`tests/api/public_api.rs` (único archivo de código tocado, +115/−1):

- `CHILD_STACK_BYTES = 32 MiB` (`:86`) — el valor que **ya** usa el gate del repo (`verify.ps1:63`),
  no un número inventado acá, y documentado con la cadena 253/`0xC00000FD` en el propio archivo.
- `child_stack_budget(ambient) -> String` (`:103`): conserva un valor del operador **si es mayor**,
  y si no aplica el piso. Función pura → testeable sin disco.
- Wiring (`:133`): `.env("RUST_MIN_STACK", child_stack_budget(...))` sobre el `Command` que arma
  `rustdoc_json`. Se fija **en el hijo**, no en `std::env::set_var` (que además es `unsafe` desde
  Rust 2024 y ensuciaría el hilo del test y los tests paralelos).
- `const _: () = assert!(CHILD_STACK_BYTES >= 8 MiB)` (`:92`): bajar el piso es **error de
  compilación**, no nota de review.
- `mod child_stack` (`:268`): 4 tests — ambiente ausente, por debajo del piso (el `4194304` de la
  doc incluido), ≥ piso (64 MiB sobrevive), y basura/`""`.

### Por qué esto **no** es "subir el stack y llamarlo arreglo"

- La recursión **no es nuestra**: está en el `rustc` nightly de terceros, compilando una dependencia.
  Acotarla exigiría `src/**` (fuera de alcance) y ni siquiera está en nuestro código.
- No se sube el stack **global**: el hilo del test sigue con el ambiente (2 MiB en el GREEN de
  abajo). Se le da presupuesto **al hijo**, que es exactamente la rama que el contrato autoriza
  (*"si el problema es del proceso hijo, es un `Command` con stack"*).
- Lo que cambia no es el tamaño: es que el resultado **deja de depender de quién exportó qué**.
  Antes: gate verde o 253 según el shell del llamador. Ahora: contrato explícito en el test.
- **Techo declarado (`ponytail:`):** el piso es **empírico** (2 MiB revienta, 32 MiB compila este
  árbol). Si un nightly futuro necesita más, el rail de compilación no lo detecta — la señal es el
  job dedicado fallando con `STATUS_STACK_OVERFLOW`. Arreglo real: upstream (recursión del compilador).

## Regresión

La propiedad *"cold-cache no produce exit 253"* **no se automatiza**: necesita un `target/` vacío
(estado de disco), y ningún test de nextest puede garantizarlo. Declarado en el header del test
(`:19-27`) con el comando exacto:

```powershell
$env:CARGO_TARGET_DIR = "<dir vacío>"; $env:RUST_MIN_STACK = "2097152"
cargo nextest run -p vantadb --test public_api --run-ignored ignored-only   # esperado: 1/1 PASS
```

Antes del fix ese comando aborta en la primera dependencia (`0xC00000FD`). Lo que **sí** es
automatizable se automatizó: la **entrada que controlamos** (el presupuesto) tiene 4 tests unitarios
+ rail de compilación, así que quitarlo o bajarlo pone el build en rojo.

## Verificación

| Comando | Resultado |
|---|---|
| `cargo nextest list --profile audit -p vantadb -E 'test(public_api)'` | **0** tests (ignored → no cuenta como verde: `FIND-794`) |
| `... -E 'test(public_api)' --run-ignored ignored-only` | **1** test (`vantadb::public_api public_api_snapshot_matches_committed_file`) |
| `cargo nextest run --profile audit -p vantadb --test public_api` | **8/8** PASS, 1 skipped |
| `cargo nextest run --profile audit -p vantadb --test public_api --run-ignored ignored-only` | **1/1** PASS |
| RED (frío + 2 MiB, sin fix) | `thread 'rustc' has overflowed its stack` → `0xC00000FD` → aborto en 8.2 s |
| GREEN (frío + 2 MiB, con fix) | 1/1 PASS, 53.7 s, exit 0 |
| `cargo fmt -p vantadb -- --check` | ✅ |
| `cargo clippy -p vantadb --test public_api -- -D warnings` | ✅ |

## Context Save Point

Cerrado. Lo único que queda fuera es `FIND-794` (el `-E` que matchea 0 sale verde) — no es de esta
fila y ya tiene dueño.
