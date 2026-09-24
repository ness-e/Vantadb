# SHOW-03 — prototipo estrella RAG-sobre-PDFs (100% local)

> **Plan:** `docs/dev/plans/2026-09-19-publicacion.md` (Wave1, primera en secuencia) · **Ruta:** vanta-worker
> **Appetite/Branch/Commit:** 3d / develop / `feat: SHOW-03 — ...` · **Estado:** ⏳ IN PROGRESS
> **SDP:** test-driven-development · systematic-debugging · documentation-and-adrs · incremental-implementation · context-engineering · source-driven-development · doubt-driven-development · api-and-interface-design (SDP v2 BUILD, §5)

## 1. TAREA

**Objetivo:** prototipo estrella RAG-sobre-PDFs 100% local (subir PDF → chunk → embed → chat con citas). Demo que vende el caso RAG local con citas; sin ella el anuncio no tiene prueba pública (Gate Justificación del plan).

**Contrato exacto (plan):** 1 comando, sin credenciales: subir PDF → chunk → embed → chat con citas verificables (assert mecánico: la cita existe en el PDF) + README del ejemplo.

**Acceptance criteria:**
- (a) 1 comando sin credenciales (`python examples/rag_pdf_chat/rag_pdf_demo.py`, exit 0, 100% local).
- (b) Cita verificable contra el PDF con assert mecánico (no visual): la cita devuelta por el chat es substring del texto extraído del PDF — el demo lo aserta en runtime y los tests lo asertan en pytest.
- (c) README del ejemplo (`examples/rag_pdf_chat/README.md`: 1 comando + salida esperada + por qué es determinista/local).

**Spec ingesta PDF (MGR-25, alcance leído primero):** MGR-25 (`docs/dev/Backlog.md:817`) es investigación+spec de ingestores txt/json/csv→html→pdf→docx — **cero implementación**. `wiki_ingest` hoy solo `.md` (`src/wiki/sources.rs:82`, verificado por codegraph en DISCOVERY). Decisión del slice: chunk-embed-chat con lo que haya — extractor PDF mínimo **stdlib-only dentro del ejemplo** (no es ingestor del core, no toca `src/`), fixture PDF simple con texto extraíble. Implementar un parser nuevo en el core = MGR-25 = **prohibido acá**.

## 2. ARCHIVOS

**Clave (con :línea):**
- `examples/rag_pdf_chat/rag_pdf_demo.py` (nuevo — extractor + chunk + embed + ingest + chat + asserts)
- `examples/rag_pdf_chat/test_rag_pdf_chat.py` (nuevo — 3 tests e2e deterministas)
- `examples/rag_pdf_chat/README.md` (nuevo — 1 comando + salida + determinismo)
- `examples/rag_pdf_chat/fixture.pdf` (nuevo — PDF mínimo con texto extraíble conocido)

**Relacionados (callers/callees codegraph_explore + trace):**
- `examples/agent_memory_cli/agent_memory_demo.py:1-94` (patrón demo 1-comando SHOW-04 — estructura a seguir: argparse `--db/--keep-db`, 2 fases, asserts de contenido, `Client`, `flush/close`)
- `examples/agent_memory_cli/test_agent_memory_cli.py:1-84` (patrón 3 tests: contenido + search hit + 1-comando subprocess)
- `examples/python/haystack_documentstore.py:16-68` (referencia lectura: `Client(db_path)`, `put(namespace, key, content, metadata, vector)`, `search(namespace, query_vector, text_query, top_k)`)
- `src/wiki/sources.rs:82` (límite actual: solo `.md` — evidencia de que no hay ingestor PDF en el core)
- MGR-25 (`docs/dev/Backlog.md:817` — alcance ingestores; esta fila = demo, MGR-25 = diseño)

**Prohibidos (WIP ajeno que NO se toca):** implementar ingestores nuevos en el core (MGR-25) · credenciales/LLM externo (100% local obligatorio) · `reparacion.bat` · `.opencode` · `Justfile` · `ocr-*` · `completions/*` · `desktop/src-tauri/Cargo.lock` · stash@{0} GOV-C4 · `docs/dev/Backlog.md` · plan file (solo recitation al cierre) · `C:/Users/Eros/.vantadb*` (datos vivos) · `src/` Rust core (cero cambios) · `web/` (SHOW-02 cerrado).

## 3. DEPENDENCIAS

**Wave:** Wave1 primera en secuencia (Wave0 ✅ ×2: FIND-98-retry + SHOW-02; DIST-10/14 después, disjuntos: examples vs docs).
**Bloqueantes:** ninguno.
**Stop del plan:** sin vía local en 3d → DEFER con diagnóstico (no meter dependencias nuevas). Vía local SÍ existe (stdlib + `vantadb.Client` con vectores explícitos, patrón SHOW-04 verificado) → no aplica el stop.
**Task previa / NextTask:** SHOW-02 ✅ → SHOW-03 (esta) → DIST-10/14 (la ejecuta el orquestador, no yo).

