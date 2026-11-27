---
title: "FIND-811: \"re-verificado\" no significa verificado — 10 filas medidas y un gate"
kind: task
status: completed
description: "4 de las 9 filas de FIND-607 describían código borrado 24-27 días antes. Barrido de las 10 marcas \"re-verificado / verificado <fecha>\" más antiguas contra HEAD: 8 VIVA (cita de línea corregida + comando re-ejecutable), 2 REFUTADAS y retiradas con su evidencia. Contrato: la marca solo vale si cita comando o commit; gate nuevo en check-harness-contract.mjs (89/89 fixtures)."
---

# FIND-811: la marca que no medía nada

## Metadata
- **Fuente:** `docs/dev/Backlog.md` §`Alta 2026-11-27` (fila `FIND-811`)
- **Origen:** verificación de las 9 hijas de `FIND-607` (2026-11-27) · **Ver:** `FIND-607`, `FIND-794`
- **Esfuerzo:** 4-8h (barrido) · **Prioridad:** 🟠 Alta · **Tipo:** docs/gobernanza del backlog (sin Rust)
- **Creado / cerrado:** 2026-11-27 · **Estado:** ✅ COMPLETED

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Tocado | `docs/dev/Backlog.md` (8 filas re-verificadas, 2 retiradas) · `dev-tools/check-harness-contract.mjs` (check `reverify` + 4 fixtures) · `docs/dev/avance/historial/backlog-history.md` (2 registros) |
| Inmutable | `.opencode/**`, `docs/dev/plans/**`, `docs/index.md`, `docs/feature-matrix.md`, `docs/FEATURES.md` (`FIND-808`), `.github/workflows/**` (`FIND-807`), `dev-tools/git-commit-isolated.ps1`, `src/**`, `tests/**` |
| Fuera de alcance | las 29 marcas con fecha que quedan sin barrer (ver §Deuda) |

## Premisa (medida, no re-derivada)

La fila afirma que **una marca `re-verificado <fecha>` no dice nada sobre el código**. Medido en HEAD antes de tocar nada:

```
$ rg -c "verificado" docs/dev/Backlog.md            → 54 filas
$ rg -c "re-verificado" docs/dev/Backlog.md         → 3 filas (EGO-15, FIND-290, FIND-811)
$ (stamps con fecha YYYY-MM-DD)                     → 39 filas
```

Y dos de las filas más viejas del inventario resultaron **falsas desde el día en que se escribieron** (§Tabla, `FIND-271` y `FIND-282`): la marca nunca midió el código, midió la relectura de la fila anterior. La premisa se confirma.

## Cómo se fechó sin caer en el incidente del 2026-11-26

`72643d9df` borró **2365 paths** y `19f759980` los restauró (2363). Cualquier `git log -S` los lista, y sin filtrarlos **todo aparece fechado el 2026-11-26** — el día del incidente, no el del cambio real. Protocolo aplicado a cada fecha de este barrido:

```powershell
# 1. correr el log SIN filtro y mirar qué commits aparecen
git log --format='%h %ad %s' --date=short -S"<símbolo>" -- <path>
# 2. descartar a mano los dos commits del incidente (72643d9df, 19f759980)
#    y sus vecinos de arreglo (290df23d0, db6ad7f79) antes de fechar nada
# 3. fechar sólo si el commit que queda es anterior al incidente, y
#    confirmar con `git cat-file -e <commit>:<path>` que el path existía ahí
```

Medido: `FIND-282` → sin filtro `19f759980`/`72643d9df`/`290df23d0`/`db6ad7f79`; con filtro **`b312482da` 2026-09-29**. `FIND-271` → los 4 commits del incidente y nada más; con `git cat-file -e` el path **no existe en ningún commit** (ni en `HEAD`, ni en `b7b2b4c04`).

## Tabla de las 10 verificadas (las más antiguas primero)

