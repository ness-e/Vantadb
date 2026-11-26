---
title: "FIND-763 - cablear el gate de rustdoc: step nuevo + context en REQUIRED"
kind: task
status: completed
description: "Cablear en CI el gate de rustdoc que está medido en verde y nadie ejecuta: step nuevo `Deny rustdoc warnings (doc gate)` con `RUSTDOCFLAGS=-D warnings` antes del artefacto, más el context `Generate API reference (rustdoc)` en `REQUIRED`"
tags: [vantadb, docs, rustdoc, gate]
---

# FIND-763 - cablear el gate de rustdoc

## Contexto

`FIND-755` dejó la superficie **pública** de rustdoc en 0 diagnósticos y
`FIND-761` la del **artefacto** en 0. Dos horas después de la primera,
`FIND-755` encontró que `vanta-memory` había vuelto a romperse sola
(`note_backfill` → `derived_health`, código nuevo de `FIND-745`). **Verde no es
durable sin el gate cableado**: la medición existía, el gate no.

Este ticket es la mitad que faltaba — las dos piezas de CI, no los lints.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `.github/workflows/docs-gates.yml` (672 líneas en
  `HEAD` — todas las referencias `:NNN` de este task file son de `HEAD` — con
  los **16** jobs, sus `name:`, sus `if:`, y las dos piezas de rustdoc: el
  comment `:553-556`, el step `:557-558`, el tarball `:560-571`, el summary
  `:573-582`)
  · `.github/workflows/ci-gate.yml` (100 líneas: el header `:7-8`, el array
  `REQUIRED` `:36-55`, el consumidor `:66-99`) · `docs/dev/tasks/FIND-761.md`
  (§Incógnitas 1-5, el YAML exacto y las tres razones) · `docs/dev/tasks/FIND-755.md:419`
  · `docs/dev/avance/activo/docs.md:45-68` (§`FIND-755`, §`FIND-761`) ·
  `docs/dev/avance/activo/ci-cd.md:958-966` (§`FIND-715`+`FIND-719`) ·
  `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`.
- **Referencias salientes:** ninguna ejecutable. `docs-gates.yml` no se importa
  desde ningún sitio; `ci-gate.yml` es un `workflow_call` consumido por los
  workflows heavy (`needs: ci-gate`).
- **Referencias entrantes:** (a) el `name:` del job `rustdoc` es lo que GitHub
  reporta como check-run y lo que `ci-gate.yml` compara por string; (b) los
  rulesets `develop` (`23692587`) y `main` (`16844590`) exigen una lista de
  contexts que **no** se edita desde estos ficheros.
- **Veredicto:** un step nuevo dentro del job `rustdoc` y una entrada en un
  array bash. Cero símbolos, cero firmas, cero cambios de comportamiento del
  artefacto. El único riesgo real es de **nombre**: un `context` que no coincide
  con el `name:` del job es un check que nunca se satisface (la clase de
  `FIND-714`/`FIND-715`), por eso se verifica carácter a carácter abajo.

## Hallazgo que corrige la premisa heredada (`FIND-719`)

Las filas de `FIND-755`, `FIND-761` y `FIND-763` dicen *"ningún job de
`docs-gates` es required en `develop`"*. **Es falso desde el 2026-11-24**: el
ruleset `23692587` (`develop`) tiene **13** contexts y **tres** son jobs de
`docs-gates` (`Internal links resolve`, `Frontmatter schema, kind agreement,
orphans`, `Generated indexes are current`). Lo cerró el orquestador vía API el
mismo día de `FIND-719` (`ci-cd.md:958-966`), y las filas posteriores quedaron
con la premisa stale.

Verificado hoy por API (solo lectura):

```
gh api repos/ness-e/Vantadb/rulesets/23692587  → 13 contexts (3 de docs-gates)
gh api repos/ness-e/Vantadb/rulesets/16844590  → 18 contexts (= REQUIRED hoy)
```

Consecuencia para esta tarea: **el gate corre en `develop`, pero no es
`required` ahí** — `Generate API reference (rustdoc)` no está en ninguno de los
dos rulesets, y este ticket **no lo añade** (los rulesets son configuración del
repo vía API, no ficheros: `FIND-763` no puede tocarlos). `FIND-719` **no se
cierra** con este cambio; lo que queda de ella es exactamente esta fila.

## Medición heredada (no se re-mide: carril CI sin `cargo`)

Precondición: `FIND-755` ✅ (`eefc5e328`) y `FIND-761` ✅ (`d9bb8f5f9`).

