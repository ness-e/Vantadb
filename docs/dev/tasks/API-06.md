# API-06: W5 IQL — versión + sintaxis + literales + AST

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 6 · investigación `docs/dev/tasks/API-STD-09.md` (+ síntesis `API-STD-15` eje IQL, checklist `API-STD-14` ítem 31, W5 `API-STD-18`)
- **Fuente:** Backlog Phase 51 (fila `API-06`)
- **Esfuerzo:** 🟡 3-5d (tool estimate: 15-30 turns)
- **Prioridad:** 🟠
- **Tipo:** Mixto (Rust core parser + docs)
- **Turns estimados:** 15-30
- **Creado:** 2026-09-25
- **last-synced:** 2026-09-25
- **Estado:** ✅ COMPLETED (2026-09-25) — contrato 4/4; Steps 0-8 ✅; review P2-01 ronda 1 ❌ → fixes R1/R2/R3 → ronda 2 ✅ APPROVE
- **Incógnitas (uphill):** 0 abiertas (case/quote/versión/AST resueltos por evidencia en §Spec; exposición del AST a bindings → `FIND-156`)
- **Pendientes (downhill):** 0/8 steps ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `parse_statement` (29 callers: `src/executor.rs:162` — camino IQL HTTP/SDK/MCP; `src/cli_handlers/data.rs:332` — `query_is_mutating`; `src/tui/repl.rs`; tests `tests/api/openapi_yaml_parity.rs`, `tests/logic/parser.rs`, `tests/storage/mutations.rs`); `autocomplete_prefix` (`src/server/handlers.rs:588`); `parse_query`/`parse_rel_op`/`parse_literal_field_value` internos al parser |
| Callees | `nom` combinators; `crate::query::*` (AST — gana `Serialize`), `crate::node::FieldValue` (ya `Serialize`), `crate::search_profile::SearchProfileConfig` (ya `Serialize`) |
| Implicaciones | (1) `42` pasa de `Float(42.0)` a `Int(42)` → cambia el tipo de los campos numéricos insertados por IQL y las comparaciones unquoted (filtros `(Int,Int)` ya soportados `filter.rs:75,242`; mixto `(Int,Float)` sigue sin coercionar → FIND-157); (2) `==` es aditivo (no rompía nada válido); (3) `from` minúscula y `'quote'` quedan como están (rechazo) — solo se documentan/pinnean; (4) default de alias en `SELECT` cambia a `target` — inerte en ejecución (`query.rs:229,298` ignoran el alias); (5) `IQL_VERSION` + derives `Serialize` aditivos |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/parser/grammar.rs` (496L) ✅, `src/parser/lexer.rs` (153L) ✅, `src/parser/mod.rs` (1288L) ✅, `src/query.rs` (749L) ✅, `src/lib.rs` (217L) ✅, `tests/logic/parser.rs` (132L) ✅, `tests/logic/snapshots/parser__dml_insert_ast.snap` ✅, `docs/api/IQL.md` (225L) ✅, `src/sdk/api.rs:280-400` (test comentario single-quote) ✅
- **Archivos leídos (secciones relevantes):** `src/executor.rs:140-249` (parse+`IqlParse`), `src/physical_plan/filter.rs` (comparación same-variant), `src/physical_plan/join.rs:205-270` (`compare_field_values`), `src/planner.rs:130-145,410-524`, `tests/api/openapi_yaml_parity.rs` (tests 24-144, 480-554), `docs/api/openapi.yaml` (1-60, 140-219, 1783-1793), `src/sdk/types/graph.rs:1-42` (`QueryResult` serde), `src/node/field.rs:1-60` (`FieldValue` serde)
- **Archivos referenciados hacia dentro (imports):** `grammar.rs` → `super::lexer::*` + `crate::query::*` + `crate::node::FieldValue`; `lexer.rs` → `nom` + `crate::node::FieldValue`; `query.rs` → `search_profile` + `node`; `lib.rs:118` `pub mod parser`
- **Archivos que referencian a los editados (referencias entrantes):** `rg parse_statement` → executor/CLI/TUI/tests; `rg parse_rel_op|parse_literal_field_value` → solo parser; `rg "query.rs"` → `query::Statement` consumido por executor/planner/CLI/MCP/HTTP
- **Veredicto impacto:** **medio** en literales (`Int` cambia el tipo almacenado por IQL — misma-variante OK, mixto documentado como FIND); **bajo** en `==`/version/`Serialize` (aditivos); **bajo** en docs (`openapi.yaml` ejemplo debe seguir parseando — test `test_yaml_drifts_fixed`)

## Contrato

"`rg IQL_VERSION src/` ≥1 (definido + gateado) Y tests parser verdes (`--lib parser` + `--test parser` + `--test openapi_yaml_parity`) Y repros `==` / `42→Int` / `from` minúscula / `'quote'` con comportamiento decidido (tests + docs) Y ejemplo YAML válido (`QueryRequest.query.example` parsea IQL)"

## Spec (SDD — feature-add: `IQL_VERSION` público + `Serialize` en el AST)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Forma de `IQL_VERSION` | A: const informativo suelto / B: const + `IQL_VERSION_MIN_PROFILE` + `iql_supports()` usado por `parse_query` / C: versión declarable en la query (sintaxis nueva) | B | ✅ decidido-por-evidencia: backlog "`IQL_VERSION` con gate (`PROFILE` y futuro)"; idioma del repo `WAL_FORMAT_VERSION` (`lib.rs:198`), `VFILE_VERSION` (`:193`); `PROFILE` es la única sintaxis post-v1 (API-STD-09 Q3) |
| 2 | Case de keywords | A: case-insensitive (rompe campos/alias minúsculas: `from`, `min`) / B: UPPERCASE-only + prohibición explícita documentada | B | ✅ decidido-por-evidencia: ya implementado y pinneado `tests/api/openapi_yaml_parity.rs:90-121`; `openapi.yaml:149-151` lo documenta (API-03); risk #1 del plan (case-insensitive rompe alias) |
| 3 | `'quote'` single | A: soportar ambos quotes (+escapes dobles, +`RELATE --'…'-->` estructural) / B: solo `"` + rechazo explícito documentado | B | ✅ decidido-por-evidencia: "1 sintaxis documentada"; `RELATE` usa quotes estructurales (`grammar.rs:241-243`) → A parcial sería F3 "single-quote a medias"; scope-only-un-quote es coherente |
| 4 | Literales numéricos | A: `parse_i64` antes que `double` (Int exacto; >i64 cae a Float) / B: `double` primero (pierde >2^53 — bug actual) | A | ✅ decidido-por-evidencia: plan Risk Register ("Int→float >2^53 → `parse_i64` primero"); contrato "`42→Int`"; `FieldValue::Int` es tipo de primera clase (`field.rs:11`, SDK `Value::Int`) |
| 5 | `==` | A: `==` antes que `=` (Eq, longest-match) / B: error | A | ✅ contrato + plan Risk ("`==` colgado → orden `==` antes que `=`") |
| 6 | Defaults de lectura | A: alinear `SELECT` → `target` (como `FROM`/`MATCH`) / B: cambiar `Query` → nombre de entidad | A | ✅ decidido-por-evidencia: `IQL.md:50` documenta default `target`; alias es inerte en ejecución (`query.rs:229,298`) |
| 7 | AST JSON | A: `Serialize` serde default (externally tagged, snake_case — igual que `QueryResult` `graph.rs:12-42`) + shape doc + test / B: solo doc (drift) / C: shape custom camelCase | A (core); exposición a bindings = DEFER → FIND | ✅ decidido-por-evidencia: ítem 31 "exponer AST plano"; hermano `QueryResult` sin renames; `FieldValue`/`SearchProfileConfig` ya `Serialize` |
| 8 | Ejemplo YAML | A: actualizar `QueryRequest.query.example` a la sintaxis v1 (con `==`) y mantener `test_yaml_drifts_fixed` (parsea IQL) / B: dejar el actual | A | ✅ decidido-por-evidencia: backlog OS "actualizar `IQL.md` + ejemplo YAML mismo-PR"; test existente ya exige que el ejemplo parsee |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `parse_statement` sigue siendo puro `IResult` sin panics; keywords UPPERCASE-only (test pin existente); `PROFILE` (MEM-01) sigue parseando en v1; `RELATE --"label"-->` y `INSERT MESSAGE` intactos; escrituras IQL con strings no cambian; no tocar `vantadb-mcp/**`, `vanta-proxy/**`, `src/server/**`, bindings; `docs/api/openapi.yaml` debe seguir parseando (`openapi_yaml_parity`); snapshot `dml_insert_ast` actualizado a propósito (no silenciar con `INSTA_FORCE_PASS`).
- **Comandos de verificación:** `cargo test --target-dir target/session-api01 -p vantadb --lib parser` · `cargo test --target-dir target/session-api01 --test parser` · `cargo test --target-dir target/session-api01 --test openapi_yaml_parity` · `rg IQL_VERSION src/` · `cargo fmt -p vantadb --check` · `cargo clippy --target-dir target/session-api01 -p vantadb --all-targets -- -D warnings`
- **Deuda pendiente:** coerción numérica mixta `(Int, Float)` en `PhysicalFilter`/`compare_field_values` (pre-existente, expuesta por literales Int) → `FIND-157`; exposición del AST JSON en bindings Py/TS → `FIND-156`; `PROFILE bogus` no consume la cláusula (pre-existente, test `:683-691`).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|---------------------------|
| `activeGoal` | Encabezado `# API-06: W5 IQL — versión + sintaxis + literales + AST` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | API-07 (W6 CLI — depende de IQL estable) |

    contract:
      verificacion: cargo test --target-dir target/session-api01 -p vantadb --lib parser → 127 passed/0 failed | cargo test --target-dir target/session-api01 --test parser → 4 passed/0 failed | cargo test --target-dir target/session-api01 --test openapi_yaml_parity → 11 passed/0 failed | rg IQL_VERSION src/ → 14 hits (mod.rs:17,20; gate grammar.rs:120; re-export lib.rs:180) | cargo fmt -p vantadb --check → 0 | cargo clippy -p vantadb --all-targets -- -D warnings → 0 | docs-coverage 0 gaps | campaign_verify_cmd 3/3
      evidencia:
        - claim: "`==` quedaba colgado: parse_rel_op(alt) intentaba `=` primero (grammar.rs:48-49) y el test :170-176 lo confirmaba (pre-fix)"
          evidencia: src/parser/grammar.rs:47-56 + src/parser/mod.rs:170-176 + RED run (5 failed)
          confianza: alta
        - claim: "`42` parseaba como Float(42.0): double precedía a parse_i64 (lexer.rs:138-144); post-fix Int exacto y >2^53 preservado"
          evidencia: src/parser/lexer.rs:128-158 + tests test_parse_literal_int_above_2_53_is_exact / TEST_parser 127/127
          confianza: alta
        - claim: "keywords UPPERCASE-only pinneado (from falla / FROM ok) y single quotes rechazadas — decidido, documentado y testeado"
          evidencia: tests/api/openapi_yaml_parity.rs:90-121 + src/parser/mod.rs::test_single_quoted_strings_are_rejected + docs/api/IQL.md §Lexical Rules
          confianza: alta
        - claim: "IQL_VERSION definido + gateado (PROFILE) + expuesto en crate root; example YAML con `==` parseado y fully-consumed"
          evidencia: src/parser/mod.rs:10-28 + src/parser/grammar.rs:120 + src/lib.rs:180 + tests/api/openapi_yaml_parity.rs::test_query_request_example_is_fully_consumed_by_parser
          confianza: alta
        - claim: "review P2-01 ronda 1 = ❌ cambios requeridos (3 findings docs/artefacto) → R1/R2/R3 aplicados"
          evidencia: docs/dev/tasks/API-06.md §Review + sesión reviewer ses_f244a6713ffeJPNbagKguM51GP
          confianza: alta
      artefactos:
        - docs/dev/tasks/API-06.md
        - docs/api/IQL.md
        - docs/dev/Backlog.md (FIND-156/157)
      invariantes: keywords UPPERCASE-only; PROFILE v1 parsea; openapi parity verde; no bindings/MCP/server; snapshot actualizado a propósito
      deuda: FIND-156 (AST en bindings) + FIND-157 (coerción Int/Float, pre-existente)
      queda_pendiente: commit local del lead + skill progreso (push solo con instrucción del owner)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva. Se paga parcialmente la deuda "AST JSON sin dueño": el core gana la proyección JSON canónica (serde) + shape pinneado por test; la exposición en bindings queda como `FIND-156` con dueño asignable. `FIND-157` documenta la coerción numérica mixta pre-existente (no introducida aquí).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ + `--lib parser` + `--test parser` + `--test openapi_yaml_parity` verdes + fmt/clippy scoped |
