# Unified Review — certify — 2026-08-05

**Profile:** vantadb
**Mode:** certify
**Duration:** ~30 min (incluye retry L1 tras liberar disco)
**Quality Gate:** ✅ PASS
**Overall score:** 9.4/10 (rating: A)
**Ponytail mode:** full

## Executive Summary

El gate pre-push de certificación (modo `certify`, profile `vantadb`) **pasó con 9.4/10**. La fase crítica **L1 (Rust workspace) está verde**: `cargo nextest run --profile audit` corrió **1829/1829 tests** tras liberar espacio en disco. Todas las fases mecánicas y cognitivas (L2-L6) también pasaron.

El primer intento falló por una causa **ambiental, no de lógica**: C: tenía solo **1.70 GB libres** y 2 tests de `security.rs` abortaron con `StorageFull (115=112- Os 112)`. Tras limpiar el Temp del OS (+~64 GB), el pipeline completo corrió sin cierre. No hubo defecto de código en los tests fallidos.

Hallazgos reales: **0 críticos, 0 high**. Dos medium accionables: (1) `npm audit` en `web/` reporta 6 vulnerabilidades (3 high) que el job `ci-web-11` no comprueba y por tanto pueden salir silenciosas; (2) `test_sdk.py` ejercita la firma deprecada posicional de `put_batch`. El resto son low/info de higiene. No hay breaking change en API, no hay bumps de versión en el working tree, y la estructura de releases (release-plz) está coherente.

Está **listo para publicar** una vez moderadas ambas vulnerabilidades web (o si no se publica web en este ciclo).

## Scoreboard

| Phase | Status | Score | Findings (C/H/M/L/I) | Duration |
|-------|--------|-------|----------------------|----------|
| L0 Diff Impact | ✅ | — | 0/0/0/0/0 | 2s |
| L1 Core Language | ✅ | 9/10 | 0/0/0/2/0 | ~9 min |
| L2 Bindings | ✅ | 9/10 | 0/0/1/2/1 | 4 min |
| L3 Web Frontend | ✅ | 10/10 | 0/0/0/0/1 | 2 min |
| L4 CI/CD + Deps | ⚠️ | 7/10 | 0/0/1/3/4 | 2 min |
| L5 Documentation | ✅ | 10/10 | 0/0/0/0/0 | 2s |
| L6 Architecture | ✅ | 9.5/10 | 0/0/0/0/1 | 7s |
| **OVERALL** | ✅ | **9.4/10** | **0/0/2/8/7** | ~30 min |

**Quality Gate (certify):** ✅ PASS — L1 (critical) verde + 0 findings critical.

## L1 — Rust Workspace (crítico)

- **1er run:** ❌ FAIL exclusivamente por disco (`StorageFull` en 2 tests de `security.rs`). `fmt/check/clippy -D warnings/deny/machete` ya pasaban.
- **Retry tras liberar disco:** ✅ **46 gates, 1829/1829 tests** (2 skipped), 0 warnings clippy, `cargo deny` limpio.
- Hallazgos low/info: `expect` en `src/binary_header.rs:67` y `src/cli_server.rs:142,176` — todos infalibles en runtime (guardados por checks previos o constantes), candidatos a `Result` por convención.

## L2 — Bindings (Python/WASM/TS)

Python prioritario (el diff tocó `vantadb-python/`): venv maturin build ✅, **62 pytest pass** ✅, PyO3 `cargo check` limpio ✅. WASM: `wasm-pack build --release` ✅ (1m09s, wasm-target presente). TS: `npm ci` + `tsc --noEmit` ✅.

## L3 — Web Frontend

`npm run lint` 0/0 · `npx tsc --noEmit` **0 errores** (check real, ya que `next.config.ts` tiene `ignoreBuildErrors: true`) · `npm run build` ✅ (35/35 páginas, Turbopack 14s). Sin regresión visible en layout.tsx (JSON-LD) ni code-playground.

## L4 — CI/CD + Dependencies

