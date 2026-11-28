---
title: "FIND-825 — el conteo de continue-on-error no se cita, se mide"
kind: task
status: completed
description: "AGENTS.md Regla 2 afirmaba «existen 8 instancias» de continue-on-error; las claves YAML son 39. La cifra se reemplaza por el comando (y por qué `rg -c` da 52: cuenta menciones en comentarios), y el invariante — toda exención lleva `# CATEGORY:` — queda dicho como Must, con las 2 que hoy no lo llevan declaradas como deuda."
id: FIND-825
tags: [vantadb, harness, ci, docs]
---

# FIND-825 — «8 instancias» de `continue-on-error` cuando hay 39

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-825`)
- **Origen:** lote evals/benches 2026-11-27 (`FIND-664` §Deuda) · **Ver:** `FIND-664`, `FIND-819`, `FIND-781`
- **Esfuerzo:** 🟢 15 min
- **Estado:** ✅ COMPLETED (local, sin push — Regla 7)

## Premisa (re-medida en HEAD)

La fila decía «8 en la regla, 40 reales». Medido hoy, con los dos comandos, porque **no miden lo
mismo**:

```
rg -c 'continue-on-error' .github/workflows/                        → 52 líneas
  demo-acceptance.yml:1  ci-rust.yml:6  certification.yml:8  docs-gates.yml:5  nightly.yml:28
  release-adapters.yml:1  release-npm-node.yml:1  release-providers.yml:1  release-wheels.yml:1

rg -n '^\s*continue-on-error:' .github/workflows/                   → 39 claves
rg -n '^\s*continue-on-error:' .github/workflows/ | rg -v CATEGORY  → 2 (certification.yml:1006, :1013)
```

- El **«8» es falso**: son **39** claves, no 8. Es la premisa de la fila y se confirma.
- El **«40»** estaba descontando ya el job-level de `FIND-664`; hoy la lectura es **39**. No hay
  cambio de premisa, hay un conteo distinto según el ancla.
- **Por qué la diferencia 52 vs 39:** 13 líneas solo **nombran** el string en un comentario
  (`# No continue-on-error, deliberately (Regla 2)`, `# sin continue-on-error (Regla 2)`). Un
  `rg -c` del string cuenta menciones, no exenciones — y esa es exactamente la clase de número que
  se desactualiza sin que nadie lo note.

## Contrato entregado

`.opencode/AGENTS.md` → **Regla 2 §Tolerancia Cero a Flaky Tests**, dos párrafos:

1. **El invariante como Must:** toda exención —nueva o vieja— lleva su `# CATEGORY:` (taxonomía en
   `docs/dev/operations/CI_POLICY.md`); una nueva requiere además justificación en el commit. Ya no
   dice «todas con CATEGORY» (era falso: 2 no la llevan).
2. **El número se mide, no se cita:** los dos comandos con su lectura fechada (39 / 2), y la
   explicación de por qué `rg -c` da 52 — para que el próximo lector no repita el error.

## Steps

1. ✅ Medir las dos cifras (`rg -c` y `^\s*continue-on-error:`) y las instancias sin tag.
2. ✅ Reemplazar la cifra por el comando + fecha; explicar 52 vs 39.
3. ✅ Convertir «todas con CATEGORY» en el Must, sin afirmar que se cumple.
4. ✅ Gate del harness: 9 checks + `--self-test` 89/89.

## Contrato de verificación

```
node dev-tools/check-harness-contract.mjs --root .    → harness contract holds (9 checks)
node dev-tools/check-harness-contract.mjs --self-test → 89/89 self-test fixtures passed
rg -n '^\s*continue-on-error:' .github/workflows/ | rg -v CATEGORY → 2 (sin cambio: no se tocó YAML)
```

**No se quitó ninguna exención:** el alcance era medir y corregir el documento. Las 2 sin
`# CATEGORY:` **no tapan un gate** — son `actions/download-artifact` de `analyze`
(`certification.yml:1006`, `:1013`), tolerancia a artefacto ausente, y el gate se re-establece
después en «Merge criterion results» (`if [ "$got" -eq 0 ]; then echo "::error::…"; exit 1`). Lo
único que les falta es la etiqueta que la regla exige: **fila propuesta en el `RESULTADO`**, no
creada acá porque `/.github/workflows/**` estaba en zona solo para medir.

## Review

- **Ronda P2-01:** tier **Fast** (`.opencode/AGENTS.md`, docs) — verify mecánico + spot-check.
- **Scope:** solo `AGENTS.md` Regla 2; cero ediciones en `.github/workflows/**`.