| **Commit** | Atómico, conventional `feat!:` + `API-06` (cambia typing de literales IQL), verificación mecánica (`campaign_verify_cmd`); commit = **lead** (esta sesión NO commitea) |
| **Release** | `dev-tools/verify.ps1` en el commit del lead; docs mismo-PR (Regla 3: IQL.md + openapi.yaml); release-plz decide versión/tag (Regla 7) |

## Herramientas necesarias
- Terminal: `cargo` con `--target-dir target/session-api01` (MCP server lockea `target/debug/vanta-cli.exe`), `rg`, `pwsh`
- `codegraph_explore` (blast radius), `campaign_*` (verify/state), `dev-tools/ocr-review.ps1` (advisory)

**Skills cargadas (SDP):** source-driven-development (valida sintaxis/spelling EBNF-like contra código+docs — nom `tag()` y serde) · systematic-debugging (Iron Law: repros `==`/`42`/`from`/`'quote'` ANTES del fix) · test-driven-development (RED→GREEN de los 4 repros + gate) · incremental-implementation (slices por fix, ≤100 líneas/step) · doubt-driven-development (cambia typing de literales → re-check adversarial) · ponytail full (mínimo: rechazo documentado > soporte nuevo). Descartadas: frontend-ui-engineering (no toca `web/`), performance-optimization (cambio de tipo de dato, sin claim de perf → Regla 9 no aplica).

