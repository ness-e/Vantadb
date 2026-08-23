# Sistema de agentes — R1-R10, FND-01..24, TSYS-06

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### R2: Crear agente vanta-research (read-only research subagent)
- **Fuente:** `docs/Backlog.md` § P19 (mejoras del sistema de agentes)
- **Fecha:** 2026-08-16
- **Objetivo:** convertir `research-agent.md` en agente real para que el lead delegue discovery y reciba digest ≤500 palabras sin gastar contexto.
- **Resultado:** ✅ `.opencode/agents/vanta-research.md` (nuevo, 114L): mode subagent, tools read-only + web (codegraph, metasearchmcp, argus, read/grep/glob; edit/bash deny), skills §6: coordinated-web-search, source-driven-development, progreso; estructura de 7 secciones idéntica a los 9 agentes existentes. Contrato mecánico verificado (grep mode:subagent + 3 skills + read-only, exit 0). Cero impacto en código; routing en vanta-lead.md es R6 (depende R2+R3). Commit `2b4cbd6b`.
- **Ids:** `R2`

### R7: Corregir comandos de verificación rotos en Output Templates
- **Fuente:** `docs/Backlog.md` § P19 (mejoras del sistema de agentes)
- **Fecha:** 2026-08-16
- **Objetivo:** checks que "pasan" sin correr = peor que no tenerlos; 2 comandos rotos en templates de agentes.
- **Resultado:** ✅ `vanta-worker.md:102` `cargo check -p vantadb-python` → `cargo check -p vantadb_py` (package real en `vantadb-python/Cargo.toml:2`; learning AUD-039: "-p vantadb-python did not match any packages"); `vanta-docs.md:102` `pytest vantadb-python/tests/` → `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py`. Ambos corren realmente ahora. Commit `5bda5662`.
- **Ids:** `R7`

### FND-09: Regla 8 — Concurrencia paranoica en PRs
- **Fuente:** `docs/Backlog.md` § P20b (instrucciones para AGENTS.md)
- **Fecha:** 2026-08-16
- **Objetivo:** no existía gate que obligara a auditar deadlocks/data races al tocar paths multi-índice o dashmap/parking_lot/Tokio (grep "Regla 8|concurrencia paranoica" solo matcheaba Regla 7).
- **Resultado:** ✅ Regla 8 redactada en `.opencode/AGENTS.md` (tabla estilo Reglas 1/2 + carga objetivo 10k w/s + 1k r/s + delegación obligatoria vanta-chaos/vanta-review, P2-01) + 1 línea de referencia en `vanta-worker.md` Verification (L104). Contrato grep verificado (L541/543/547/548/551/553 AGENTS.md). Cambio docs aditivo, sin deuda. Commit `c34a0dc8`.
- **Ids:** `FND-09`

### FND-15: Crash recovery / WAL en la práctica (verificación con vanta-chaos)
- **Fuente:** `docs/Backlog.md` § P20c (verificación y análisis)
- **Fecha:** 2026-08-16
- **Objetivo:** verificar que kill a mitad de escritura recupera estado consistente (`chaos_integrity`/`wal_resilience`).
- **Resultado:** ✅ verificación escrita en `docs/research/FND-15-crash-recovery-verificacion.md` — sin gap de producto (WAL + recovery funcionan), gap de infra de tests documentado. Commit `8c6044a1`.
- **Ids:** `FND-15`

### FND-17: API reference automatizada (docs-as-code)
- **Fuente:** `docs/Backlog.md` § P20c (verificación y análisis)
- **Fecha:** 2026-08-16
- **Objetivo:** ¿rustdoc/pydoc/typedoc se generan en CI y se versionan junto al código? Lo primero que evalúa un dev antes de adoptar la DB.
- **Resultado:** ✅ análisis + plan en `docs/research/FND-17-api-reference-docs-as-code.md`: Fase 1 = cargo doc en CI (sin deps nuevas), defer justificado de typedoc/pydoc/site (deps nuevas, sin demanda aún). Citas archivo:línea verificadas (gate-docs-21.yml:30/62, ci-rust-10.yml:154, Cargo.toml:11, pyproject.toml:41-42, vantadb_py.pyi:1) + URLs resueltas. Commit `5dc71f0d`.
- **Ids:** `FND-17`

