---
title: "Imponer -p <crate> y --locked en los 5 pasos Cargo sin candado de ci-rust.yml"
kind: task
description: Cinco invocaciones cargo check / cargo nextest run en .github/workflows/ci-rust.yml
id: FIND-392
type: fix
state: IN_PROGRESS
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
depende_de: FIND-466
precede_a: FIND-455
---

# FIND-392 — `-p` + `--locked` en los steps Cargo de test/test-windows/test-macos

## Objetivo

Cinco invocaciones `cargo check` / `cargo nextest run` en `.github/workflows/ci-rust.yml`
corrían sin `--locked` y sin selector de paquete, permitiendo deriva de `Cargo.lock`
en CI y compilaciones no acotadas. Acceptance: en los jobs `test`, `test-windows`
y `test-macos`, toda línea `cargo check|nextest run` lleva `--locked` y `-p <crate>`
o `--workspace`.

## Contrato (literal del plan)

```
python -c "import yaml,re; d=yaml.safe_load(open('.github/workflows/ci-rust.yml',encoding='utf-8')); bad=[(j,l.strip()) for j in ('test','test-windows','test-macos') for s in d['jobs'][j]['steps'] for l in (s.get('run') or '').replace(chr(92)+chr(10),' ').splitlines() if re.search(r'cargo (check|nextest run)',l) and not ('--locked' in l and re.search(r'(^| )(-p |--workspace)',l))]; assert not bad, 'infractor: '+repr(bad); print('OK')"
```

**Estado ANTES (medido contra HEAD):** AssertionError con exactamente los 5
infractores listados abajo.

## DISCOVERY — decisión por step (no en bloque)

Hecho estructural medido en `Cargo.toml` (raíz, línea ~779): el workspace tiene
paquete raíz `vantadb` y `default-members = [".", "vantadb-python",
"vanta-memory", "vantadb-server", "vantadb-mcp"]`. Por tanto, `cargo check` /
`cargo nextest run` **sin `-p` cubrían los 5 default-members**, no solo `vantadb`.

Consecuencia: la alternativa `--workspace` NO era neutra — añadiría
`vantadb-wasm`, `vanta-proxy` y `vantadb-ffi-core`, excluidos a propósito de
`default-members` como EXPERIMENTAL (ADR-031 / STABLE-09: wasm32 toolchain extra
y proxy = heavy compile que romperían el Fast Gate). Tampoco `-p vantadb` solo:
recortaría cobertura en 4 de los 5 crates (CI verde probando menos — el riesgo
principal del brief).

**Decisión aplicada (idéntica en los 5):** enumerar los default-members con
`-p` explícito + `--locked`. Cobertura byte-a-byte equivalente a la anterior.

| Job | Step | Antes | Después | Cobertura |
|-----|------|-------|---------|-----------|
| test (Linux) | Run tests (Audit profile, core) | `cargo nextest run --profile audit --features "cli,arrow,tls,opentelemetry"` | `... -p vantadb -p vantadb-python -p vanta-memory -p vantadb-server -p vantadb-mcp --locked` | idéntica (default-members) |
| test-windows | Cargo check (Windows smoke test) | `cargo check` | `cargo check -p <5 default-members> --locked` | idéntica |
| test-windows | Run quick integrity tests | `cargo nextest run --profile ci-windows --features ...` | `... -p <5> --locked` | idéntica |
| test-macos | Cargo check (macOS smoke test) | `cargo check` | `cargo check -p <5> --locked` | idéntica |
| test-macos | Run tests (Audit profile, core) | `cargo nextest run --profile audit --features ...` | `... -p <5> --locked` | idéntica |

`--features "cli,arrow,tls,opentelemetry"` sigue activando features solo en los
paquetes seleccionados que las definen (comportamiento cargo multi-paquete),
igual que antes.

## Alcance acotado (NO tocado por decisión del owner)

`public-api-snapshot`, `test-embedding-features`, `msrv`, `experimental-check`,
`release-combo` quedan fuera aunque tampoco cumplen la regla; `minimal-versions`
no admite `--locked` por diseño. Sin ampliar alcance.

## Referencias

- `docs/dev/workflow/RULES.md` — Regla `-p <crate> --locked`, `--build-jobs 2`.
- Regla 1 de `.opencode/AGENTS.md`.
- `--locked`: "Asserts that the exact same dependencies and versions are used
  as when the existing Cargo.lock was originally generated... deterministic
  builds, such as in CI pipelines." — Cargo Book,
  https://doc.rust-lang.org/cargo/commands/cargo-build.html (verificado).
- `cargo nextest run --workspace` / `-p` / `--profile` — nextest docs,
  https://nexte.st/docs/running/ (verificado).
- FIND-466 (commit 29a27e56f): añadió entradas `paths:` a este fichero — respetado.

## Evidencia

- ANTES: `AssertionError: infractor: [('test', 'cargo nextest run --profile audit --features "cli,arrow,tls,opentelemetry"'), ('test-windows', 'cargo check'), ('test-windows', 'cargo nextest run --profile ci-windows --features "cli,arrow,tls,opentelemetry"'), ('test-macos', 'cargo check'), ('test-macos', 'cargo nextest run --profile audit --features "cli,arrow,tls,opentelemetry"')]`
- DESPUÉS: `OK`. YAML re-parseado válido; las 5 líneas resueltas listan
  `-p` ×5 + `--locked` (dump del YAML confirmado).

## Resultado

- CIERRE: contrato verde, cobertura preservada, alcance respetado.