## Investigation Notes
- **Gate P (API-STD-15):** eje IQL = "`IQL_VERSION` + 1 sintaxis documentada + case definido + `==`/int/string fixes + AST JSON", breaking `feat!:`; ítem 31 (API-STD-14): "exponer AST plano → SÍ". W5 verify (API-STD-18): "`cargo test` parser (`mod.rs`) + YAML ejemplo válido".
- **W5 no toca MCP `query_iql`** (prohibido en esta tarea; el path queda intacto).
- **`--target-dir target/session-api01`** ya existe (warm) — nota de entorno API-01/03.
- `tests/api/openapi_yaml_parity.rs:501-517` ya exige que `QueryRequest.query.example` parsee como IQL → es el gate "ejemplo YAML válido".
- `openapi.yaml:149-151` (API-03) ya documenta UPPERCASE-only → consistente con la decisión de case.
- Coerción numérica mixta: `evaluate_condition` (`filter.rs:66-102`) y `compare_field_values` (`join.rs:233-269`) solo comparan misma variante → `Int` estored vs `Float` literal (y viceversa) no matchean; pre-existente (SDK escribe `Int`), enumerado como `FIND-157`.
- `PROFILE` gate: única sintaxis post-v1; el `if iql_supports(...)` en `parse_query` es el mecanismo pedido ("gate PROFILE y futuro").

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — todas resueltas por evidencia en §Spec |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% (8/8 steps ✅ · contrato 4/4) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no toca trust boundaries nuevos: parser puro (sin `unsafe`, sin FFI, sin deps nuevas). La decisión "rechazo explícito" para `'quote'`/lowercase evita semántica ambigua (inyección no aplica: el AST se ejecuta por planner tipado). Sin cambios en manejo de errores de bindings.
- [x] **PERFORMANCE** — no toca hot path más allá de comparaciones existentes; `parse_i64`-first agrega un intento de parseo (nanosegundos, input corto) — sin claim cuantificado → Regla 9 no aplica. `Serialize` derive no se invoca en runtime de queries.

