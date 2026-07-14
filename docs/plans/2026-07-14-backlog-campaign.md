# Plan de Ejecución: Backlog Campaign (Jul 14)

> **Inicio:** 2026-07-14
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/Backlog.md`
> **Método:** Triage Gate (backlog-executor Prompt 0) aplicado a todos los items ❌
> **Principio:** Solo tareas ✅ DO — excluidas contenido humano (copy, case studies, blogs, posts), trámites externos (trademark, certificados), y redes sociales

## Triage Summary

| Resultado | Count |
|-----------|-------|
| **✅ DO** | ~80 tasks |
| 🟡 DEFER | ~40 tasks |
| ❌ SKIP | ~10 tasks |
| 🔴 BLOQUEADO | 0 tasks |

Total ❌ en Backlog.md: ~130+ items. Gate aplicado con criterios: relevancia, impacto real, costo/beneficio, dependencias, riesgo, scope.

---

## Priority 🔴 — Bloqueantes de Release

### Task 1: REV-001 — CI Rust TSan ABI mismatch

- **Fuente:** Backlog.md:375
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔴
- **Archivos clave:** `.github/workflows/ci-rust-10.yml`
- **Gate Result:** ✅ DO
- **Gate Justificación:** CI roto en main impide releases. TSan incompatible con Rust 1.94.1. Solución: remover `-Zsanitizer=thread` o gatear tras nightly.
- **Contrato:** "`cargo nextest run --profile audit --workspace --build-jobs 2` pasa en CI"
- **Estado:** ✅ FIXED
- **Notas:** Removidos flags `-Zsanitizer=thread` y `-Cunsafe-allow-abi-mismatch=sanitizer` del job `sanitizer-tsan`; job ahora corre como nightly test regular. Además, corregido error de indentación YAML en job `msrv` (3 espacios → 2) que invalidaba todo el workflow.

### Task 2: REV-002 — CI Web 21 ESLint errors

- **Fuente:** Backlog.md:376
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔴
- **Archivos clave:** `web/src/routes/demo.lazy.tsx`, `web/src/routes/why-vantadb.tsx`
- **Gate Result:** ✅ DO
- **Gate Justificación:** CI web roto impide releases. 14 errors + 7 warnings conocidos y localizados.
- **Contrato:** "`npm run lint` 0 errors, 0 warnings"
- **Estado:** ✅ FIXED
- **Notas:** 19 prettier errors auto-fixed via `--fix`. 3 react-hooks/exhaustive-deps warnings fixed by adding `reducedMotion` to dep arrays. Commit `35873e6`.

### Task 3: DRV-099 — Haystack protocolo Document real

- **Fuente:** Backlog.md:344
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-haystack/src/python.rs:42-173`, `vantadb-haystack/vantadb_haystack.pyi`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Integración rota con Haystack real. Métodos aceptan/retornan dicts en vez de `haystack.dataclasses.Document`. No es compatible con pipelines.
- **Contrato:** "`cargo check -p vantadb-haystack` pasa, Python tests pasan"
- **Estado:** ✅ FIXED
- **Notas:** write_documents now accepts both dict and haystack.dataclasses.Document objects. filter_documents returns Document instances. Fixed meta extraction for both paths. Updated .pyi stubs.

### Task 4: DRV-102 — Langchain missing GIL release (all methods)

- **Fuente:** Backlog.md:352
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-langchain/src/python.rs:82-85,109-112,126-133`
- **Gate Result:** ✅ DO
- **Gate Justificación:** GIL no liberado en `add_texts`, `similarity_search_by_vector`, `delete`. Bloquea Python threads.
- **Contrato:** "`cargo check -p vantadb-langchain` pasa, tests pasan"
- **Estado:** ✅ COMPLETED
- **Commit:** `3cc6888`
- **Notas:** PyO3 0.29 cambió API: `allow_threads` → `detach(self, f)` que consume el token Python. Los métodos se reestructuraron en fases: prepare (GIL) → detach (bloqueante) → post (GIL renovado via `Python::attach`). Colateral: `__init__.py` expone `__version__`.

### Task 5: DRV-109 — LlamaIndex missing GIL release (all methods)

- **Fuente:** Backlog.md:366
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-llamaindex/src/python.rs:82-85,104-107,124-126`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mismo bug que DRV-102 (código byte-for-byte idéntico a langchain). GIL no liberado.
- **Contrato:** "`cargo check -p vantadb-llamaindex` pasa, tests pasan"
- **Estado:** ⬜ PENDING

### Task 6: SEC-13 — CSP unsafe-inline + HSTS + nonce system

- **Fuente:** Backlog.md:85
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `web/` (CSP headers, nonce generation)
- **Gate Result:** ✅ DO
- **Gate Justificación:** Seguridad: `style-src 'unsafe-inline'` + sin HSTS + `/metrics` público sin auth. Bloqueante de release.
- **Contrato:** "CSP nonce funcional en prod build, HSTS headers presentes"
- **Estado:** ⬜ PENDING