### FND-18: Time-to-first-query <5 min en SDKs Python/TS (Fase 0)
- **Fuente:** `docs/Backlog.md` § P20d (Fase 0 pre-launch)
- **Fecha:** 2026-08-16
- **Objetivo:** medir y reducir "instalar → primera query" en ambos SDKs; pulir quickstart hasta <5 min.
- **Resultado:** ✅ fricción real era docs rotas, no instalación: fix metadata shape en `vantadb-ts/README.md` (`{ lang: { String: "en" } }`, discriminated union vs shape roto), PyPI primario + `hit.key`/`hit.score` en `vantadb-python/README.md` (VantaSearchHit no es subscriptable), `docs/QUICKSTART.md` desactualizado corregido + sección métrica. **Medición local (2026-08-16): Python 6.2s (install 5.52s + query 0.67s), TS 1.6s (install 1.32s + query 0.30s)** — muy por debajo del objetivo de 5 min. Cero código tocado; gaps de API documentados para FND-05. Commit `ae39516e`.
- **Ids:** `FND-18`

### FND-19: Auditoría Arc<Mutex<>> en todo el core (Fase 0)
- **Fuente:** `docs/Backlog.md` § P20d (Fase 0 pre-launch)
- **Fecha:** 2026-08-16
- **Objetivo:** grep `Arc<Mutex<` en `src/` y justificar cada instancia (¿necesaria o canal mpsc / Arc<DashMap>?); heurística: anidado en core = alerta roja.
- **Resultado:** ✅ inventario en `docs/research/FND-19-arc-mutex-inventario.md` — 2 instancias en core, 1 acción recomendada (ingestion canal). Commit `5df79635`.
- **Ids:** `FND-19`

### FND-20: Documentar trade-off HNSW (ef_search/M: recall vs latencia) + argumento vs IVF/FAISS
- **Fuente:** `docs/Backlog.md` § P20d (Fase 0 pre-launch)
- **Fecha:** 2026-08-16
- **Objetivo:** nota técnica defensible para Show HN ("¿por qué no FAISS?"), con parámetros actuales citados.
- **Resultado:** ✅ `docs/architecture/FND-20-hnsw-tradeoff.md` (inglés): parámetros HNSW actuales (M=32, ef=100) con citas archivo:línea (`src/index/graph.rs:255-269`, `search/nearest.rs:71-77`, `neighbors.rs:57-62`, `ivf.rs:79-228`, `auto_tune.rs:11-53`), trade-off recall vs latencia/memoria, sección "Why not FAISS/IVF" para local-first. Drift documentado: ADR 005 (ef_construction=200) y PERFORMANCE_TUNING.md (=400) no coinciden con el código (=100) — la nota cita el código como fuente de verdad. Commit `4051a850`.
- **Ids:** `FND-20`

### FND-21: ADRs retroactivos de decisiones ya tomadas (Fjall vs RocksDB, zero-copy Arrow, WAL async/batch)
- **Fuente:** `docs/Backlog.md` § P20d (Fase 0 pre-launch)
- **Fecha:** 2026-08-16
- **Objetivo:** decisiones ya tomadas en código sin ADR que las documente; complementa FND-12 (método).
- **Resultado:** ✅ 3 ADRs en `docs/architecture/adr/` con Contexto/Decisión/Consecuencias/Status, numeración sin colisión (ADR-020/021/022; ADR-019 ya ocupado), evidencia archivo:línea: **ADR-020** consolidación backend default Fjall vs RocksDB (relaciona ADR 004; `Cargo.toml:97`, `config.rs:582-598`, `init.rs:269-289`), **ADR-021** zero-copy Arrow en bindings (nuevo genuino; `columnar.rs:22`, wasm `lib.rs:1428-1447`; estado bindings Python/Node sin Arrow → FND-04 pendiente), **ADR-022** consolidación WAL async/batch (relaciona DRV-014/DRV-015; `wal.rs:297/340/342/358`, `wal_sharded.rs:9-14/191/198-218`). Commit `b4a86030`.
- **Ids:** `FND-21`