## Steps

### Step 0: Impacto mapeado + lecturas Regla 0
- **Archivos:** todos los de §Impacto mapeado
- **Acción:** leer completos los archivos a tocar; mapear referencias entrantes/salientes; completar la tabla de impacto
- **Verify:** tabla §Impacto mapeado completa + `codegraph_explore` ejecutado
- **Estado:** ✅ DONE 2026-09-25 — lecturas completas (grammar/lexer/mod/query/lib/IQL.md/tests/snap) + secciones relevantes (executor/filter/join/openapi/parity). CodeGraph: `parse_statement` 29 callers; `SelectStatement` 12.

### Step 1: RED — repros `==`, `42→Int`, big-int, `from`, `'quote'`, `IQL_VERSION`, example YAML
- **Archivos:** `src/parser/mod.rs` (tests), `tests/api/openapi_yaml_parity.rs` (no editar: usar), `docs/api/openapi.yaml` (ejemplo con `==`)
- **Acción:** tests nuevos que deben fallar HOY: `parse_rel_op("==")` consume ambos; `parse_condition("edad == 18")` ok; `42` → `Int`; `9007199254740993` exacto; `IQL_VERSION`/`iql_supports` (RED = no compila, símbolo inexistente); example YAML con `==` debe parsear
- **Verify:** `cargo test --target-dir target/session-api01 -p vantadb --lib parser` → fallos/compilación roja esperada (documentar el fallo)
- **Estado:** ✅ DONE 2026-09-25 — RED verificado: 5 tests nuevos fallando (`==` deja `=` colgando; `42`→Float; big-int pierde precisión; condition/query `==` no parsean) + parity `test_query_request_example_is_fully_consumed_by_parser` FAILED con leftover `AND archived == false FETCH title, score RANK BY score DESC`. Baseline pre-edit: 117 passed.

