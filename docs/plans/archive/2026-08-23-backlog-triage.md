# Plan de Ejecución: Correctness Sprint + Agent Exposure (triage Backlog 2026-08-23)

> **Campaign ID:** 82c5ed20-2086-4619-b471-dbafeb63aead
> **Inicio:** 2026-08-23
> **Estado:** ✅ COMPLETED (cierre 2026-08-23 — 16/17 ✅ + 1 SKIP tardío; W1 verificada en código + tests re-ejecutados por el lead)
> **Fuente:** `docs/Backlog.md` (triage Gate P confirmado por owner: set base 16 + REVIEW-09)
> **FAIL_MODE:** parallel · MAX_CONCURRENT=3 · waves por dominio disjunto

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 17 |
| 🟡 DEFER | ~35 (campañas propias: DESKTOP-23..39, MKT/CLD/BLOG pre-launch, BND-04/05+MEM-51 bindings, REVIEW-10/12 god-files, MOD-02/04/05 refactors, PRO/FUT roadmap, DISC-01/02, BIZ-01b, OLD-01, GOV-TK*, TIR follow-ups, LEG-01 humano) |
| ❌ SKIP | 2 |
| 🔴 BLOQUEADO | 4 |

**SKIP:** REVIEW-07 (resuelto por BND-06 `db337b00` — exclusiones scope-safe en `.config/nextest.toml`, verificado hoy) · REVIEW-20 (STALE — `scripts/validate-docs-coverage.ps1` existe y corrió 0 gaps hoy).
**BLOQUEADO:** AUD-042 (upstream tantivy <0.27) · CLD-04 (requiere pilot enterprise real) · BND-07 (acción externa owner: Discord invite + DNS) · DESKTOP-38 (pre-requisito endpoint metrics en `vanta-proxy` no existe).

Status: ⬆️ uphill = 4 incógnitas abiertas (MOD-01 orden exacto validate↔WAL, CORE-02 root cause WASM, MOD-17 semántica GIL/detach, MCP-31 exposición del compresor) · ⬇️ downhill = 13 tareas con steps definidos.

---

## Wave 1 — Bugs críticos (🔴 correctness)

### Task 1: MOD-01 — WAL escrito ANTES de validar duplicado (resucita datos tras restart)

- **Appetite:** max 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴
- **Archivos clave:** `src/engine.rs:226,150-157,260`, `src/storage/engine/insert.rs`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — símbolos existen (`insert` @ insert.rs:33, `WalWriter.append`, blast radius 173 callers vía codegraph); orden exacto validate→WAL por confirmar contra reporte `docs/reviews/modulos/core.md` H-1
- **Gate Justificación:** bug de corrupción vía API pública (insert/update rechazado resucita datos tras restart), expuesto en WASM standalone; fix acotado al orden write-path
- **Contrato:** test nuevo: insert/update RECHAZADO (duplicado inválido) → reopen → registro AUSENTE (el WAL no contiene la op); `cargo nextest run -p vantadb` verde + suite durabilidad
- **Pre-mortem:** (1) el fix asume un solo write path pero hay batch_append/batch paths paralelos que esquivan la validación; (2) mover validación antes del WAL cambia semántica de upsert legítimo; (3) regresión en throughput del hot path (medir si el bench lo detecta — Regla 9 si se alega perf)
- **Stop conditions:** appetite 1d excedido → abortar y re-triar como campaña propia; rabbit hole >2 iteraciones sin test rojo→verde
- **Cynefin:** 🟨 complicado — requiere trazar los 3 write paths (put/batch/bulk) para validar el orden único
- **Task file:** `.opencode/skills/campaign-executor/tasks/MOD-01.md` · **Estado:** ✅ COMPLETED (`18fd2c80` — validate→WAL→apply; RED reproducido; nextest 2049/2049 + audit 2718/2718)

### Task 2: MOD-07 — Notifications JSON-RPC sin `id` rechazadas como -32700
- **Appetite:** max 2h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-mcp/src/protocol.rs:8-14`, `vantadb-mcp/src/server.rs:102`
- **Verificación real:** ✅ CÓDIGO-REAL — `RpcRequest.id: Value` campo requerido verificado hoy (`protocol.rs:11`); notificación sin id falla deserialize → -32700 espurio
- **Gate Justificación:** rompe handshake con clientes MCP estrictos; fix ~10 líneas (`#[serde(default)]` + Option) + test
- **Contrato:** test: request sin `id` (notification) → NO produce error -32700 (se procesa/silencia); `cargo nextest run -p vantadb-mcp` verde
- **Pre-mortem:** respuesta a notification está prohibida por spec — asegurar que el fix no empiece a responderlas
- **Task file:** `tasks/MOD-07.md` · **Estado:** ✅ COMPLETED (`4cb3abec` — nextest 37/37, contrato verificado en disco)

### Task 3: MOD-12 — `ensure_indexes_current` ausente en path HTTP (text search rota DB fresca)

- **Appetite:** max 2h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴
- **Archivos clave:** `src/cli_server.rs:1758` (arranque server HTTP), `src/sdk/builder.rs:35-42`
- **Verificación real:** ✅ CÓDIGO-REAL — `rg ensure_indexes_current src/cli_server.rs` = 0 matches (hoy); el fix MCP-01 cubrió solo el path stdio
- **Gate Justificación:** misma clase de bug que MCP-01 (resuelto), canal HTTP queda roto para búsqueda textual/híbrida en DB fresca; fix 1-3 líneas + e2e
- **Contrato:** e2e HTTP: put con texto → search textual vía `/api/v2/*` en DB fresca devuelve hits SIN rebuild manual; rg ≥1 match en cli_server.rs
- **Pre-mortem:** el server HTTP puede abrir el engine por otro constructor que ya garantice índices — verificar antes de duplicar la llamada
- **Task file:** `tasks/MOD-12.md` · **Estado:** ✅ COMPLETED (`5623e41f` — RED reproducido en HTTP real, e2e 12/12, grep=1)

### Task 4: MOD-16 — Suite pytest default rota (66 failed: DBs sin cerrar acumulan RSS)

