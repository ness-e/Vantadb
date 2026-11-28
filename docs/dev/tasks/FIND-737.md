---
title: "FIND-737 - Devolver el detector de data races: TSan con std instrumentado y una regla medible"
kind: task
status: completed
description: "Los 243 reportes del run 45668284667 no eran adjudicables porque std va precompilado: ni el interceptor de pthread_mutex ni la instrumentacion de atomicos lo ven. Se entrega job TSan con -Zbuild-std, nightly pinneado, supresiones por clase y un criterio que si distingue ruido de un defecto real: un solo reporte con frame del motor falla el job."
id: FIND-737
tags: [vantadb, ci, tsan, sanitizer, concurrencia]
---

# FIND-737 - La lane TSan vuelve con `std` instrumentado

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-737`, 🟠 Alta / 🟠 Alta)
- **Origen:** `FIND-734` (2026-11-25) — **Ver:** `FIND-714`, `FIND-227`, `FIND-730`
- **Esfuerzo:** 🟡 3-4 h
- **Estado:** ✅ COMPLETED (local, sin push — Regla 7)

## Premisa (re-medida al abrir — no re-derivada)

La fila afirma que **no hay job de TSan en CI** y que el obstáculo es `std` precompilado. Medido en
HEAD el 2026-11-28:

```
rg -n 'sanitizer-tsan|tsan-suppressions' .github/workflows/ci-rust.yml   -> 0
git log --oneline -S'tsan-suppressions'                                   -> el borrado de FIND-734
```

**Confirmada.** Cero TSan en CI; el único sanitizer vivo es `sanitizer-asan`.

## Blast Radius

- **Paths tocados:** `.github/workflows/ci-rust.yml` (job `sanitizer-tsan` al final, junto a
  `sanitizer-asan`), `.github/tsan-suppressions.txt` (nuevo), `docs/dev/operations/CI_POLICY.md`
  (§ TSan), `docs/dev/workflow/TRIGGERS.md` + `docs/dev/workflow/README.md` (nota de trigger),
  `docs/dev/Backlog.md` (fila), este task file.
- **Sin tocar:** `src/**`, `vanta-memory/**`, `vantadb-{python,node,wasm,ts}/**`, `vantadb-mcp/**`,
  `dev-tools/**`, `.opencode/**`, `docs/index.md`, `Cargo.toml`, cualquier otro workflow.
- **Carril ajeno (SKIP declarado):** `src/storage/**`, `src/attestation.rs`,
  `src/storage/engine/maintenance.rs`, `tests/certified_delete.rs` y
  `docs/dev/avance/activo/core-engine.md` los tiene tomados `FIND-903`; no se tocan ni se
  interpretan sus fallos de gate.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rust.yml` (1.349 líneas: `on:`, `env`,
  `permissions`, `concurrency`, `rust-scope`, `tests-guard`, `miri` y `sanitizer-asan` —
  el job nuevo copia **ese** estilo), `.github/actions/rust-setup/action.yml` (inputs
  `toolchain`/`components`/`free-disk`/`swap-mb`/`shared-key`; exporta `RUSTUP_TOOLCHAIN`, FIND-701),
  `docs/dev/operations/CI_POLICY.md` (§ Fast Gate + § TSan completo), `docs/dev/workflow/TRIGGERS.md`
  y `README.md` (índice de workflows), `rust-toolchain.toml` (pin 1.95.0 + `rust-src`).
- **Referencias hacia dentro:** `rust-scope` decide si corre `clippy`; el job nuevo **no** entra en
  `needs` de nada (no es required, no bloquea PRs) — por eso no puede reintroducir el deadlock de
  `FIND-765` ni el hueco de `FIND-787`. `tests-guard` no lo nombra: no hay suite que se pueda
  "saltar" en silencio porque el job no corre fuera de `workflow_dispatch`.
- **Referencias entrantes:** `rg -l 'sanitizer-tsan|TSan' docs/` → `CI_POLICY.md`, `TRIGGERS.md`,
  `README.md` (índice de workflows) y filas del Backlog (`FIND-714`, `FIND-734`). Ninguna otra.
- **Veredicto impacto:** **bajo en código** (cero líneas de `src/**`, cero binarios), **alto en
  consecuencia**: hoy la ausencia de la lane está cubierta *por nombre* (`vanta-chaos`,
  `memory_concurrency`, `FIND-328`, `miri`) y ninguna de esas señales ve un data race silencioso.
  Un double-free real en el teardown de `FjallInner` imprime la misma forma que warn #42, así que
  la lane no era "ruidosa": era **ciega**.

## Spec — decisiones

| Decisión | Elección | Por qué | Alternativa rechazada |
|---|---|---|---|
| Instrumentar `std` | `-Zbuild-std` + `components: rust-src` | Es la única forma de que TSan vea futex y atómicos de `std`; el Unstable Book lo pide explícitamente | Una lista de supresiones más larga: expiró en 24 h (FIND-714) y para tapar #42 habría que nombrar un tipo del motor |
| Pin del nightly | `nightly-2026-10-02` = `1.101.0-nightly (c36f14571)` | Es la toolchain exacta donde se midió el coste (3,2 min / 164 MB); `-Zsanitizer` y `build-std` se rompen entre nightlies | `toolchain: nightly` (deriva silenciosa en cada bump de rama) |
| Invocación | `cargo`, no `cargo +nightly` | `rust-setup` exporta `RUSTUP_TOOLCHAIN` con el pin (FIND-701); `+nightly` lo **override** y anula el pin | `cargo +nightly` (parece pinneado y no lo está) |
| Trigger | `workflow_dispatch` | Lane pesada: nunca un check rojo por PR (dos niveles de CI_POLICY) | Cron en `ci-rust.yml` (arrastraría los 20+ jobs a diario); cron en `nightly.yml` (otro workflow, fuera de zona) |
| Criterio | **falla si un reporte tiene frame del motor** | Es lo único que distingue ruido de defecto real; el volumen de warnings no lo distingue | `race reports: 0` (imposible y sin significado con ruido de harness) |
| Supresiones | una entrada por **clase**, con razón escrita | Cuatro clases medidas en vez de 243 líneas que caducan | Fichero por reporte (FIND-714) o ninguno (la lane vuelve a nacer roja) |
| Margen | `timeout-minutes: 120` | Un timeout no es un veredicto | 50 min (el presupuesto que la fila citaba para la lane vieja) |

## Steps

1. 🟢 ✅ Pin verificado contra el canal oficial
   (`https://static.rust-lang.org/dist/2026-10-02/channel-rust-nightly.toml` →
   `version = "1.101.0-nightly (c36f14571 2026-10-01)"`), que es el `rustc` de la medición.
2. 🟢 ✅ `.github/tsan-suppressions.txt`: cuatro clases (`test::run_test`, `std::sys::sync::mutex`,
   `std::sys::sync::once`, `alloc::sync::Arc`) con su razón y el invariante "nunca un nombre con
   `vantadb` ni un `src/`".
3. 🟢 ✅ Job `sanitizer-tsan` en `ci-rust.yml`: pin + `rust-src` + `-Zbuild-std` + `--target`, sin
   `--locked`, `TSAN_OPTIONS` con `suppressions`/`halt_on_error=0`/`exitcode=0`/`print_suppressions=1`.
4. 🟢 ✅ Paso de veredicto: strip de ANSI, guard de "la suite terminó" (`Summary … N tests run`),
   regla de frame del motor (awk por bloques), canario de supresiones, artefacto `tsan-log`.
5. 🟢 ✅ Regla dry-run offline contra fixtures cortadas del log medido (warn #42 → marcado;
   reporte sólo-std → no marcado; `library/std/src/…` y `vantadb_py/src/…` → no marcados) con
   `gawk 5.3.2` y `mawk 1.3.4`.
6. 🟢 ✅ `actionlint .github/workflows/ci-rust.yml` → exit 0, sin salida.
7. 🟢 ✅ `CI_POLICY.md` § TSan reescrito (job, pin, receta, regla medible, deuda) y § Fast Gate con
   la fila del job; `TRIGGERS.md` + índice de workflows con la nota de dispatch.

## Verificación

- `python -c "yaml.safe_load(...)"` sobre `ci-rust.yml` → parsea, 27 jobs, claves del job correctas.
- `actionlint .github/workflows/ci-rust.yml` → **exit 0**.
- Regla de veredicto: `bash tsan-verdict-test.sh` → **ALL PASS** (6 checks) con `gawk` y `mawk`.
- `cargo ...` bajo TSan: **no ejecutado en este host** — ver deuda 1.

## Review

Gate aplicado: **fast** (tier mecánico; los paths son `.github/**` y `docs/dev/**`, ninguno de la
lista adversarial). Verificación mecánica: `actionlint` exit 0 + dry-run de la regla + parseo YAML.

## Deuda declarada (a confirmar en el primer run)

1. **El job nunca se ejecutó.** Diseñado y validado por `actionlint` + dry-run offline; **lo confirma
   el primer run de CI**. Cuatro cosas dependen de él: que `-Zbuild-std` + TSan enlace en
   `ubuntu-latest` con el nightly pinneado; que el wall time entre en 120 min; que las cuatro clases
   sumen cero hits; que el conteo de frames del motor sea cero.
2. **Sin schedule todavía.** Dispatch-only. Programarlo exige un `schedule:` en `ci-rust.yml` (que
   arrastraría todos los jobs) o un step de dispatch en `nightly.yml` (otro workflow): ambos fuera de
   la zona de esta tarea.
3. **Tests timing-bound bajo TSan** sin curar. El perfil `audit` ya excluye el tier Heavy; si un test
   con presupuesto propio no aguanta el slowdown (precedente ASan: `multi_namespace_stress`, 265 s vs
   su assert de 60 s), el primer run lo nombra y entra en la tabla RESOURCE-GUARD existente.
4. **Nada de `src/**`.** Esta lane no arregla hallazgos de TSan: abre la vía que hoy no puede
   distinguirlos.
