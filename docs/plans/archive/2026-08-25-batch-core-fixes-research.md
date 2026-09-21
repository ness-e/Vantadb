# Plan de Ejecución: Batch Core Fixes + Research P38 (2026-08-25)

> **Campaign ID:** aa2cde2b-e52f-4dae-910a-b274373a5bda
> **Inicio:** 2026-08-25
> **Estado:** ✅ COMPLETADO (reanudado, 9/9)
> **Fuente:** docs/Backlog.md (selección del lead + confirmación del usuario 2026-08-25)
> **Modo:** FAIL_MODE=parallel, MAX_CONCURRENT=3

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 9 |
| 🟡 DEFER | ~30 (P34/P37 ya cubiertos, Pro/FUT/RES menor) |
| ❌ SKIP | 2 (AUD-043 resuelto, REVIEW-07 duplicado) |
| 🔴 BLOQUEADO | 1 (CORE-01 ADR) |

Status: ⬆️ uphill = 3 (RES-01/02/03 research requieren DISCOVERY) · ⬇️ downhill = 6

> **Nota:** el conteo del backlog (75) está inflado — P34/P37 (UX/DAUD) y MOD-15/FIND-11/17 ya fueron completados en batches previos con task files agrupados. Este plan cubre los hallazgos REALES restantes (AUD-044..047, FIND-22/23) + research P38 prioritaria.

## Tasks

### Task 1: FIND-23 — vanta-http-map envía namespace vacío en ingest/get

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴 (detectado por E2E-VISUAL)
- **Archivos clave:** `desktop/src/vanta-http-map.ts:93,114`
- **Verificación real:** ✅ CÓDIGO-REAL — `item.namespace ?? ""` (línea 93) y `q.namespace ?? ""` (114): el server embebido rechaza ingest/get con namespace vacío (ValidationError). El mapping WASM sí defaulta `DEFAULT_NS`. Bug real.
- **Gate Justificación:** rompe ingest en web build embebido; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** ingest/get con namespace omitido usan `DEFAULT_NS` en vanta-http-map; `npm run build` (desktop) exit 0
- **Task file:** `skills/campaign-executor/tasks/FIND-23.md`
- **Estado:** ✅ COMMITTED `fix(desktop)` - DEFAULT_NS en http-map, node test 21/21

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) DEFAULT_NS no existe como constante en TS.

### Task 2: AUD-044 — shim MmapMut sin write-back: compact_layout pierde datos

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴 (data loss)
- **Archivos clave:** `src/storage/vfile_mmap.rs:131-133`, `src/storage/archive.rs:95,135`
- **Verificación real:** ✅ CÓDIGO-REAL — `MmapMut.flush()` (shim no-memmap2) es no-op; `compact_layout` (archive.rs:95,135) llama `tmp_mmap.flush()` esperando write-back al disco → en builds sin memmap2 el tmp file se renombra SIN el buffer escrito. Data loss real.
- **Gate Justificación:** data loss en compact_layout sin feature memmap2; fix acotado (flush del shim debe escribir el buffer al file).
- **Gate Result:** ✅ DO
- **Contrato:** test: compact_layout sin memmap2 → los datos sobreviven al reopen; `cargo nextest run -p vantadb archive` (o compact) pasa
- **Task file:** `skills/campaign-executor/tasks/AUD-044.md`
- **Estado:** ✅ COMMITTED `fix(storage)` - shim flush write-back, compact_layout data loss resuelto

  **Pre-mortem:**
  - Fallo 1: el shim MmapMut no guarda el File para flush (solo AlignedBytes) — necesita guardar handle
  - Fallo 2: modificar el shim rompe la API parity con memmap2
  - **Stop conditions:** si el fix requiere cambiar la estructura MmapMut (guardar File), hacerlo manteniendo API. **Cynefin:** 🟨 complicado — storage FFI. **Top 3 riesgos:** (1) estructura shim; (2) API parity; (3) data loss residual.

