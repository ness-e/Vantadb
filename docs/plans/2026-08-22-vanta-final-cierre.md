# Plan de Ejecución: Vanta Cierre Final — integración, recall semántico y gobierno de decisiones

> **Inicio:** 2026-08-22
> **Estado:** ⏳ EN PROGRESO (5/8 tareas)
> **Fuente:** auditoría final post-P30 (vanta-research `ses_fd8c2c26`, 2026-08-22) + decisiones del usuario (2026-08-21/22) + deudas vigentes de task files
> **Predecesores:** P27 F1-F4 ✅ 24/24 · P29 F5 ✅ 9/9 · P30 F6+F7 ✅ 9/9 — **roadmap TDAM F1-F7 cerrado**, suites 2568+ tests
> **Modo:** waves — Wave 0 (integraciones y tests independientes) → Wave 1 (embeddings fundación) → Wave 2 (semantic recall + scoring) → Wave 3 (gobierno humano + meta-tarea).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 8 |
| 🟡 DEFER | 1 (MEM-36 → su meta-tarea ES Task 8 de este plan; la campaña bindings se crea desde acá) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

**Objetivo:** cerrar el port TDAM al 100% funcional — cablear el context engine productivamente, probar el roundtrip cross-crate del wiki, portar auto-sync, pagar la deuda #1 (recall semántico con embeddings), consumir scores reales en compresión, y articular humanamente las decisiones (Regla 5).

**Decisiones fijadas por este plan (no re-debatir):**
- **D38:** recall/dedup/query usan **vector similarity cuando el record tenga vector**; fallback keyword-overlap para records sin vector — nunca se rompe un record legacy.
- **D39:** fuente de embeddings = lo que el core ya exponga. **Paso 0 obligatorio de Task 4:** verificar si existe auto-embedding real (COMP-010 del SYNTHESIS mapping NO está verificado contra código). Si no existe → trait `EmbeddingProvider` opcional en vanta-memory con implementación host, sin deps nuevas.
- **D40:** MEM-45 auto-sync = port del scheduler programado (decisión usuario), sobre el run_id/throttle de MEM-31.
- **D41:** ADR-029 y las decisiones D24-D37 requieren **articulación humana** (Regla 5) — Task 7 es human-in-loop: la IA prepara el material, el autor escribe.
- **Principios heredados vigentes:** P4 LLM/embedding opcional · sanitización · D19 · sin unwrap/expect · errores #[non_exhaustive] · verify mecánico del lead por tarea · SARL completo.

Status: ⬆️ uphill = 1 (existencia de auto-embedding en core — Task 4 Paso 0 decide) · ⬇️ downhill = ~28 steps estimados

---

## Tasks

### Task 1: MEM-43 — Wire context engine → pipeline worker
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `vanta-memory/src/services/pipeline_worker.rs` (editar), `context_engine/engine.rs` (facade si falta)
- **Verificación real:** ✅ AUDITORÍA — cero referencias a context_engine en services/ (solo tests e2e_flow.rs); decisión usuario: wire productivo
- **Gate Justificación:** convierte la killer feature F5 en productiva dentro del ciclo L0→L1→L2→L3
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: el worker ejecuta assemble_with_recall como fase post-L3 (compresión del historial + inyección MMD + recall con budget compartido); e2e extendido demuestra compresión activa dentro del pass completo"
- **Pre-mortem:** (1) doble compresión (worker + caller externo) → el worker es UNO de los callers; API existente intacta; (2) compresión automática puede sorprender → config flag `context_compression_enabled` default true documentado
- **Stop conditions:** si el wiring exige reescribir assemble → ⬛ y escalar diseño
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | compresión dispara costos ocultos | es LLM-free; flag config + telemetría genlog | diseño |
  | 🟢×🟡 | orden de fases ambiguo | test e2e assertiona orden L0→L1→L2→L3→compress→recall | primer test |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-43.md`
- **Notas:** Ruta: vanta-worker. Auditoría: ses_fd8c2c26 hallazgo C3.

### Task 2: MEM-44 — E2e ingest→tools wiki_* roundtrip
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** test cross-crate nuevo (ubicar según dónde vivan las primitivas MCP — probablemente `vantadb-mcp/tests/` consumiendo el crate core)
- **Verificación real:** ✅ AUDITORÍA — ambos extremos verificados por separado contra el MISMO WikiStore, sin test encadenado
- **Gate Justificación:** prueba la integración cruzada completa: .md temporales → ingest worker → wiki_search/wiki_read vía handlers MCP
- **Gate Result:** ✅ DO
- **Contrato:** "test único: fixture .md temporales → worker::run (con runner fake o fallback P4) → wiki_search encuentra términos de los archivos → wiki_read devuelve el contenido mergeado → wiki_graph conecta; todo verde"
- **Pre-mortem:** (1) ingest vive en vanta-memory pero tools en vantadb-mcp → el test usa el core como punto común o depende de ambos crates según grafo de deps; verificar dirección válida antes
- **Stop conditions:** si la dirección de dependencia impide el test único → 2 tests hermanos compartiendo fixture spec documentada
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | dirección de deps inválida (ciclo) | verificar cargo tree primero; plan B: 2 tests hermanos | DISCOVERY |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-44.md`
- **Notas:** Ruta: vanta-worker.