### Task 7: DEVOPS-13 — Pin all workflow actions to SHA + Node 22

- **Fuente:** Backlog.md:90
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `.github/workflows/*.yml` (11 workflows)
- **Gate Result:** ✅ DO
- **Gate Justificación:** Supply chain security. 11 workflows sin SHA pinning. Node 20 deprecated.
- **Contrato:** "Todos los `actions/*` usan `@<sha>` en vez de `@vX`"
- **Estado:** ⬜ PENDING

---

## Priority 🟡 — Código y Correctitud

### Task 8: DRV-006 — Race condition en delete() (write lock dropped before index cleanup)

- **Fuente:** Backlog.md:149
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔴
- **Archivos clave:** `src/engine.rs:235-248`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Data integrity. `drop(nodes)` libera write lock L241, luego actualiza índices sin protección. Ventana de corrupción.
- **Contrato:** "`cargo check -p vantadb` pasa, tests existentes pasan"
- **Estado:** ⬜ PENDING

### Task 9: DRV-007 — Data race en filter_field() (scalar_index sin lock)

- **Fuente:** Backlog.md:150
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `src/engine.rs:354`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Comportamiento indefinido. `filter_field()` no adquiere `nodes` RwLock mientras mutaciones concurrentes modifican `scalar_index`.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 10: DRV-057 — OpenAI client recreado en cada embed()

- **Fuente:** Backlog.md:263
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-openai/src/python.rs:63-69`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Performance: TLS handshake + connection pool se recrean por request. Cachear `Py<PyAny>` del cliente elimina overhead.
- **Contrato:** "`cargo check -p vantadb-openai` pasa"
- **Estado:** ⬜ PENDING

### Task 11: DRV-058 — OpenAI metadata no-string ignorado

- **Fuente:** Backlog.md:264
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-openai/src/python.rs:149-155`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Bug: metadata bool/int/float se pierden sin warning. Impacto directo en usuarios.
- **Contrato:** "`cargo check -p vantadb-openai` pasa"
- **Estado:** ⬜ PENDING

### Task 12: DRV-062 — Ollama client recreado en cada embed()

- **Fuente:** Backlog.md:275
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-ollama/src/python.rs:63-70`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mismo bug que DRV-057. Cachear cliente.
- **Contrato:** "`cargo check -p vantadb-ollama` pasa"
- **Estado:** ⬜ PENDING

### Task 13: DRV-063 — Ollama metadata no-string ignorado

- **Fuente:** Backlog.md:276
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-ollama/src/python.rs:139-145`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mismo bug que DRV-058. Copy-paste del adapter openai.
- **Contrato:** "`cargo check -p vantadb-ollama` pasa"
- **Estado:** ⬜ PENDING

### Task 14: DRV-068 — LiteLLM GIL no liberado en search()

- **Fuente:** Backlog.md:288
- **Esfuerzo:** 🟢 15min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-litellm/src/python.rs:94-122`
- **Gate Result:** ✅ DO
- **Gate Justificación:** GIL bloqueado durante búsqueda vectorial, impide concurrencia Python.
- **Contrato:** "`cargo check -p vantadb-litellm` pasa"
- **Estado:** ⬜ PENDING

### Task 15: DRV-069 — LiteLLM store() sin parámetro py

- **Fuente:** Backlog.md:289
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-litellm/src/python.rs:124-148`
- **Gate Result:** ✅ DO
- **Gate Justificación:** No puede liberar GIL. Necesita cambio de firma para agregar `py: Python`.
- **Contrato:** "`cargo check -p vantadb-litellm` pasa"
- **Estado:** ⬜ PENDING

### Task 16: DRV-070 — LiteLLM metadata no-string ignorado

- **Fuente:** Backlog.md:290
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-litellm/src/python.rs:136-142`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Tercera copia del mismo bug DRV-058/063.
- **Contrato:** "`cargo check -p vantadb-litellm` pasa"
- **Estado:** ⬜ PENDING

### Task 17: DRV-103 — LangChain metadata no-string ignorado

- **Fuente:** Backlog.md:353
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-langchain/src/python.rs:70-78`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-langchain` pasa"
- **Estado:** ⬜ PENDING

### Task 18: DRV-104 — LangChain similarity_search no retorna metadata

- **Fuente:** Backlog.md:354
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-langchain/src/python.rs:114-121`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Metadata almacenada via `add_texts` se pierde en respuesta. LangChain espera metadata para filtering.
- **Contrato:** "`cargo check -p vantadb-langchain` pasa"
- **Estado:** ⬜ PENDING

### Task 19: DRV-105 — LangChain delete() silenciosamente no-op en IDs malformados

