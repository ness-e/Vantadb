---
title: "FIND-752 - los seis binarios fuera del default-filter: 119 de 154 tests no tienen lane que los ejecute"
kind: task
status: completed
description: "Los seis binarios excluidos por nombre en .config/nextest.toml suman 154 tests. Medido contra gh: other_heavy lleva 0 exitos y 36 fallos en 30 dias y muere antes de correr los tests; security_audit no esta en ningun workflow; la lane de snapshot corre pero su selector deja 14 tests fuera. Total: 119 tests sin lane que los ejecute."
tags: [vantadb, ci, nextest, coverage, audit]
---

# FIND-752 — los seis binarios fuera del `default-filter`, contados

Auditoría, no arreglo. No se tocó `.config/nextest.toml` ni `.github/**`. El
entregable es la tabla de `docs/dev/operations/CI_POLICY.md` §2c y los números de abajo.

## Contrato

- (a) cuántos tests contienen los seis binarios → **medido**, 154.
- (b) dónde corre cada uno hoy, con último resultado real → tabla.
- (c) cuáles no corrieron con éxito en 30 días → **119 tests sin lane**.
- **Prohibido** cambiar el filtro o los workflows. Las propuestas quedan como texto (§Propuestas).

## Impacto mapeado (Regla 0)

| Archivo | Leído | Referencias hacia dentro | Referencias entrantes | Impacto |
|---|---|---|---|---|
| `docs/dev/operations/CI_POLICY.md` | 906 L (estructura y tono) | `AGENTS.md`, `dev-tools/verify.ps1:114`, `dev-tools/coverage-budget.ps1`, `docs/dev/workflow/RULES.md` | 25 ficheros (`rg -l CI_POLICY`) | **Se añade §2c** (tabla). Ninguna sección existente se reescribe; los números de §"Fast Gate Test Exclusions" no cambian |
| `.config/nextest.toml` | completo (108 L) | `Cargo.toml` tests, workflows | `verify.ps1`, `justfile`, `run-all-tests.ps1` | **sólo lectura** — se audita, no se edita |
| `.github/workflows/certification.yml` | jobs `other-heavy`, `memory-concurrency`, `on:` | `Cargo.toml` test targets | schedule/dispatch | **sólo lectura** (`FIND-763` en vuelo) |
| `.github/workflows/snapshot-certification.yml` | fases 01-07 | `.config/nextest.toml` | push/PR/dispatch | **sólo lectura** |
| `dev-tools/verify.ps1` | steps 104-139 | `CI_POLICY` | `.githooks/pre-push` | **sólo lectura** (fuera de scope) |

Veredicto: el cambio es **documental**. El riesgo no es de código: es publicar una
cifra sin medirla. Cada celda de la tabla lleva su comando o su `gh api`.

## (a) Inventario — medido

El `default-filter` de `profile.default` (heredado por `audit`) excluye los seis
binarios por nombre (`(package(vantadb) and binary(X))`). Con el filtro puesto
**el selector no alcanza ninguno**, así que el inventario exige
`--ignore-default-filter` (la forma del contrato devuelve 0 tests y exit 0 — el
mismo modo de fallo de `FIND-738`).

```bash
cargo nextest list -p vantadb --ignore-default-filter --run-ignored all \
  -E 'binary(benchmark_internal) or binary(benchmark_datasets) or binary(cli_tests) \
      or binary(concurrency_parity) or binary(snapshot_certification) or binary(security_audit)'
```

| Binario | Tests | Notas |
|---|---:|---|
| `benchmark_internal` | **1** | `test_benchmark_internal_10k` |
| `benchmark_datasets` | **1** | `test_glove100_hnsw_basic` — `return` temprano si falta GloVe-100 (`tests/benchmark_datasets.rs:18`) |
| `cli_tests` | **96** | `required-features = ["cli"]` (`Cargo.toml:605`) |
| `concurrency_parity` | **5** | 1 `#[ignore]`: `test_triple_backend_parity_validation` |
| `snapshot_certification` | **45** / **56** con `--features failpoints` | la lane compila con `failpoints`: el número que importa es 56 |
| `security_audit` | **6** | inyección IQL, auth bypass, input validation, resource exhaustion, timing attack, summary |
| **Total** | **154** (features por defecto) | la columna suma 1+1+96+5+45+6 |

