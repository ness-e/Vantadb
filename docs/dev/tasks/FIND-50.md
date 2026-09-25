# FIND-50 — Split src/parser/mod.rs 1682L (grammar.rs + lexer.rs)

> **Plan:** docs/dev/plans/2026-09-08-backlog.md (Task 3, Wave0)
> **Estado:** ✅ COMPLETO (2026-09-08, resume SARL1: bloqueo Wave0 resuelto)
> **Appetite / Branch / Commit:** max 1d / develop / `refactor: FIND-50 split parser grammar lexer`
> **Wave:** Wave0 — archivos disjuntos de FIND-48/49; NO tocar `src/index/` ni `src/sdk/`
> **Gate Justificación:** parser IQL monolítico 64546 bytes; grammar/lexer separados facilitan insta snapshots (TBH-06)
> **Pre-mortem:** macros pegadas lexer rompen split a ciegas; snapshots cambian por reordenamiento
> **Contrato:** `cargo check -p vantadb` 0 + parser tests 0 failed + `cargo insta test --check` 0 nuevos `.snap.new`
> **SDP:** campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design + base auto (progreso, ponytail full). Excluida: frontend-ui-engineering (sin UI/web en scope).

## Gate D — evaluación (question-gates.md)

**Veredicto: NO dispara.** Blast radius = 3 archivos (`executor.rs:8`, `server/handlers.rs:569`, `lib.rs:115` decl) + `tests/logic/parser.rs` vía glob `vantadb::parser::*` (preservado por re-exports). Sin símbolos públicos nuevos (move puro, `pub use` re-exports). Sin cambio semántico (código movido byte-verbatim, solo visibilidad `fn`→`pub(crate)` intra-parser). Refactor mecánico, contrato mecánico. Sin `question` al usuario.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/parser/mod.rs` (1897L Read / 1682L wc / 64546 bytes — verificado 2026-09-08): líneas 1-625 lógica + 627-1897 `mod tests`
- `src/lib.rs:115` (`pub mod parser;`), `src/executor.rs:8`, `src/server/handlers.rs:569`
- `tests/logic/parser.rs` (132L: usa `vantadb::parser::*` → `parse_query`, `parse_statement` + 3 insta snapshots)
- Rules: `.opencode/rules/query-dsl.md` (R-4: parser delega frases al planner — el split no toca `parse_condition`), `.opencode/rules/core-engine.md` (R-3 sin unwrap en código movido — verificado: el código usa `?`/combinadores; R-2: no exportar internos nuevos al SDK — `pub(crate)` interno, `pub` surface idéntica)

**Referencias hacia dentro (qué usa el parser):**
- `nom` (branch/bytes/character/combinator/multi/number/sequence), `crate::node::FieldValue`, `crate::query::*`, `crate::sdk::{SearchProfileConfig, SearchProfileMode}`

**Referencias entrantes (quién usa el parser):**
- `src/executor.rs:8` → `crate::parser::parse_statement`
- `src/server/handlers.rs:569` → `crate::parser::autocomplete_prefix`
- `tests/logic/parser.rs:10` → `vantadb::parser::*` (parse_query, parse_statement)
- `src/parser/mod.rs` tests internos → `super::*` (ident, parse_number, ws, etc. — privados hoy)

**Veredicto de impacto:** BAJO y contenido. Corte por nivel léxico vs gramatical, sin mover tests (`mod tests` queda en `mod.rs`, resuelve vía re-exports). Superficie `pub` idéntica post-split. Riesgo pre-mortem #1 (macros pegadas) mitigado: nom es funciones/combinadores, no `macro_rules!` locales — no hay macros que mover. Riesgo #2 (snap churn) mitigado: AST types viven en `crate::query`, no se tocan; Debug output idéntico.

## Corte (qué va a cada archivo)

- **`src/parser/lexer.rs`** (~190L): `ws` (pub, se mantiene), `ident`, `RESERVED_KEYWORDS` (→`pub(crate)`), `non_keyword_ident`, `parse_number`, `string_literal`, `parse_u128_id`, `parse_i64`, `parse_literal_field_value`, `parse_vector_lit` (movido desde §DML, es literal-level). Todo `pub(crate)` salvo `ws`.
- **`src/parser/grammar.rs`** (~420L): `parse_traversal`, `parse_rel_op`, `parse_condition`, `parse_query` (pub), `parse_profile_mode`, `parse_field_assign`, `parse_insert`, `parse_update_field_expr`, `parse_update`, `parse_delete`, `parse_relate`, `parse_insert_message`, `parse_join_on`, `parse_join_clause`, `parse_subquery_condition_inner`, `parse_where_item`, `WhereItem` (pub), `parse_select` (pub), `parse_statement` (pub), `EXTRA_AUTOCOMPLETE_KEYWORDS`, `is_keyword_token`, `autocomplete_prefix` (pub). Imports explícitos `use super::lexer::{...}` (sin ciclos: lexer no importa grammar).
- **`src/parser/mod.rs`** (~1280L): docs + `pub mod grammar; pub mod lexer;` + `pub use grammar::*; pub use lexer::*;` + `mod tests` intacto.

## Steps atómicos

| # | Step | Contrato step | Estado |
|---|------|---------------|--------|
| 1 | Crear `src/parser/lexer.rs` + `mod.rs` declara módulos + re-exports | `cargo check -p vantadb` 0 | ✅ |
| 2 | Crear `src/parser/grammar.rs`, `mod.rs` thin + tests | `cargo check` 0 + clippy 0 + `--all-targets` 0 errores parser | ✅ |
| 3 | Verify full contrato + fmt + commit | parser tests 0 failed + insta 0 `.snap.new` + fmt 0 + commit | ✅ |

## Context Save Point

- Repo: rama `develop`, medido `src/parser/mod.rs` = 64546 bytes / 1682L (wc) — coincide backlog 2026-09-02.
- `insta = "1.48"` dev-dep workspace (Cargo.toml:208); snapshots parser en `tests/logic/snapshots/parser__*.snap` (3); target `[[test]] parser` (Cargo.toml:351).
- Estado git al inicio: `M .opencode`, `M opencode.jsonc`, `?? docs/dev/plans/2026-09-08-backlog.md` — FUERA de scope, no commitear.
- TDD: refactor mecánico sin lógica nueva → RED no aplica (verificación = suite existente verde antes/después); baseline: suite parser debe estar verde pre-split (step 0 implícito en step 1 verify).

## Iteraciones

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| 1 | DISCOVERY: codegraph + grep blast radius + rules + SDP | Gate D no dispara; corte definido | codegraph_explore, grep, read |
| 2 | Slice 1: `lexer.rs` creado, `mod.rs` re-exports | `cargo check` 0 warnings | edit, bash |
| 3 | Slice 2: `grammar.rs` creado, `mod.rs` thin + tests-header fix + `pub(crate)` uniform | `--all-targets`: 0 errores en `src/parser` | edit, bash |
| 4 | Slice 3 verify: fmt ✅; tests/clippy/insta BLOQUEADOS por breakage ajeno (FIND-48/49 mid-split) | Parcial — ver Bloqueo | rustfmt, bash |
| 5 | RESUME SARL1 (bloqueo resuelto: FIND-48 fcd339b7 + FIND-49 e3711dea): check 0 + nextest lib 117/117 + `--test parser` 4/4 + `cargo insta test --check` exit 0 (0 `.snap.new`) + clippy `--all-targets` 0 + fmt 0 → commit pathspec | ✅ COMPLETO | bash |

## Bloqueo (RESUELTO 2026-09-08 ✅)

- ~~`src/sdk/types.rs` (FIND-49) + `src/index/graph.rs` (FIND-48) en split concurrente~~ → ambos COMPLETO; lib verde.
- Evidencia verify final: `cargo check -p vantadb` exit 0 · `cargo nextest run -p vantadb --lib parser` 117 passed · `cargo nextest run -p vantadb --test parser` 4 passed (3 insta snapshots sin drift) · `cargo insta test --check -p vantadb -- --test parser` exit 0 "no snapshots to review" · `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0 · `rustfmt --check` 3 archivos exit 0 · 0 `*.snap.new` en repo.
- Commit pathspec solo: `src/parser/mod.rs src/parser/lexer.rs src/parser/grammar.rs docs/dev/tasks/FIND-50.md` (plan file queda sin commitear — ownership lead).

## Context Save Point (final)

- `src/sdk/types.rs` (FIND-49) + `src/index/graph.rs` (FIND-48) en split concurrente → lib no compila (253 errores, 0 en `src/parser` — probado vía `git stash` baseline: 616 errores sin mis cambios).
- `cargo fmt --check` global falla al parsear `types.rs` ajeno; mis 3 archivos fmt ✅.
- NO commitear en rojo; NO tocar staging ajeno (`graph.rs` staged por FIND-48).
- **Resume:** cuando Wave0 estabilice lib → `cargo test -p vantadb --test parser` + `cargo insta test --check` (0 `.snap.new`) + `cargo clippy -p vantadb -- -D warnings` + commit pathspec solo `src/parser/* docs/dev/tasks/FIND-50.md`.
