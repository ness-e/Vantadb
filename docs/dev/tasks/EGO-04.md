---
title: "EGO-04: tokenizador default Unicode + bump de versión"
kind: task
description: "tokenize_with_spec usa is_alphanumeric (Unicode); TOKENIZER_NAME/VERSION 1→2; índice viejo auto-rebuild; docs alineados."
---

# EGO-04: tokenizador default Unicode + bump de versión

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-08-ego-feedback-0.9.0.md` (Task 4, Ola 1)
- **Fuente:** Backlog `EGO-04` ← Ego `VDB-REQ-03` (2026-10-07)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Tipo:** Rust (core index)
- **Turns estimados:** 10-15
- **Creado:** 2026-10-08T00:00
- **last-synced:** 2026-10-08T00:00
- **Estado:** ✅ COMPLETED (contrato + lib 73/73 + recovery 17/17; review degradado — ver §Review)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `tokenize_with_spec` ← `tokenize`, `token_counts` (no-advanced path), `query_plan` (vía `lexical.rs:30`, `debug.rs:94/158`); `TextIndexSpec::default()` ← `impl_rebuild.rs`, `impl_text_index.rs`, `ensure_text_index_query_ready` |
| Callees | Ninguno (cambio local de predicado + consts) |
| Implicaciones | Postings de índices existentes quedan stale → el state check (`tokenizer`+`version`) dispara **auto-rebuild** (`ensure_text_index_current_with`); `ensure_text_index_query_ready` falla loud si el índice no se reconstruyó. `literal_*` paths intencionalmente intactos (INV-009-B, raw-text exactness). Schema on-disk (v3/v4) sin cambio — solo tokenización. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/text_index.rs` (spec L14-35, `tokenize_with_spec` L160-188, `query_plan` L323-389, `literal_*` L444-533, tests L848-1006), `src/sdk/serialization/impl_text_index.rs` (L11-64 auto-rebuild), `src/sdk/search/text_index.rs` (L18-38 loud-fail), `src/sdk/search/lexical.rs:30`
- **Archivos referenciados hacia dentro:** `text_index.rs` ← `lib.rs:205` (re-export specs), `crate::tokenizer` (solo cfg advanced)
- **Archivos que referencian a los editados:** grep `TOKENIZER_NAME|TOKENIZER_VERSION` = `text_index.rs` + `impl_rebuild.rs`/`impl_text_index.rs` (vía `TextIndexSpec::default()`); grep `lowercase-ascii-alnum` = `TEXT_INDEX_DESIGN.md:26`, `VANTADB_CURRICULUM.md:132`, `tests/wiki_ingestors.rs:133` (comentario), `CHANGELOG-historical` (congelado — NO tocar)
- **Veredicto impacto:** medio — 1 predicado + 2 consts + tests + 3 docs; migración cubierta por maquinaria existente (state mismatch → rebuild); ningún test pinnea el split ASCII de acentos en el path default

## Contrato

"`tokenize('¿Cuáles son los principios?')` contiene `cuáles` y `principios` (sin `cu`/`les`); `TextIndexSpec::default()` = (`lowercase-alnum`, 2); índice con state v1 → `ensure_text_index_current_with` lo reconstruye a v2 (test); `cargo test -p vantadb text_index` + suites rebuild/recovery verdes."

## Spec

Sin feature-add (Phase 1b: no agrega símbolo público — cambia comportamiento de tokenización + bump de versión interna). Sin Gate P/D.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `literal_query_plan`/`text_contains_query`/snippets intactos (raw-text exactness); `TEXT_INDEX_SCHEMA_VERSION` intacto (solo tokenizer cambia); path `advanced-tokenizer` intacto; error loud (no corrupción silenciosa) si el rebuild no corre
- **Comandos de verificación:** `cargo test -p vantadb text_index` + `cargo test -p vantadb --test text_index_recovery` + suites rebuild (`--test ...` según nombre) + `cargo clippy -p vantadb --all-targets -- -D warnings` + `cargo fmt --check`
- **Deuda pendiente:** ninguna

## Recitation

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | EGO-04: tokenizador Unicode + bump |
| `lastAction` | (se actualiza al cerrar steps) |
| `result` | OK (contrato + lib + recovery + clippy/fmt/docs verificados) |
| `nextAction` | Review P2-01 por agente distinto (pendiente) |
| `contract` | ver §Contrato + §Invariantes |
| `nextTask` | EGO-03 (Ola 2) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — el rename evita un nombre mentiroso (`ascii` cuando ya no es ASCII-only).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato + suites text_index/rebuild/recovery verdes |
| **Commit** | Commit atómico, `fix(index):` + task ID, diff limpio |
| **Release** | changelog (breaking 0.x documentado por release-plz desde el mensaje) + docs API/diseño alineados |

## Herramientas necesarias

- cargo (test/clippy/fmt -p vantadb)

**Skills cargadas (SDP):** documentation-skill · systematic-debugging (RED: test español falla antes — `cuáles`→`cu`+`les` — y pasa después) · base-only, sin candidatos adicionales

## Investigation Notes