### TSYS-06: Chaos/resilience del task-system — decisión (runner DEFER)
- **Fuente:** `docs/Backlog.md` § P17 (P17 TSYS-06) + P18 TIR-07 (misma brecha)
- **Fecha:** 2026-08-16
- **Objetivo:** decidir si construir un chaos runner que fuzzee `campaign-server.mjs`/máquina de estados vs tests de inyección de fallos puntuales (gap-01 §3.3-24).
- **Resultado:** ✅ decisión documentada en `docs/research/TSYS-06-chaos-runner.md`: **runner DEFERIDO** — tests de inyección de fallos puntuales cubren el riesgo real con fracción del costo; runner re-evaluable cuando el MCP server tenga superficie crítica. Resuelve también TIR-07. Commit `bd4c3e22`.
- **Ids:** `TSYS-06`

### R1: Skills obligatorias en §6 de los 9 agentes
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** añadir línea OBLIGATORIO al inicio de §6 para que sub-agentes vía `task` tool carguen sus skills de dominio.
- **Resultado:** ✅ línea "> **OBLIGATORIO:** al inicio de cada sesión cargá con skill <nombre> las skills de esta sección." al inicio de §6 en los 9 agentes (`.opencode/agents/vanta-*.md`). Commit `ec7f947a`.
- **Ids:** `R1`

### R3: Delegar fase DISCOVERY a vanta-research
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** delegar solo las fases pesadas (DISCOVERY 🟡/🔴) a vanta-research — ataca la pérdida de contexto en /compact (híbrido recomendado P6).
- **Resultado:** ✅ `commands/pipeline.md` (modo task) y `task.md` Phase 2-3 referencian el fork a vanta-research: el lead arma el task file con el digest. Commit `1885f64e`.
- **Ids:** `R3`

### R5: Sync §6 ↔ `campaign_load_skills`
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** que lo que lista cada agente en §6 coincida con lo que devuelve el MCP (evita duplicación/desfase).
- **Resultado:** ✅ §6 sincronizado con `campaign_load_skills` en los 9 agentes; 0 refs desfasadas (grep verificado). Commit `ec7f947a`.
- **Ids:** `R5`

### R6: Routing table + manual con vanta-research
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** añadir fila "Research/Discovery → vanta-research" en `vanta-lead.md` §8 y actualizar el operating manual.
- **Resultado:** ✅ fila en `vanta-lead.md` §8 + `.opencode/VANTADB-OPERATING-MANUAL.md` actualizado (dependía de R3 ✅). Commit `7c21c8a4`.
- **Ids:** `R6`

### R8: Eliminar referencia colgante a skill `typescript-expert`
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** reemplazar en `vanta-worker.md:125` la skill REMOVED/Dangling `typescript-expert` por una viva.
- **Resultado:** ✅ `vanta-worker.md:125` → `source-driven-development` (validado contra SKILLS-MANIFEST). Commit `ec7f947a`.
- **Ids:** `R8`

### R9: Alinear bloques `permission:` con tablas MCP ❌/✅ (deuda TSYS-11)
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** el permission block SÍ filtra tools → deny los servers ❌ de la tabla MCP en cada agente.
- **Resultado:** ✅ permission blocks de los 9 agentes denegan los servers ❌ MCP (playwright/discord/lottiefiles/cargo-mcp/rust-analyzer-mcp según cada tabla). Commit `ec7f947a`.
- **Ids:** `R9`

### R10: Consolidar bloque §7 duplicado en reference compartido
- **Fuente:** `docs/Backlog.md` § P19
- **Fecha:** 2026-08-16
- **Objetivo:** eliminar ~10 líneas idénticas × 9 agentes + tabla MCP (drift real documentado).
- **Resultado:** ✅ `.opencode/references/task-system.md` creado (patrón `definition-of-done.md`) y §7 reemplazado por 1 línea por agente. Commit `ec7f947a`.
- **Ids:** `R10`

### FND-01: Regla de presupuesto de memoria + benchmark OOM
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** investigar qué vive en RAM vs disco hoy (HNSW/LSM) y confirmar si el riesgo OOM es real bajo carga.
- **Resultado:** ✅ 🔴 CONFIRMADO: RSS sin límite real — guard existente subestima ~6.5× el uso bajo carga 10k w/s + 1k r/s (bench OOM). Regla must en `.opencode/rules/memory-budget.md` (compute/storage separation + back-pressure). Follow-ups F1/F4 delegados a core-engine. Commit `a159211b`.
- **Ids:** `FND-01`