- **Appetite:** max 4h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/tests/test_async_smoke.py:35`, conftest fixture autouse
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — reporte `docs/reviews/modulos/python.md` H1 de hoy; reproducir `pytest -q` antes de tocar
- **Gate Justificación:** la suite canónica del binding Python no ejecuta — bloquea CI/confianza de cada cambio PyO3
- **Contrato:** `pytest -q` exit 0 en vantadb-python (suite default completa); fixture autouse cierra todas las DBs abiertas
- **Pre-mortem:** algunas failures pueden ser bugs reales (no solo leaks) — si >3 failures persisten post-fixture, escalar hallazgos como tareas nuevas y no maquillar
- **Task file:** `tasks/MOD-16.md` · **Estado:** ✅ COMPLETED (`deefc919` — fixture autouse; verify pytest pendiente de re-corrida del orquestador)

### Task 5: MOD-17 — Deadlock potencial `OpGate::drain()` espera condvar sosteniendo GIL en `close()`

- **Appetite:** max 4h · **Esfuerzo:** 🟡 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/lib.rs:1711-1717,132-139`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — reporte python.md H2 de hoy; confirmar patrón condvar+GIL leyendo el código antes del fix
- **Gate Justificación:** deadlock potencial en cierre desde threads Python (crash colgado del intérprete); fix acotado (drain dentro de `py.detach`) + test estrés
- **Contrato:** test estrés concurrente (N threads usando la DB + close() simultáneo) termina sin hang; `pytest -q` verde
- **Pre-mortem:** `py.detach` durante interpret shutdown puede no estar permitido — verificar ciclo de vida PyO3; el fix puede requerir reordenar drop
- **Cynefin:** 🟨 complicado — semántica GIL/PyO3 drop ordering
- **Task file:** `tasks/MOD-17.md` · **Estado:** ✅ COMPLETED (`50319e30` — drain en py.detach; RED real vía stash; pytest -q 111 passed)

### Task 6: CORE-02 — Bug IQL transporte WASM: query lee graph-store vacío en modo standalone

- **Appetite:** max 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠
- **Archivos clave:** `vantadb-wasm/src/lib.rs`, `desktop/src/vanta-wasm-map.ts`, init graph-store (`src/graph.rs`)
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — hallazgo análisis desktop 2026-08-22 (`vanta-wasm-map.ts`); reproducir en modo OPFS antes de tocar
- **Gate Justificación:** bloquea confiabilidad del modo standalone del Studio (F4); nativo y HTTP funcionan — gap acotado al init WASM
- **Contrato:** test wasm roundtrip: INSERT edge (RELATE) → query IQL devuelve el edge en modo standalone OPFS
- **Pre-mortem:** (1) root cause puede estar en persistencia OPFS (no en init) → alcance crece; (2) graph-store puede ser in-memory por diseño en wasm → decisión de producto, no bug
- **Stop conditions:** si el fix requiere rediseñar persistencia OPFS → abortar, registrar ADR y re-triar
- **Cynefin:** 🟧 complejo — causa-efecto emerge al experimentar (probe-sense-respond)
- **Task file:** `tasks/CORE-02.md` · **Estado:** ✅ COMPLETED (`3a8bf366` — H1: nodos grafo fuera del snapshot OPFS; fix graph_state.json + collect/restore SDK; wasm roundtrip test; nextest 2712/2712)

### Task 7: REVIEW-09 — Bug lógico cache_warmer: latch `saturated` monotónico mata el aprendizaje

- **Appetite:** max 2h · **Esfuerzo:** 🟢 · **Prioridad:** 🟠
- **Archivos clave:** `src/cache_warmer.rs:143,197`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — derivada review-full-20260822 H09-CODE-001; confirmar latch+decay leyendo el código
- **Gate Justificación:** warming degrada silenciosamente en servers long-running (decisión owner: incluir); fix acotado 1 archivo
- **Contrato:** test: ciclos decay que reducen tabla → latch se resetea cuando post-decay total < max_pairs → vuelve a aprender pares
- **Pre-mortem:** la condición de reset puede oscilar (reset thrashing) — elegir histéresis simple
- **Task file:** `tasks/REVIEW-09.md` · **Estado:** ✅ COMPLETED (`8b8924b3` — TDD RED→GREEN, P2-01 approve, nextest audit 2714/2714)

---

## Wave 2 — Quick wins (🟢 dominios disjuntos, paralelizables)

### Task 8: REVIEW-08 — h2 0.4.15 RUSTSEC-2026-0258 → `cargo deny check advisories` FALLA

- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟡
- **Archivos clave:** `Cargo.lock`
- **Verificación real:** ✅ CÓDIGO-REAL — h2 `0.4.15` en Cargo.lock verificado hoy
- **Gate Justificación:** bloquea deny gate antes del próximo release; fix = `cargo update -p h2`
- **Contrato:** `cargo deny check advisories` exit 0; Cargo.lock con h2 ≥0.4.16 commiteado
- **Pre-mortem:** bump puede arrastrar breaking de dependientes — si `cargo check` falla, documentar pin y re-triar
- **Task file:** — · **Estado:** ✅ COMPLETED (`ff9b2933` inline — h2 0.4.18, deny advisories ok)

### Task 9: REVIEW-19 — CHANGELOG.md stub en raíz

- **Appetite:** max 15min · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
- **Archivos clave:** `CHANGELOG.md` (nuevo)
- **Verificación real:** ✅ CÓDIGO-REAL — Test-Path CHANGELOG.md = False hoy
- **Gate Justificación:** tooling externo espera changelog en raíz; stub 1 línea apuntando a `docs/CHANGELOG.md` (regla: NUNCA editar changelog manual — release-plz)
- **Contrato:** Test-Path CHANGELOG.md = True y contenido referencia docs/CHANGELOG.md
- **Task file:** — · **Estado:** ✅ COMPLETED (inline — stub CHANGELOG.md raíz)

### Task 10: REVIEW-16 — 5 imports muertos en `debug_ops.rs`

- **Appetite:** max 15min · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
- **Archivos clave:** `src/sdk/search/debug_ops.rs:2`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — derivada review L2-CODE-001; `cargo fix --lib -p vantadb` los detecta
- **Gate Justificación:** warnings en cada build maturin/cargo; fix mecánico
- **Contrato:** `cargo clippy -p vantadb --no-deps` sin warnings de unused imports en debug_ops.rs
- **Task file:** — · **Estado:** ❌ SKIP (imports muertos ya eliminados colateralmente; cargo check 0 warnings en debug_ops.rs)

### Task 11: REVIEW-14 — Panics frágiles: unwrap en `version_history.rs:283` (key <8 bytes) + unwraps `explain.rs`

- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
- **Archivos clave:** `src/sdk/version_history.rs:283`, `src/sdk/search/explain.rs:103,147,183`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — derivada review H09-CODE-005; confirmar unwraps en disco
- **Gate Justificación:** panic con store corrupto viola contract de errores (`VantaError::Corrupt`); fix acotado
- **Contrato:** test: key <8 bytes → `VantaError::Corrupt` (sin panic); explain.rs sin unwrap frágil (bindear Some en pattern)
- **Task file:** `tasks/REVIEW-14.md` · **Estado:** ✅ COMPLETED (`4044a588` — helper version_from_key + explain sin unwraps; nextest 2051/2051)

### Task 12: REVIEW-15 — Cast `from_raw_parts` a f32 sin assert de alineación (`vector_data.rs:167`)

- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
- **Archivos clave:** `src/node/vector_data.rs:167`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — derivada review H07-UNSAFE-002
- **Gate Justificación:** invariante page-aligned implícito sin evidencia; UB potencial documentable con `align_to()` seguro
- **Contrato:** código usa `align_to()` o `debug_assert!(ptr % 4 == 0 && len % 4 == 0)`; suite storage verde
- **Pre-mortem:** cambiar a `align_to()` puede revelar desalineación real en producción — preferir assert explícito primero
- **Task file:** `tasks/REVIEW-15.md` · **Estado:** ✅ COMPLETED (`57090e0e` — align_to + guard; −1 unsafe; P2-01 approve)

### Task 13: REVIEW-18 — Warning Next.js package-lock stray (turbopack root)

- **Appetite:** max 30min · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
- **Archivos clave:** `web/next.config.ts`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — derivada review H03-CODE-001; reproducir warning en build web
- **Gate Justificación:** warning de build confunde CI/dev; config 1 línea
- **Contrato:** `npm run build` en web/ sin warning turbopack/package-lock
- **Task file:** `tasks/REVIEW-18.md` · **Estado:** ✅ COMPLETED (`6ea5e545` — turbopack root; build 35/35 limpio)

### Task 14: MOD-03 — `trigger_compaction()` es stub que solo loguea

- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟡
- **Archivos clave:** `src/storage/engine/maintenance.rs:22-48`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — reporte core.md M-1; confirmar stub y callers en disco
- **Gate Justificación:** API pública que promete compaction y no hace nada; renombrar/delegar a vacuum existente (decisión en DISCOVERY)
- **Contrato:** sin método público que solo loguee: o delega a vacuum real (test bytes reclaim) o renombrado deprecated con doc
- **Pre-mortem:** callers existentes pueden depender del no-op — grep callers antes
- **Task file:** `tasks/MOD-03.md` · **Estado:** ✅ COMPLETED (`28ce0a57` — delega a merge_segments; RED confirmado; nextest 2052/2052)

---

## Wave 3 — Exposición agéntica MCP (diferenciadores; SECuenciales entre sí — mismos archivos)

> Las 3 tareas tocan `vantadb-mcp/src/handlers/tools.rs` + `SKILL.md` ×2 → ejecutar EN SERIE dentro de la wave. Absorben MOD-10 (versions/supersede/similar_to_key/vacuum/remove_edge quedan como candidatos a añadir en estas mismas PRs si el contexto lo permite — no bloquean el contrato).

### Task 15: MCP-31 — Context engine vía MCP: tool `context_assemble`

- **Appetite:** max 1d · **Esfuerzo:** 🟠 · **Prioridad:** 🔴
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `vanta-memory/src/context_engine/engine.rs` (handlers gateway `assemble_with_recall`)
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — handlers existen como funciones puras sobre `&VantaEmbedded` (nota P26); confirmar firma
- **Gate Justificación:** LA funcionalidad distintiva del memory OS hoy inaccesible para agentes externos (solo desktop IPC MEM-58); wrapper directo bajo riesgo
- **Contrato:** tool `context_assemble(session_key, token_budget, query?)` en tools/list; test round-trip con sesión seedada (vanta-seed) devuelve contexto ≤ budget; SKILL.md ×2 hash SAME
- **Pre-mortem:** (1) exposición del compresor MMD puede filtrar internals — decidir solo assemble v1; (2) vanta-memory puede no linkear en el binario server — verificar deps del crate mcp
- **Stop conditions:** si requiere refactor de vanta-memory (threads/wasm) → abortar y re-triar como BND-04
- **Cynefin:** 🟨 complicado · **Top 3 riesgos:** acoplamiento compresor, deps crate, shape del resultado para agentes
- **Risk Register:** | 🟡×🟠 deps de vanta-memory en binario MCP | verificar Cargo.toml del server primero | DISCOVERY |
  | 🟡×🟡 shape no determinista del contexto | schema JSON-RPC con campos estables | diseño tool |
- **Task file:** `tasks/MCP-31.md` · **Estado:** ✅ COMPLETED (`4d752f14` — tool context_assemble + context_tests.rs; SKILL.md hash SAME verificado)

### Task 16: MCP-30 — Scenes API vía MCP (`scene_read`/`scene_list`/`scene_query`)

- **Depende de:** Task 15 (misma wave, serial) · **Appetite:** max 4h · **Esfuerzo:** 🟢 · **Prioridad:** 🔴
- **Archivos clave:** ídem + `vanta-memory/src/gateway/knowledge_handlers.rs`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — handlers puros existen (nota P26)
- **Gate Justificación:** navegación semántica por escenas accesible solo desde desktop/proxy hoy; wrappers directos
- **Contrato:** 3 tools en tools/list; test round-trip seed sesión → scene_list > 0 → scene_read por id; SKILL.md ×2
- **Task file:** `tasks/MCP-30.md` · **Estado:** ✅ COMPLETED (`d03b6517` — scenes.rs + 7 round-trips; nextest mcp 51/51; docs ×2 hash SAME)

### Task 17: MCP-32 — Threads CRUD vía MCP (historial conversacional persistente)

- **Depende de:** Task 16 (misma wave, serial) · **Appetite:** max 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `src/agentic/thread.rs:89-203`
- **Verificación real:** 🟡 VERIFICAR-IN-DISCOVERY — API completa existe en SDK (`create_thread/send_message/get_thread/list_threads/delete_thread/purge_expired_threads`)
- **Gate Justificación:** hoy solo `inject_context` expuesto; CRUD completo habilita historial conversacional gestionable por el agente; wrappers sin cambio de SDK
- **Contrato:** tools `thread_create/thread_send/thread_get/thread_list/thread_delete` (+purge si trivial); test round-trip create→send→get→list→delete; SKILL.md ×2
- **Pre-mortem:** semántica de sesiones/namespace de threads puede chocar con multi-tenant RBAC del proxy — revisar D34 antes de exponer writes
- **Task file:** `tasks/MCP-32.md` · **Estado:** ✅ COMPLETED (implementada por el lead inline — SARL step 3; threads.rs + 7 tests; nextest mcp 58/58; docs hash SAME)

---

## Protocolo

Igual que campañas previas: skills base al inicio, MCP tools por paso, C0 PLAN→ACT→VERIFY, Question Gates D/V/C, verify mecánico (`campaign_verify_cmd`) antes de cada commit, commit Conventional con task ID, SARL ante sub-agente incompleto. Waves: W1 (bugs, paralelo máx 3) → W2 (quick wins, paralelo) → W3 (MCP, serial). Al cierre: retrospectiva + skill progreso + archive.

## Notas de triaje

- Fuente de evidencia Paso 0: checks mecánicos 2026-08-23 (rg/Test-Path/codegraph/Cargo.lock) + reportes `docs/reviews/modulos/*.md` (P32, 2026-08-23) + review-full-20260822.
- Los DEFER agrupados por campaña futura sugerida: **Desktop Polish** (DESKTOP-23..37+39), **Pre-Launch Marketing** (MKT-04/18f/g/h/i, CLD-01/02, BLOG-CTA — MKT-18g es la primera), **Bindings vanta-memory** (BND-04, MEM-51, BND-05), **Core Refactors** (MOD-02 txn atomicity, MOD-04 TTL index con bench Regla 9, MOD-05 InMemoryEngine, REVIEW-10/12 god-file splits, REVIEW-13), **Docs/Prompt hygiene** (TIR follow-ups a/b/c, GOV-TK3/5/7/8/9).