- **Fuente:** Backlog.md:355
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-langchain/src/python.rs:125-133`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Bug silencioso. `id.split(':')` con `parts.len() != 2` ignora delete sin error.
- **Contrato:** "`cargo check -p vantadb-langchain` pasa"
- **Estado:** ⬜ PENDING

### Task 20: DRV-106 — LangChain from_texts class method no implementado

- **Fuente:** Backlog.md:356
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-langchain/src/python.rs:11`
- **Gate Result:** ✅ DO
- **Gate Justificación:** LangChain usa `from_texts` como entry point principal. Docstring afirma implementarlo. Gap real.
- **Contrato:** "`cargo check -p vantadb-langchain` pasa, `from_texts` funciona"
- **Estado:** ⬜ PENDING

### Task 21: DRV-110 — LlamaIndex metadata no-string ignorado

- **Fuente:** Backlog.md:367
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-llamaindex/src/python.rs:70-78`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-llamaindex` pasa"
- **Estado:** ⬜ PENDING

### Task 22: DRV-111 — LlamaIndex query() no retorna metadata

- **Fuente:** Backlog.md:368
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-llamaindex/src/python.rs:109-116`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-llamaindex` pasa"
- **Estado:** ⬜ PENDING

### Task 23: DRV-112 — LlamaIndex delete() no-op en IDs malformados

- **Fuente:** Backlog.md:369
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-llamaindex/src/python.rs:120-128`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-llamaindex` pasa"
- **Estado:** ⬜ PENDING

### Task 24: DRV-086 — CrewAI metadata no-string ignorado

- **Fuente:** Backlog.md:321
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-crewai/src/python.rs:141-148`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-crewai` pasa"
- **Estado:** ⬜ PENDING

### Task 25: DRV-092 — DSPy metadata no-string ignorado

- **Fuente:** Backlog.md:332
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-dspy/src/python.rs:100-107`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mismo bug, misma solución, esfuerzo mínimo.
- **Contrato:** "`cargo check -p vantadb-dspy` pasa"
- **Estado:** ⬜ PENDING

### Task 26: DRV-098 — Haystack metadata inconsistencia intra-archivo

- **Fuente:** Backlog.md:343
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-haystack/src/python.rs:88-98`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Metadata no-string se pierde al escribir pero se parsea al filtrar. Documentos escritos no son encontrables por filtro.
- **Contrato:** "`cargo check -p vantadb-haystack` pasa"
- **Estado:** ⬜ PENDING

### Task 27: DRV-050 — MCP LISP injection vector

- **Fuente:** Backlog.md:249
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1154-1187`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Security: naive escaping antes de interpolación en query LISP. No escapa paréntesis, newlines.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 28: DRV-044 — MCP shutdown via process::exit(0)

- **Fuente:** Backlog.md:236
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-server/src/main.rs:46-57`
- **Gate Result:** ✅ DO
- **Gate Justificación:** In-flight JSON-RPC requests se pierden sin respuesta. Fix: `CancellationToken` en vez de `exit(0)`.
- **Contrato:** "`cargo check -p vantadb-server` pasa"
- **Estado:** ⬜ PENDING

### Task 29: DRV-048 — JSON-RPC no-2.0 descartado silenciosamente

- **Fuente:** Backlog.md:247
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-mcp/src/lib.rs:359-363`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Spec violation (§7): servidor DEBE responder error -32600. Cliente nunca sabe que su request fue rechazado.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 30: DRV-049 — MCP collection_delete no atómico

- **Fuente:** Backlog.md:248
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1269-1305`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Crash a mitad de delete → namespace parcialmente borrado. Sin transacción.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 31: DRV-054 — MCP read_axioms hardcoded

- **Fuente:** Backlog.md:253
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1190-1198`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Axioms hardcoded como JSON inline. Si se actualizan en metadata module, copia MCP deriva.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 32: DRV-025 — TOCTOU race en ResourceGovernor::request_allocation()

- **Fuente:** Backlog.md:196
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `src/governor.rs:41-57`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Dos threads pueden pasar el check OOM y sobre-asignar 2× del límite. Sin CAS.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 33: DRV-040 — unsafe en simd.rs sin // SAFETY: comment

- **Fuente:** Backlog.md:225
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-wasm/src/simd.rs:34-77`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Bloque `unsafe` con `v128_load` requiere punteros alineados. Sin SAFETY docs el invariante no es verificable.
- **Contrato:** "`cargo check -p vantadb-wasm` pasa"
- **Estado:** ⬜ PENDING

### Task 34: DRV-043 — Core crate compilation errors (visibility)

- **Fuente:** Backlog.md:228
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb/src/sdk/serialization/impl_index.rs:20,210`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Bloquea `cargo check -p vantadb-wasm` + todos los adapters que dependen de core. `ensure_text_index_current_with` y `adjust_text_index_state_after_replace` privados en `impl_text_index.rs` pero llamados desde `impl_index.rs`.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 35: DRV-035 — TypeScript metadata type mismatch

- **Fuente:** Backlog.md:220
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/__tests__/*.test.ts` + `src/types.ts:1-10`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Bug dormido: tests usan `{ source: { String: "test" } }` pero VantaValue define `{ type: "String", value: "test" }`. No se serializa correctamente por bridge WASM.
- **Contrato:** "`npx tsc --noEmit` pasa"
- **Estado:** ⬜ PENDING

