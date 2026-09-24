# D1c — Adelgazar `records_list` (~103) moviendo paginación a caso de uso 🟡

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: lógica/frontera (handler HTTP gordo → caso de uso). Subagente: `vanta-worker`.
- Scope: `src/server/handlers.rs:390-492` (actual 405-507: `records_list` + `merge_all_namespaces_pages` 367-379 + `clamp_limit` 385-390).
- Contexto verificado: `records_list` hace fan-out namespaces (sort nombre asc) + `db.list` por ns + `merge_all_namespaces_pages` + slice `start/end/window/next_cursor` inline en handler; patrón humilde ya establecido por B1 en `src/server/conversation.rs` (`StartConversationUseCase` + trait puerto + handler humilde DTO→uso→respuesta).
- OJO D5c: `clamp_limit` en :385-390 (def) y usos :434 (`records_list`) y :547-549 (`records_search`) — preservarlo tal cual, sin mover ni renombrar.
- Baseline: `/cleanCA src/server/handlers.rs` (A2 🟡/F1: handler no humilde, función >20 líneas).
- Blast radius: `records_list` ← `router.rs:172` (`GET /api/v2/list`); callees `run_db_op`, `Embedded::namespace_stats`, `Embedded::list`, `merge_all_namespaces_pages`; <10 archivos, sin símbolos públicos nuevos salvo `ListRecordsUseCase` interno → Gate D = GO.
- Skills: campaign-executor, progreso, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, planning-and-task-breakdown, codebase-memory, security-and-hardening (input HTTP: `filter_ops` JSON, `namespace/limit/cursor`).

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia:
  - Nuevo módulo `src/server/list_records.rs` (patrón B1): struct `ListRecordsCommand { namespace: Option<String>, filter_ops: Option<MemoryFilter>, limit: usize, cursor: Option<usize> }` + trait puerto `ListRecordsPorts { list_namespaces_sorted, list_in }` + `ListRecordsUseCase::execute` (fan-out + sort + merge + slice movidos del handler) + `ServerListPorts<'a> { db: &Embedded }` prod + `AllNamespacesListPage` movida o re-exportada para wire.
  - Handler `records_list` queda humilde: parse `ListParams` → `filter_ops` JSON (400 si inválido) → `clamp_limit` → DTO → `run_db_op` con `ListRecordsUseCase::execute` → respuesta. `merge_all_namespaces_pages` se mueve al caso de uso (o queda como helper privado del caso de uso, NO duplicada).
  - Tests AAA/FIRST del caso de uso en memoria sin HTTP (FakePorts con namespaces fijos, paginación, truncación).
- NO cambia:
  - Wire HTTP idéntico: `{records, next_cursor}` single-ns y `{records, next_cursor, truncated_namespaces}` all-ns; `next_cursor = (end < len).then_some(end)`, `start = cursor.unwrap_or(0).min(len)`, `end = (start+limit).min(len)`.
  - `clamp_limit` definición y sus 3 usos (records_list :434, records_search :547/:549) — tal cual.
  - `conversation_add` (B1) intacto; `records_search` intacto salvo clamp ya existente.
  - Orden estable namespaces (nombre asc) y detección truncación (`page.next_cursor.is_some()`).
- Archivos exactos:
  - `src/server/list_records.rs` (nuevo).
  - `src/server/mod.rs` (nuevo `pub mod list_records;`).
  - `src/server/handlers.rs` (solo `records_list` + mover `merge_all_namespaces_pages`; `clamp_limit` y `records_search` intocados).
  - `docs/dev/tasks/D1c.md` (este file).
- Verify:
  - `cargo fmt --check`
  - `cargo check -p vantadb --tests --features server`
  - `cargo nextest run --profile audit -p vantadb --features server list_records` (nuevos) + HTTP existentes (`rbac_namespace`, `list_window` si aplica)
  - `cargo clippy -p vantadb --features server --all-targets -- -D warnings`

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [x] Slice 1: caso de uso + tests en memoria (RED→GREEN, focado primero): `ListRecordsCommand` + `ListRecordsPorts` + `ListRecordsUseCase::execute` + `merge_all_namespaces_pages` movida + `FakePorts` (namespaces ordenados, multi-página, truncación, cursor/limit bordes, filter_ops passthrough); `cargo nextest run -p vantadb --features server list_records` verde → CÓDIGO OK, ejecución bloqueada por colateral D1b (maintenance.rs E0624, ver BLOQUEO).
- [x] Slice 2: handler delgado (DTO→uso→respuesta): `records_list` delega a `ListRecordsUseCase` vía `run_db_op` + `ServerListPorts`; preserva parse `filter_ops`→400, `clamp_limit`, wire single/all; handler 103→82 líneas.
- [x] Slice 3: borrar código muerto del handler (`merge_all_namespaces_pages` vieja eliminada; `rg` = 1 definición en caso de uso); `rustfmt --check` archivos D1c verde; `cargo clippy -D warnings + check` bloqueados por colateral (cero errores en archivos D1c).
- [x] Cierre: verify completo del contrato por el lead (colateral D1b resuelto y commiteado edf28979): nextest list_records/rbac_namespace/list_window 10 passed, clippy server -D warnings verde, fmt verde + RESULTADO ✅.

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO: ✅/🟡/❌ + STEPS_OK + PROXIMO_STEP + COMMIT_HASH (ninguno — commitea el lead) + ARCHIVOS + VERIFY_CONTRATO + BLOQUEO + GATES_EVALUADOS + SKILLS_CARGADAS (≥10).
- Recitation: objetivo, estado, última acción, próximo paso, contrato+resultado, invariantes, deuda.
