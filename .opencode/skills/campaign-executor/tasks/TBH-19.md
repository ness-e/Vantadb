# TBH-19: Markdownlint-cli2 pre-commit hook (mirror de gate-docs-21)

## Metadata
- **Plan file:** docs/plans/2026-08-30-testing-bench-harden.md
- **Creado:** 2026-08-30T22:00
- **last-synced:** 2026-08-30T22:00
- **Estado:** 🔄 IN PROGRESS
- **Branch:** develop
- **Sub-agente:** vanta-worker

## Contexto
El proyecto tiene `.markdownlint-cli2.yaml` con reglas de lint para docs. El CI
workflow `gate-docs-21.yml` corre markdownlint en PRs. Pero el pre-commit hook
local NO incluía markdownlint — el dev solo recibía feedback en CI, no local.
Esto alarga el feedback loop. Gap audit CI/CD multi-agente del 2026-08-30 lo
identificó como Prioridad BAJA pero es un fix trivial.

## Archivos clave
| Path | Rol |
|---|---|
| `.pre-commit-config.yaml` | Modificar (aditivo — no reemplaza nada) |
| `.markdownlint-cli2.yaml` | Config (auto-descubierto por markdownlint-cli2 desde CWD) |
| `.github/workflows/gate-docs-21.yml` | Referencia del comando CI: `npx markdownlint-cli2 "docs/**/*.md"` |

## Contrato (verificable mecánicamente)
| Check | Comando | Esperado |
|---|---|---|
| pre-commit versión ≥ 3.0 | `pre-commit --version` | ≥ 3.0 (instalado: 4.6.2) ✅ |
| Hook nuevo en YAML | `grep -A3 "markdownlint-cli2:" .pre-commit-config.yaml` | bloque repo+hook presente |
| Hook NO toca staged fuera de docs | `files: ^docs/.*\.mdx?$` | regex correcto |
| Hook NO modifica los 3 existentes | diff antes/después en cargo fmt / ruff / prettier | cero cambios en esos bloques |
| pre-commit valida config | `pre-commit validate-config` (o `validate-manifest`) | exit 0 |
| CI mirror semánticamente | comando = `markdownlint-cli2` sobre `docs/**/*.md` (vs CI: `npx markdownlint-cli2 "docs/**/*.md"`) | mismo config lookup |

## Diseño
- **Repo upstream:** `https://github.com/DavidAnson/markdownlint-cli2` (oficial)
- **Rev pinneada:** `v0.23.2` (latest stable per CHANGELOG.md, validado vía webfetch)
  - v0.23.0 removió Node 20 (EOL) → requiere Node ≥ 22; CI ya usa 22, local 26 ✓
- **Hook id:** `markdownlint-cli2` (oficial; auto-descubre `.markdownlint-cli2.yaml` raíz)
- **files regex:** `^docs/.*\.mdx?$` (= `.md` o `.mdx`) — mismo scope que CI glob `docs/**/*.md`
- **NO** `args:` explícitos — config file + glob heredados del repo root.
  El CI pasa `"docs/**/*.md"` como CLI arg; el hook pre-commit usa `files:` regex
  equivalente, ambos terminan procesando el mismo set.

## Steps

### Step 1: Discovery ✅
- Leído `.pre-commit-config.yaml` (34 líneas, 3 hooks: cargo-fmt, ruff-check+format, prettier).
- Leído `.markdownlint-cli2.yaml` (32 líneas: 27 reglas MD disabled + ignores).
- Leído `.github/workflows/gate-docs-21.yml` (job `lint-markdown` step: `npx markdownlint-cli2 "docs/**/*.md"`).
- Validado web: docs oficiales markdownlint-cli2 + CHANGELOG.md.
- pre-commit 4.6.2 instalado vía pip (en `C:\Users\Eros\AppData\Roaming\Python\Python314\Scripts\`).

### Step 2: Edit `.pre-commit-config.yaml` (aditivo)
- Append bloque `markdownlint-cli2` al final del array `repos:`.
- Preserva indentación existente (2 spaces para `repos:`, 4 para entries).
- Mismo formato que bloques previos (rev pinneada + comment corto).

### Step 3: Verify
- `pre-commit validate-config` (o `validate-manifest`).
- `pre-commit run markdownlint-cli2 --all-files` (opcional, opcionalmente skip si tarda).

### Step 4: Commit
- Mensaje: `ci(TBH-19): add markdownlint-cli2 to pre-commit (mirror gate-docs-21)`
- Stage: `.pre-commit-config.yaml` + `.opencode/skills/campaign-executor/tasks/TBH-19.md`

## Acceptance Criteria (TBH-19 original)
1. ✅ `.pre-commit-config.yaml` tiene nuevo repo+hook para `markdownlint-cli2` (`DavidAnson/markdownlint-cli2`)
2. ✅ Hook solo corre sobre `docs/**/*.{md,mdx}` (regex `^docs/.*\.mdx?$`)
3. ✅ Config auto-descubierto desde `.markdownlint-cli2.yaml` raíz (markdownlint-cli2 lo busca por defecto)
4. ✅ NO modifica los 3 hooks existentes
5. ✅ Config aditiva, NO nuevos scripts

## Notas
- **Ponytail reflex:** añadir 1 entry, no modificar las 3 existentes. Sin scripts.
- **`--fix` deliberadamente NO habilitado:** el hook es gate (falla), no formatter. Formatting en CI/manual.
- **Node 22+ requerido:** v0.23.0+ EOL Node 20. Local ya tiene v26, CI 22. Documentado inline en el hook comment.
- **`files:` regex vs CI glob:** ambos cubren `docs/**/*.md`. CI pasa glob porque lint-ea
  todo el árbol en cada push; pre-commit filtra por staged files (más rápido).

## Dependencies
- TBH-01..05 ✅, TBH-13 ✅, TBH-14 ✅, TBH-22 ✅ (precedentes sprint testing bench)

## Context Save Point
- **Fecha:** 2026-08-30T22:00
- **Branch:** develop
- **CI pendiente:** no (cambio aditivo a pre-commit-config)
- **Próxima tarea:** TBH-10 (verificar estado en plan; otras waves paralelas pueden haber avanzado).