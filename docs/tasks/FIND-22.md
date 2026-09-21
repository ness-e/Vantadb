# FIND-22 — Formalizar exclusiones del fast gate en CI_POLICY.md

**Plan:** docs/plans/2026-08-25-batch-core-fixes-research.md (Task 6)
**Contrato:** CI_POLICY.md documenta las exclusiones; docs coverage 0 gaps
**Tipo:** docs (vanta-docs, leaf) — NO código
**Appetite:** max 1h · 🟢 · Cynefin 🟦 obvio

## Steps

- [x] Step 1 — DISCOVERY: leer verify.ps1 completo + nextest.toml + CI_POLICY.md; localizar tests excluidos en fuente
- [x] Step 2 — ACT: añadir sección "Fast Gate Test Exclusions" a docs/operations/CI_POLICY.md (inglés)
- [x] Step 3 — VERIFY: grep cross-check exclusiones ↔ doc + pwsh scripts/validate-docs-coverage.ps1 (0 gaps)

## Discovery

### Exclusiones reales encontradas

**A. dev-tools/verify.ps1** (`-E` filter, líneas 73/79 nextest+coverage; fallback `--skip` líneas 75/81):

| Test | Ubicación fuente | Motivo (comentario inline) | Categoría |
|------|------------------|---------------------------|-----------|
| `deserialize_absurd_node_count` | `src/index/core.rs:414` | input de tamaño absurdo diseñado para OOM el runner; no es flaky, es bomba de memoria | RESOURCE-GUARD |
| `test_search_with_bizarre_text_query` | `tests/security.rs:639` | inputs malformados gigantes (100KB queries, NUL bytes); cubiertos por fuzzing dedicado | RESOURCE-GUARD |
| `test_malformed_payload_extremely_large` | `tests/security.rs:324` | payload de 1MB+10KB metadata; cubiertos por fuzzing dedicado | RESOURCE-GUARD |

Nota: los 3 tests SÍ corren fuera del fast gate (suite completa local, heavy certification) y están cubiertos por fuzz-40.yml para el caso de inputs malformados.

**B. .config/nextest.toml `default-filter`:** 55 exclusiones binarias estructurales (`not (package(vantadb) and binary(X))`) — ya documentadas implícitamente como split two-tier (Heavy Certification corre esos bins). Se referencian, no se duplican.

### Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `dev-tools/verify.ps1` (95L), `.config/nextest.toml` (107L), `docs/operations/CI_POLICY.md` (268L), snippets de `src/index/core.rs`, `tests/security.rs`
- **Referencias hacia dentro (lo que CI_POLICY referencia):** ci-rust-10.yml, scripts/validate-docs-coverage.ps1, ADR-015/ADR-018, dev-tools/verify*.ps1
- **Referencias entrantes (quién referencia CI_POLICY.md):** Cargo.toml:647, CONTRIBUTING.md:136, dev-tools/verify.ps1:48,66,72, .github/workflows/desktop.yml:3, .opencode/AGENTS.md:356,444, .opencode/rules/release-ci.md:38, README.md:428, README_ES.md:394, docs/book/src/operations/CI_POLICY.md (`{{#include}}` — hereda cambios automáticamente), master-index ×2
- **Veredicto:** cambio additivo de solo-documentación (una sección nueva en CI_POLICY.md). El include de mdBook propaga el contenido sin acción extra. Ningún archivo de código se toca. Sin riesgo.

## Context Save Point

- ✅ COMPLETADO. CI_POLICY.md § "Fast Gate Test Exclusions" (líneas ~76-100): tabla de 3 exclusiones RESOURCE-GUARD con fuente/por qué/dónde vive/quién revierte + nota sobre las 55 exclusiones estructurales de nextest.toml + reglas para exclusiones nuevas. `last_reviewed` → 2026-08-25.
- Verify: rg cross-check 3/3 exclusiones doc ↔ verify.ps1 ✅ · pwsh scripts/validate-docs-coverage.ps1 → **0 gaps** ✅
- Pendiente lead: commit (`docs:`), skill progreso.