| Comando | Resultado | Fuente |
|---|---|---|
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --keep-going` | `exit 0` / 0 diagnósticos | `FIND-755` §Verificación (re-corrido por el reviewer) |
| el mismo, tras `FIND-761` | `exit 0` / 0 (sin regresión) | `FIND-761` §Verificación (2) |
| `cargo doc --no-deps --workspace --all-features --document-private-items` **sin** `-D warnings` | `exit 0` con **36 warnings** → el step del artefacto no juzga | `FIND-761` §Medición |
| `RUSTDOCFLAGS="-D warnings"` + `--all-features --document-private-items` | `exit 0` / 0 **después** de `d9bb8f5f9` | `FIND-761` §Verificación (1) |

`FIND-761` sólo cambió líneas de comentario (`git diff -U0` filtrado a no-doc
= vacío), así que la superficie pública no cambió entre `eefc5e328` y
`d9bb8f5f9`: los dos verdes aplican al árbol actual.

## Las tres razones ya medidas

1. **El step del artefacto no se toca.** Corre `--all-features
   --document-private-items`, que es **otra superficie** (los 36 lints que
   `FIND-761` cerró, entre ellos `private_intra_doc_links` y los docstrings de
   Python que rustdoc lee como links). Su semántica es "construye y publica el
   tarball"; meterle `-D warnings` lo convierte en dos cosas a la vez y lo
   acopla a la vista privada. **Dos steps, uno de cada cosa.**
2. **`env:` va en el step.** En el job, `RUSTDOCFLAGS` se heredaría también en
   el build del artefacto: cambiaría la semántica del tarball y un lint nuevo de
   la vista privada rompería la publicación de docs.
3. **Va ANTES del build `--all-features`** porque no comparte fingerprint
   (`default` vs `--all-features`): no se ahorra nada reordenando, y así el
   árbol caliente que encuentra primero es el del workspace default.

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | `docs-gates.yml`: step nuevo `Deny rustdoc warnings (doc gate)` en el job `rustdoc`, entre `rust-setup` y `Generate rustdoc` | ✅ |
| 2 | `ci-gate.yml`: `"Generate API reference (rustdoc)"` en `REQUIRED` + header honesto (19 = 18 del ruleset + este) | ✅ |
| 3 | `actionlint` sobre los dos ficheros (baseline previo: 0 en ambos) + coincidencia literal del `context` | ✅ |
| 4 | Responder con evidencia el alcance: ¿corre en `develop`? | ✅ (corre; no es required) |
| 5 | Cierre: task file, fila del Backlog, entrada de avance, commit local | ✅ |

## YAML final

```yaml
# docs-gates.yml, job `rustdoc`, step NUEVO (indice 2 de 7, antes de
# "Generate rustdoc" y de "Compress API reference")
- name: Deny rustdoc warnings (doc gate)
  env:
    RUSTDOCFLAGS: "-D warnings"
  run: cargo doc --no-deps --workspace --keep-going