Nota sobre el total: el comando de arriba compila con las features por defecto y
devuelve **154**. La lane de snapshot compila con `failpoints` y ese binario
crece de 45 a 56 (+11 medidos), o sea **165** si se cuenta con las features de
esa lane. El denominador del % es el medido con el comando del contrato: **154**.

## (b) y (c) Dónde corre cada uno — 30 días (2026-10-27 → 2026-11-26)

### Runs de `certification.yml` (schedule)

```bash
gh run list --workflow certification.yml --created ">=2026-10-27" --limit 200 --json databaseId,event,conclusion
gh api repos/{owner}/{repo}/actions/runs/<id>/jobs --jq '.jobs[] | "\(.conclusion) \(.name)"'
```

**39 runs programados: 37 `failure`, 2 `cancelled`, 0 `success`.**

Por job (agregado de los 39):

| Job | success | failure | skipped |
|---|---:|---:|---:|
| `Certification: other_heavy` | **0** | **36** | 3 |
| `Certification: memory_concurrency` | 35 | 0 | 3 |
| `Certification: mutation_test` | 0 | 36 | 3 |
| `Light benchmarks` / `High density` | 0 | 36 | 3 |

`other_heavy` no falla en un test: falla en el **step 4**, antes de ejecutar nada.
Run `45598671538` (2026-11-25, `c634e532`): `Verify required benchmark datasets
(TBH-01) -> failure`, `Download benchmark datasets -> skipped`, `Run remaining
heavy tests -> skipped`.

Causa raíz verificada: `main` es `c634e532`; la corrección de `FIND-328`
(descarga **antes** del gate, job acotado a domingos) está en `develop` en
`3f7fdd3fa`, **552 commits por delante de `main`**. La lane programada corre
`main`, así que sigue ejecutando el orden viejo: el gate pide GloVe-100 antes de
descargarlo → falla siempre.

### Runs de `snapshot-certification.yml`

**48 runs en 30 días: 39 `success`, 6 `failure`, 3 `cancelled`.** Último:
`45802050950` (2026-11-26 10:30 UTC) = `success`.

Pero la lane **no corre el binario entero**: `snapshot-certification.yml:148-164`
selecciona por nombre con `-E 'test(snapshot) or test(pending_restore) or test(schedule)'`.
Medido con las features de la lane:

```bash
cargo nextest list -p vantadb --ignore-default-filter --run-ignored all \
  --features failpoints --test snapshot_certification -E 'test(snapshot) or test(pending_restore) or test(schedule)'
# total 56, matched 42
```

**14 tests del binario no los selecciona ninguna fase** (ni la 05, que es el
rocksdb nativo por nombre): 4 `export_format_*`, **`find319_child_flag_ignores_empty_values`**
(el caso de `FIND-738`), 5 `vantafile_*`, 4 `wal_*`.

### `security_audit`

`git log -S"security_audit" -- .github/workflows` → **0 commits**: nunca estuvo
en un workflow. Además `dev-tools/verify.ps1:117` lo salta por nombre en el
fallback `cargo test`, `dev-tools/coverage-budget.ps1:79` también, y el nextest
del pre-push lo excluye por filtro. **Ningún lane lo ejecuta.**

## La tabla (publicada en `CI_POLICY.md` §2c)

| Binario | Tests | Lane hoy | Último resultado real | Veredicto |
|---|---:|---|---|---|
| `benchmark_internal` | 1 | `certification.yml` job `other_heavy` | run `45598671538`, 2026-11-25: **failure antes de los tests**; 0 success/36 failure en 30 d | **no corre** |
| `benchmark_datasets` | 1 | idem | idem | **no corre** (y si corriera sin dataset, retorna en verde) |
| `cli_tests` | 96 | idem | idem | **no corre** |
| `concurrency_parity` | 5 (4+1 `#[ignore]`) | `certification.yml` job `memory_concurrency` | run `45598671538`: **success**; 35 success/0 failure en 30 d | **corre** 4/5; el `#[ignore]` **no corre en ningún lane** |
| `snapshot_certification` | 56 (`failpoints`) | `snapshot-certification.yml` fases 03/04/05 | run `45802050950`, 2026-11-26: **success**; 39/48 en 30 d | **corre parcial**: 42/56; **14 no corren** |
| `security_audit` | 6 | ninguno | nunca | **no corre** |