=== RECITATION MOD-07 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MOD-07: aceptar notifications JSON-RPC sin id — handshake clientes MCP estrictos
Estado: completed
Última acción: Fix implementado TDD: RED reprodujo el -32700 espurio exacto; GREEN con #[serde(default, deserialize_with=keep_explicit_null)] id: Option<Value> + routing notification-silenciosa en serve_lines (extraída para testability con duplex pipes); verify full verde; commit 4cb3abec
Resultado: OK
Próxima acción: Ninguna para MOD-07. Orquestador continúa con MOD-12 (text search roto en path HTTP, cli_server.rs:1758)
Contrato: verificacion: cargo fmt --check ✅ | cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings ✅ | cargo nextest run -p vantadb-mcp = 37/37 ✅ || evidencia || claim: notification sin id NO produce -32700 (contrato) → test server::tests::notification_without_id_is_not_answered PASS; antes del fix falló reproduciendo el bug exacto {"error":{"code":-32700}} | confianza: alta || claim: requests con id y JSON malformado conservan comportamiento → tests request_with_id_still_answered + malformed_json_still_parse_error + explicit_null_id_is_a_request_not_a_notification PASS | confianza: alta || claim: sin regresiones → nextest 37/37 PASS | confianza: alta || artefactos: vantadb-mcp/src/protocol.rs, vantadb-mcp/src/server.rs, .opencode/skills/campaign-executor/tasks/MOD-07.md, commit 4cb3abec || invariantes: NUNCA emitir respuesta a mensaje sin id (JSON-RPC 2.0 §4.1); -32700 solo para JSON no-deserializable; "id": null explícito sigue siendo request respondible; notificaciones desconocidas nunca llegan a handlers || deuda: ninguna (nota: EnteredSpan no-Send en dispatch_request es issue latente preexistente, candidato a Backlog, no introducido por este PR) || queda_pendiente: orquestador ejecuta MOD-12 (próxima Wave 1); verify full workspace al cierre del sprint
Próxima tarea si completa: MOD-12
=== END RECITATION ===

=== RECITATION MOD-12 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MOD-12: ensure_indexes_current en arranque del server HTTP — text search funcional en DB fresca
Estado: completed
Última acción: Fix implementado (guard read_only + state.db.ensure_indexes_current() en run() ~1793), helper e2e espeja producción, test de regresión agregado. RED (404) verificado pre-fix, GREEN post-fix. Verify full crates tocados verde. Commit 5623e41f.
Resultado: OK
Próxima acción: Ninguna para MOD-12. Orquestador ejecuta MOD-16 (suite pytest default rota)
Contrato: verificacion: rg -c "ensure_indexes_current" src/cli_server.rs = 1 >= 1 ✅ | cargo fmt --check ✅ | cargo clippy -p vantadb -p vantadb-server --all-targets -D warnings ✅ | cargo test -p vantadb-server --test e2e = 12/12 (incl. test_e2e_text_search_fresh_db) ✅ | nextest -p vantadb-server 5/5 ✅ | nextest -p vantadb --features server cli_server 49/49 ✅ || evidencia || claim: el path HTTP no garantizaba indices → RED verificado: sin fix, search textual por HTTP devuelve 404 (NotFound text_index bm25) con put 201 previo; confianza alta || claim: fix restaura text search en DB fresca → GREEN test_e2e_text_search_fresh_db (put texto → /api/v2/search → hits>0 sin rebuild); confianza alta || claim: sin regresiones → e2e 12/12 + cli_server 49/49 + server 5/5 PASS; confianza alta || artefactos: src/cli_server.rs, vantadb-server/tests/e2e.rs, vantadb-server/tests/helpers/mod.rs, .opencode/skills/campaign-executor/tasks/MOD-12.md, commit 5623e41f || invariantes: semántica de ensure_indexes_current intacta; guard read_only (deployments read-only no ejecutan ensure); fallo de ensure loggeado NO aborta arranque (twin MCP-01); sin unwrap nuevos en prod || deuda: ninguna || queda_pendiente: orquestador continúa con MOD-16; el test de regresión vive en binario e2e excluido del perfil default nextest (heavy certification lo corre)
Próxima tarea si completa: MOD-16
=== END RECITATION ===

=== RECITATION MOD-16 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MOD-16: suite pytest default verde con fixture autouse que cierra todas las DBs
Estado: in-progress
Última acción: DISCOVERY completo: repro 66 failed/43 passed clasificado por archivo (test_perf 8 + test_sdk 58; test_load envenena con cero close()); root cause = guard mide RSS proceso vs memory_limit por-DB 128MB; claim '70 passed' confirmado como corrida parcial; PyO3 no gc-visible ni weakref-able; close() idempotente. Task file creado con Regla 0.
Resultado: PARTIAL
Próxima acción: Step 1: crear tests/conftest.py (registry + fixture autouse)
Contrato: verificacion: pytest -q en vantadb-python/tests exit 0 | evidencia | claim: 66 failed reproducidos (perf 8 + sdk 58), test_sdk.py solo = 70 passed (claim histórico parcial confirmado) | confianza: alta || claim: instancias PyO3 no visibles en gc ni weakref → registry por wrapper de constructor | confianza: alta || claim: close() idempotente verificado | confianza: alta
Próxima tarea si completa: MOD-17
=== END RECITATION ===