- `cargo deny check` ✅ (como sección: advisores/bans/letras). `cargo audit` ✅ con 1 warning permitido (`RUSTSEC-2026-0577 lru 0.12.5` vía ratatui, tier declarado unsound-warning).
- `npm audit` en `web/`: **6 vulns (3 high, 3 moderate)** → ci-web-11 NO tiene gate npm audit → van silenciosas.
- Parity: working tree **sin bumps de versión**; release-plz.co.toml correcto (semver de based on conventional commits); el workflow windows/release actualiza tanto wheels como el MCP; vendría sin conflictos.

## L5 — Documentation

`scripts/validate-docs-coverage.ps1` → **0 gaps** (63+13 sdk, 46 config, 33 error, 40 CLI, 42 Python, 15 MCP). CHANGELOG actualizado a [0.5.0] 2026-07-31. Markdown viable.

## L6 — Architecture

Layering OK: los bindings importan solo API pública del core. `wal.rs` no importa `crate::index` (bien). Grafo acyclic, budget de acoplamiento ≤5 por crate. 1 info: mcp/server usan `vantadb::storage::StorageEngine` directo en vez de la capa `sdk::VantaEmbedded` — no viola privacidad y expansión `pub`. No-blocker.

## Findings by category

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 2 |
| Low | 8 |
| Info | 7 |

### Medium

- **[H02-CODE-001]** `vantadb-python/tests/test_sdk.py:498,517` — put_batch() se llama con la firma posicional deprecada (`db.put_batch(entries)`); la nueva API keyword (`keys=`, `vectors=`) no la usa como primaria en tests. *Fix:* migrar a keyword-args (≠bug de runtime, solo deprecation en pytest/warning).
- **[H04-CI-002]** `web/package.json` — npm audit = 6 vulnerabilities (3 high). No gate active. *Mitigue:* `npm audit fix` y añadir `npm audit --audit-level=high` al job `ci-web-11` antes de next web release.

### Low

- `src/binary_header.rs:67` — expect infalible en librero (producción); sugerencia a `Result`.
- `src/cli_server.rs:142,176` — expect/unwrap infalibles.
- `vantadb/src/metrics/core/mod.rs:298` — `get_native_memory` dead_code en wasm.
- `vantadb-wasm/Cargo.toml` — LICENSE file ausente (bloquea publish a crates.io). (high prior, atenuable).
- `deny.toml` vs `.cargo/audit.toml` — divergencia de ignores (RUSTSEC-2024-0436); sync.
- Cargo.lock — duplicados benignos (rand, thiserror, syn).

### Info

- `vantadb-ts/package.json` — npm audit 2 high (no gate; SDK pre-1.0).
- deny.toml license list broader than MIT/Apache-only (standard).
- `next.config.ts` — Turbopack multi-package-lock false root; set `turbopack.root`.

## Cross-Cutting Patterns

Ningún patrón tipificado transversal (la deprecación put_batch es solo L2; el gap npm-audit solo L4). No hay tendencia única de proyecto en este snapshot.

## Recommendations (priorized)

1. **(medium, antes de publicar web)** Mitigar 6 vulns npm en `web/` y añadir gate `npm audit --audit-level=high` a `ci-web-11`.
2. **(medium, cleanup)** Migrar `put_batch` posicional → keyword en `test_sdk.py:498,517`.
3. **(low, pre-wasm-publish)** Añadir `LICENSE` en `vantadb-wasm/`.
4. **(low, hygiene)** Sincronizar ignores RUSTSEC-2024-0436 entre `deny.toml` y `.cargo/audit.toml`.
5. **(info)** Considerar rutear `mcp/server` a `vantadb::sdk` para consistencia con python/wasm.

## Nota de lección (Regla 5)

Conectar el `StorageFull` al run de nextest: verificar espacio libre en disco (min. 3-5 GB) antes de correr el certificado pesado. Este host roza el límite de espacio cuando el working tree crece.

---
*Generated by unified-review skill. Profile: vantadb. Mode: certify.*