### Task 36: DRV-041 — Worker.rs Promise cuelga si mensaje nunca llega

- **Fuente:** Backlog.md:226
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-wasm/src/worker.rs:201-229`
- **Gate Result:** ✅ DO
- **Gate Justificación:** `_reject` nunca se invoca → Promise cuelga para siempre. Response parsing vía `serde_json::from_str` agrega round-trip JSON innecesario.
- **Contrato:** "`cargo check -p vantadb-wasm` pasa"
- **Estado:** ⬜ PENDING

---

## Priority 🟡 — Performance y Calidad

### Task 37: DRV-074 — mem0 delete_col solo pagina 1 (data loss)

- **Fuente:** Backlog.md:299
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mem0/src/python.rs:324-344`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Si collection >100 records, los de páginas posteriores sobreviven al delete. No atómico.
- **Contrato:** "`cargo check -p vantadb-mem0` pasa"
- **Estado:** ⬜ PENDING

### Task 38: DRV-079 — Letta list_memories solo pagina 1 (truncación)

- **Fuente:** Backlog.md:309
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-letta/src/python.rs:122-139`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mismo patrón que DRV-074. Si user/agent >100 memorias, extra no aparecen.
- **Contrato:** "`cargo check -p vantadb-letta` pasa"
- **Estado:** ⬜ PENDING

### Task 39: DRV-085 — CrewAI clear() solo pagina 1 (data loss)

- **Fuente:** Backlog.md:320
- **Esfuerzo:** 🟡 2h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-crewai/src/python.rs:120-138`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mismo bug que DRV-074/079.
- **Contrato:** "`cargo check -p vantadb-crewai` pasa"
- **Estado:** ⬜ PENDING

### Task 40: DRV-097 — Haystack count_documents() truncates at 100

- **Fuente:** Backlog.md:342
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `vantadb-haystack/src/python.rs:167-171`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Usa `Default::default()` con `limit: Some(100)`. Si namespace >100 docs, devuelve 100. Same bug.
- **Contrato:** "`cargo check -p vantadb-haystack` pasa"
- **Estado:** ⬜ PENDING

### Task 41: VFY-001 — TS SDK catch {} silencia errores

- **Fuente:** Backlog.md:103
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/src/vantadb.ts:176,215,249`
- **Gate Result:** ✅ DO
- **Gate Justificación:** 4+ bloques catch vacíos tragan errores.
- **Contrato:** "`npx tsc --noEmit` pasa"
- **Estado:** ⬜ PENDING

### Task 42: VFY-002 — TS get_nns_by_id spawn por llamada

- **Fuente:** Backlog.md:104
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-ts/src/vantadb.ts:325`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Sin batching, spawn overhead en cada llamada.
- **Contrato:** "`npx tsc --noEmit` pasa"
- **Estado:** ⬜ PENDING

### Task 43: VFY-003 — Python reindex_hnsw_from_text riesgo OOM

- **Fuente:** Backlog.md:105
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-python/src/lib.rs:1584`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Sin batch processing, dataset grande → OOM.
- **Contrato:** "`cargo check -p vantadb-python` pasa"
- **Estado:** ⬜ PENDING

### Task 44: VFY-004 — flat.rs O(n²) en filter

- **Fuente:** Backlog.md:106
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `src/index/flat.rs:32`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Sin índice para filtros, O(n²) en filter.
- **Contrato:** "`cargo check -p vantadb` pasa, bench sin regresión"
- **Estado:** ⬜ PENDING

### Task 45: VFY-005 — TS OperationalMetrics 70% incompleto

- **Fuente:** Backlog.md:107
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-ts/src/types.ts:148-168`
- **Gate Result:** ✅ DO
- **Gate Justificación:** 3 de 10+ métricas mapeadas.
- **Contrato:** "`npx tsc --noEmit` pasa"
- **Estado:** ⬜ PENDING

### Task 46: VFY-006 — add_node escribe lock durante toda inserción

- **Fuente:** Backlog.md:108
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `src/index/graph.rs:476-490`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Lock retention durante toda inserción bloquea lecturas concurrentes.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 47: VFY-007 — remove_node O(n²) neighbor fixup

- **Fuente:** Backlog.md:109
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟢
- **Archivos clave:** `src/index/core.rs`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Deletes costosos O(n²).
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 48: VFY-008 — WAL fsync por escritura

- **Fuente:** Backlog.md:110
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `src/storage/wal.rs`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Write amplification por fsync en cada escritura.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 49: VFY-010 — ACID Phase 2: Buffered write transactions

- **Fuente:** Backlog.md:112
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔵
- **Archivos clave:** `src/wal.rs`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Phase 1 implementada, Phase 2 no. Requerido para ACID completo.
- **Contrato:** "`cargo nextest run --profile audit --workspace --build-jobs 2` pasa"
- **Estado:** ⬜ PENDING