### Task 3: AUD-045 — clones de vector en hot path IVF search

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `src/index/ivf.rs` (IvfEntry.vector Vec<f32>, search loop)
- **Verificación real:** ✅ CÓDIGO-REAL — `IvfEntry { vector: Vec<f32> }` (ivf.rs:18-22); search clona vectores por candidato. Es PERF → Regla 9 obligatoria (bench before/after).
- **Gate Justificación:** hot path IVF; effort 🟡 con bench.
- **Gate Result:** ✅ DO
- **Contrato:** bench before/after documentado (cargo bench ivf o canonical_p99); sin regresión; `cargo nextest run -p vantadb ivf` pasa
- **Task file:** `skills/campaign-executor/tasks/AUD-045.md`
- **Estado:** ✅ COMMITTED `perf(index)` - f32_slice_similarity, bench -59%, 21/21

  **Pre-mortem:**
  - Fallo 1: cambiar Vec<f32> a Arc/slice rompe deserialize_from_bytes
  - Fallo 2: bench sin dataset representativo → regresión no medida
  - **Stop conditions:** si el cambio de representación es invasivo (serialización), documentar y DEFER. **Cynefin:** 🟨 complicado — perf + serialización. **Top 3 riesgos:** (1) serialización; (2) bench; (3) regresión.

### Task 4: AUD-046 — fan-out all-namespaces en list HTTP trunca a NS_CAP=10000

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `src/cli_server.rs` (list HTTP handler)
- **Verificación real:** 🟡 VERIFICAR — backlog: list HTTP trunca por namespace a NS_CAP=10000 sin señal al cliente; re-fan-out completo por página. Confirmar en DISCOVERY.
- **Gate Justificación:** datos truncados silenciosamente vía HTTP; effort 🟡.
- **Gate Result:** ✅ DO
- **Contrato:** list HTTP no trunca silenciosamente (paginación completa o señal al cliente); test e2e con >10000 records (o el límite actual)
- **Task file:** `skills/campaign-executor/tasks/AUD-046.md`
- **Estado:** ✅ COMMITTED `fix(server)` - truncated_namespaces aditivo, 59/59
- **last-synced:** 2026-08-25T00:00

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) cambio de API del handler.

### Task 5: AUD-047 — duplicación ~50 líneas del match métrico en layer.rs

- **Appetite:** max 2h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `src/index/search/layer.rs` (match Cosine/Euclidean/SparseDot)
- **Verificación real:** 🟡 VERIFICAR — backlog: bloque match métrico duplicado (~50 líneas), anidado peor desde f2c2141e. Confirmar en DISCOVERY (codegraph).
- **Gate Justificación:** refactor de duplicación; effort 🟢. Hot path → verificar suite completa.
- **Gate Result:** ✅ DO
- **Contrato:** helper extraído; `cargo nextest run -p vantadb search_layer` + suite index pasa; clippy/fmt
- **Task file:** `skills/campaign-executor/tasks/AUD-047.md`
- **Estado:** ✅ COMMITTED `refactor(index)` - metric_score closure, -35 lineas, 358/358

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) hot path search.

### Task 6: FIND-22 — documentar exclusiones del fast gate en CI_POLICY.md

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `docs/operations/CI_POLICY.md`, `dev-tools/verify.ps1`
- **Verificación real:** 🟡 VERIFICAR — backlog: formalizar las 3 exclusiones de tests del fast gate (deserialize_absurd_node_count, etc.). Confirmar en DISCOVERY.
- **Gate Justificación:** docs CI; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** CI_POLICY.md documenta las exclusiones; docs coverage 0 gaps
- **Task file:** `skills/campaign-executor/tasks/FIND-22.md`
- **Estado:** ✅ COMMITTED `docs(ci)` - exclusiones documentadas, coverage 0 gaps

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio.

### Task 7: RES-01 — ACID Phase 4a: WAL v2 con WalRecord::Prepare (research)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Archivos clave:** `src/wal.rs`, `src/wal_sharded.rs`, `docs/research/`
- **Verificación real:** 🟡 VERIFICAR — backlog P38: keystone de rollback multi-capa; habilita errores truthful y MVCC. Research → análisis → plan.
- **Gate Justificación:** ⬆️ uphill — research con decisión de diseño. El output es un doc de investigación/análisis + plan, NO código.
- **Gate Result:** ✅ DO (research)
- **Contrato:** doc de investigación generado en docs/research/ con análisis + plan; hallazgos ruteados a backlog si hay decisión
- **Task file:** `skills/campaign-executor/tasks/RES-01.md`
- **Estado:** ✅ COMMITTED `docs(research)` - GO condicional tras flag wal_prepare + bench

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟧 complejo — requiere experimentar. **Top 3 riesgos:** (1) scope; (2) decisión ADR.

