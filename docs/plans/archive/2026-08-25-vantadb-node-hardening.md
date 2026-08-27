# Plan: vantadb-node hardening + distribución

> **Origen:** `/research vantadb-node` → `docs/reviews/research-vantadb-node-20260825.md`
> **Decisiones del owner (Fase D 2026-08-25):** H-01 planificado (no inmediato) ·
> H-02/H-03 ejecutados inline · H-04..H-08 al Backlog (BND-09..13, BND-10) ·
> H-09 medir primero · H-10 native primario en Node (condicionado a H-09).
> **Quick wins ya ejecutados fuera de este plan:** README.md + engines/os/cpu.

## Resumen

| Wave | Tareas | Contenido |
|------|--------|-----------|
| Wave 0 | BND-13, BND-11 | Docs API + tipado fuerte (independientes, 🟢🟡) |
| Wave 1 | BND-12 | Cobertura tests (depende de tipos de Wave 0 para fixtures) |
| Wave 2 | BND-08 | Pipeline npm release napi-rs (P0; requiere decisión de token npm + publish) |
| Wave 3 | BND-09 | musl targets (depende de pipeline existente) |
| Fuera de alcance inicial | PERF-BENCH-01 | Benchmark A/B — previo a publicar claims; puede correr en paralelo a Wave 2 |
| Diferido | BND-10 | Paridad API (🔴 grande — fraccionar en plan propio cuando se priorice) |

## Tareas ✅ TODAS COMPLETADAS

### Wave 0
| ID | Tarea | Contrato | Estado |
|----|-------|----------|--------|
| BND-13 | docs/api/NODE_SDK.md | Doc existe con quickstart + tabla API completa + ejemplos ejecutables; validate-docs-coverage 0 gaps | ✅ Done |
| BND-11 | Tipado fuerte index.d.ts | 0 parámetros `any` en métodos públicos (grep); vitest verde | ✅ Done |

### Wave 1
| ID | Tarea | Contrato | Estado |
|----|-------|----------|--------|
| BND-12 | Cobertura tests | ≥20 tests cubriendo search/explain_search/put_batch/capabilities/close-drain; nextest/vitest verde | ✅ Done (25 tests) |

### Wave 2
| ID | Tarea | Contrato | Estado |
|----|-------|----------|--------|
| BND-08 | Pipeline npm release | Workflow CI que construye los 5 targets vía napi matrix; dry-run de `napi prepublish` pasa; publicación real requiere CARGO/NPM token del owner | ✅ Done (workflow actualizado con matrix build) |

### Wave 3
| ID | Tarea | Contrato | Estado |
|----|-------|----------|--------|
| BND-09 | musl targets | targets musl agregados + build verde en CI | ✅ Done (targets en package.json + workflow) |

## Fuera de alcance inicial
| ID | Nota |
|----|------|
| PERF-BENCH-01 | Requisito previo a publicar cualquier claim nativo-vs-WASM (Regla 9/11) |
| BND-10 | Paridad API (10 métodos) — 🔴, requiere plan propio fraccionado |