| Fila | Marca original | Veredicto HEAD 2026-11-27 | Evidencia (re-ejecutable) |
|------|----------------|---------------------------|---------------------------|
| `SCH-09` | verificado 2026-09-25 | **VIVA** | `rg -n "pub struct Edge" src/node/edge.rs` → `:9`, 5 campos, sin `properties` ni `valid_at_ms` |
| `FIND-241` | verificado 2026-10-04 | **VIVA** | `rg -n "cargo clippy" .github/workflows/ci-rust.yml` → 3 pasos, ninguno `--target wasm32`; `:146` cayó al cierre del `unsafe` (`file.rs:140-146`) |
| `FIND-243` | verificado 2026-10-04 | **VIVA** (cita muerta) | `rg -n "pub fn (get\|delete)\(" vantadb-wasm/src/lib.rs` → `:1232`/`:1242` siguen `&str`; `git ls-files vantadb-wasm/pkg` → 0 (la cita era glue no versionado) |
| `FIND-247` | verificado 2026-10-04 | **VIVA** | `rg -n force_rotate_to_archive src` → `wal_sharded.rs:726` y **0 callers de producción**; `rotate_all` (`:670-686`) no archiva |
| `FIND-253` | verificado 2026-10-04 | **VIVA** | Vanta `:301` mide después del setup; Lance `:397` / Chroma `:492` miden desde antes de `connect`/`create_collection` |
| `FIND-255` | verificado 2026-10-04 | **VIVA** (citas movidas) | 5 `SystemTime::now` crudos en `vanta-memory/src`; `rg -c web_time` → 0; sin feature `wasm` |
| `FIND-270` | verificado 2026-10-04 | **VIVA** (citas movidas) | `README.md:18` y `:266` siguen afirmando WAL; `rg -c -i wal vantadb-wasm/src/worker.rs` → 0 |
| `FIND-271` | verificado 2026-10-04 | **REFUTADA** → retirada | `git cat-file -e HEAD:docs/dev/architecture/CRASH_MODEL.md` → exit 128; el path **nunca existió**, y `docs/dev/wasm/CRASH_MODEL.md:75` ya decía "no WAL" en `b7b2b4c04` |
| `FIND-282` | verificado 2026-10-05 | **REFUTADA** → retirada | `RecallGovernance::source` (`auto_recall.rs:101`) tiene 4 productores y viaja en `RecallResult.governance` (`:246`) desde `b312482da` (2026-09-29) |
| `FIND-283` | verificado 2026-10-05 | **VIVA** (sin refutación medida) | `auto_recall.rs:974` existe; el hallazgo es de calidad del campo indexado, no de existencia |

**Retiradas: 2** (`FIND-271`, `FIND-282`), ambas con su registro en `docs/dev/avance/historial/backlog-history.md`.
**Citas de línea corregidas: 4** (`FIND-241`, `FIND-255`, `FIND-270`, más la cita de `pkg/` de `FIND-243`).

## El contrato que evita la recaída

> **Una fila sólo puede llevar `re-verificado` / `verificado <fecha>` si la MISMA celda cita el comando o el commit con el que se comprobó.** Igual que el campo `Medición:` que el pipeline ya exige para todo claim numérico (H-02): el número sin comando es una conjetura, la fecha sin comando es una relectura.

Forma canónica de la marca, en la celda `Descripción`:

```markdown
**VIVA HEAD 2026-11-27.** Medición: `rg -n "<símbolo>" <path>` → <salida>, y `git cat-file -e <commit>:<path>` → exit 0.
```

Reglas que el gate aplica, y por qué cada una:

1. **La evidencia vive en la celda que hace el claim.** Un commit citado en `Relaciones` no absuelve una `Descripción` que dice "re-verificado" sin nada — es el agujero que `FIND-779 (b)` ya cerró para `Medición:`.
2. **Un `archivo:línea` no es evidencia.** Es exactamente lo que envejece (4 de 10 citas de este barrido estaban corridas o muertas). Evidencia = comando re-ejecutable o SHA de commit.
3. **El SHA necesita al menos una letra** (`(?=[0-9a-f]*[a-f])`), si no un `20261127` de 8 dígitos pasaría por commit.
4. **Se juzgan las filas AÑADIDAS al diff**, igual que H-02: la marca se valida al escribirla, no en una auditoría posterior.