### Task 3: MEM-45 — Auto-sync scheduler (re-ingest programado del wiki)
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/ingest/auto_sync.rs` (crear) + wiring ManagedTimer (MEM-16)
- **Verificación real:** ✅ AUDITORÍA — hueco de triaje confirmado (research 08 §2 auto-sync-scheduler.ts); decisión usuario: portar
- **Gate Justificación:** los .md locales cambian; re-ingest manual-only exige que el usuario recuerde
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19 con FakeClock (MEM-16): (a) intervalo configurable dispara re-ingest del wiki; (b) respeta busy guard (no re-ingesta si pending/processing — 409 MEM-28); (c) disabled by default; (d) usa run_id fresco por build (paquetes tardíos descartados — MEM-31)"
- **Pre-mortem:** (1) timer thread vs ManagedTimer pull-based → reusar ManagedTimer/Clock de MEM-16 (ya resuelto, cero threads); (2) detección de cambios → mtime/hash del directorio (hash simple por archivo, cap)
- **Stop conditions:** si la detección de cambios exige watcher de FS (deps nuevas) → hash periódico simple y documentar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | re-ingest storm si el intervalo es corto | min interval clamp + busy guard | test b |
  | 🟢×🟢 | disabled by default olvidado | test c | primer test |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-45.md`
- **Notas:** Ruta: vanta-worker. TDAM ref: `MemoryKnowledge/src/store/auto-sync-scheduler.ts`.

### Task 4: MEM-46 — Embeddings para records L1 (fundación recall semántico)
- **Appetite:** max 2d
- **Esfuerzo:** 🔴 | **Prioridad:** 🔴 (deuda #1)
- **Archivos clave:** DISCOVERY decide: `vanta-memory/src/core/record/l1_writer.rs` (editar) + posible facade en core o trait `EmbeddingProvider`
- **Verificación real:** 🟡 VERIFICAR — COMP-010 auto-embedding del SYNTHESIS mapping NO verificado contra código; **Paso 0 obligatorio:** codegraph_explore "auto embedding vector generation put text" + leer `src/vector/` y sdk search; decidir rama:
  - **Rama A:** core ya auto-embeddea texto en put → L1 records YA tienen vector al escribir; Task 4 = verificar + facade de búsqueda vectorial multi-namespace para vanta-memory
  - **Rama B:** core NO auto-embeddea → trait `EmbeddingProvider` opcional (host implementa; sin deps nuevas); L1 writer almacena vector cuando hay provider
- **Gate Justificación:** deuda #1 vigente (MEM-11/18/21); decisión usuario: atacar ahora
- **Gate Result:** ✅ DO (condicionado a Paso 0)
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) records escritos con provider activo llevan vector consultable; (b) sin provider → record válido sin vector (P4); (c) búsqueda vectorial multi-namespace `l1/*` devuelve por similitud"
- **Pre-mortem:** (1) embeddings requieren modelo externo → provider trait host-implementado, nunca hardcodear API; (2) dimensión inconsistente entre providers → validar dimensión al configurar
- **Stop conditions:** Paso 0 revela que ni Rama A ni B son viables sin deps pesadas → ⬛ y escalar al usuario con opciones
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🔴×🔴 | COMP-010 inexistente (SYNTHESIS mapping erróneo) | Rama B: provider trait propio — ya prevista | Paso 0 |
  | 🟡×🟠 | embedder lento bloquea writes | async/best-effort: write sin vector + backfill | benchmarks |
