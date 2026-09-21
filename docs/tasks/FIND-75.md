# FIND-75 — WASM README + zero-copy input + d.ts

> **Plan:** `docs/plans/2026-09-15-find-correcciones.md` (Task 20, Wave6)
> **Estado:** ✅ COMPLETE (2026-09-15; review vanta-review APPROVE ses_f598b3891ffeusxg1e8cEH2uUY)

## Review P2-01

- Verdict **APPROVE**, 0 required changes; 5/5 puntos contrato verificados con comandos exactos por el reviewer (re-medición byte a byte match, ratio 28.2, 0 Rust en diff, d.ts --check exit 0, Regla 11 OK).
- Opcional no bloqueante (NO aplicado, scope discipline): `README.md:19,65` cita `dev-tools/build-wasm.ps1` inexistente (arrastre WSM-13 pre-existente) → follow-up.
- NOTICED BUT NOT TOUCHING: `vantadb-wasm/demo/README.md:38-39,58` cita números viejos (1.35MB/578KB) — fuera de Archivos clave → orquestador decide (nueva fila FIND o extensión).
- Colateral conocido: clippy `-D warnings` rojo solo por `txn.rs:158` (FIND-93 ya ticketado); 0 warnings en `vantadb-wasm`.
> **Appetite:** 4h · 🟢 · 🟢 · **Ruta:** vanta-worker
> **Archivos clave:** `vantadb-wasm/README.md`, `vantadb-wasm/src/lib.rs:2377-2378`, `vantadb-wasm/src/vantadb_wasm.d.ts`
> **SDP:** campaign-executor, progreso, source-driven-development (base) + incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (lifecycle BUILD) — resto descartado: frontend-ui-engineering (sin UI web/), systematic-debugging (no es bug), codebase-memory usada vía codegraph_explore directo.

## Contrato

- Números README re-medidos desde `pkg/` real (NO copiar 1.35MB/1.411.870B a ciegas) + evidencia de medición.
- Input zero-copy implementado o DEFER-ratificado con evidencia (parseo mayor → DEFER salida válida, no refactor).
- Diff `src/vantadb_wasm.d.ts` vs `pkg/` post-build + comando documentado + clippy 0.
- Prohibidos (NO TOCAR): `.opencode/`, `completions/`, tauri lock, stash@{0}, FIND-66 (Formula/), FIND-69 (dspy/), resto del workspace Rust fuera de `lib.rs:2377-2378`.

## Discovery — type & blast radius