=== RECITATION CORE-02 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: CORE-02: graph-store visible para query IQL en modo standalone WASM (roundtrip persistencia OPFS/IDB)
Estado: completed
Última acción: Fix completo: VantaEdgeRecord += reverse/created_at_ms (serde default), collect_graph_nodes + restore_graph_nodes en SDK, wiring graph_state.json en save/load ×opfs/idb/worker. Contrato wasm verde (wasm-pack --node). Verify full: fmt OK, clippy workspace 0 warnings, nextest audit 2712/2712. Commit 3a8bf366.
Resultado: OK
Próxima acción: Ninguna para CORE-02. Orquestador continúa con REVIEW-09 (cache_warmer latch saturado)
Contrato: verificacion: cargo nextest run --profile audit --workspace --build-jobs 2 = 2712/2712 PASS | cargo fmt --check OK | cargo clippy --workspace --all-targets --all-features exit 0 | wasm-pack test --node: core02_graph_persist_tests::graph_roundtrip_through_snapshot_payload ok || evidencia || claim: root cause = snapshot OPFS solo exporta memory records, nodos de grafo sin FIELD_NAMESPACE invisibles → reopen deja grafo vacío | evidencia: persist_payload lib.rs:661 + collect_all_deduped list_namespaces/list | confianza: alta || claim: fix restaura el grafo — insert edge → query IQL FROM lo devuelve in-session Y cross-snapshot en engine WASM | evidencia: test bindgen core02_graph_persist_tests::graph_roundtrip_through_snapshot_payload ok (wasm-pack --node) + tests nativos sdk::api::tests::test_core02_* 2/2 | confianza: alta || claim: sin regresiones | evidencia: nextest audit workspace 2712/2712; fallas wasm-pack restantes (11) pre-existentes en develop verificado por stash-run | confianza: alta || artefactos: src/sdk/api.rs, src/sdk/serialization/graph_types.rs, vantadb-wasm/src/lib.rs, tests/proptest_serialization_roundtrip.rs, tests/sdk_serialization.rs, docs/api/EMBEDDED_SDK.md, task file CORE-02.md, commit 3a8bf366 || invariantes: formato db_state.json NO cambia (archivo lateral graph_state.json); snapshots viejos restauran sin grafo; edge.reverse sobrevive restore; labels re-internadas al restaurar; sin unwraps nuevos en prod || deuda: ninguna introducida || queda_pendiente: orquestador registra FIND-CORE02a (11 unit tests lib.rs wasm fallan bajo node, pre-existentes) y FIND-CORE02b (VantaFields difícil desde JS puro por enums tagged) en Backlog; desbloquear vanta_query en vanta-wasm-map.ts es trabajo UI/wire separado
Próxima tarea si completa: REVIEW-09
=== END RECITATION ===

=== RECITATION REVIEW-09 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: REVIEW-09: resetea latch saturated tras decay — warming vuelve a aprender pares en long-running
Estado: completed
Última acción: Tarea completa: RED confirmado (bug reproducido), fix de 3 líneas en decay() + docs honestos, 2 tests nuevos, verify full verde (2714/2714), review P2-01 con hallazgo corregido, commits 8b8924b3 + 4dabcbff.
Resultado: OK
Próxima acción: Ninguna para REVIEW-09. Orquestador continúa con Wave 2: REVIEW-08 (h2 0.4.15 RUSTSEC-2026-0258, cargo update -p h2)
Contrato: verificacion: cargo fmt --check ✅ | cargo clippy --workspace --all-targets --all-features -- -D warnings ✅ | cargo nextest run --profile audit --workspace --build-jobs 2 = 2714/2714 PASS | cargo nextest run -p vantadb cache_warmer = 11/11 PASS (re-verificado post-review) || evidencia || claim: latch set-once era la causa raíz — record_co_access:143-145 satura y decay():189-201 reducía pair_count sin tocar saturated → evidencia: RED pre-fix, tests fallando en :426/:468 con el bug exacto | confianza: alta || claim: fix restaura el ciclo de aprendizaje — saturar → decay bajo cap → latch reseteado → aprende pares nuevos → re-satura al cruzar umbral; sin thrash: decays con post-total ≥ cap mantienen latch (una transición por cruce) → evidencia: test_decay_below_cap_resets_saturation_and_learning_resumes + test_no_thrash_latch_persists_while_post_decay_total_at_cap, ambos GREEN; reviewer independiente reprodujo 11/11 | confianza: alta || claim: bound AUDIT-04 intacto y sin regresiones → evidencia: nextest audit workspace 2714/2714 + clippy -D warnings | confianza: alta || claim: review P2-01 por agente distinto aprobó tras corregir hallazgo 🔴 (doc del campo saturated :49-50 seguía diciendo monotonic) → evidencia: vanta-review ses_fcf48fec6ffeAuUiOahukhpCnV, veredicto approve condicional cumplido | confianza: alta || artefactos: src/cache_warmer.rs, .opencode/skills/campaign-executor/tasks/REVIEW-09.md, commits 8b8924b3 + 4dabcbff || invariantes: bound de memoria intacto (satura al cruzar max_pairs); una transición de latch por cruce de umbral (reset solo dentro de decay, ≤1×/1000 eventos); clear() intacto; sin unwraps nuevos; sin API pública cambiada (CacheWarmer pub(crate)) || deuda: ninguna introducida || queda_pendiente: orquestador continúa Wave 2 (REVIEW-08 h2 RUSTSEC); hallazgo 🟡 del review (derivar saturación de pair_count en vez de AtomicBool) como candidato a Backlog; campaign_verify_cmd MCP inusable esta sesión por planes ambiguos — verify full corrido vía bash directo
Próxima tarea si completa: REVIEW-08
=== END RECITATION ===

=== RECITATION MOD-17 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MOD-17: drain del OpGate fuera del GIL — close() concurrente sin deadlock del intérprete
Estado: completed
Última acción: Tarea completa: fix de 2 líneas (drain dentro de py.detach + derive Clone) con TDD honesto. El primer RED era un bug del propio test (factory llamada eager en listcomp ejecutaba workers en main thread); corregido y re-probado RED verdadero vía stash del fix: watchdog faulthandler capturó el bloqueo mutuo real. Commit 50319e30.
Resultado: OK
Próxima acción: Ninguna para MOD-17. Orquestador continúa Wave 1 restante / Wave 2 según plan
Contrato: verificacion: pytest tests/test_close_concurrency.py RED(buggy: watchdog mata @30s con traceback closer-en-drain + worker-esperando-GIL)→GREEN 5/5 estable | pytest -q completo = 111 passed exit 0 (88s) | cargo fmt --check ✅ | cargo clippy --workspace --all-targets --all-features -D warnings ✅ | cargo nextest --profile audit --workspace --build-jobs 2 = 2714/2714 ✅ | validate-docs-coverage.ps1 (pwsh) = 0 gaps ✅ | commit 50319e30 || evidencia || claim: causa raíz = drain() espera condvar con GIL tomado mientras workers in-flight re-adquieren GIL para salir de py.detach → evidencia: traceback faulthandler contra binario buggy muestra closer clavado en close() (:65) y worker en put() (:30), bloqueo mutuo real con threads reales | confianza: alta || claim: fix elimina el deadlock sin cambiar semántica observable → evidencia: GREEN 5/5 + test sanity closing-rechaza-ops-nuevas + suite completa 111/111 | confianza: alta || artefactos: vantadb-python/src/lib.rs, vantadb-python/tests/test_close_concurrency.py, .opencode/skills/campaign-executor/tasks/MOD-17.md, commit 50319e30 || invariantes: semántica de durabilidad intacta (closing rechaza ops nuevas, espera in-flight, engine.close después); drain sigue siendo método Rust puro usable sin token Python; sin unwrap/unsafe nuevos; Regla 8 no disparada (std::sync only, sin dashmap/parking_lot/tokio) || deuda: ninguna introducida (Regla 6: saldo negativo — elimina un deadlock) || queda_pendiente: orquestador — (1) suite pytest era dependiente de espacio en disco: 78 failed por os error 112 StorageFull con disco en 0.3GB libres; limpiados ~143GB de %TEMP% → 111 passed; considerar guard de disco en CI local; (2) drift autogenerado en completions/_vanta-cli* (subcomando tui) apareció durante builds, NO commiteado — rutear como FIND; (3) campaign_verify_cmd MCP ambiguo con 2 planes activos (schema no acepta planFile) — verify full corrió directo en terminal
Próxima tarea si completa: REVIEW-09 ya completada; siguiente pendiente del plan según orquestador (MOD-01 o Wave 2)
=== END RECITATION ===