- **Cynefin:** 🟧 complejo — el comportamiento emerge al probar; steps cortos
- **Uphill/Downhill:** ⬆️ 1 (Paso 0) · ⬇️ 4 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-46.md`
- **Notas:** Ruta: vanta-worker (Rama B) o vanta-engine (si Rama A toca core vector). Decidir ruta tras Paso 0.

### Task 5: MEM-47 — Semantic recall end-to-end (swap overlap→vector + fallback)
- **Appetite:** max 2d
- **Esfuerzo:** 🔴 | **Prioridad:** 🔴
- **Archivos clave:** `core/hooks/auto_recall.rs`, `core/record/{l1_reader,l1_dedup}.rs`, `gateway/knowledge_handlers.rs` (editar los 3 — mismo swap)
- **Verificación real:** ✅ AUDITORÍA — deuda vigente en MEM-11:38, MEM-18:27, auto_recall.rs:492-496; depende de Task 4
- **Gate Justificación:** paga la deuda #1: paráfrasis y cross-idioma en recall/dedup/query
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: (a) record CON vector matchea por similitud semántica (paráfrasis sin keywords comunes); (b) record SIN vector cae a keyword-overlap (D38 fallback); (c) dedup usa similitud para candidatos; (d) knowledge_handlers query usa vector; (e) RecallScope sigue respetándose en modo vector"
- **Pre-mortem:** (1) mezcla vector+keyword ranking inconsistente → normalizar scores o fusionar RRF (ya existe en core); (2) performance HNSW sobre namespaces múltiples → medir; search_multi ya existe
- **Stop conditions:** appetite excedido → entregar recall-only (dedup/query a tarea propia ⬛)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🔴 | records mixtos (con/sin vector) rankean injusto | dos pools rankeados + merge RRF; documentado | test b |
  | 🟡×🟡 | regresión en tests keyword existentes | fallback D38 preserva comportamiento | suite completa |
- **Cynefin:** 🟧 complejo
- **Uphill/Downhill:** ⬆️ 0 (post Task 4) · ⬇️ 4 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-47.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 4.

### Task 6: MEM-48 — Compresión consume scores L1 reales
- **Appetite:** max 1d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡
- **Archivos clave:** `vanta-memory/src/context_engine/compressor.rs` (editar)
- **Verificación real:** ✅ AUDITORÍA — scoring heurístico vigente (MEM-22:72); upgrade "consumir scores L1" no ocurrió
- **Gate Justificación:** el cascade score heurístico (ToolResult=6>...) era placeholder mientras no existieran memories puntuadas; ahora existen con priority 0-100
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vanta-memory` pasa; tests D19: mensajes vinculados a memories con priority alta sobreviven la cascada antes que los de priority baja; sin memories vinculadas → heurístico actual como fallback"
- **Pre-mortem:** join mensaje↔memoria vía source_message_ids (ya existe en MemoryRecord) — el score de un mensaje = max(priority de sus memories)
- **Stop conditions:** si el join resulta O(n²) → índice message_id→score precomputado
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟢×🟡 | join O(n²) | índice HashMap precomputado | diseño |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-48.md`
- **Notas:** Ruta: vanta-worker. Independiente — cualquier wave.

### Task 7: MEM-49 — ADR-029 articulación humana + gate D24-D37 (human-in-loop)
- **Appetite:** max 1d (humano) / 🟢 prep IA
- **Esfuerzo:** 🟢 (prep IA) | **Prioridad:** 🟠 (gobierno)
- **Archivos clave:** `docs/architecture/adr/ADR-029-*` (el AUTOR edita), `docs/architecture/adr/ADR-0XX-proxy-knowledge.md` (borrador nuevo)
- **Verificación real:** ✅ REAL — Regla 5: forcing function del autor humano; ADR-029 en borrador desde P29; D24-D37 de P30 sin ADR
- **Gate Justificación:** las decisiones arquitectónicas no están cerradas hasta que el autor las articule con sus palabras
- **Gate Result:** ✅ DO
- **Contrato:** "Tarea HUMANA: (1) IA genera documento-guía con cada decisión + evidencia + preguntas socráticas (NO redacta la decisión final); (2) el AUTOR edita ADR-029 con sus palabras y aprueba; (3) IA transcribe a ADR nuevo el racional del proxy/knowledge una vez articulado. Gate: commit del autor con las decisiones en primera persona"
- **Pre-mortem:** la IA redacta por el humano → pierde la función (Regla 5 explícito) — IA solo aporta datos y estructura
- **Stop conditions:** si el autor no dispone tiempo → tarea queda abierta honestamente (no marcar completed)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | tarea humana sin dueño de tiempo | agendar con el usuario explícitamente | inicio de la tarea |
- **Cynefin:** 🟦 obvio (proceso)
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-49.md`
- **Notas:** Ruta: vanta-docs prepara guía → AUTOR HUMANO edita → lead commitea. Única tarea del plan que requiere al usuario en el loop.