### Task 50: VFY-012 — DEVOPS-03 musllinux target gap

- **Fuente:** Backlog.md:114
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟢
- **Archivos clave:** CI config
- **Gate Result:** ✅ DO
- **Contrato:** "CI musllinux targets build correctos"
- **Estado:** ⬜ PENDING

### Task 51: RC5 — Mejorar mensaje expect crypto.rs

- **Fuente:** Backlog.md:123
- **Esfuerzo:** 🟢 15min
- **Prioridad:** 🟡
- **Archivos clave:** `src/crypto.rs:104`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Mensaje genérico, incluir razón técnica en panic.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 52: RC8 — auth_middleware debe devolver 401 en vez de panic

- **Fuente:** Backlog.md:126
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `src/cli_server.rs:758`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Middleware paniquea cuando invariante se viola. Debe devolver 401.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 53: DEVOPS-14 — Extract composite action para Rust setup

- **Fuente:** Backlog.md:91
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟡
- **Archivos clave:** `.github/workflows/*.yml`
- **Gate Result:** ✅ DO
- **Gate Justificación:** 5+ workflows duplican inline Rust setup. DRY.
- **Contrato:** "CI workflows pasan"
- **Estado:** ⬜ PENDING

---

## Priority 🟡 — Refactoring y DRY

### Task 54: DRV-002 — put_batch duplica lógica de put()

- **Fuente:** Backlog.md:138
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟢
- **Archivos clave:** `src/sdk/api.rs:117-193`
- **Gate Result:** ✅ DO
- **Gate Justificación:** ~40 líneas idénticas (validación, node_id collision, timestamp, version). DRY violation.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 55: DRV-003 — purge_expired llama replace_derived_indexes por nodo

- **Fuente:** Backlog.md:139
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `src/sdk/api.rs:380-383`
- **Gate Result:** ✅ DO
- **Gate Justificación:** O(n) index rebuilds en loop. 10K purges = 10K rebuilds.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 56: DRV-004 — list() carga ALL records a memoria antes de paginar

- **Fuente:** Backlog.md:140
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `src/sdk/api.rs:296-315`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Namespace con 100K+ registros → OOM.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 57: DRV-008 — Duplicate scoring pipeline en vector_search() y hybrid_search()

- **Fuente:** Backlog.md:151
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔵
- **Archivos clave:** `src/engine.rs:288-305,399-413`
- **Gate Result:** ✅ DO
- **Gate Justificación:** ~25 líneas idénticas (sort_by, truncate, collect, QueryResult build).
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 58: DRV-009 — node_count() O(n) full scan

- **Fuente:** Backlog.md:152
- **Esfuerzo:** 🟢 1h
- **Prioridad:** ⚪
- **Archivos clave:** `src/engine.rs:424-426`
- **Gate Result:** ✅ DO
- **Gate Justificación:** 1M nodos → 1M iteraciones por cada node_count(). Sin contador cacheado.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 59: DRV-011 — Scan-forward recovery duplicado en WalWriter y WalReader

- **Fuente:** Backlog.md:161
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🔵
- **Archivos clave:** `src/wal.rs:287-332,593-630`
- **Gate Result:** ✅ DO
- **Gate Justificación:** ~40 líneas idénticas de byte-scan para localizar registro válido tras corrupción.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 60: DRV-012 — append() y batch_append() duplican lógica de sync

- **Fuente:** Backlog.md:162
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `src/wal.rs:390-396,430-436`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Extraer a fn `maybe_sync()`.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 61: DRV-016 — Inconsistencia Mutex: governor.rs usa std::sync::Mutex vs parking_lot

- **Fuente:** Backlog.md:173
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `src/vector/governor.rs:94,111,147,160`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Todo el codebase usa parking_lot. governor.rs usa std Mutex con `.lock().unwrap()`.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 62: DRV-022 — governance/ completo gated tras feature no-default sin consumidores

- **Fuente:** Backlog.md:193
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `src/governance/`
- **Gate Result:** ✅ DO
- **Gate Justificación:** 4 módulos (1235L) bajo `#[cfg(feature = "governance")]`. Ningún crate externo ni módulo interno importa. Feature existe como flag vacío.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 63: DRV-023 — ResourceGovernor + ALLOCATED_BYTES sin callers

- **Fuente:** Backlog.md:194
- **Esfuerzo:** 🟢 15min
- **Prioridad:** 🔵
- **Archivos clave:** `src/governor.rs`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Struct exportado + static global, cero referencias fuera del archivo.
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 64: DRV-046 — Blocking stdio I/O en tokio runtime

