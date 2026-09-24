# PRX-04 — Cache-preserving injection

- **Estado:** ✅ COMPLETED
- **Commit:** (pendiente push por lead)
- **Plan:** docs/dev/plans/2026-09-08-backlog.md (Task 7, Wave2)
- **Contrato:** `cargo test -p vanta-proxy` 0 failed + test nuevo prefijo estable byte-a-byte ✅ + `cargo clippy -p vanta-proxy -- -D warnings` 0
- **Branch:** develop
- **SDP:** test-driven-development, incremental-implementation, context-engineering (+ source-driven-development base; frontend-ui-engineering / api-and-interface-design no aplican — sin UI ni API nueva)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vanta-proxy/src/inject.rs` (384L), `vanta-proxy/src/server.rs` (process_inner D29 + forward_raw + tool loop), `vanta-proxy/src/forward.rs` (verbatim passthrough), `vanta-proxy/tests/pipeline.rs` (365L, patrón fixtures reales), `vanta-proxy/tests/proxy_wire.rs` (verbatim asserts).
- **Referencias hacia dentro (inject.rs usa):** `serde_json::Value`, `bytes::Bytes`, `crate::error::ProxyError`, `vanta-memory` persona/scene (solo `build_memory_block`).
- **Referencias entrantes (quién usa inject):** `server.rs:225-230` (`build_memory_block` + `inject_into` en process_inner paso 5); nadie más. Tests unitarios en `inject.rs:270-384`.
- **Veredicto:** impacto LOCAL — solo `inject_into` + helpers privados. Sin símbolos `pub` nuevos (tests `#[cfg(test)]` no cuentan). Gate D no dispara. `forward.rs` verbatim intacto. Scope SOLO exacto (PRX-09 semántico fuera).

## Defectos (DISCOVERY, zero-code)

1. **Anthropic `system` string sin idempotencia** (`inject.rs:207-209`): reinjecta siempre → doble bloque en retry → prefijo inestable. OpenAI/Responses sí tienen `starts_with`.
2. **Anthropic `system` array inserta incondicional** (`inject.rs:210-214`): `insert(0)` duplica en re-inject → inestable.
3. **`inject_into` siempre re-serializa** (`inject.rs:265-267`): aunque nada cambie (bloque vacío + tools completas) devuelve `Some` → el caller forwardea bytes re-serializados en vez de verbatim. Debe devolver `None` cuando no hay cambio (callers ya tratan `None` como verbatim).
4. `serde_json` sin `preserve_order` (workspace `Cargo.toml:53,205`) → `Map` = BTreeMap, claves ordenadas → re-serialización determinista: mismo `Value` ⇒ mismos bytes. La estabilidad byte-a-byte depende SOLO de idempotencia (1-3). Sin cambio de dependencia.

## Steps

- [x] **Step 1 RED:** tests prefijo estable byte-a-byte por provider — 4 FAILED por razón correcta (RESUME: fixture noop tenía `}` faltante que lo hacía pasar vacuo; probe `fixture must parse` lo expuso, fixture corregido).
- [x] **Step 2 GREEN:** guards idempotencia + flag `changed` → `None` sin cambio. Solo `inject.rs`; `forward.rs`/`server.rs` intactos.
- [x] **Step 3 VERIFY + commit:** `cargo test -p vanta-proxy` 125 passed/0 failed (100 lib + 5 pipeline + 10 wire + 5 prx01 + 5 tool_loop) + clippy `--all-targets -D warnings` 0 + fmt 0.

## Verify (contrato)

- `cargo test -p vanta-proxy` → 125 passed, 0 failed ✅ (7 tests nuevos `prx04_*`)
- `cargo clippy -p vanta-proxy --all-targets -- -D warnings` → 0 ✅ (1 collapsible_match corregido)
- `cargo fmt -p vanta-proxy -- --check` → 0 ✅

## NOTICED BUT NOT TOUCHING

- OpenAI `messages[].content` en forma array-de-partes no se inyecta (`inject.rs:235-243` solo `Value::String`) — fuera de scope (ni Anthropic ni Responses lo necesitan); ¿crear FIND? no, comportamiento pre-existente, no regresión.