## (c) El número que nadie tenía

**119 de 154 tests (77 %) no tienen ningún lane que los ejecute de verdad:**

| Origen | Tests |
|---|---:|
| `cli_tests` (job `other_heavy`, 0/36) | 96 |
| `snapshot_certification` no seleccionados por ninguna fase | 14 |
| `security_audit` (ningún workflow) | 6 |
| `benchmark_internal` (`--skip benchmark` también en local) | 1 |
| `benchmark_datasets` (sólo corre en un verde que no mide) | 1 |
| `concurrency_parity::test_triple_backend_parity_validation` (`#[ignore]`, sin `--include-ignored`) | 1 |
| **Total** | **119** |

## Los tres estados, separados

1. **Corre en otro lane y pasa:** `concurrency_parity` (4/5, weekly green) y
   `snapshot_certification` (42/56, dedicated lane green).
2. **Corre y falla:** ninguno de los seis — `other_heavy` falla **antes** de
   correr tests, que es distinto y es lo que hacía invisible el agujero.
3. **No corre en ningún sitio:** los 119 de arriba.

## Propuestas (texto; NO registradas en Backlog)

- **P-1 — `security_audit` no está cableado:** nunca estuvo en un workflow
  (`git log -S"security_audit" -- .github/workflows` → 0 commits) y los dos gates
  locales lo saltan por nombre. 6 tests (5 de seguridad real) sin lane. Es
  `FIND-664`/`FIND-752` en su forma pura. Propuesta: job `nightly` propio
  (`cargo test --release --test security_audit`), o devolverlo al `default-filter`
  midiendo antes el wall-time.
- **P-2 — el selector de la lane de snapshot es más estrecho que el binario:**
  `test(snapshot) or test(pending_restore) or test(schedule)` deja 14/56 fuera,
  incluido el test de `FIND-738`. Propuesta: `-E 'binary(snapshot_certification)'`
  en las fases 03/04 (el `--no-tests fail` ya protege el vacío), con las features
  por fase como hoy.
- **P-3 — `cli_tests` no necesita dataset y depende de un job que sí:** 96 tests
  atados al gate `TBH-01` de GloVe-100. Propuesta: separar `--test cli_tests` a un
  job sin dataset; `benchmark_internal`/`benchmark_datasets` sí se quedan detrás del gate.
- **P-4 — el `#[ignore]` de `concurrency_parity`:** `test_triple_backend_parity_validation`
  no lo ejecuta nadie (ningún workflow pasa `--include-ignored`). Propuesta:
  `--include-ignored` en ese step o `#[ignore = "razón"]` con lane que lo corra.
- **P-5 — la lane programada mide `main`, no `develop`:** el rojo de `other_heavy`
  es la corrección de `FIND-328` sin mergear (552 commits). No es un workflow roto;
  es una lane que prueba un árbol viejo. Propuesta: registrar la regla en
  `docs/dev/workflow/RULES.md` y verificar el verde de `other_heavy` en el primer
  run posterior al merge.

## Verificación

- Inventario: `cargo nextest list -p vantadb --ignore-default-filter --run-ignored all -E 'binary(...) or …'` → **154** (salida completa en `target/find752-list-all.txt`).
- Filtro de snapshot: mismo comando con `--features failpoints --test snapshot_certification -E 'test(snapshot) or test(pending_restore) or test(schedule)'` → **42 de 56**.
- Runs: `gh run list … --created ">=2026-10-27"` + `gh api …/runs/<id>/jobs` (agregado arriba).
- `main` vs fix: `gh api repos/{owner}/{repo}/compare/main...3f7fdd3fa` → `diverged, ahead_by 552, behind_by 1`.
- Docs: `node scripts/docs/check-docs.mjs` · `node scripts/docs/check-links.mjs` · `node scripts/docs/gen-index.mjs --check` (drift esperado: no se regenera en este commit).

## Review

_(P2-01 al cierre; ver `RESULTADO` del commit.)_