- **Fuente:** Backlog.md:245
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/lib.rs:320-384`
- **Gate Result:** ✅ DO
- **Gate Justificación:** `stdin.lock().lines()` bloquea worker de tokio. Ctrl+C no ejecuta shutdown graceful.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 65: DRV-047 — MCP validation limits hardcoded

- **Fuente:** Backlog.md:246
- **Esfuerzo:** 🟢 15min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mcp/src/lib.rs:549,553,575`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Usar `config.max_namespace_length` / `config.max_key_length` en vez de literales.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 66: DRV-051 — MCP search_semantic N+1 query pattern

- **Fuente:** Backlog.md:250
- **Esfuerzo:** 🟢 1h
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1114-1122`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Por cada hit de HNSW, llama `get_node()` individualmente. top_k=1000 → 1000 queries.
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 67: DRV-056 — MCP stdout write errors ignorados

- **Fuente:** Backlog.md:255
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mcp/src/lib.rs:394-399,378-383`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 68: DRV-064 — Ollama embed() llama API secuencialmente por texto

- **Fuente:** Backlog.md:277
- **Esfuerzo:** 🟢 1h
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-ollama/src/python.rs:73-90`
- **Gate Result:** ✅ DO
- **Gate Justificación:** N textos = N RPCs vs 1 batch. Ollama soporta `client.embed(model=..., input=[...])`.
- **Contrato:** "`cargo check -p vantadb-ollama` pasa"
- **Estado:** ⬜ PENDING

### Task 69: DRV-075 — mem0 search() ignora text_query

- **Fuente:** Backlog.md:300
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mem0/src/python.rs:201-202`
- **Gate Result:** ✅ DO
- **Gate Justificación:** `let _ = query;` descarta texto de búsqueda. Solo vector search.
- **Contrato:** "`cargo check -p vantadb-mem0` pasa"
- **Estado:** ⬜ PENDING

### Task 70: DRV-076 — mem0 update() TOCTOU entre get() y put()

- **Fuente:** Backlog.md:301
- **Esfuerzo:** 🟢 1h
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mem0/src/python.rs:254-291`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Dos `py.detach()` separados permiten modificación concurrente.
- **Contrato:** "`cargo check -p vantadb-mem0` pasa"
- **Estado:** ⬜ PENDING

### Task 71: DRV-077 — mem0 Collection name sanitization lazy

- **Fuente:** Backlog.md:302
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mem0/src/python.rs:88-93,137-146`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Namespace efectivo ≠ collection_name si contiene chars inválidos.
- **Contrato:** "`cargo check -p vantadb-mem0` pasa"
- **Estado:** ⬜ PENDING

### Task 72: DRV-080 — Letta retrieve_memory expone distancia VantaDB raw

- **Fuente:** Backlog.md:310
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-letta/src/python.rs:111-118`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Cosine distance (0→2) directa. Consumidores esperan score 0-1. Sin normalización.
- **Contrato:** "`cargo check -p vantadb-letta` pasa"
- **Estado:** ⬜ PENDING

### Task 73: DRV-081 — Letta AtomicU64 counter reset en restart

- **Fuente:** Backlog.md:311
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-letta/src/python.rs:64,76-77`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Counter inicializado en 0. Tras cerrar/reabrir store, nuevos inserts pueden colisionar.
- **Contrato:** "`cargo check -p vantadb-letta` pasa"
- **Estado:** ⬜ PENDING

### Task 74: DRV-088 — CrewAI serde_json dependency extra

- **Fuente:** Backlog.md:323
- **Esfuerzo:** 🟢 15min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-crewai/Cargo.toml:20`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Único adapter que añade serde_json. Podría ser format!/join sin dependencia.
- **Contrato:** "`cargo check -p vantadb-crewai` pasa"
- **Estado:** ⬜ PENDING

### Task 75: DRV-090 — CrewAI search() threshold filtering post-GIL

- **Fuente:** Backlog.md:325
- **Esfuerzo:** 🟢 1h
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-crewai/src/python.rs:105-116`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Score normalization tras `py.detach()`. top_k=1000 con threshold alto → trae 1000 para devolver 0.
- **Contrato:** "`cargo check -p vantadb-crewai` pasa"
- **Estado:** ⬜ PENDING

### Task 76: DRV-093 — DSPy forward() expone distancia VantaDB raw

- **Fuente:** Backlog.md:333
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-dspy/src/python.rs:76-81`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-dspy` pasa"
- **Estado:** ⬜ PENDING

### Task 77: DRV-094 — DSPy AtomicU64 counter reset en restart

- **Fuente:** Backlog.md:334
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ℹ️
- **Archivos clave:** `vantadb-dspy/src/python.rs:38,95-96`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-dspy` pasa"
- **Estado:** ⬜ PENDING

### Task 78: DRV-101 — Haystack AtomicU64 doc_counter reset

- **Fuente:** Backlog.md:346
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ℹ️
- **Archivos clave:** `vantadb-haystack/src/python.rs:38,68-70`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-haystack` pasa"
- **Estado:** ⬜ PENDING

### Task 79: DRV-108 — LangChain AtomicU64 counter reset