### Step 2: GREEN — lexer: `parse_i64` anti-float + orden Int antes de Float
- **Archivos:** `src/parser/lexer.rs:128-145`
- **Acción:** `parse_i64` con `not(peek('.'|'e'|'E'))`; `parse_literal_field_value` con `parse_i64` antes que `double`; docs del bloque
- **Verify:** `cargo test --target-dir target/session-api01 -p vantadb --lib parser` (los repros de literales pasan)
- **Estado:** ✅ DONE 2026-09-25 — `parse_i64` con `not(peek('.'|'e'|'E'))` + Int antes de Float; repros de literales verdes.

### Step 3: GREEN — grammar: `==` antes que `=`, gate `PROFILE`, default alias SELECT
- **Archivos:** `src/parser/grammar.rs:47-56,116-121,362-363`
- **Acción:** orden longest-match (`==`); `if iql_supports(IQL_VERSION_MIN_PROFILE)` alrededor del `opt(PROFILE…)`; `from_alias` default `"target"`
- **Verify:** `cargo test --target-dir target/session-api01 -p vantadb --lib parser` + `cargo test --target-dir target/session-api01 --test parser`
- **Estado:** ✅ DONE 2026-09-25 — `==` longest-match (`grammar.rs:47-60`); PROFILE gateado (`grammar.rs:120`); SELECT alias default `target` (`grammar.rs:363`). GREEN: `--lib parser` 127/127 + `--test parser` 4/4.

### Step 4: `IQL_VERSION` + gate + re-export + tests
- **Archivos:** `src/parser/mod.rs`, `src/lib.rs:196-198`
- **Acción:** `IQL_VERSION=1`, `IQL_VERSION_MIN_PROFILE=1`, `iql_supports()` const fn; re-export `pub use parser::IQL_VERSION;`; tests de gate
- **Verify:** `rg IQL_VERSION src/` ≥1 · `cargo test --target-dir target/session-api01 -p vantadb --lib parser`
- **Estado:** ✅ DONE 2026-09-25 — 14 hits en `src/`; re-export `lib.rs:180`; `test_iql_version_defined_and_gated` verde.

### Step 5: AST JSON — `Serialize` en AST + test de shape
- **Archivos:** `src/query.rs:10-221`
- **Acción:** `#[derive(Serialize)]` + `use serde::Serialize;` en `Statement`/`Query`/`SelectStatement`/`FromClause`/`JoinClause`/`SubqueryCondition`/`Condition`/`RelOp`/`RankBy`/`Traversal`/DML; test `serde_json::to_value` del shape externo
- **Verify:** `cargo test --target-dir target/session-api01 -p vantadb --lib parser`
- **Estado:** ✅ DONE 2026-09-25 — `Serialize` solo en tipos AST (`LogicalPlan`/`LogicalOperator` revertidos a propósito); `test_ast_json_projection_shape` verde.