### FND-02: Regla de coordinación multi-índice + auditoría de deadlocks
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** escanear paths multi-índice (vector + grafo + text), mapear orden de locks y buscar inversión/contención.
- **Resultado:** ✅ fix de deadlocks en evicción multi-índice (lock no reentrante + write guard) + regla en `.opencode/rules/concurrency-async.md` + audit P2-01 approve. Follow-ups menores 2/3 delegados a core-engine. Commits `c104f1f2` + `93a1e311`.
- **Ids:** `FND-02`

### FND-03: Aislamiento de features Cargo + compile matrix
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** verificar que un consumidor vector-only no compile el motor de grafos; feature set mínimo compila + wheels.
- **Resultado:** ✅ feature set mínimo compila (`--no-default-features --features fjall`) + wheels empaquetan set mínimo; compile matrix CI verde. Commit `71c58753`.
- **Ids:** `FND-03`

### FND-04: Zero-copy Arrow en bindings — DIFERIDO con ADR
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** analizar viabilidad de exponer buffers Arrow sin copia en Python/Node y firmar plan (implementación o ADR de diferimiento).
- **Resultado:** ✅ DIFERIDO con ADR-021 + señal de reapertura explícita en `docs/research/FND-04-arrow-zero-copy.md` (umbrales de reapertura documentados). Commit `95a67fd3`.
- **Ids:** `FND-04`

### FND-05: SDK idiomático (no wrapper 1:1 de Rust)
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** identificar gaps de API idiomática en `vantadb-python`/`vantadb-ts` (context managers, type hints, async nativo).
- **Resultado:** ✅ análisis en `docs/research/FND-05-sdk-idiomatico.md` (lista de gaps PY-*/TS-*) + prototipos: `with VantaDB(path) as db` (Python) y `await using db` (TS). Sin rewrite — no hacer async nativo (cubre FND-04). Commit `14183fc4`.
- **Ids:** `FND-05`

### FND-06: Regla de boundaries core ↔ bindings (Ports & Adapters)
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** verificar si server/bindings dependen de detalles internos del core que deberían estar tras un trait.
- **Resultado:** ✅ regla R-8 core-bindings (lógica de negocio NUNCA en PyO3/WASM/server) en `.opencode/rules/api-contract.md` + TODO(core) marcado + drift ERR-028 documentado. Commit `bea0f513`.
- **Ids:** `FND-06`

### FND-07: Regla de observabilidad real (prometheus) + probe endpoint
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** cerrar el gap entre `prometheus` declarado en Cargo.toml y lo realmente consultable; /metrics con datos reales.
- **Resultado:** ✅ `/metrics` responde con feed real de latencia de queries (prometheus) + regla R-3 en `.opencode/rules/server-mcp.md` (todo endpoint nuevo expone métricas reales, no placeholders). Commit `8820bdaf`.
- **Ids:** `FND-07`

### FND-08: Regla de backend validado contra patrón de acceso real + auditoría de compactación
- **Fuente:** `docs/Backlog.md` § P20a
- **Fecha:** 2026-08-16
- **Objetivo:** evaluar si la compactación de fjall/rocksdb está tuneada para random reads (similitud vectorial) vs default de escritura secuencial.
- **Resultado:** ✅ ADR-023 (backend compaction — diferir marginal, justificado con bench de lectura) + regla en `.opencode/rules/durability.md`. Commit `e5e76684`.
- **Ids:** `FND-08`

### FND-10: Regla 9 — No optimizar sin medir + benchmark canónico P99
- **Fuente:** `docs/Backlog.md` § P20b
- **Fecha:** 2026-08-16
- **Objetivo:** establecer benchmark canónico P99 (insert 100k×1536d + search) como baseline de no-regresión.
- **Resultado:** ✅ Regla 9 en `.opencode/AGENTS.md` + `benches/canonical_p99.rs` ejecutable con baseline registrado: **3.07ms p99** (documentado en `docs/operations/BENCHMARKS.md`). Commit `89943c7d`.
- **Ids:** `FND-10`