- **Fuente:** Backlog.md:358
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ℹ️
- **Archivos clave:** `vantadb-langchain/src/python.rs:23,59,63`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-langchain` pasa"
- **Estado:** ⬜ PENDING

### Task 80: DRV-114 — LlamaIndex AtomicU64 counter reset

- **Fuente:** Backlog.md:371
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ℹ️
- **Archivos clave:** `vantadb-llamaindex/src/python.rs:23,59,63`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-llamaindex` pasa"
- **Estado:** ⬜ PENDING

### Task 81-86: RwLock<String> dead overhead (6 adapters)

- **Fuente:** DRV-059 (openai), DRV-065 (ollama), DRV-071 (litellm), DRV-087 (crewai), DRV-091 (dspy), DRV-096 (haystack)
- **Esfuerzo:** 🟢 15min c/u
- **Prioridad:** ⚪
- **Gate Result:** ✅ DO (agrupado)
- **Gate Justificación:** RwLock<String> nunca escrito, solo `.read().unwrap().clone()`. Cambiar a `String` plano.
- **Contrato:** "`cargo check -p vantadb-<adapter>` pasa"
- **Estado:** ⬜ PENDING

### Task 87: DRV-020 — unwrap() en serialize_to_bytes

- **Fuente:** Backlog.md:177
- **Esfuerzo:** 🟢 5min
- **Prioridad:** ℹ️
- **Archivos clave:** `src/index/serialize.rs:21`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 88: DRV-026 — redundant unwrap() en three_way_merge()

- **Fuente:** Backlog.md:197
- **Esfuerzo:** 🟢 5min
- **Prioridad:** ℹ️
- **Archivos clave:** `src/governance/conflict.rs:272-273`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb` pasa"
- **Estado:** ⬜ PENDING

### Task 89: DRV-031 — Comentario doc duplicado

- **Fuente:** Backlog.md:209
- **Esfuerzo:** 🟢 2min
- **Prioridad:** ℹ️
- **Archivos clave:** `vantadb-python/src/lib.rs:1140-1142`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo check -p vantadb-python` pasa"
- **Estado:** ⬜ PENDING

### Task 90: DRV-037 — types.test.ts usa types incorrectos

- **Fuente:** Backlog.md:222
- **Esfuerzo:** 🟢 15min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-ts/src/__tests__/types.test.ts:11-14`
- **Gate Result:** ✅ DO
- **Gate Justificación:** `created_at_ms: 1000` (number) vs type `string`, `node_id: 42` (number) vs type `string`. Pasa porque `__tests__/` excluded.
- **Contrato:** "`npx tsc --noEmit` pasa"
- **Estado:** ⬜ PENDING

### Task 91: DRV-039 — No ESLint config en vantadb-ts

- **Fuente:** Backlog.md:224
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ℹ️
- **Archivos clave:** `vantadb-ts/`
- **Gate Result:** ✅ DO
- **Contrato:** "ESLint config presente, `npx eslint` corre"
- **Estado:** ⬜ PENDING

### Task 92: DRV-045 — Test setup factory duplicado en 3 test files

- **Fuente:** Backlog.md:237
- **Esfuerzo:** 🟢 30min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-server/tests/server.rs:26-39, e2e.rs:52-66, benchmarks.rs:23-55`
- **Gate Result:** ✅ DO
- **Gate Justificación:** ~40L de duplicación de patrón TempDir + StorageEngine + ServerState.
- **Contrato:** "`cargo test -p vantadb-server` pasa"
- **Estado:** ⬜ PENDING

### Task 93: DRV-055 — MCP test testea serde_json, no lógica MCP