### Task 8: RES-02 — backup/restore físico completo (research)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Archivos clave:** `src/storage/`, `docs/research/`
- **Verificación real:** 🟡 VERIFICAR — backlog P38: gap de backup/restore (MCP-34 DEFER parcial). Research → análisis → plan.
- **Gate Justificación:** ⬆️ uphill — research. Output: doc + plan.
- **Gate Result:** ✅ DO (research)
- **Contrato:** doc de investigación + plan en docs/research/; hallazgos ruteados
- **Task file:** `skills/campaign-executor/tasks/RES-02.md`
- **Estado:** ✅ COMMITTED `docs(research)` - restore físico recomendado (S1-S5), MCP-34b/FIND-25/26 ruteados

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟧 complejo. **Top 3 riesgos:** (1) scope; (2) solapamiento con MCP-34a.

### Task 9: RES-03 — go/no-go session layer VantaDB MCP (research + decisión)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-mcp/`, `docs/research/`
- **Verificación real:** 🟡 VERIFICAR — backlog DEC-01: roadmap 4 fases (session cache, Claude Code plugin, sync/improve, lesson extraction). Research go/no-go.
- **Gate Justificación:** ⬆️ uphill — requiere decisión (Regla 5 ADR). Output: doc + recomendación go/no-go para el owner.
- **Gate Result:** ✅ DO (research)
- **Contrato:** doc de investigación con go/no-go recomendado; DEC-01 resuelto o ruteado
- **Task file:** `skills/campaign-executor/tasks/RES-03.md`
- **Estado:** ✅ COMMITTED `docs(research)` - F1/F3/F4 no-go, F2 defer docs-only; DEC-01 resuelta defer-as-scoped

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟧 complejo. **Top 3 riesgos:** (1) decisión sin owner.

## DEFER

| ID | Motivo |
|----|--------|
| UX-02..19, DAUD-01..08 | ✅ YA COMPLETADOS en batch P40 (task files agrupados) — filas del backlog a limpiar |
| MOD-15, FIND-11/17 | ✅ YA COMPLETADOS en batch P40 |
| MOD-05 | Deprecar InMemoryEngine — refactor grande |
| REVIEW-10/12 | God-file splits — refactors grandes |
| FIND-20/21 | Tauri window/menu — investigación dedicada |
| PRO-01..06, FUT-02..11, RES-04..15, DEC-01/02 | Roadmap / research menor — fuera de este batch |

## SKIP

| ID | Motivo |
|----|--------|
| AUD-043 | ✅ ya resuelto (FIND-30: `_ns` en cli_server.rs:1330) |
| REVIEW-07 | Duplicado (BND-06 resuelto) |

## BLOQUEADO

| ID | Motivo |
|----|--------|
| CORE-01 | Persistencia Binary on-disk — requiere ADR de formato (Regla 5) |

## Waves

- **Wave 0**: FIND-23 (1) · AUD-044 (2) · AUD-047 (5)
- **Wave 1**: AUD-045 (3) · AUD-046 (4) · FIND-22 (6)
- **Wave 2**: RES-01 (7) · RES-02 (8) · RES-03 (9)

> MAX_CONCURRENT = 3. Sub-agentes NO commitean; el lead verifica mecánico y commitea por tarea. RES-01/02/03 son research (vanta-research, read-only → digest) o vanta-docs según el output esperado.

## Notas

- plan-adjust [2026-08-25]: creado desde docs/Backlog.md. El backlog tenía 75 filas pero ~30 ya cubiertas por batches previos — este plan toma los hallazgos reales restantes + research P38 prioritaria.
- Paso 0: AUD-044 (shim MmapMut flush no-op → data loss) y AUD-045 (IvfEntry Vec clones) verificados reales; FIND-23 (namespace vacío HTTP) confirmado; AUD-043 ya resuelto → SKIP.
- CodeGraph auto-sync deshabilitado (lock de otro proceso) — sub-agentes deben leer archivos directos.
- Otra sesión activa está trabajando (commits fix(build) Justfile/Dockerfile, fix(docs)) — no tocar esos archivos.
- ⬆️ uphill = 3 (RES-01/02/03) · ⬇️ downhill = 6