## 4. REFERENCIAS

**Rules del área (lectura completa antes de ACT):** `.opencode/rules/server-mcp.md` (R-1..R-3, demo corre sobre el mismo motor que las tools MCP) · `.opencode/rules/python-bindings.md` (R-1/R-2, uso de `Client.put/search` con vectores explícitos, GIL-released).
**Refs:** `.opencode/references/definition-of-done.md` (standing checklist + DoD v1) · `test-suite.md` (pytest para SDK) · `testing-patterns.md` (AAA, naming) · `clean-code-clean-architecture.md` Ap. V (norma transversal).
**Commands:** `pipeline.md` (ejecución) · `audit.md` (verify post-tarea).
**SPEC.md raíz:** Success Criteria (usuario nuevo 1 comando → `fallback:false` + recuerdo por sinónimo <30 min) — el demo usa vectores explícitos (sin proveedor), luego `fallback` no aplica; se cita como N/A-motivado.
**Tabla Spec (símbolos públicos nuevos):** N/A — demo aislada en `examples/`, cero símbolos públicos nuevos, cero cambios de API. Decisión registrada: no agregar API (YAGNI/ponytail).

## 5. SKILLS (SDP Paso 0b — `campaign_discover_skills_v2` phase=BUILD, keywords [rag, pdf, chunk, embed, citas, demo, local, e2e])

| Skill | Cuándo aplica (1 línea) |
|---|---|
| test-driven-development | Demo como e2e: RED (test cita-verificable falla) → GREEN (demo mínima) → REFACTOR |
| systematic-debugging | Si el extractor falla en el fixture o un test falla: root-cause antes de parches |
| documentation-and-adrs | README del ejemplo + decisión "sin API nueva" registrada acá (§4) |
| incremental-implementation | 1 slice vertical delgado (extract→chunk→embed→chat) + commit atómico |
| context-engineering | Context pack del slice: SHOW-04 como ejemplo-patrón, Client API verificada |
| source-driven-development | Client.put/search verificados contra `help()` vivo, no memoria del modelo |
| doubt-driven-development | Stakes medios (showcase público): verificación adversarial de la cita |
| api-and-interface-design | Garantizar que NO se agrega API pública (decisión N/A de §4) |

## 6. HERRAMIENTAS+MCP

**Comandos exactos:**
- `python examples/rag_pdf_chat/rag_pdf_demo.py` (contrato 1-comando; `PYTHONUTF8=1` por codepage cp1252 vs smokes Python — precedente FIND-114/116)
- `$env:PYTHONUTF8=1; python -m pytest examples/rag_pdf_chat/test_rag_pdf_chat.py -v` (e2e determinista)
- `campaign_verify_cmd` para el gate (BUG exit -1 conocido → bash directa + mención en RESULTADO)
- Cargo `-j 2` si compila (no aplica: cero cambios Rust; se declara N/A)

**MCP:** codegraph_explore (ya usado en DISCOVERY para ingestores/límites) · campaign (task-system) · ocr CLI en CIERRE (`pwsh dev-tools/ocr-review.ps1`; Critical/High=bloquea).
**Internet:** SOLO si formato PDF incierto → N/A (fixture propio, bytes controlados; sin red → igual aplica: cero citas externas, cero deuda TSYS-13).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — generado en DISCOVERY)

- `src/wiki/sources.rs:22-121`: `SourceFile{rel_path, content}`, `scan_local_sources` solo `.md` (`:82` skip non-.md), budget 28k chars, guard `ensure_within_root`. Sin ingestores PDF en el core → el demo no puede reusar ingesta PDF del core (no existe).
- `examples/agent_memory_cli/`: patrón demo 1-comando SHOW-04 (argparse, 2 sesiones mismo dir, asserts contenido, vectores fijos `[1.0,0.0,0.0]`, `MEMORY_LIMIT=128MB`, tests AAA con `tempfile.mkdtemp` aislado + subprocess 1-comando).
- `examples/python/haystack_documentstore.py:61-68,142-148`: `Client(db_path, memory_limit_bytes=1G)`, `put(ns, id, content, metadata, vector)`, `search(ns, query_vector, text_query, top_k)`, hit attrs `key/payload/metadata/score`.
- `Client` vivo (`help()`): `put(ns, key, payload, metadata, vector, ttl_ms)`, `search(ns, query_vector, filters, text_query, top_k, ...)`, `memory.get(ns, key)`, `flush/close`. Sin `unwrap` en Python (binding ya mapea a excepciones).
- **Veredicto impacto (Regla 0):** archivos nuevos aislados en `examples/rag_pdf_chat/`; cero callers entrantes (nadie importa el demo); cero cambios a archivos existentes salvo este task file; rollback = `git revert` del commit o borrar el dir. Impacto: nulo fuera del ejemplo.