=== RECITATION MOD-01 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MOD-01: validar antes de escribir WAL en InMemoryEngine — insert/update rechazado no resucita datos tras restart
Estado: completed
Última acción: Tarea completa TDD: RED reprodujo la resurrección exacta (impostor pisa original; update recrea eliminado); fix reordenó validate→WAL→apply bajo write-lock único en insert/update/delete; verify full verde (2049/2049 + 2718/2718 audit); commit 18fd2c80
Resultado: OK
Próxima acción: Ninguna para MOD-01. Orquestador continúa según plan (Wave 1 restante / Wave 2)
Contrato: verificacion: cargo fmt --check ✅ | cargo clippy --workspace --all-targets --all-features -D warnings ✅ | cargo nextest run -p vantadb = 2049/2049 ✅ | nextest --profile audit --workspace --build-jobs 2 = 2718/2718 ✅ | validate-docs-coverage.ps1 = 0 gaps ✅ | commit 18fd2c80 || evidencia || claim: RED pre-fix — insert duplicado rechazado resucita impostor [9.9,9.9] sobre original [1.0,2.0] tras reopen; update rechazado sobre nodo eliminado lo recrea via replay | evidencia: fallos test_mod01_* capturados antes del fix (left:[9.9,9.9] right:[1.0,2.0]) | confianza: alta || claim: fix cierra el bug — validate→WAL→apply bajo sección crítica única nodes.write() en los 3 mutadores; GREEN 4/4 mod01 + suite completa verde | evidencia: commit 18fd2c80, nextest 2049/2049 y audit workspace 2718/2718 | confianza: alta || claim: flujo legítimo intacto — insert→update→reopen conserva versión actualizada; delete persiste; reject-delete inerte | evidencia: test_mod01_legitimate_insert_update_survives_reopen + test_mod01_delete_persists_and_rejected_delete_is_inert_after_reopen PASS | confianza: alta || claim: StorageEngine/batch/bulk/txn NO violan el invariante — upsert por diseño, guards previos al WAL, buffer txn solo commitea validado | evidencia: src/storage/engine/insert.rs:33-342 leído completo + review core.md N-1 | confianza: alta || artefactos: src/engine.rs, .opencode/skills/campaign-executor/tasks/MOD-01.md, commit 18fd2c80 || invariantes: formato/semántica del WAL sin cambios (solo orden); durabilidad-antes-de-visibilidad para ops VALIDADAS; sin unwrap/unsafe nuevos; API pública sin cambios || deuda: ninguna introducida — preexistentes documentadas en task file: N-1 deprecar InMemoryEngine (~850L), replay sigue aplicando como upsert incondicional (defense-in-depth para WALs ya-corrompidos = cambio semántico aparte) || queda_pendiente: orquestador registra candidatos FIND (deprecación InMemoryEngine N-1) si lo desea; skill progreso ejecutada al cierre
Próxima tarea si completa: Definir por orquestador (MOD-17 ya completada)
=== END RECITATION ===

=== RECITATION REVIEW-18 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: REVIEW-18: warning turbopack package-lock stray eliminado en build de web/
Estado: completed
Última acción: Tarea completa: warning reproducido en DISCOVERY; fix turbopack.root=__dirname en web/next.config.ts; build limpio exit 0 35/35 paginas; commits 6ea5e545 (fix) + c5ef4dfe (docs); lesson escrita
Resultado: OK
Próxima acción: Ninguna para REVIEW-18. Orquestador continúa Wave 2 (REVIEW-08 h2 RUSTSEC o según prioridad)
Contrato: verificacion: npm run build en web/ = tail SIN linea ⚠ Warning, EXIT_CODE=0, 35/35 paginas || evidencia || claim: warning reproducido con config actual → output capturado en task file §DISCOVERY (ignored package-lock.json in C:\Users\Eros) | confianza: alta || claim: fix elimina el warning sin romper build → build post-fix limpio exit 0, output standalone intacto | evidencia: bash EXIT_CODE=0 | confianza: alta || artefactos: web/next.config.ts, task file REVIEW-18.md, docs/Backlog.md (-1 fila), docs/avance/activo/web-frontend.md (+REVIEW-18), commits 6ea5e545 + c5ef4dfe || invariantes: output standalone intacto; lockfile stray fuera del repo NO tocado || deuda: ninguna introducida || queda_pendiente: campaign_verify_cmd MCP inusable por planes ambiguos (schema sin planFile) — verify directo en terminal (precedente MOD-17); fila Backlog de REVIEW-15 sigue presente — orquestador decide
Próxima tarea si completa: REVIEW-08
=== END RECITATION ===

=== RECITATION REVIEW-14 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: REVIEW-14: keys cortas en version_history devuelven error de corrupción sin panic; unwraps frágiles de explain.rs eliminados
Estado: completed
Última acción: Tarea completa: helper version_from_key() (BackendError si key <8B) + validación en versions() + test nuevo RED→GREEN; explain.rs reestructurado bindeando Some(query_sparse) en patterns (3 unwraps eliminados, has_sparse removido); verify full verde 2051/2051; commit 4044a588; FIND-27 registrada para siblings mod.rs/debug_ops.rs
Resultado: OK
Próxima acción: Ninguna para REVIEW-14. Orquestador continúa Wave 2 según plan (REVIEW-08 h2 RUSTSEC o REVIEW-15 vector_data align)
Contrato: verificacion: cargo fmt --check exit 0 ✅ | cargo clippy -p vantadb --all-targets --all-features -D warnings ✅ | cargo nextest run -p vantadb = 2051/2051 PASS (baseline 2050 +1 nuevo) ✅ | rg unwrap explain.rs = 0 matches ✅ | commit 4044a588 || evidencia || claim: key <8 bytes devuelve BackendError tipado sin panic → test short_versions_partition_key_returns_corruption_error_not_panic PASS (loop len 0..8 + boundary 8B) | confianza: alta || claim: explain.rs sin unwraps frágiles, semántica idéntica arm-por-arm (Keyword nullifica Option local no solo bool) → grep 0 matches + suite 2051/2051 verde | confianza: alta || claim: site :283 del review era test-only sin path prod que parsee keys → reclasificado; validación añadida a versions() como defensa real ante store corrupto | evidencia: grep workspace len()-8 = único match en tests + lectura completa version_history.rs | confianza: alta || artefactos: src/sdk/version_history.rs, src/sdk/search/explain.rs, .opencode/skills/campaign-executor/tasks/REVIEW-14.md, docs/Backlog.md (+FIND-27), commit 4044a588 || invariantes: sin cambios API pública (versions() solo cambia ante keys truncadas imposibles en stores sanos); sin unwraps nuevos en prod; sin variante nueva en VantaError (protege matches exhaustivos bindings P2-6) || deuda: ninguna introducida || queda_pendiente: FIND-27 en Backlog (unwraps hermanos mod.rs/debug_ops.rs — mismo fix mecánico, tocar search hot path); desviación de mensaje de commit documentada (no existe variante Corrupt, se usó BackendError)
Próxima tarea si completa: REVIEW-08
=== END RECITATION ===

