---
title: "EGO-01 (épico): contrato crítico — EGO-05/09 + FIND-221 (EGO-01..04 ya implementados, review pendiente)"
kind: task
description: "Value acepta escalares JS planos; Embedded::open_readonly + patrón daemon/socket; put_batch contrato documentado + tests. EGO-01/02/03/04 NO reimplementar (commits 82317d89, c82f4b2d, 36ac14ff, b04d33bf)."
---

# EGO-01 (épico): contrato crítico Ego — EGO-05/09 + FIND-221

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (Task 8, Wave 1 tras API-10 ✅)
- **Fuente:** Backlog §EGO-01/05/09 + §FIND-221 ← Ego `VDB-BUG-01/02`, `VDB-INC-02`, análisis externo F1
- **Esfuerzo:** 🟡 max 3d · **Prioridad:** 🔴
- **Tipo:** Rust core SDK + tests (sin tocar `src/storage/`, `src/vector/`, `src/wal.rs`)
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE + commit orquestador)
- **Next:** EGO-03
- **SDP:** codebase-memory, source-driven-development, progreso (lectura), ponytail
- **Reglas área:** `.opencode/rules/api-contract.md` + `js-ecosystem.md` + `durability.md` (leídas)

## DISCOVERY — qué ya está hecho (NO reimplementar)

| ID | Commit | Estado verificado |
|----|--------|-------------------|
| EGO-01 cursor Null/string | `82317d89` | ✅ en árbol (`vantadb-node/src/lib.rs:825-838`, comentario EGO-01 presente) |
| EGO-02 query_vector opcional | `c82f4b2d` | ✅ en historial (no verificado en árbol — fuera de scope) |
| EGO-03 comodines nativos | `36ac14ff` | ✅ en historial (`src/sdk/api/search.rs` PROHIBIDO tocar — EGO-03 futuro) |
| EGO-04 tokenizador Unicode | `b04d33bf` | ✅ en historial |
| Review P2-01 + backfill FIND-335 | — | ⏳ pendiente orquestador (NO es este task) |

- **FIND-221:** `docs/api/PYTHON_SDK.md:218` ya documenta "atómico por chunk, no todo-o-nada". Falta paridad en docstring Rust + tests que pinneen el contrato.
- **EGO-09 lock:** `src/storage/engine/init.rs:171-255` — `read_only` ya usa `try_lock_shared` (fs2); el lock exclusivo del writer bloquea también al lector compartido; `FjallBackend::open` ignora `read_only` (fjall 3.1.x sin modo read-only en registry local) → lock-free total requiere Engine/backends (FUERA de scope worker: `src/storage/` prohibido).
- **EGO-05 wire:** `Value` es externally-tagged (`{"Int":42}`); `vantadb-node` usa `VantaValue` propio (no le afecta); WASM/Python deserializan el tipo core vía serde → ganan tolerancia automáticamente, serialización intacta (retrocompatible, Gate D no dispara).

## Contrato

