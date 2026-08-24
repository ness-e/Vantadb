# Plan de Ejecución: Batch REVIEW/MOD/FIND (pipeline paralelo)

> **Inicio:** 2026-08-24
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md (selección del lead, sesión 2026-08-24)
> **Modo:** FAIL_MODE=parallel, MAX_CONCURRENT=3

## Resumen
| DO | SKIP | BLOQUEADO |
|----|------|-----------|
| 10 | 2    | 0         |

| ID | Descripción | Archivos | Ruta | Contrato | Estado |
|----|-------------|----------|------|----------|--------|
| `REVIEW-06` | OOM rustc en `cargo test --workspace` — fix `[profile.test]` | root `Cargo.toml`, `src/lib.rs` | vanta-tuner | `cargo nextest run -p vantadb --profile audit` compila sin OOM | ⬜ PENDING |
| `REVIEW-07` | nextest default-filter stale | — | — | — | ⛔ SKIP — resuelto por BND-06/`db337b00` (filtro scope-safe verificado en `.config/nextest.toml`) |
| `REVIEW-11` | Dependabot sin pip | `.github/dependabot.yml` | vanta-lead | yaml válido con ecosistema pip | ✅ COMMITTED `ci(deps)` |
| `MOD-02` | Transacciones no crash-atómicas | `src/storage/engine/txn.rs`, `wal_sharded.rs` | vanta-worker | tests txn + chaos pasan; replay respeta Commit marker | ⬜ PENDING |
| `MOD-08`+`MOD-09` | Loop stdio serial + shutdown descarta respuesta in-flight | `vantadb-mcp/src/server.rs` | vanta-worker | mcp_tests pasan; respuesta in-flight se escribe antes de salir | ⬜ PENDING |
| `MOD-19` | ~30% API core sin exponer en Python | `vantadb-python/` | vanta-worker | pytest pasa; similar_to_key/count/delete_by_filter expuestos | ⬜ PENDING |
| `FIND-27` | Provider Ollama endpoint legacy roto | `src/llm.rs` | vanta-worker | test contra mock; POST /api/embed {model,input} | ⬜ PENDING |
| `FIND-28` | Casts u8*→f32* sin align check ×3 | `src/index/ivf.rs:69`, `src/storage/engine/mapper.rs:191`, `src/sdk/serialization/bytes.rs:136` | vanta-worker | cargo check + clippy limpios; align_to aplicado | ⬜ PENDING |
| `UX-01`+`UX-05` | LensShell compartido + token `.label-tech` | `desktop/src/components/*`, `desktop/src/index.css` | vanta-worker | `npm run build` (desktop) exit 0; 6 lenses usan LensShell | ⬜ PENDING |
| `FIND-04` | Tabla cross-SDK search() Python↔TS | READMEs SDK, `docs/api/BINDINGS_NAMESPACES.md` | vanta-docs | tabla presente en ambos READMEs, link al doc de namespaces | ⬜ PENDING |

## Waves
- **Wave 0:** REVIEW-06 · MOD-02 · FIND-27
- **Wave 1:** FIND-28 · MOD-19 · MOD-08+09
- **Wave 2:** UX-01+05 · FIND-04
- Inline (lead): REVIEW-11 ✅ · REVIEW-07 ⛔

## Notas
- Árbol tenía trabajo desktop P34 a medio hacer → checkpoint commiteado (`5a7f31e0`, `89ab5e2c`) antes de arrancar.
- Segunda sesión activa editando WorkspaceShell.tsx → re-verificar churn antes de Wave 2.
- Sub-agentes NO commitean (evita race del index); el lead verifica mecánico y commitea por tarea.