- **Fuente:** Backlog.md:254
- **Esfuerzo:** 🟢 15min
- **Prioridad:** ⚪
- **Archivos clave:** `vantadb-mcp/tests/mcp_tests.rs:697-721`
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo test -p vantadb-mcp` pasa"
- **Estado:** ⬜ PENDING

### Task 94: REV-013 — spin 0.9.8 yanked dependency

- **Fuente:** Backlog.md:387
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Archivos clave:** `deny.toml`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Dependencia yanked, monitoreada vía transitive dependency (fjall/flume). Podría romper CI deny check.
- **Contrato:** "`cargo deny check` pasa"
- **Estado:** ⬜ PENDING

### Task 95: REV-014 — 24 stale dependabot branches

- **Fuente:** Backlog.md:388
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🔵
- **Archivos clave:** `origin/dependabot/*`
- **Gate Result:** ✅ DO
- **Gate Justificación:** Ramas stale sin auto-delete después de merge. Housekeeping.
- **Contrato:** "No hay ramas dependabot abiertas con PR merged"
- **Estado:** ⬜ PENDING

### Task 96: NUEVO-14 — WASM bundle size <500KB gzip

- **Fuente:** Backlog.md:401
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-wasm/Cargo.toml`, build config
- **Gate Result:** ✅ DO
- **Gate Justificación:** Bundle size impacta load time en navegador. Medible.
- **Contrato:** "`wasm-pack build` produce <500KB gzip"
- **Estado:** ⬜ PENDING

### Task 97: NUEVO-15 — Code coverage report en CI

- **Fuente:** Backlog.md:402
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟡
- **Archivos clave:** CI config
- **Gate Result:** ✅ DO
- **Gate Justificación:** CII Silver requiere ≥80% coverage. Sin herramienta configurada.
- **Contrato:** "CI genera coverage report"
- **Estado:** ⬜ PENDING

### Task 98: DEVOPS-PY313 — Python 3.13 wheels en CI matrix

- **Fuente:** Backlog.md:413
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Archivos clave:** CI config
- **Gate Result:** ✅ DO
- **Gate Justificación:** Python 3.13 estable, wheels necesarios para compatibilidad.
- **Contrato:** "CI build matrix incluye Python 3.13"
- **Estado:** ⬜ PENDING

---

## 🟡 DEFER (not in plan — documented for reference)

| ID | Razón |
|----|-------|
| MKT-14 | Case studies — contenido humano |
| TSK-106 | GitHub Discussions — no verificable desde repo |
| NUEVO-01 | README hero — contenido visual/humano |
| NUEVO-07 | Migration tools — esfuerzo 3-5d, no bloqueante |
| NUEVO-08 | Learning path — contenido humano |
| NUEVO-10 | Benchmark suite publish — 3-5d |
| TSK-107 | Community showcase — contenido humano |
| LEG-01 | Trademark — trámite externo |
| MKT-03/04/05/10/15/16/17 | Contenido humano/campañas |
| TSK-103/104 | Benchmarks/demo agent — post-launch |
| DEVOPS-12 | PyPI signing — devops, no bloqueante |
| DEVOPS-10 | Windows signing — necesita cert externo |
| SEC-14 | Migrar bincode → postcard/rkyv — alto riesgo, serialización |
| WEB-02 | False claims landing — contenido humano |
| WEB-03 | Async WAL batching — 2-3d perf optimization |
| WEB-04 | Storage format versioning — 3-5d heavy |
| DEVOPS-15 | Mover features heavies — optimización compilación |
| TEST-11/12 | Frontend/security tests — expansión |
| DOC-20 | mdBook adoption — docs infra |
| VFY-009 | 637 inline styles — 3-5d cosmético |
| VFY-011 | ACID Phase 3 MVCC — 3-5d heavy |
| DRV-001 | search.rs 1085L god file — 2-3d refactor |
| DRV-005 | SDK unit tests — expansión tests |
| DRV-019 | 14 .expect() hot-path SIMD — micro-optim|
| DRV-028/029/030 | LRU, cache-key, conversores — funciona, refactor si necesario |
| DRV-034 | 30+ try-catch — funciona, refactor si crece |
| DRV-036 | _mapRecord validación — bajo impacto |
| DRV-038 | TS numeric fields string — breaking change |
| DRV-042 | Test duplicación wasm — infra tests |
| DRV-052 | Metrics no reportadas — feature gap |
| DRV-060/066/072 | No setter namespace — YAGNI |
| DRV-061/067/073/078/082/089/095/100/107/113 | Test coverage expansión |
| DRV-083/084 | Dedup/delete_col — feature gap |
| NUEVO-11/12/13 | WASM features — post-launch |
| NUEVO-16/17/18 | PQ/LSM/sparse — muy alto esfuerzo |
| CLI-01 | CLI polish — 3-5d |
| DEVOPS-HOMEBREW | Homebrew — post-launch |
| DEVEX-DEMO/EXAMPLES | Demo/examples — post-launch |
| TSK-107b/ENT-04/BIZ-01 | Enterprise — post-launch |
| COM-02/03/04 | Discord — necesita acceso Discord |
| NUEVO-19/20/21 | Mover/misc — post-launch |
| NUEVO-15 | Code coverage — ✅ DO (Task 97) |
| PUBLICAR crates.io | Post-launch |
| WEB-001 | Re-add demo — post-launch |

---

===
**This plan file was created by Prompt 0 (backlog-executor) on Jul 14, 2026.**
**98 tasks ✅ DO, ready for execution via `iter-prompt.md` or harness.**
===

=== RECITATION ===
Objetivo activo: Task 4 — DRV-102 Langchain missing GIL release
Estado: completed
Última acción: wrapped add_texts, similarity_search_by_vector, delete in py.detach() (pyo3 0.29 API); fixed __init__.py __version__ export
Resultado: ✅ cargo check -p vantadb-langchain passes, cargo build passes, commit 3cc6888
Próxima acción: Task 5 — DRV-109 LlamaIndex missing GIL release (byte-for-byte identical code)
Contrato: "cargo check -p vantadb-langchain pasa, tests pasan"
Próxima tarea si completa: Task 5 — DRV-109 LlamaIndex missing GIL release
=== END RECITATION ===