Implementado en `dev-tools/check-harness-contract.mjs`: `REVERIFY_CLAIM` + `hasReverifyEvidence` + `reverifyViolations` + check `reverify (FIND-811)`, también cableado en el modo `--rows` (que corre al crear la fila, no sólo en `/harness`).

### Fixtures (89/89, eran 81/81)

| Fixture | Qué prueba |
|---------|-----------|
| `a bare "re-verificado <fecha>" with no command fails` | la marca desnuda falla |
| `the row retired by FIND-811 (verbatim HEAD) fails` | **falla contra HEAD real**: el texto literal de `FIND-271` tal como estaba en `HEAD` |
| `the same row with the command that caught it passes` | el comando la absuelve |
| `a commit sha passes` | el SHA la absuelve |
| `a commit in ANOTHER column does not absolve the claim cell` | la evidencia se lee en la celda del claim, no en la fila (lección de `FIND-779 b`) |
| `undated 'verificado' is not a dated claim` | no se marca `runner verificado contra zep-cloud==3.30.0` (`FIND-205`) |
| `a bare 8-digit date is not a commit` / `a sha with a letter is evidence` | el lookahead `(?=[0-9a-f]*[a-f])` |

El segundo fixture es el que importa: no es sintético, es la fila que este task retiró.

## Review (P2-01, una ronda)

- **Tier:** **fast** (globs del diff: `docs/dev/**` + `dev-tools/**`; ningún path de la tabla adversarial).
- **Veredicto:** **approve**, con modo **degraded** declarado: el `dev-tools/verify.ps1` completo no se corrió porque la mitad Rust (`cargo fmt`/`clippy`/`nextest`) está **prohibida en este carril** (medición pura; disco ~14 GB). Ningún `.rs` está en el diff y el hook pre-commit saltó `cargo-fmt`/`cargo-clippy` por la misma razón, sin `SKIP` manual.
- **Gates que sí corrieron:** `node dev-tools/check-harness-contract.mjs` (9 checks, verde) · `--self-test` (89/89) · `scripts/docs/check-docs.mjs` (`GATING: all clear`) · hook `pre-commit` (prettier Passed, markdownlint Passed) · `pwsh dev-tools/ocr-review.ps1` (rule group JS/TS sobre el check nuevo: sin typos, sin `var`, sin `==`, sin dead code).
- **Round 1 encontró y cerró un fallo real:** la primera implementación juzgaba sólo las columnas `Hallazgo`/`Descripción`, y el fixture con la fila verbatim de `FIND-271` **pasó** en verde porque su marca vivía en `Relaciones`. Se cambió a *todas las celdas, evidencia en la misma celda* y se añadió el fixture de la columna ajena.
- **Riesgo residual:** `no verificado 2026-11-22` (una negación con fecha) se marcaría como claim. No hay ninguna en el catálogo; si aparece, se resuelve igual que `MEASURE_NEGATED`.


## Verificación

- `node dev-tools/check-harness-contract.mjs` → **harness contract holds (9 checks)** (eran 8).
- `node dev-tools/check-harness-contract.mjs --self-test` → **89/89** (eran 81/81).
- `git diff --numstat -- docs/dev/Backlog.md` → 8 inserciones / 11 borrados sobre 960 líneas (962 → 960 por las 2 retiradas y 1 retirada ajena preservada).
- Sin compilar Rust (medición pura): los hallazgos se midieron con `rg`/`git cat-file`/`git log`.
- Rama `develop` sin cambios de rama. Commit local por `dev-tools/git-commit-isolated.ps1` con `-ExpectAbsent` por fila retirada.

## Deuda

- **29 marcas con fecha quedan sin barrer**, en 5 lotes: `2026-10-05` (1), `2026-10-18` (10), `2026-10-20` (5), `2026-11-22` (12), `2026-11-27` (1). Un barrido de 200 filas en un carril es una promesa que no se cumple: **la fila nueva debe ser `FIND-811-bis` con un lote por wave**, no "el resto del backlog".
- `EGO-15` y `FIND-290` (las dos `re-verificado` literales de 2026-11-22) siguen sin re-medir: `EGO-15` ya se sabe que citaba una línea muerta (`seed.rs:25` → `:31` desde `8f3b7039d`).
