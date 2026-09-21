# GOV-F1 — Auditoría raíz pública (README ×2 + governance files)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** README.md, README_ES.md, CONTRIBUTING.md, SECURITY.md, SUPPORT.md, CLA_INDIVIDUAL.md, CLA_CORPORATE.md + plan file GOV-F1 entry.
- **Referencias hacia dentro:** 7 archivos raíz públicos enlazados desde docs/master-index, docs/QUICKSTART y entre sí.
- **Referencias hacia afuera:** workflows (.github/workflows/*), assets/, examples/, scripts/install.*, fuzz/, dev-tools/, docs/operations/*, registries (PyPI/npm/crates), Discord API, vantadb.dev.
- **Veredicto:** auditoría read-mostly; fixes triviales inline permitidos por contrato. Sin impacto en código. PROHIBIDO git.

## Steps

1. ✅ DISCOVERY — lectura de los 7 archivos + mapeo de claims verificables
2. ✅ VERIFY — Test-Path 48 paths; greps contra cli.rs/config.rs/cli_server.rs/Cargo.toml/pyproject.toml/nextest.toml/justfile; fetch Discord API + vantadb.dev
3. ✅ FIXES — 10 fixes triviales inline (ver reporte)
4. ✅ REPORTE — docs/reviews/auditoria-raiz-publica-2026-08-22.md (12 findings)
5. ✅ VERIFY FINAL — markdownlint-cli2 exit 0 sobre 8 archivos

## Context Save Point

Tarea completa. Deuda → 2 tickets owner (dominio vantadb.dev sin DNS; copy wheels ARM64 ↔ MKT-18h). Commit pendiente: delegar al lead (`docs: GOV-F1 audit root public files`).