=== RECITATION 14 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MOD-03: trigger_compaction() deja de ser stub — delega a merge_segments()
Estado: completed
Última acción: Delegación implementada (maintenance.rs trigger_compaction→merge_segments), test disk-backed bytes-reclaim RED→GREEN, glosario sync, verify full verde, commit 28ce0a57 con mensaje pedido.
Resultado: OK
Próxima acción: Ninguna — tarea completa. Orquestador toma próxima task del plan (Wave 3 MCP o REVIEW-15).
Contrato: verificacion: cargo nextest run -p vantadb → 2052/2052 ✅ + cargo clippy -p vantadb --all-targets --all-features -- -D warnings ✅ (exit 0) + cargo fmt --check ✅ + pre-commit hooks ✅; evidencia: {claim: stub solo logueaba, evidencia: RED test before=67108864 after=67108864 contra stub, confianza: alta}, {claim: delegación reclama bytes reales, evidencia: GREEN test shrink+survivor legible (commit 28ce0a57), confianza: alta}, {claim: cero impacto producción, evidencia: rg trigger_compaction → solo 5 tests internos, sin exposición SDK/bindings/MCP, confianza: alta}; artefactos: commit 28ce0a57, task file .opencode/skills/campaign-executor/tasks/MOD-03.md; invariantes: firma pública Result<()> preservada, threshold configurado no hardcodeado, árbol HEAD consistente (fix cli_tests.rs NO incluido); deuda: FIND candidato — merge_segments/vacuum no ven tombstones huérfanos post-delete (delete.rs:165 remove_hnsw_entry) — decisión Engine/Arch; queda_pendiente: orquestador decide fila FIND-* para blind-spot de contadores
Próxima tarea si completa: REVIEW-15 o siguiente del plan (decisión del orquestador)
=== END RECITATION ===