## 8. INVESTIGACIÓN PROBLEMA

**Vía de ingesta PDF con lo que hay:** no hay extractor PDF en el repo (ni core ni Python). Opciones: (a) dependencia nueva (`pypdf`) — prohibida por el stop del plan ("no meter dependencias nuevas"); (b) fixture `.txt` renombrado — deshonesto (el contrato dice PDF); (c) extractor mínimo stdlib-only dentro del ejemplo + fixture PDF generado con bytes controlados que ese extractor garantiza leer. **Elegida (c):** regex sobre streams `BT…ET` con operadores `Tj/TJ` (el fixture solo usa `(texto) Tj` ASCII + `TJ` con arrays simples) + `chat` = retrieval híbrido (`text_query` keyword + vector determinista por hash de tokens) y la cita = payload del chunk ganador, verificada con `assert cita in texto_pdf`.
**Embed 100% local sin credenciales:** vector determinista 8-d por hash de tokens (normalizado) — sin ONNX, sin red, sin proveedor; el motor nunca llama afuera (mismo truco que SHOW-04 con vectores fijos, pero derivado del texto del chunk para que el retrieval discrimine).
**Chunk:** tamaño fijo ~500 chars con overlap 50, metadata `source={pdf,page,chunk}` (proveniencia estilo MGR-25 sin implementar MGR-25).

## 9. INVESTIGACIÓN INTERNET

N/A — fixture propio con bytes controlados; extractor y formato verificados mecánicamente por los tests (el test falla si el extractor no lee el fixture). Cero URLs citadas → cero gate TSYS-13, cero deuda.

## 10. VALIDACIÓN+CIERRE

- Verify contrato: demo exit 0 + `CITA VERIFICADA` en stdout + pytest 3/3 + `campaign_verify_cmd` (o bash directa por bug -1 + mención).
- OCR delegation (`pwsh dev-tools/ocr-review.ps1`; Critical/High=bloquea) + DoD 3 niveles (Correctness/Quality/Integration-Docs/Ship) + commit conventional con task ID (staging SELECTIVO solo `examples/rag_pdf_chat/` + este task file). NO PUSH (solo vanta-lead).
- P2-01 lo hace el orquestador (no yo). Gates D/V/C vía `question` (D: no-disparado — blast radius 4 archivos nuevos, sin API pública; V/C al cierre si aplica).
- RESULTADO §7 obligatorio al final.

## Impacto mapeado (Regla 0) — MUST antes del primer edit

- **Archivos leídos completos:** `src/wiki/sources.rs` (vía codegraph verbatim :15-121) · `examples/agent_memory_cli/agent_memory_demo.py` · `.../README.md` · `.../test_agent_memory_cli.py` · `examples/python/haystack_documentstore.py` · `examples/README.md` · `docs/dev/Backlog.md` filas SHOW-03/MGR-25 · plan file Wave1 · rules `server-mcp.md` + `python-bindings.md` · `definition-of-done.md`.
- **Referencias hacia dentro (qué importa el slice):** `vantadb.Client` (binding existente, sin cambios) — `put/search/memory.get/flush/close`.
- **Referencias entrantes (quién depende del slice):** nadie (dir nuevo, sin importers).
- **Veredicto:** impacto nulo fuera de `examples/rag_pdf_chat/`; seguro para ACT.

## Steps atómicos (~100 líneas c/u)

- [x] **Step 0 — DISCOVERY + task file** (este archivo; contrato + blast radius + Context Save Point) ✅
- [x] **Step 1 — Slice vertical: demo + fixture + tests + README** ✅ (RED: ModuleNotFoundError sin demo; GREEN: 3/3 pytest + demo exit 0 `CITA VERIFICADA`)
- [x] **Step 2 — CIERRE** ✅ (pytest 6/6 con SHOW-04 sin regresión · `cargo fmt --check` EXIT=0 · `campaign_verify_cmd` bug exit -1 → bash directa · OCR advisory sin Critical/High · commit selectivo, NO PUSH)

## Context Save Point

Si la sesión se interrumpe: el trabajo vive en `examples/rag_pdf_chat/` (sin commit hasta el cierre) + este task file (steps marcados). Reanudar en el primer step ⬜ (Step 1 o 2). Invariantes: sin `src/`, sin deps nuevas, sin credenciales. Comando de re-verificación: `$env:PYTHONUTF8=1; python -m pytest examples/rag_pdf_chat/test_rag_pdf_chat.py -v`.