### Step 6: actualizar tests existentes + snapshot + integración
- **Archivos:** `src/parser/mod.rs` (expectativas Float→Int, comentarios), `tests/logic/parser.rs:63`, `tests/logic/snapshots/parser__dml_insert_ast.snap`
- **Acción:** actualizar esperados a `Int`; regenerar/reemplazar snapshot a propósito; agregar repros lower/quote
- **Verify:** `cargo test --target-dir target/session-api01 --test parser`
- **Estado:** ✅ DONE 2026-09-25 — expectativas Float→Int actualizadas (`mod.rs`, `tests/logic/parser.rs:63`); snapshot `parser__dml_insert_ast.snap` actualizado a propósito (sin `INSTA_FORCE_PASS`); repros lower/quote pinneados; `--test parser` 4/4.

### Step 7: docs — `IQL.md` + `openapi.yaml`
- **Archivos:** `docs/api/IQL.md`, `docs/api/openapi.yaml:144-164,1792`
- **Acción:** documentar versión+gate, 7 statements (SELECT/JOIN/subquery), defaults idénticos, case, literals/operators (`==`), quotes, AST JSON; actualizar example
- **Verify:** `cargo test --target-dir target/session-api01 --test openapi_yaml_parity` + `scripts/validate-docs-coverage.ps1`
- **Estado:** ✅ DONE 2026-09-25 — IQL.md (Language Version, Lexical Rules, SELECT/JOIN/subquery, `==`, AST JSON) + `openapi.yaml` example con `==` + línea de versión; parity 11/11; docs-coverage 0 gaps. R2 del review: nota type-strict en Lexical Rules.

### Step 8: verify final + cierre (review P2-01, FIND rows, recitation)
- **Archivos:** task file + Backlog (FIND rows)
- **Acción:** fmt/clippy scoped, suite scoped completa, OCR advisory, review `vanta-review`, marcar FIND rows, recitation
- **Verify:** contrato 4/4 + `campaign_verify_cmd`
- **Estado:** ✅ DONE 2026-09-25 — `campaign_verify_cmd` 3/3 (127+, 4/4, 11/11); fmt/clippy verdes; subsets lib query 78/executor 24/physical 51/planner 13 + `--test executor` 1/1 + `--test mutations` 1/1; OCR preview advisory; review ronda 1 ❌ → R1/R2/R3 aplicados; FIND-156/157 en Backlog.

## Dependencias
- API-01 ✅ (tipos base `u128_serde`; sin dependencia de código nueva)
- Paralela con API-04/API-05. Desbloquea: API-07 (W6 CLI `query`).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (subagente, contexto fresco, sesión `ses_f244a6713ffeJPNbagKguM51GP`) — **ronda 1 (2026-09-25): ❌ cambios requeridos** (3 findings, todos de artefacto/docs)
- **Enfoque:** ¿la decisión de typing `Int` + rechazo de `'quote'`/lowercase es correcta para un lenguaje versionado? ¿el gate `iql_supports` es el mecanismo correcto? ¿el shape AST JSON documentado coincide con lo que emite serde? ¿alternativas mejores?
- **Cómo se probó:** evidencia de verificación real (comandos + outputs), no auto-reporte. El revisor hizo spot-check propio: `rg IQL_VERSION` (consts/gate/re-export) + re-ejecutó `--test openapi_yaml_parity` → 11 passed; verificó snapshot/expectativas sin `INSTA_FORCE_PASS` y la fila FIND-157.
- **Checklist anti-hábitos tóxicos** (contrato §12 agent-02-task-execution) — verificado por el revisor:
  - [x] No inventar salidas de comandos que no se ejecutaron → spot-check propio OK.
  - [x] No saltarse la clarificación por "ya sé qué quiere" → decisiones resueltas por evidencia en §Spec (Gate P/plan).
  - [x] No declarar done sin verificar contra acceptance criteria → (finding R1 corregido: estados sincronizados).
  - [x] No ignorar fallos ni reportar "todo OK" con fallo parcial → RED documentado + hallazgos (FIND-157).
  - [x] No reintentar en bucle sin diagnóstico → un solo ciclo RED→GREEN.
  - [x] No gastar presupuesto infinito; paradas explícitas → OOM de suite completa acotado a subsets deterministas.