=== RECITATION MCP-31 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MCP-31: tool MCP context_assemble exponiendo el context engine de vanta-memory
Estado: completed
Última acción: Tarea completa: modulo context.rs (wrapper thin assemble_with_recall + perform_auto_recall), 7 tests RED->GREEN, docs x7 archivos sincronizados, verify full verde, commit 4d752f14
Resultado: OK
Próxima acción: Orquestador: skill progreso y lanzar MCP-30 (misma wave W3, serial — mismos archivos tools.rs/SKILL.md)
Contrato: verificacion: cargo fmt --check exit 0 ✅ | cargo clippy -p vantadb-mcp --all-targets --all-features exit 0 ✅ | cargo nextest run -p vantadb-mcp = 44/44 PASS (37 previos + 7 nuevos) ✅ | scripts/validate-docs-coverage.ps1 exit 0 (0 gaps, fix inline allow_insecure) ✅ | commit 4d752f14 con pre-commit hooks ok || evidencia: [claim: tool context_assemble aparece en tools/list con schema JSON-RPC valido, evidencia: test_tools_list_registers_context_assemble PASS, confianza: alta], [claim: round-trip sesion seedada devuelve contexto <= token_budget, evidencia: test_context_assemble_seeded_persona_roundtrip_within_budget PASS (seed persona via import_seed vanta-seed, estimator re-verifica <= 4000), confianza: alta], [claim: query surfacea L1 memories relevantes, evidencia: test_context_assemble_query_surfaces_l1_memories PASS (<relevant-memories> + contenido), confianza: alta], [claim: SKILL.md hash SAME en ambos lados, evidencia: Get-FileHash skills/ vs .opencode/ True (tambien api-reference y mcp-protocol), confianza: alta], [claim: sin regresiones, evidencia: nextest -p vantadb-mcp 44/44, confianza: alta] || artefactos: vantadb-mcp/src/context.rs, vantadb-mcp/tests/context_tests.rs, vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/src/lib.rs, skills/vantadb-mcp/{SKILL.md,references/*} x2 copias, docs/api/MCP.md, docs/operations/CONFIGURATION.md, task file MCP-31.md, commit 4d752f14 || invariantes: compresor MMD NO expuesto (solo assemble); errores de dominio como error_content nunca protocol errors; wire shape = tipos serde existentes de vanta-memory (IntegratedContext); read-only; sin deps nuevas || deuda: ninguna introducida; colateral documentado: docs/api/MCP.md incluye seccion Getting Started ajena (inseparable del worktree compartido, anotado en body del commit) || queda_pendiente: orquestador ejecuta skill progreso y decide siguiente tarea de W3 serial (MCP-30)
Próxima tarea si completa: MCP-30
=== END RECITATION ===

=== RECITATION FIND-08 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: FIND-08: corregir instalación TestPyPI en examples/demo/README.md e indexar los 10 ejemplos huérfanos (6 Python + 4 Rust) en README raíz
Estado: completed
Última acción: Editado examples/demo/README.md (instalación vía PyPI real con pip install 'vantadb-py>=0.5.0', link a pypi.org); agregada sección 'Examples' al README raíz con tablas Python (11 filas) y Rust (4 filas), cada entrada verificada con Test-Path y descripción extraída de la docstring real del archivo
Resultado: OK
Próxima acción: Orquestador: commit de examples/demo/README.md + README.md (docs: FIND-08)
Contrato: verificacion: rg -i 'test.pypi' examples/demo/README.md = 0 matches ✅; Test-Path de los 10 archivos = True ✅; rg -c 'examples/' README.md = 23 vs baseline 7 ✅ | evidencia: [1] claim: 0 refs a test.pypi en demo README (antes: 2), evidencia: rg exit=1, confianza: alta; [2] claim: los 10 ejemplos listados existen, evidencia: Test-Path → 10x True, confianza: alta; [3] claim: referencias examples/ en README raíz 7→23, evidencia: rg -c = 23, confianza: alta | artefactos: examples/demo/README.md, README.md | invariantes: no se tocó contenido de los .py/.rs — solo indexados | deuda: ninguna | queda_pendiente: commit por parte del orquestador (instrucción explícita del usuario)
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION MKT-18g ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MKT-18g — Corregir claims técnicos falsos en docs/strategy/SHOW_HN_PREP.md y REDDIT_POSTS.md
Estado: completed
Última acción: Verifiqué cada claim técnico contra código real y corregí los falsos en ambos archivos manteniendo el tono del copy
Resultado: OK
Próxima acción: ninguna — tarea completa; commit pendiente para el orquestador (solo los 2 archivos de docs/strategy/)
Contrato: artefactos: docs/strategy/SHOW_HN_PREP.md, docs/strategy/REDDIT_POSTS.md | evidencia: verificación mecánica rg contra Cargo.toml (croaring/wide/fjall/rocksdb/rayon/failpoints, sin cpufeatures), src/hardware/mod.rs:236-246 (is_x86_feature_detected + NEON), src/cost_estimator.rs:22 (PREFILTER_THRESHOLD=0.01), benches/hnsw_recall_ef.rs:21-23 (dataset SINTÉTICO 10K×128d — no SIFT), tests/storage/chaos_integrity.rs (failpoints wal_append/storage_insert/mmap_flush/hnsw_serialize), vantadb-python/src/lib.rs:1473-1495 (search_batch GIL-released + rayon par_iter), docs/operations/BENCHMARKS.md §1/§7 | invariantes: tono/estrategia del copy intacto; Regla 11 (todo número con fuente reproducible) | deuda: benchmark SIFT1M sin correr [TO VERIFY marcado en ambos archivos]; fila MKT-18g de docs/Backlog.md sin tocar (worktree compartido con otra tarea activa)
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION MCP-30 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: MCP-30: tools scene_read/scene_list/scene_query wrapper directo de knowledge_handlers de vanta-memory
Estado: completed
Última acción: Implementado scenes.rs (3 wrappers thin + schemas), dispatch en tools.rs, 7 tests RED→GREEN. Verify full verde (fmt/clippy/nextest 51/51/docs parity 0 gaps). Docs SKILL.md x2 hash SAME + api-reference x2 + mcp-protocol x2 + MCP.md 60 tools. Commit d03b6517.
Resultado: OK
Próxima acción: Orquestador: skill progreso y lanzar MCP-32 (misma wave W3, serial)
Contrato: verificacion: cargo fmt --check exit 0 ✅ | cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings exit 0 ✅ | cargo nextest run -p vantadb-mcp = 51/51 PASS (44 previos + 7 nuevos) ✅ | scripts/validate-docs-coverage.ps1 exit 0 (0 gaps) ✅ | commit d03b6517 con pre-commit hooks ok || evidencia: [claim: 3 tools registradas en tools/list con schema valido, evidencia: test_tools_list_registers_scene_tools_with_valid_schemas PASS, confianza: alta], [claim: round-trip sesion seedada scene_list > 0 y scene_read por id devuelve contenido, evidencia: scene_list_roundtrip_lists_seeded_scenes_heat_desc + scene_read_roundtrip_returns_block_content PASS (seed via upsert_scene publica, sin L0 ni LLM), confianza: alta], [claim: scene_query keyword matchea y el hit resuelve via scene_read, evidencia: scene_query_finds_scene_by_keyword_and_reads_it PASS, confianza: alta], [claim: errores de dominio como error_content nunca protocol errors, evidencia: scene_read_missing_scene_is_error_content_not_protocol_error + reject_empty_session_and_keyword PASS, confianza: alta], [claim: SKILL.md hash SAME ambos lados, evidencia: Get-FileHash skills/ vs .opencode/ = SAME x3 archivos (SKILL/api-reference/mcp-protocol), confianza: alta] || artefactos: vantadb-mcp/src/scenes.rs, vantadb-mcp/tests/scene_tests.rs, vantadb-mcp/src/handlers/tools.rs (+4L), vantadb-mcp/src/lib.rs, skills/vantadb-mcp/* x2 copias, docs/api/MCP.md, task file MCP-30.md, commit d03b6517 || invariantes: read-only; wire shape = tipos serde existentes de vanta-memory (id de navegacion = filename); errores de dominio como error_content; sin deps nuevas; embed=None en scene_query (keyword-only D38) || deuda: ninguna introducida || queda_pendiente: orquestador ejecuta skill progreso y decide siguiente tarea wave W3 serial (MCP-32 threads CRUD)
Próxima tarea si completa: MCP-32
=== END RECITATION ===

---

## Retrospectiva de cierre (Start / Stop / Continue)

**Start:** verificación post-hoc del lead contra código+tests (atrapó que los RESULTADO vacíos no significaban trabajo ausente) · bloque "Skills obligatorias" explícito en cada delegación (SKILLS_CARGADAS en RESULTADO) · implementación inline por el lead cuando la delegación falla 3× (SARL step 3).

**Stop:** confiar en el canal de respuesta de sub-agentes (~40% vacíos en esta sesión; el trabajo estaba intacto al menos una vez) · lanzar de a UN sub-agente por mensaje cuando las tareas son disjuntas (desperdicia paralelismo; MAX_CONCURRENT=3 es el límite real) · asumir que el matcher MCP `campaign_load_skills` sugiere las skills críticas (devuelve lista mínima).

**Continue:** waves por dominio disjunto con commits por tarea · claim:true + guard R5 para multi-instancia · docs ×2 hash SAME en cada cambio de skill MCP.

**Acción medida:** tasa first-try de delegaciones = 9/13 launches útiles (~70%, bajo el baseline North Star >90%) — causa raíz canal de reporte, no capacidad; métrica a vigilar: resultados vacíos/launch. Follow-ups registrados como FIND: sitios hermanos align_to (ivf.rs/mapper.rs/bytes.rs), FIND-27 unwraps mod.rs/debug_ops, drift completions/_vanta-cli, fila MOD-16 residual en Backlog.

=== RECITATION FIND-01 ===
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: FIND-01: metadata plano en la API TS (vantadb-ts)
Estado: completed
Última acción: DISCOVERY confirmó que WASM y nativo exigen forma tagged; implementé normalización a tagged en capa TS (helper src/metadata.ts), tipos union VantaFlatValue/VantaMetadataInput, aplicado en put/putBatch/importRecords/list/search/deleteByFilter/exportNamespace/insertNode (WASM) y put/putBatch/list/search (nativo); README+JSDoc+examples migrados a forma plana; test nuevo flat-metadata.test.ts.
Resultado: OK
Próxima acción: Orquestador decide commit del changeset
Contrato: verificacion: cd vantadb-ts && npm test (253 passed) OK; npx tsc --noEmit OK; rg '\{ String' vantadb-ts/README.md = 0; rg 'type: "String"' src/vantadb.ts = 0. evidencia: tests flat-metadata.test.ts (7) + suite completa verde. artefactos: vantadb-ts/src/metadata.ts, src/types.ts, src/vantadb.ts, src/native.ts, src/__tests__/flat-metadata.test.ts, README.md, examples/langchain/index.ts, examples/llamaindex/index.ts. invariantes: records leidos del engine siguen devolviendo metadata tagged (Map en runtime WASM); forma tagged sigue aceptada (backward compat). deuda: tipo VantaMetadata=Record miente en runtime WASM (Map real) — preexistente, fuera de scope FIND-01. queda_pendiente: commit (usuario pidió NO commitear).
Próxima tarea si completa: siguiente tarea del backlog
=== END RECITATION ===