### FND-11: No mergear código IA sin poder explicarlo (AI Guardian)
- **Fuente:** `docs/Backlog.md` § P20b
- **Fecha:** 2026-08-16
- **Objetivo:** gate que impida aceptar código generado por IA sin poder explicar cada decisión no trivial.
- **Resultado:** ✅ Regla 10 (AI Guardian) en `.opencode/AGENTS.md` + referenciada en workflow de PR (el desarrollo dicta el syllabus). Commit `3b0d2a3b`.
- **Ids:** `FND-11`

### FND-12: ADRs como forcing function (escrito por humano, no IA)
- **Fuente:** `docs/Backlog.md` § P20b
- **Fecha:** 2026-08-16
- **Objetivo:** reforzar que el ADR lo escribe el autor humano articulando el trade-off; la IA solo aporta evidencia.
- **Resultado:** ✅ Regla 5 reforzada en `.opencode/AGENTS.md` con formato mínimo (Contexto/Decisión/Consecuencias — quién lo articula). Commit `3b0d2a3b`.
- **Ids:** `FND-12`

### FND-13: Benchmarks honestos (extiende FND-10)
- **Fuente:** `docs/Backlog.md` § P20b
- **Fecha:** 2026-08-16
- **Objetivo:** los claims de performance deben citar benchmark reproducible + números, no adjetivos.
- **Resultado:** ✅ Regla 11 en `.opencode/AGENTS.md` + claims del README revisados y alineados. Commit `d61a006c`.
- **Ids:** `FND-13`

### FND-14: Ritual de inicio — validación de feature stack
- **Fuente:** `docs/Backlog.md` § P20b
- **Fecha:** 2026-08-16
- **Objetivo:** el ritual de sesión debe verificar que el feature set default + mínimo compila.
- **Resultado:** ✅ paso 5 del Ritual de Inicio en `.opencode/AGENTS.md` (`cargo check --no-default-features --features fjall`). Commit `3b0d2a3b`.
- **Ids:** `FND-14`

### FND-16: Multi-target CI (wheels + WASM por PR)
- **Fuente:** `docs/Backlog.md` § P20c
- **Fecha:** 2026-08-16
- **Objetivo:** analizar si compilar wheels Windows/Mac/Linux + WASM en cada PR vale el costo.
- **Resultado:** ✅ plan multi-target CI implementado: job wasm/TS por PR con paths filter (análisis en FND-16) + fix path CONTRIBUTING + dictamen P2-01 en FND-02. Commits `0f15a817` + `fb878cba`.
- **Ids:** `FND-16`

### FND-22: CONTRIBUTING.md + triage de issues (post-launch)
- **Fuente:** `docs/Backlog.md` § P20d
- **Fecha:** 2026-08-16
- **Objetivo:** formalizar proceso de contribución y triage antes de que el volumen comunitario desborde.
- **Resultado:** ✅ `CONTRIBUTING.md` (commit convention, PR flow, gates) + guía de triage en `.github/`. Commit `d9beaa9a`.
- **Ids:** `FND-22`

### FND-23: Decidir grafos default-on vs opt-in con telemetría real (post-launch)
- **Fuente:** `docs/Backlog.md` § P20d
- **Fecha:** 2026-08-16
- **Objetivo:** usar señales reales de adopción para decidir si el motor de grafos queda default-on o pasa a opt-in.
- **Resultado:** ✅ ADR-024: motor de grafos **default-on hasta señal de telemetría** (métrica `vanta_graph_ops_total`) — no decidir por intuición; complementa FND-03. Commit `bde23fd3`.
- **Ids:** `FND-23`

### FND-24: JTBD/ICP: entrevistas post-Show HN
- **Fuente:** `docs/Backlog.md` § P20d
- **Fecha:** 2026-08-16
- **Objetivo:** usar las primeras conversaciones reales para definir ICP y job-to-be-done.
- **Resultado:** ✅ `docs/research/FND-24-icp-jtbd.md`: **0 evidencia de usuarios reales — todo hipótesis** (4 perfiles ICP, 10 JTBD) + plan de validación accionable (semana 4-8 post-Show HN). No se inventa evidencia donde no existe (regla de la tarea). Commit `a93e7932`.
- **Ids:** `FND-24`