`cargo nextest run -p vantadb --test sdk_serialization --test multi_process_lock` verde + `list({cursor: undefined})` y `cursor: "0"` no fallan (ya cubierto por EGO-01) + `put_batch` atómico documentado.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/types.rs` (`Value` L106-144), `src/sdk/builder.rs` (`Embedded::open/open_with_config` L94-142, sin `open_readonly`), `src/sdk/api/memory.rs` (`put_batch` L901-929, `put_batch_inner` L931-1170, `check_read_only` L264, `materialize_confidence` L290-352), `src/storage/engine/init.rs` (`init_storage` L152-281, solo lectura), `src/backends/fjall_backend.rs` (`open` L56-59, solo lectura), `tests/storage/multi_process_lock.rs` (157L), `tests/sdk_serialization.rs` (estilo tests L1-45)
- **Referencias hacia dentro:** `Value` ← `Fields/MemoryMetadata` ← `MemoryInput.metadata`, `MemoryRecord`, shred, text/sparse index; `Embedded::open*` ← bindings + CLI + server; `put_batch_inner` ← `put_batch` ← bindings
- **Referencias entrantes:** `Value::Deserialize` ← serde_json en bindings (WASM/Python) + tests; `read_only` ← `Config.read_only` + CLI `--read-only`
- **Veredicto:** medio-bajo — Deserialize aditivo (output idéntico), constructor nuevo (aditivo), docstring + tests; NADA en `src/storage/` ni `src/sdk/api/search.rs`

## Slices

- [x] **Slice 0 — EGO-01:** verificado presente, NO reimplementar
- [x] **Slice 1 — EGO-05:** `Deserialize` tolerante para `Value` (rama `is_human_readable`: escalares planos + tagged + homogéneos; rama binaria postcard intacta byte-ident) + 5 tests en `tests/sdk_serialization.rs` + 1 unit postcard all-variants en `types.rs`. Hallazgo: serde serializa unit `Null` como `"Null"` pelado → `"Null"` mapea a `Null` (como derived) para no romper roundtrip.
- [x] **Slice 2 — EGO-09:** `Embedded::open_readonly` (SDK) + doc patrón daemon/socket + 3 tests en `tests/embedded_readonly.rs` (nuevo); lock-free backend → FIND Engine (fuera de scope)
- [x] **Slice 3 — FIND-221:** contrato exacto en docstring `put_batch` (paridad PYTHON_SDK.md) + 2 tests que pinnean atomicidad de validación upfront en `src/sdk/api.rs`; 2PC total → FIND Engine

## Spec

Sin símbolos públicos nuevos salvo `Embedded::open_readonly` (constructor aditivo sobre `Config.read_only` existente — documentado, sin Gate P/D: no agrega endpoint ni cambia wire output).

## Invariantes (handoff — MUST)

- `Value` serializa EXACTAMENTE igual que antes (externally-tagged); solo `Deserialize` acepta un superconjunto
- `put_batch_inner`: sin cambios de lógica de commit en este task (doc + tests); `purge_lock` intacto (Regla 8: cualquier cambio futuro → vanta-chaos + vanta-review)
- `src/storage/`, `src/vector/`, `src/wal.rs`, `src/sdk/api/search.rs`: NO tocar
- `vantadb-python/**` (WIP ajeno unstaged), stash, `.opencode/**`, CHANGELOG, `docs/index.md`, `llms.txt`, workflows/, `docs/dev/plans/`: NO tocar
- Comandos con Regla dura `-p`; nextest timeout ≥600s; cargo vía `$env:USERPROFILE\.cargo\bin`

## Steps

- [x] DISCOVERY + task file (este archivo)
- [x] Slice 1 EGO-05 RED→GREEN + `sdk_serialization` verde (28/28 directo)
- [x] Slice 2 EGO-09 `open_readonly` + docs + tests (3/3 directo)
- [x] Slice 3 FIND-221 doc + tests (en lib 595/595)
- [x] `cargo fmt --check` + `cargo clippy -p vantadb --lib --test sdk_serialization --test embedded_readonly -- -D warnings` + `git diff --check`
- [x] Save Point + RESULTADO 🟡 (PROXIMO_STEP="review + commit", orquestador)

## Save Point (2026-10-16 — trabajo completo, sin commit)

- **Archivos tocados (solo estos):** `src/sdk/types.rs`, `src/sdk/builder.rs`, `src/sdk/api/memory.rs`, `src/sdk/api.rs`, `tests/sdk_serialization.rs`, `tests/embedded_readonly.rs` (nuevo), `docs/dev/tasks/EGO-01.md` (este archivo). `Cargo.toml` y `src/bin/` restaurados intactos (verificado `git status` vacío en ellos).
- **Evidencia:** lib `sdk::` 595/595 · `sdk_serialization` 28/28 · `embedded_readonly` 3/3 · `multi_process_lock` 2/2 (sin tocar) · fmt/clippy/diff--check limpios.
- **Desviación conocida:** `cargo nextest run` literal NO ejecutado — `target/debug/vanta-cli.exe` bloqueado por 2 servidores MCP ajenos (`server --mcp`, PIDs rotativos, NO matar). Se ejecutaron los mismos test binaries (libtest harness) compilados por cargo contra la lib actual. Re-correr nextest al commitear cuando los servidores liberen. Clippy full `--all-targets` omitido por el mismo lock; corrido en `--lib` + 2 test targets con `-D warnings`.
- **Deuda para Engine (FINDs sugeridos):** 2PC todo-o-nada real de `put_batch`; lectura lock-free Fjall (`FjallBackend::open` ignora `read_only`, fjall 3.1.x sin modo read-only).
- **Si la sesión se interrumpe:** nada a mitad — todos los artefactos están commiteables tal cual; solo falta review + commit del orquestador.

## Review

- **Revisor:** vanta-review, sesión distinta al implementador (veredicto 2026-10-15, contrato re-ejecutado con binarios directos + clippy + fmt).
- **Veredicto:** ✅ APPROVE EGO-01 — Deserialize quirúrgico (red-team sin rupturas), open_readonly enforced, FIND-221 pinneado con honestidad de alcance, Regla 8 limpia. 2 Optional follow-up (assert Validation en readonly test, visit_i64 TagVisitor).
- **Nota previa superada:** (Pendiente orquestador — P2-01 + Regla 8 concurrencia + FIND-335 backfill EGO-01..04) — Regla 8 verificada limpia por el revisor; backfill FIND-335 sigue en Backlog como fila propia.

## Recitation

| Campo | Valor |
|-------|-------|
| `activeGoal` | EGO-01 épico: EGO-05/09 + FIND-221 (EGO-01..04 hechos) |
| `lastAction` | Slices 1-3 GREEN + verify completo (ver Save Point) |
| `result` | PARTIAL (implementación + evidencia completas; falta review + commit orquestador) |
| `nextAction` | Orquestador: P2-01 review + re-correr `cargo nextest run -p vantadb --test sdk_serialization --test multi_process_lock` + commit |
| `contract.verificacion` | lib sdk:: 595/595 · sdk_serialization 28/28 · embedded_readonly 3/3 · multi_process_lock 2/2 (binarios directos; nextest bloqueado por lock ajeno) · fmt/clippy/diff--check limpios |
| `contract.deuda` | review P2-01 + commit (orquestador); 2PC put_batch + lock-free Fjall read (Engine); backfill FIND-335 EGO-01..04 |