=== RECITATION FIND-23 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: FIND-23: vanta-http-map envía namespace vacío en ingest/get
Estado: completed
Última acción: Fix aplicado: DEFAULT_NS="default" + `|| DEFAULT_NS` en 6 sitios de vanta-http-map.ts (ingest/search/get/get_version/versions/delete). Test RED→GREEN en vanta-http-map.test.ts. Cleanup WIP guard: FIND-11/UX-POLISH sync a completed.
Resultado: OK
Próxima acción: Lead: verifica mecánico (node --test + npm run build), commitea solo archivos de FIND-23 + cleanup FIND-11/UX-POLISH, ejecuta skill progreso
Contrato: verificacion: cd desktop && node --test src/vanta-http-map.test.ts = 21/21 pass; cd desktop && npm run build = exit 0; node --test src/vanta-wasm-map.test.ts = 14/14 (idempotencia) | evidencia: claim='ingest/search/get con namespace omitido usan DEFAULT_NS' evidencia='vanta-http-map.ts:25 const DEFAULT_NS, :95/:116/:246/:255/:265/:292' confianza=alta; claim='RED verificado antes del fix' evidencia='node --test output 20 pass / 1 fail (test FIND-23) antes de editar vanta-http-map.ts' confianza=alta; claim='DEFAULT_NS=default canonico' evidencia='vanta-wasm-map.ts:45, native.rs:34, types.rs:369' confianza=alta | artefactos: desktop/src/vanta-http-map.ts, desktop/src/vanta-http-map.test.ts, .opencode/skills/campaign-executor/tasks/FIND-23.md | invariantes: cero cambios src-tauri/web/core; namespace explicito intacto (tests existentes verdes); wasm-map sin cambio de comportamiento | deuda: ninguna; DEFAULT_NS duplicado wasm/http aceptado | queda_pendiente: lead commitea (NO incluir archivos de agentes paralelos: AUD-047.md, layer.rs); review P2-01 vanta-review; skill progreso fila Backlog FIND-23
Próxima tarea si completa: AUD-044
=== END RECITATION ===

=== RECITATION AUD-047 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: AUD-047: extraer helper del match métrico duplicado en src/index/search/layer.rs
Estado: completed
Última acción: Implementación completa: closure metric_score (Fn inline) extraído; 2 bloques match metric (Cosine/Euclidean/SparseDot, ~54 líneas) reemplazados por 2 llamadas; neto -35 líneas (409→374). No se tocó aritmética ni fast_similarity. Verify full: fmt ✅, check ✅, nextest 358/358 ✅, clippy workspace all-targets ✅.
Resultado: OK
Próxima acción: Lead verifica mecánico y commitea (refactor:) — el worker NO commitea por regla
Contrato: verificacion: cargo nextest run -p vantadb -E 'test(search_layer)|test(index)' → 358/358 passed; cargo check -p vantadb ✅; cargo fmt --check ✅; cargo clippy --workspace --all-targets --all-features -- -D warnings ✅; evidencia: layer.rs:35-55 (closure), :97 y :247 (call sites) — diff en worktree sin commit; invariantes: misma aritmética, misma firma pub(crate) search_layer, no tocar fast_similarity; deuda: ninguna; artefactos: .opencode/skills/campaign-executor/tasks/AUD-047.md actualizado
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION AUD-044 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: Fix shim MmapMut flush no-op → data loss en compact_layout (builds sin memmap2)
Estado: completed
Última acción: Implementación completa: write_back en shim MmapMut (flush/flush_async/flush_range), seek(0) en map/map_mut (posición compartida), flush en grow_to (no remap_mut — replace_backing_file tiene buffer stale por diseño), 4 tests de regresión (shim flush, grow preserve, compact reopen, compact reorder). 3 ciclos de review adversarial vanta-review: 2 hallazgos válidos arreglados, 2 misreads reconciliados con evidencia directa. Verify full: shim 1932/1932, default 2062/2062, fmt, clippy -D warnings, wasm32 check — todos ✅
Resultado: OK
Próxima acción: Lead: commitear src/storage/vfile_mmap.rs + vfile.rs + archive.rs + task file; ejecutar progreso (migrar Backlog AUD-044 → docs/avance)
Contrato: verificacion: cargo nextest run -p vantadb --no-default-features --features fjall,roaring,cli,advanced-tokenizer,sysinfo,rayon,fs2 --lib = 1932/1932 + cargo nextest run -p vantadb = 2062/2062 + cargo fmt --check = 0 + cargo clippy --workspace --all-targets --all-features -- -D warnings = 0 + cargo check -p vantadb-wasm --target wasm32-unknown-unknown = 0 | evidencia: claim=flush shim escribe buffer a disco evidencia=test shim_mmap_mut_flush_writes_buffer_to_disk (RED→GREEN, vfile_mmap.rs) confianza=alta; claim=compact_layout sin memmap2 sobrevive reopen evidencia=test_compact_layout_disk_backed + reorder test (archive.rs) confianza=alta; claim=root cause confirmado evidencia=sync_to_mmap_preserves_data FAIL pre-fix / PASS post-fix (index/core.rs:321) confianza=alta; claim=builds memmap2 intactos evidencia=2062/2062 default + cfg-gate confianza=alta | artefactos: .opencode/skills/campaign-executor/tasks/AUD-044.md | invariantes: API shim intacta (pub(crate), firmas iguales), buffer.len()==file.len() en map time, File clonado dropea con mapping (Windows rename), builds memmap2 no tocados, flush≠fsync (no barrera durabilidad) | deuda: ninguna (Regla 6 saldo 0); colateral resuelto inline (posición compartida en clones, test_vfile_grow_to_on_disk pre-existente FAIL ahora pasa) | queda_pendiente: lead verifica mecánico y commitea (Regla 2 — NO commiteé); progreso pendiente para el lead; otra sesión activa tocó layer.rs/vanta-http-map/FIND-11/UX-POLISH — NO incluir en este commit
Próxima tarea si completa: AUD-045
=== END RECITATION ===