- **Fixes del review (2026-09-25):**
  - **R1 ✅ (High — bloqueaba cierre):** el task file se auto-declaraba COMPLETED/APPROVE con Steps ⬜ → estados sincronizados con evidencia real; §Review registrado con el dictamen.
  - **R2 ✅ (Medium — bloqueaba merge):** `IQL.md` §Numeric literals documenta la comparación **type-strict** (`Int` vs campo `Float` almacenado → `false` silencioso, incluidos datos pre-v1 IQL) + guía `28` vs `28.0`.
  - **R3 ✅ (Low):** comentario obsoleto de `grammar.rs:82-86` ("bare numbers parse as Float / Float/Float branch") corregido a Int-first.
- **Veredicto ronda 1:** ❌ cambios requeridos — "el contrato pasó 4/4 y el código/approach es aprobable; tras aplicar 1-2, cierre sin más revisiones".
- **Re-verificación (ronda 2):** ✅ **APPROVE** (2026-09-25, misma sesión de review): R1 ✅ (estados sincronizados + §Review completo; header "Fases explícitas" intacto), R2 ✅ (nota type-strict coincide con `filter.rs:66-102`/`join.rs:233-269` y no contradice los docs de operadores), R3 ✅ (comentario describe el comportamiento actual). Nuevos hallazgos: ninguno funcional; nit de naming interno limpiado a FIND-156/157. "Cierre y commit autorizados".
- **Veredicto:** ✅ **APPROVE** (ronda 2)

## Notas
- Estrategia de slice: fixes chicos y aditivos primero (`==`, Int), luego versión/gate, luego AST (aditivo), docs al final. Cada slice compila y testea.
- **No commit**: el dueño/lead commitea localmente (política 2026-09-25); push solo con instrucción explícita.
- `--target-dir target/session-api01` obligatorio (lock de `target/debug/vanta-cli.exe`).
- Riesgos del plan cubiertos: F1 versionar sin migrar PROFILE → `IQL_VERSION_MIN_PROFILE=IQL_VERSION=1` + test de gate; F2 SELECT sin JOIN real → JOIN ya ejecuta (`tests/logic/joins.rs`) y se documenta tal cual; F3 single-quote a medias → decisión de rechazo explícito (no soporte parcial).
- `span`: **FIND-156** (AST JSON en bindings Py/TS — ítem 31, DEFER por prohibición de bindings en API-06) y **FIND-157** (coerción numérica `Int`/`Float` en filtros, pre-existente) registradas en `docs/dev/Backlog.md` durante la ejecución.
- **Progreso 2026-09-25 (vanta-worker):** Steps 0-8 ✅. Entorno: disco completo → `target/session-api01/debug/incremental` liberado (5.2 GB) + `CARGO_INCREMENTAL=0`; suite lib completa crasheó por OOM del entorno (alloc 2.1 MB fallida) → verificación acotada a subsets deterministas (parser/query/executor/physical/planner + targets parser/executor/mutations/openapi_yaml_parity), todos verdes.
- **RED→GREEN (evidencia):** pre-fix 5 repros rojos (`==` colgado, `42`→Float, big-int, condition/query `==`) + parity leftover `AND archived == false FETCH title, score RANK BY score DESC`; post-fix `--lib parser` 127/127, `--test parser` 4/4, parity 11/11.
- **Review P2-01:** ronda 1 ❌ (3 findings: R1 task-file auto-declarado, R2 doc type-strict, R3 comentario obsoleto) → R1/R2/R3 aplicados; el revisor autorizó cierre tras fixes ("tras aplicar 1-2, cierre sin más revisiones").
- **No commit:** política owner 2026-09-25 — el commit local lo hace el lead; push solo con instrucción explícita.
