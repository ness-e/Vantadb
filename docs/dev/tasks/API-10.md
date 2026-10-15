---
title: "API-10 — Schema-first slice 1 + pins DIST-08"
kind: task
status: active
description: "JSON Schema canonico de DTOs wire + paridad Rust-TS extendida + pins QUICKSTART 0.6.1-0.8.0."
tags: [vantadb, api-10, schema]
---

# API-10 — Schema-first slice 1 + pins DIST-08

> Origen: Backlog §API-10 + §DIST-08 · Wave 0 (tras SURF-01 ✅) · Next: EGO-01.
> Regla: `.opencode/rules/api-contract.md` (leída antes de tocar `src/sdk/` — solo lectura, sin cambios) ·
> Verdad de superficies: `docs/api/SURFACE_MANIFEST.md` · Skill docs: `documentation-skill` cargada antes de editar `.md`.
> SDP: grep SKILLS-MANIFEST por schema/parity/dto → sin candidatos específicos (solo `reference-design-contract`, `database-schema-designer`, quality-bar genericas);
> declara `SDP: documentation-skill + codebase-memory/codegraph (blast radius) + ponytail + base-only`.
> SKILLS_CARGADAS: documentation-skill (edición .md), codegraph/codebase-memory (DTOs + callers), ponytail full.

## Objetivo

Slice 1 SOLAMENTE: schema canónico de DTOs wire + paridad extendida Rust→TS (NO codegen — post-1.0, scope creep prohibido) + pins QUICKSTART/DIST-08.

## Contrato

- `cargo nextest run -p vantadb --test public_api --test version_coherence --test sdk_serialization --test openapi_yaml_parity` verde
- `node scripts/docs/check-npm-versions.mjs` exit 0
- `check-links`/`check-docs` sin nuevos issues + `git diff --check` + `cargo fmt --check` (se tocó `.rs`)

## Spec (decisiones)

| # | Decisión | Opción elegida | Alternativa rechazada + por qué |
|---|----------|----------------|--------------------------------|
| 1 | Formato schema | JSON Schema draft 2020-12 hand-written en `docs/api/wire-schema.json` | Derive `schemars`: nueva dependencia + build impact por un slice anti-drift |
| 2 | Alcance DTOs | 10 DTOs wire (record/search/graph + filter/window) | Todos los tipos SDK: la superficie de drift es el wire de bindings |
| 3 | Paridad Python | Excluida del test: `vantadb-python/**` es WIP ajeno (unstaged, no tocar) | Leer `.pyi` en test = frágil contra WIP ajeno |
| 4 | Pins target | `0.8.0` = `[workspace.package] version` (R-2 fuente única; avance ci-cd confirma 0.6.1/0.7.0/0.8.0 ✓ en PyPI) | Live-registry check: sin red en este runner → re-verificación diferida, anotada en deuda |
| 5 | WASM-sync (JS-02) | NO ejecutar: uniformar a Promise-based rompe consumidores WASM-sync | Decisión documentada para DIST-19 (spec decide ventana 0.x vs major) |
| 6 | Codegen real | Post-1.0 (slice 2) | Scope creep prohibido por contrato |

## Impacto mapeado (Regla 0)

- Archivos leídos completos: `src/sdk/types/record.rs`, `src/sdk/types/search.rs` (parcial 1-220), `src/sdk/serialization/graph_types.rs` (vía codegraph), `src/sdk/serialization/vector_types.rs` (grep campos), `src/sdk/types.rs` (grep `Value`), `tests/sdk_serialization.rs`, `tests/version_coherence.rs`, `tests/api/openapi_yaml_parity.rs`, `docs/user/QUICKSTART.md`, `scripts/docs/check-npm-versions.mjs`, `docs/api/SURFACE_MANIFEST.md`.
- Referencias hacia dentro: schema → structs Rust (nombres de campos serde verificados 1:1); test → `docs/api/wire-schema.json` + `vantadb-ts/src/types.ts` (solo lectura).
- Referencias entrantes: ninguna (archivo nuevo + test aditivo + docs) — cero callers afectados.
- Veredicto: impacto aditivo/doc-only; ningún símbolo `pub` nuevo; ningún hot path; Gate D no disparado.