=== RECITATION AUD-045 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: AUD-045: eliminar clones de vector completo per-candidate en hot path IVF search
Estado: completed
Última acción: Fix de 4 sitios en src/index/ivf.rs: calculate_similarity(&VectorRepresentations::Full(x.clone())) → f32_slice_similarity(slice) (helper pub(crate) preexistente, mapper.rs:201, ya usado por search/layer.rs). Search (scoring centroides + per-candidato), build (asignación Lloyd + convergencia). Cero cambios de serialización/layout/API.
Resultado: OK
Próxima acción: Lead: commitear SOLO src/index/ivf.rs + tasks/AUD-045.md + plan file (worktree compartido con AUD-046/FIND-22 — no incluir sus archivos); ejecutar skill progreso fila Backlog AUD-045
Contrato: verificacion: cargo nextest run -p vantadb -E 'test(ivf)' = 21/21 passed; cargo clippy --workspace --all-targets --all-features -- -D warnings = OK; cargo fmt --check = ivf.rs sin diffs | evidencia: claim='mejora medida Regla 9' evidencia='cargo bench -p vantadb --bench ivf_bench -- search --measurement-time 2: search −48%..−65% (p.ej. 100/10: 52.6ms→21.8ms; 100/1: 8.3ms→2.9ms); build k-means nlist=100 3.36s→0.96s (−71%), nlist=400 5.80s→2.29s (−61%); entorno i5-1235U/32GB/Win11/rustc 1.95; tabla completa en tasks/AUD-045.md' confianza=alta; claim='recall sin regresión' evidencia='recall@10 before/after ~igual (tabla task file)' confianza=media; claim='serialización/layout intactos' evidencia='diff ivf.rs: serialize_to_bytes/deserialize_from_bytes/IvfEntry sin cambios' confianza=alta | artefactos: src/index/ivf.rs, .opencode/skills/campaign-executor/tasks/AUD-045.md | invariantes: no tocar archivos de sesiones paralelas (cli_server.rs/AUD-046, CI_POLICY.md/FIND-22, completions/, opencode.jsonc); API pública estable | deuda: ninguna | queda_pendiente: commit del lead + skill progreso
Próxima tarea si completa: AUD-046
=== END RECITATION ===