```

```yaml
# ci-gate.yml, REQUIRED (entrada 18 de 19, entre actionlint y Cross-language)
"Generate API reference (rustdoc)"
```

## Verificación

| Comprobación | Comando | Resultado |
|---|---|---|
| lint de workflows | `actionlint .github/workflows/docs-gates.yml` | **exit 0** (baseline previo al cambio: 0) |
| lint de workflows | `actionlint .github/workflows/ci-gate.yml` | **exit 0** (baseline previo al cambio: 0) |
| step en el job correcto y en la posición correcta | parseo YAML (`js-yaml`): `jobs.rustdoc.steps` | `[0]` checkout · `[1]` rust-setup · **`[2]` nuevo** · `[3]` Generate rustdoc · `[4]` Compress · `[5]` Upload · `[6]` Summary |
| `RUSTDOCFLAGS` sólo en el step | parseo YAML | `steps[2].env = {"RUSTDOCFLAGS":"-D warnings"}` · `jobs.rustdoc.env = {CARGO_TERM_COLOR, CARGO_INCREMENTAL}` (sin `RUSTDOCFLAGS`) |
| sin `continue-on-error` | parseo YAML | ausente en el step y en el job |
| comando exacto | parseo YAML | `"cargo doc --no-deps --workspace --keep-going"` (sin `--all-features`, sin `--document-private-items`) |
| coincidencia literal del `context` | parseo del array `REQUIRED` + comparación por code points | `name:` del job y entrada de `REQUIRED` = **32 chars, code points idénticos** |
| ningún otro job roto | parseo YAML | **16 jobs** (antes y después), mismo conjunto |
| enlaces / schema de docs | `node scripts/docs/check-links.mjs` · `check-docs.mjs` | 0 broken · **GATING all clear** |

No se corrió `cargo`: carril CI (otro carril compila) y la medición del comando
ya estaba hecha por `FIND-755`/`FIND-761` sobre el mismo árbol.

Notas de fidelidad: `actionlint` local es **v1.7.12** y CI pinea **1.7.7**; la
sintaxis usada (`env:` por step) existe en ambas. El orfandazgo informativo de
este task file (`check-docs` ORPHANS) es el esperado: `gen-index.mjs` no se
corre en esta wave, igual que en `FIND-755`/`FIND-761`.

## Review (P2-01)

Tier **fast** por glob (`.github/workflows/**` → "cualquier otro path"), pero el
riesgo real del `context` merecía contexto fresco: **`vanta-review`,
report-only, no implementó nada**.

- **Enfoque:** re-corrió `actionlint`, parseó los dos YAML por su cuenta
  (posición del step, `env` por nivel, `continue-on-error`, array `REQUIRED` con
  conteo y duplicados), contrastó `REQUIRED − {rustdoc}` contra el ruleset real
  de `main` (`18/18`, `missing: []`) y el alcance en `develop` contra los dos
  rulesets por API.
- **Veredicto:** **`approve`**, 0 Blocking. 3 Medium + 3 Low. Cerrados acá los
  que caen en la superficie escribible de esta fila:
  - **L-1** — "15 jobs / 672 líneas" cuando son **16**; las referencias `:NNN`
    eran de `HEAD` y no lo decían. Corregido.
  - **L-3** — el gate mide sólo la superficie **default**; los `pub` que existen
    bajo features no-default no tienen gate propio. Declarado como techo
    (arriba), no como olvido: la superficie estricta está medida verde en
    `FIND-761` y meterla en cada PR cuesta rocksdb + bindgen.
- **Reportados al orquestador (fuera de la superficie de esta fila):**
  - **M-1** — `on.push.paths` de `docs-gates.yml` no incluye `web/**`, y con 3
    jobs de `docs-gates` required un PR que sólo toque `web/` quedaría en
    *Expected* para siempre (clase `FIND-714`). Preexistente.
  - **M-3** / **L-2** — `docs/dev/tasks/FIND-761.md:401-403` sigue afirmando
    que "ningún job de `docs-gates` es required en `develop`" (falso desde
    2026-11-24) y `:381-382` da por hecho que la fila `FIND-763` está en el
    Backlog (estaba, y esta tarea la retira). Ese fichero no es escribible acá.

## Deuda / pendiente

- `FIND-719`: **no se cierra**. Lo que queda es que el job `rustdoc` no es
  required en ninguno de los dos rulesets; añadirlo es configuración del repo
  vía API (orquestador/owner), no un fichero.
- `web/**` en las `paths` de `docs-gates.yml` (M-1) — fila aparte.
- `FIND-761.md` con la premisa stale (M-3/L-2) — fila aparte.

## Alcance (escritura)

`.github/workflows/docs-gates.yml` · `.github/workflows/ci-gate.yml` ·
`docs/dev/tasks/FIND-763.md` · `docs/dev/Backlog.md` (sólo la fila `FIND-763`) ·
`docs/dev/avance/activo/ci-cd.md` (dominio elegido: **ci-cd**, porque el
hallazgo `FIND-719` que esta tarea corrige vive en su §Rulesets).

**Prohibido y respetado:** `docs/dev/operations/CI_POLICY.md` (hay `FIND-752`
en vuelo), `src/**`, `vanta-memory/**`, `vantadb-mcp/**`, `.githooks/**`,
`dev-tools/**`, `docs/dev/workflow/**`, `docs/dev/plans/**`, `.opencode/**`.

## Verificación (contrato mecánico)

No se puede disparar CI desde acá. Lo que sí se verifica:

```powershell
actionlint .github/workflows/docs-gates.yml
actionlint .github/workflows/ci-gate.yml
```

más, por inspección mecánica: el step cae en el job `rustdoc` (indentación,
posición antes de `- name: Generate rustdoc`), el comando es **exactamente**
`cargo doc --no-deps --workspace --keep-going` con `RUSTDOCFLAGS: "-D warnings"`
en el `env:` **del step**, sin `continue-on-error`, y el string de `REQUIRED`
coincide carácter a carácter con el `name:` del job.