## Blast radius (DTOs wire → consumidores)

```
src/sdk/types/{record,search,graph}.rs + serialization/{vector_types,graph_types}.rs
 ├─ vantadb-python/src/ (PyO3 mapping; WIP AJENO — no tocar)
 ├─ vantadb-ts/src/types.ts + guards.ts (mirror; solo lectura en test)
 ├─ vantadb-wasm/src/vantadb_wasm.d.ts (mirror; sin cambios slice 1)
 ├─ vantadb-server/ handlers + vantadb-mcp/ tools (consumen serde; sin cambios)
 ├─ docs/api/openapi.yaml schemas RecordInput/... (paridad existente intacta)
 └─ docs/api/wire-schema.json (NUEVO — canónico slice 1, gate: wire_schema_parity)
```

## Steps

- [x] S1 — `docs/api/wire-schema.json` canónico (10 DTOs, required ⊆ properties verificados contra fuentes Rust)
- [x] S2 — `tests/sdk_serialization.rs::wire_schema_parity` (paridad exacta key-set ×9 DTOs + required-subset request + u128-string + mirror TS; RED real: TS sin `superseded_by`/`superseded_at_ms` → GREEN con 2 campos opcionales aditivos en `vantadb-ts/src/types.ts`)
- [x] S3 — QUICKSTART pins `0.6.1`→`0.8.0` (6 puntos: descripción ×2, nota §0, install wheel, wheel filename, TestPyPI note; histórico "since 0.6.1" intacto)
- [x] S4 — VERIFY contrato: nextest 36 passed/1 skipped + check-npm-versions exit 0 + check-links exit 0 + diff --check + rustfmt ✅; check-docs: solo ORPHAN (clase pre-existente, comparten C2T1/SRV-10; se enlaza al cierre)

## Decisiones registradas

- DIST-19: cambio WASM-sync (JS-02 Promise uniforme) NO ejecutado — rompería consumidores; lo decide el spec de DIST-19.
- NOTICED BUT NOT TOUCHING: `docs/user/FAQ.md:23,32` (`0.6.1`), `docs/user/tutorials/04-hybrid-search-basics.md:185` (texto de ejemplo, no pin), `docs/user/index.md` (índice generado — lo regenera el orquestador), `docs/CHANGELOG.md`/`docs/index.md`/`llms.txt` (prohibidos), `examples/` demo pins (candidato slice 2 tras verificar registries con red).
- Deuda: re-verificar `pip index vantadb-py==0.8.0` + `npm view vantadb` con red (runner offline); FAQ pins pendientes de decisión owner.

## Context Save Point

Trabajo en worktree (sin commit, sin stage): `docs/api/wire-schema.json` (nuevo), `tests/sdk_serialization.rs` (mod `wire_schema_parity` aditivo), `docs/user/QUICKSTART.md` (6 edits pins), este task file (nuevo).
Verificación pendiente (S4): `$env:USERPROFILE\.cargo\bin\cargo nextest run -p vantadb --test public_api --test version_coherence --test sdk_serialization --test openapi_yaml_parity` + `node scripts/docs/check-npm-versions.mjs` + `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs`.
PROXIMO_STEP: review P2-01 + commit (lo organiza el orquestador).

## Review (Gate P2-01)
- **Revisor:** vanta-review, sesión distinta al implementador (veredicto 2026-10-15, contrato re-ejecutado).
- **Veredicto:** ✅ APPROVE API-10 — required muestreados vs Rust OK, TS aditivo no-breaking, QUICKSTART histórico intacto, sin codegen ni firmas. 3 opcionales follow-up (SearchHit legacy sin paridad, required laxo, fecha QUICKSTART). Orphan reporting-only (clase compartida C2T1/SRV-10).