=== RECITATION FIND-22 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: FIND-22: formalizar exclusiones del fast gate en docs/operations/CI_POLICY.md
Estado: completed
Última acción: ACT+VERIFY: sección 'Fast Gate Test Exclusions' añadida a CI_POLICY.md (tabla 3 exclusiones RESOURCE-GUARD + nota nextest.toml default-filter ~55 bins + reglas de governance); last_reviewed→2026-08-25. Cross-check grep 3/3 + docs coverage 0 gaps.
Resultado: OK
Próxima acción: Lead: git add docs/operations/CI_POLICY.md + task file, commit 'docs: FIND-22 — formalizar exclusiones fast gate', skill progreso
Contrato: verificacion: rg cross-check 3/3 exclusiones (deserialize_absurd_node_count, test_search_with_bizarre_text_query, test_malformed_payload_extremely_large) doc↔verify.ps1 ✅ + pwsh scripts/validate-docs-coverage.ps1 → 0 gaps ✅ | evidencia: claim=3 exclusiones RESOURCE-GUARD documentadas con fuente/por qué/dónde/quién revierte evidencia='docs/operations/CI_POLICY.md:80-100 §Fast Gate Test Exclusions' confianza=alta; claim=no se cambió verify.ps1 ni nextest.toml evidencia='git diff solo CI_POLICY.md' confianza=alta; claim=tests viven en src/index/core.rs:414 y tests/security.rs:324/639 evidencia='grep en fuente' confianza=alta | artefactos: docs/operations/CI_POLICY.md, .opencode/skills/campaign-executor/tasks/FIND-22.md | invariantes: cero cambios de código; taxonomía EXPERIMENTAL/BEST-EFFORT/NON-CRITICAL/INFORMATIONAL intacta; mdBook include hereda automáticamente (docs/book/src/operations/CI_POLICY.md) | deuda: ninguna | queda_pendiente: lead commitea (docs: FIND-22) y ejecuta skill progreso
Próxima tarea si completa: AUD-045
=== END RECITATION ===

=== RECITATION AUD-045 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: AUD-045: eliminar clones de vector completo per-candidate en hot path IVF search
Estado: completed
Última acción: Tarea completa: fix de 4 clones via f32_slice_similarity (helper pub(crate) preexistente). Bench before/after documentado: search −48%..−65% segun config, build k-means hasta −71%. Verify: nextest ivf 21/21, clippy workspace -D warnings OK, fmt limpio en ivf.rs.
Resultado: OK
Próxima acción: Lead: commitear src/index/ivf.rs + tasks/AUD-045.md + plan; ejecutar skill progreso
Contrato: verificacion: cargo nextest run -p vantadb -E 'test(ivf)' = 21/21 passed; cargo clippy --workspace --all-targets --all-features -- -D warnings = OK; cargo fmt --check = ivf.rs sin diffs | evidencia: claim='clones eliminados sin cambio de API ni serializacion' evidencia='src/index/ivf.rs diff: 4 sitios calculate_similarity(Full(clone)) → f32_slice_similarity(slice), helper preexistente mapper.rs:201, serialize/deserialize intactos' confianza=alta; claim='mejora medida' evidencia='cargo bench -p vantadb --bench ivf_bench -- search --measurement-time 2: search -48%..-65%, build k-means nlist=100 -71%; entorno i5-1235U/32GB/Win11; numeros en .opencode/skills/campaign-executor/tasks/AUD-045.md' confianza=alta; claim='recall sin regresion' evidencia='tabla recall@10 before/after en task file (~igual)' confianza=media | artefactos: src/index/ivf.rs, .opencode/skills/campaign-executor/tasks/AUD-045.md | invariantes: NO tocar cli_server.rs/CI_POLICY.md/completions/opencode.jsonc (sesiones paralelas); serialize/deserialize_from_bytes y layout IvfEntry intactos; API publica estable | deuda: ninguna | queda_pendiente: lead commitea solo src/index/ivf.rs + tasks/AUD-045.md + plan file; skill progreso fila Backlog AUD-045
Próxima tarea si completa: AUD-046
=== END RECITATION ===