- `campaign_detect_task_type` → `wasm` (WASM binding; check: `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`).
- codegraph_explore "vantadb-wasm lib zero-copy MemoryInput": 46 símbolos / 4 files. `from_js` (lib.rs:2366) tiene **14 callers** en `lib.rs`, sin tests covering. `NodeInput` 11 callers cross-crate.
- Gate D: NO disparado — 0 símbolos públicos nuevos (docs + DEFER, sin cambio de API), contrato no ambiguo (Stop explícito), blast radius edición = 1 archivo docs.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-wasm/README.md` (263 líneas), `vantadb-wasm/src/vantadb_wasm.d.ts` (794 líneas), `dev-tools/build-wasm-types.mjs` (67 líneas), `lib.rs:100-219` (structs input), `lib.rs:1120-1204` (put/put_batch), `lib.rs:1680-1711` (insert_node), `lib.rs:2298-2378` (output zero-copy + comentario pendiente).
- **Referencias hacia dentro:** README cita `pkg/` (generado, gitignored vía `vantadb-wasm/pkg/.gitignore:1` → `*`), `Cargo.toml` (`wasm-opt -Oz`), `build-wasm.ps1`, `vantadb-ts/README.md`, `docs/QUICKSTART.md`.
- **Referencias entrantes:** `vantadb-ts/src/types.ts:47-50` documenta PERF-08 output Float32Array (coherente, no se toca); ningún workflow/Cargo cita los números del README.
- **Veredicto:** edición segura acotada a `vantadb-wasm/README.md` (docs-only); `lib.rs` y `.d.ts` SOLO lectura (DEFER, sin refactor); `pkg/` solo lectura + sync local via script documentado (gitignored, no commiteado).

## Verificación línea por línea (divergencia vs plan)

| # | Plan asumía (2026-09-15) | Real en disco hoy | Veredicto |
|---|---|---|---|
| V1 números | "YA dice 1.35MB, al día, re-verificar no re-escribir" | `pkg/vantadb_wasm_bg.wasm` = **1.658.202 B (1,58 MB)**, gzip **676.239 B (660 KB)**; pkg build 2026-09-11 01:44, `lib.rs` modificado después (mismo día 15:29) | README STALE en ~17% — HALLAZGO, se re-escribe con números medidos + nota staleness |
| V2 input | `lib.rs:2377-2378` pendiente explícito post-PERF-08 | Confirmado verbatim; input va por `from_js` whole-object serde en 14 call sites (put/put_batch/search×3/explain/list/import/insert_node-fields/filters) + adapters custom (`deserialize_sparse_vector` string→u32, `deserialize_cursor` string\|number, maps json_compatible) | Parseo mayor → DEFER-ratificado (H2), no refactor |
| V3 d.ts | "`src/vantadb_wasm.d.ts` existe vs `pkg/` → diff post-build" | `pkg/vantadb_wasm.d.ts` = stubs generados con `any` (NO aplicado el hand-written); `node dev-tools/build-wasm-types.mjs --check` → **exit 3 FAIL** | Out-of-sync real; sync = correr script (gitignored) + documentar comando en README (H3) |

**Medición completa pkg actual (2026-09-15, PowerShell .NET GzipStream):**

| File | Raw bytes | Raw | Gzip bytes | Gzip |
|------|----------:|-----|----------:|------|
| `vantadb_wasm_bg.wasm` | 1.658.202 | 1,58 MB | 676.239 | 660 KB |
| `vantadb_wasm_bg.js` | 56.925 | 55,6 KB | 10.886 | 10,6 KB |
| `vantadb_wasm.js` | 245 | 245 B | 156 | 156 B |
| **TOTAL transfer** | **1.715.372** | **1,64 MB** | **687.281** | **~671 KB** |

## Spec (decisiones)

| ID | Decisión | Alternativa descartada | Por qué |
|---|---|---|---|
| H1 | Re-escribir tabla §1 + intro + fila comparativa + Last reviewed con números medidos hoy + nota staleness (pkg 09-11 < lib.rs 09-11) + comando repro | "Re-verificar sin tocar" (asunción plan) | Evidencia V1: 1.658.202 ≠ 1.411.870; Regla 11 exige números reproducibles; no re-escribir sería dejar doc mintiendo |
| H2 | DEFER-ratificado input zero-copy: documentar en task file (no en código) por qué no se toca | Implementar Float32Array directo en 6 shapes de input | 14 call sites + 3 adapters serde custom + validación MAX_F32_VEC_LEN por shape; ganancia solo en vectores grandes, riesgo de romper validación; Stop del plan lo declara salida válida |
| H3 | Correr `node dev-tools/build-wasm-types.mjs` local (sync pkg gitignored) + `--check` como evidencia + documentar comando en README §1/Referencias | Rebuildear wasm completo (`wasm-pack build`) | Rebuild pesado/OOM fuera de appetite; el contrato pide "diff post-build + documentar comando", no rebuild; script es el sync canónico (release-npm-61.yml lo invoca post-build) |

## Steps atómicos

- [x] **STEP-1 — README números + comando d.ts** (1 archivo): aplicar H1+H3 en `vantadb-wasm/README.md`; verify: `git diff --check` + `cargo check -p vantadb-wasm -j 2` + `cargo clippy -p vantadb-wasm -- -D warnings` + script `--check` OK post-sync + lectura final. Commit `docs: FIND-75 — WASM README re-medido + d.ts sync doc`.
- [x] **STEP-2 — Cierre:** review P2-01 (vanta-review o self doble-lectura si no disponible) + recitation plan (unstaged, race Wave6) + RESULTADO.

## Context Save Point

- Discovery 2026-09-15; H1/H2/H3 cerradas con evidencia; `lib.rs`/`.d.ts` intactos (solo lectura); `pkg/` gitignored (`pkg/.gitignore:1`); WIP ajeno (`.opencode/`, `completions/`, tauri lock, FIND-69) NO incluir en commit (solo `vantadb-wasm/README.md` + este task file).