### Task 8: MEM-36 (meta) — Crear plan campaña Bindings SDK
- **Appetite:** max ½d
- **Espec:** COMPLETA en `.opencode/skills/campaign-executor/tasks/MEM-36.md` (creada 2026-08-22 — leéla: contexto, superficies, referencias TDAM, riesgos VS-CORE-05, decisiones heredadas y las que el plan debe cerrar)
- **Entregable:** `docs/plans/<FECHA>-vantadb-bindings-sdk.md` listo para `/pipeline run`
- **Gate Result:** ✅ DO
- **Contrato:** "Existe el plan de bindings con ≥4 tareas ✅ DO, contratos mecánicos (`cargo check -p vantadb-wasm` + wasm-pack build + tsc + pytest) y task files referenciados; commit hecho"
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟢×🟡 | plan genérico sin superficie real | Step 1 obligatorio: listar métodos públicos hoy | DISCOVERY |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 (spec completa) · ⬇️ 2 steps
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-36.md` (YA EXISTE)
- **Notas:** Ruta: vanta-lead (planning). Independiente — cualquier wave.

---

## Checkpoints

| # | Después de | Verificación |
|---|---|---|
| CP1 | Tasks 1-3 (Wave 0) | worker comprime en pass completo + roundtrip wiki verde + auto-sync FakeClock green |
| CP2 | Tasks 4-5 (embeddings) | paráfrasis matchea por vector; fallback keyword intacto; suite completa sin regresiones |
| CP3 | Tasks 6-8 | scoring real activo + ADRs articulados por el autor + plan de bindings creado |

## Lecciones aplicadas (vigentes)

1. Verify mecánico del lead tras CADA sub-agente (bash real).
2. SARL: RESUME con feedback exacto > RESUME genérico > RETRY fresco > STRATEGY.
3. Decisiones cerradas upfront (D21-D41) — este plan mantiene el patrón.
4. Corregir header del plan tras cada update_task_state.
5. Métrica P30 ~44% primer-intento — si esta campaña <50%, ESCALAR el diagnóstico de vanta-worker al usuario antes de la campaña bindings.

---

=== RECITATION ===
Campaign ID: 1c1a36bd-d27c-49f6-919b-e4fd231e7240
Objetivo activo: MEM-47 semantic recall end-to-end: swap overlap→vector + fallback D38 en auto_recall/l1_dedup/knowledge_handlers
Estado: pending ⏳
Última acción: MEM-47 completo: campo MemoryRecord.vector poblado en reads; helpers cosine_similarity/rrf_merge(k=60)/MIN_COSINE_SIMILARITY(0.35) en l1_reader; swap dual-pool+RRF en perform_auto_recall(embed), recall_candidates(embed)→batch_dedup vía config.embed, scene_query(embed query-time); pipeline_worker pasa dedup_config.embed al recall; RecallMode.effective(bool) honesto; 5 tests D19 nuevos con fake embedding determinista 64-dim
Resultado: OK
Próxima acción: Lead: git add vanta-memory/ && verify_changed.ps1 && commit feat(vanta-memory): MEM-47 — luego delegar Task 6 (MEM-48)
Contrato: por tarea — cargo check/nextest/fmt/clippy del crate tocado exit 0 + tests D19
Próxima tarea si completa: 6