=== RECITATION AUD-046 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: AUD-046: list HTTP fan-out all-namespaces no trunca silenciosamente
Estado: completed
Última acción: Steps 1-3 completos: señal aditiva truncated_namespaces + helper puro merge_all_namespaces_pages + unit test semántica NS_CAP + assertion e2e del campo. Verify full verde (fmt/clippy/check/nextest 59/59). FIND-24 registrado. Plan+task files actualizados.
Resultado: OK
Próxima acción: Lead: commitear src/cli_server.rs + docs/Backlog.md + task file; ejecutar progreso
Contrato: verificacion: cargo nextest run -p vantadb --lib --features server -E 'test(v2_list) | test(merge_all_namespaces)' = 3/3 pass; cli_server suite 59/59; fmt/clippy/check = 0 | evidencia: claim='fan-out señaliza truncamiento' evidencia='helper merge_all_namespaces_pages + AllNamespacesListPage.truncated_namespaces (cli_server.rs)' confianza=alta; claim='wire format aditivo no-breaking' evidencia='VantaMemoryListPage SDK intacto, campo solo en struct local' confianza=alta; claim='e2e HTTP >10k inviable debug (timeout 30s vs list 67s medido)' evidencia='probe temporal + FIND-24 en Backlog; semantica NS_CAP cubierta por unit de pages sinteticas' confianza=alta | artefactos: src/cli_server.rs, docs/Backlog.md (FIND-24), task file AUD-046.md | invariantes: rama single-namespace intacta; VantaMemoryListPage SDK sin cambios; cli_server.rs NO dividido | deuda: FIND-24 (perf list + cursor cross-ns) | queda_pendiente: lead commitea (src/cli_server.rs + docs/Backlog.md + task file); skill progreso
Próxima tarea si completa: AUD-045 o siguiente pendiente wave 1
=== END RECITATION ===

=== RECITATION AUD-046 ===

Objetivo activo: AUD-046 — list HTTP fan-out all-namespaces no trunca silenciosamente
Estado: accept (worktree listo, sin commit)
Última acción: Implementado señal aditiva + helper + tests; verify full verde; FIND-24 registrado en Backlog
Resultado: OK
State: ACCEPT (desde: VERIFY)
Próxima acción: Lead: commitear src/cli_server.rs + docs/Backlog.md (fila FIND-24) + task file AUD-046.md; ejecutar progreso
Contrato: verificacion: cargo nextest run -p vantadb --lib --features server -E 'test(v2_list) | test(merge_all_namespaces)' = 3/3 pass (0.9s); nextest cli_server suite = 59/59; cargo fmt --check = 0; cargo clippy -p vantadb --all-targets --all-features -- -D warnings = 0; cargo check -p vantadb --features server = 0 | evidencia: claim='fan-out señaliza truncamiento' evidencia='helper merge_all_namespaces_pages cli_server.rs (next_cursor.is_some() => truncated_namespaces) + struct AllNamespacesListPage con campo aditivo' confianza=alta; claim='wire format no-breaking' evidencia='VantaMemoryListPage del SDK intacto; campo nuevo solo en struct local del fan-out; clientes tolerantes a campos extra' confianza=alta; claim='e2e HTTP >10k inviable en debug' evidencia='probe medido: put_batch 10_005=1s, db.list(limit 10k)=60-70s vs REQUEST_TIMEOUT=30s → FIND-24 en Backlog; semántica NS_CAP cubierta por unit test de pages sintéticas' confianza=alta | artefactos: src/cli_server.rs, docs/Backlog.md, .opencode/skills/campaign-executor/tasks/AUD-046.md | invariantes: rama single-namespace intacta; VantaMemoryListPage SDK sin cambios; cli_server.rs NO dividido; NS_CAP=10_000 sin cambio | deuda: FIND-24 (perf list O(ventana) ~6.7ms/nodo debug + re-fan-out por página + cursor cross-ns requiere SDK) | queda_pendiente: lead commitea (NO incluir archivos de agentes paralelos); review P2-01 vanta-review opcional; skill progreso
Próxima tarea si completa: AUD-045 o siguiente pendiente wave 1
=== END RECITATION ===