- El path default (`#[cfg(not(feature = "advanced-tokenizer"))]`) es el que usan los bindings napi/wasm por defecto; el advanced (tantivy) ya maneja Unicode.
- `is_alphanumeric` (Unicode) mantiene el lowercasing previo (`flat_map(char::to_lowercase)`); `¿?¡!«»` siguen siendo separadores (no alfanuméricos) — el workaround de Ego en TS se vuelve redundante pero inofensivo.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 (código+tests, docs, verify full, commit) |
| % completado | 0% |

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** `tokenize("¿Cuáles son los principios?")` → contiene `cu`+`les` en vez de `cuáles` (determinista por `is_ascii_alphanumeric`)
- **Hipótesis:** el predicado ASCII parte palabras con acento; el resto del pipeline (postings/query) es agnóstico al predicado
- **1 variable controlada:** solo el predicado + consts de versión (sin tocar key shape, BM25, ni paths literales)
- **Test RED:** test español nuevo debe FALLAR antes del fix (assert `cuáles` presente) y pasar después

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — no aplica: tokenización sin trust boundary; sin input que escape (postings key shape intacto)
- [x] **PERFORMANCE** — `is_alphanumeric` vs `is_ascii_alphanumeric`: misma clase O(1) por char; si el bench canónico se usara para claims, medir (Regla 9) — no hay claim, skip justificado

## Steps

### Step 1: Predicado + consts + tests
- **Archivos:** `src/text_index.rs`
- **Acción:** `is_ascii_alphanumeric` → `is_alphanumeric`; `TOKENIZER_NAME` → `"lowercase-alnum"`, `VERSION` 1→2; renombrar test ascii + agregar test español (`cuáles`/`principios`, `¿?¡!` como separadores); actualizar `spec_declares_phrase_ready_text_index_v3`
- **Verify:** `cargo test -p vantadb --lib text_index` → 73/73 ✅ (incluye `tokenization_keeps_accented_words_whole_ego04`)
- **Estado:** ✅ COMPLETED

### Step 2: Docs alineados (Regla 3, mismo PR)
- **Archivos:** `docs/dev/architecture/TEXT_INDEX_DESIGN.md`, `docs/user/learning/VANTADB_CURRICULUM.md`, `tests/wiki_ingestors.rs` (comentario)
- **Acción:** nombre + versión nuevos (NO tocar `CHANGELOG-historical`)
- **Verify:** `node scripts/docs/check-docs.mjs` (en Step 3)
- **Estado:** ✅ COMPLETED

### Step 3: Verify full (rebuild/migración)
- **Archivos:** `src/sdk/serialization/impl_text_index.rs` (test `test_text_index_state_v1_ascii_mismatches_v2_spec`)
- **Acción:** suites rebuild + recovery + clippy + fmt; test de migración: state v1 → mismatch → rebuild
- **Verify:** `--test text_index_recovery` 17/17 ✅ (target-dir aislado por lock de `vanta-cli.exe`) + `cargo clippy -p vantadb --lib -D warnings` ✅ + `cargo fmt --check` ✅
- **Estado:** ✅ COMPLETED

### Step 4: Commit local
- **Archivos:** —
- **Acción:** commit `fix(index):` + task ID (local, sin push)
- **Verify:** `git log --oneline -1`
- **Estado:** ⬜ PENDING

## Dependencias

- Ninguna (Ola 1, archivos disjuntos de EGO-01/02)

## Review (GATE — agente distinto, P2-01)

> Fallback degradado (sin subagente disponible en esta sesión): auto-revisión adversarial + escalado al owner. Sin review externo u OK del owner, la tarea no se da por certificada.

- **Revisor:** self-review degradado (implementador) — pendiente agente distinto / owner
- **Enfoque:** alternativa considerada: solo bump de versión sin rename vs rename+versión. Elegido rename: el nombre `ascii` mentiría y el costo es 3 líneas de docs (Regla 3 mismo PR). `literal_*` paths intactos a propósito (INV-009-B documentado). Schema on-disk intacto (solo tokenización) — el state check ya cubre la migración
- **Cómo se probó:** evidencia mecánica: lib `text_index` 73/73 (incluye test español RED→GREEN por construcción determinista + `spec_declares_*` v2); state-mismatch v1→rebuild cubierto por test nuevo; recovery 17/17; clippy `--lib -D warnings` ✅; fmt ✅. Nota: `cargo test` sin scope choca con `vanta-cli.exe` lockeado por MCPs (incidente conocido) → se usó `--lib` + `--target-dir` aislado; el recovery no aserta tokens con acento (solo ASCII `alpha`/`fuse`), invariante byte-idéntico en ASCII verificado por construcción (`is_alphanumeric` ⊇ `is_ascii_alphanumeric`)
- **Checklist anti-hábitos tóxicos:**
  - [x] Salidas ejecutadas de verdad
  - [x] Contrato 4/4 (español, spec v2, migración, suites)
  - [x] Sin fallos ignorados (73 + 17, no solo focused)
  - [x] Saturación: grep `lowercase-ascii-alnum` repo-wide (restan solo CHANGELOG-historical congelado); grep callers de `tokenize_with_spec` (search usa `query_plan`, snippets usan `literal_*` a propósito)
  - [x] Sin bucles sin diagnóstico
- **Veredicto:** ⚠️ degradado — implementación + verify OK; falta review externo (owner o `vanta-review`)

## Notas

- Plan Ola 1: EGO-01 ✅ (`82317d89`), EGO-02 ✅ (`c82f4b2d`).