=== RECITATION RES-03 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: RES-03: go/no-go session layer VantaDB MCP (DEC-01)
Estado: in-progress
Última acción: DISCOVERY+research completos: mapeado lo existente (ThreadStore 6 tools MCP-32, assemble_with_recall MCP-31 context_assemble, scenes MCP-30, axioms MCP-33, genlog por-sesion, skills MCP); origen DEC-01 leido (COGNEE_EVALUATION.md §8-9); transporte MCP validado contra spec oficial 2025-06-18. Analisis por fase producido: F1 no-go (duplicado), F2 defer (docs-only hoy), F3 no-go, F4 no-go.
Resultado: PARTIAL
Próxima acción: Lead: persistir docs/research/res03-session-layer-gonogo.md y .opencode/skills/campaign-executor/tasks/RES-03.md desde el contenido inline entregado; commitear como docs:
Contrato: verificacion: doc persistido por lead en docs/research/res03-session-layer-gonogo.md con analisis por fase + go/no-go recomendado (contenido completo entregado en respuesta del agente); evidencia: claim=threads CRUD ya expuestos via MCP evidencia='vantadb-mcp/src/threads.rs:87-102 (6 tools: thread_create/send/get/list/delete/purge_expired)' confianza=alta; claim=context engine expuesto evidencia='vanta-memory/src/context_engine/engine.rs:199 assemble_with_recall + handlers/tools.rs:1893 context_assemble' confianza=alta; claim=scenes sandboxed por session_key ya existen evidencia='vanta-memory/src/core/scene/scene_tools.rs:104-200' confianza=alta; claim=axioms read/write/delete expuestos evidencia='handlers/tools.rs:1134,1139,1177' confianza=alta; claim=genlog por sesion ya existe evidencia='vanta-memory/src/core/memory_generation_log/store.rs:24-36' confianza=alta; claim=stdio es transporte estandar MCP y Streamable HTTP reemplazo HTTP+SSE evidencia='https://modelcontextprotocol.io/specification/2025-06-18/basic/transports (VERIFICADA 2026-08-25)' confianza=alta | artefactos: contenido de docs/research/res03-session-layer-gonogo.md + tasks/RES-03.md entregados inline al lead | invariantes: no tocar codigo; decision final del owner (Regla 5); doc en ingles | deuda: lead debe persistir 2 archivos desde el contenido entregado (agente sin tool de escritura) | queda_pendiente: lead persiste doc + task file, rutea DEC-01 a resuelto-con-recomendacion-defer en Backlog, decide si abre ADR
Próxima tarea si completa: RES-01
=== END RECITATION ===

=== RECITATION RES-01 ===
Campaign ID: aa2cde2b-e52f-4dae-910a-b274373a5bda
Objetivo activo: RES-01: ACID Phase 4a — WAL v2 con WalRecord::Prepare (research, output DOC no código)
Estado: in-progress
Última acción: DISCOVERY+research completos: leídos wal.rs/wal_sharded.rs/txn.rs/init.rs(replay MOD-02)/ADR DRV-014. Doc EN de 185 líneas persistido en docs/research/res01-acid-wal-v2-prepare.md con estado actual file:line, diseño Prepare, recovery dos-pasadas, costos/riesgos, plan S1-S6, recomendación GO condicional. Task file creado. Verify mecánico: secciones + refs True/True.
Resultado: PARTIAL
Próxima acción: Lead: (1) rutear a docs/Backlog.md fila FIND (truthful-error gap txn.rs:133-190) + fila ADR-humano Regla 5; (2) verificar doc y commitear docs/research/res01-acid-wal-v2-prepare.md + tasks/RES-01.md
Contrato: verificacion: pwsh check secciones+refs del doc = True/True/TASKFILE True (exit 0); evidencia: claim='commit point actual es durabilidad del batch Begin+ops+Commit' evidencia='txn.rs:146-160 + wal_sharded.rs:215-235' confianza=alta; claim='MOD-02 da crash-atomicidad via skip-mask de slots contiguos' evidencia='init.rs:505-551' confianza=alta; claim='gap truthful-error real: apply failure post-WAL-Commit resucita ops en restart' evidencia='txn.rs:133-190 buffer dropeado pre-apply + replay init.rs:559-599 re-aplica Commit durable' confianza=alta; claim='clon por shard-grouping es tradeoff intencional' evidencia='docs/architecture/adr/DRV-014-wal-batch-tradeoff.md cae92db3' confianza=alta | artefactos: docs/research/res01-acid-wal-v2-prepare.md, .opencode/skills/campaign-executor/tasks/RES-01.md | invariantes: cero cambios en src/ (read-only respetado); DRV-014 NO revertir (tradeoff vigente); doc técnico en inglés (Doc Language Split) | deuda: filas FIND+ADR pendientes de crear en docs/Backlog.md (fuera de mi scope de escritura autorizado: solo doc+task file)
Próxima tarea si completa: RES-02
=== END RECITATION ===
