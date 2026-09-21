# SKL-01: Corregir y modernizar `skills/vantadb/SKILL.md`

## Metadata
- **Plan file:** `docs/plans/2026-08-17-skills-vantadb.md` (wave SKL)
- **Fuente:** diagnóstico del lead 2026-08-17 (Backlog P21) — skill desactualizada contra código real
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Docs (skills de proyecto) — no toca código core ni bindings
- **Turns estimados:** 3-5
- **Creado:** 2026-08-17
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 — diagnóstico completo con evidencia en sección Blast Radius
- **Pendientes (downhill):** 3 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `skills/vantadb/SKILL.md` (666L, skill del proyecto referenciable por cualquier agente), `SKILLS-MANIFEST.md` (catálogo raíz, puede listarla), docs de usuario |
| Callees | API Python real (`vantadb-python/vantadb_py/vantadb_py.pyi` — fuente de verdad de firmas), API Rust (`src/sdk/api.rs`), integraciones (`integrations/langchain/`, `integrations/llamaindex/`), docs vivos (`docs/api/MCP.md`, `docs/operations/BENCHMARKS.md`) |
| Implicaciones | La skill es la referencia de uso de VantaDB para agentes → versiones/paths/claims falsos propagan errores a cualquier agente que la cargue. Cambios son docs-only: riesgo bajo, sin impacto en código |

## Hallazgos verificados (lead, 2026-08-17)

1. **🔴 `scripts/install-vantadb.sh` = basura**: contenido literal `test` (6 bytes). No instala nada.
2. **🔴 Versiones viejas**: SKILL.md:651-654 dice "Current Version: 0.1.4", "Rust Minimum: 1.70+", "Python Minimum: 3.8+". Real: 0.5.0 (`Cargo.toml:642`), Rust 1.94.1 (`Cargo.toml:644`), PyPI requiere Python ≥3.11 (verificado en registro PyPI).
3. **🔴 Dep Rust stale**: SKILL.md:48 `vantadb = "0.1.4"` → real 0.5.0 publicado en crates.io.
4. **🟡 Benchmarks viejos**: tabla SKILL.md:364-370 (Put 10.7ms, Vector Search 62ms…) — README actual afirma hybrid 1.2ms. Regla 11: claims de performance necesitan benchmark reproducible o se reformulan sin número.
5. **🟡 Claims falsos de features**:
   - SKILL.md:616 "IQL/LISP query language (experimental)" en Not Supported → **IQL SÍ está soportado** (tool MCP `query_iql`, `vantadb-mcp/src/handlers/tools.rs:72`).
   - SKILL.md:619 "Advanced tokenization (stemming, stopwords)" Not Supported → **falso**: `advanced-tokenizer` (tantivy) es feature default (`Cargo.toml:97`).
   - SKILL.md:626-627 "No stemming or stopwords support" → falso por lo anterior.
6. **🟡 Paths muertos** (verificados con Test-Path):
   - `docs/BENCHMARKS.md` → real `docs/operations/BENCHMARKS.md`
   - `docs/adr/` → real `docs/architecture/adr/`
   - `packages/langchain-vantadb/` → real `integrations/langchain/`
   - `examples/python/langchain_rag.py` → **no existe**
7. **🟡 Features reales faltantes**: `ttl_ms` (pyi put), `put_batch` (kwargs), `purge_expired`, `compact_wal`, `AsyncVantaDB` (`vantadb-python/vantadb_py/__init__.py`), `:memory:` mode (pyi `__init__`), `hardware_profile`.
8. **🟡 Directorios vacíos**: `skills/vantadb/references/`, `skills/vantadb/assets/` no tienen contenido.

## Contrato
"`skills/vantadb/SKILL.md` sin claims falsos ni paths muertos: (1) versiones 0.5.0 / Rust 1.94.1 / Python ≥3.11; (2) `rg "0.1.4|1.70|3\.8\+|docs/BENCHMARKS|docs/adr|packages/langchain|langchain_rag" skills/vantadb/SKILL.md` → 0 matches; (3) `install-vantadb.sh` instalador real (>30 bytes, no "test"); (4) features reales documentadas (IQL, ttl_ms, purge_expired, compact_wal, AsyncVantaDB)." Verificación mecánica: los 4 checks.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. NO inventar firmas: toda API documentada DEBE existir en `vantadb-python/vantadb_py/vantadb_py.pyi` o `src/sdk/api.rs`. Si dudas de un método → verificar con rg antes de documentarlo.
  2. NO tocar `docs/Backlog.md`, task files, `docs/api/MCP.md`, `docs/plans/*` — solo `skills/vantadb/`.
  3. Regla 11 (claims perf): número de latencia SOLO si cita benchmark reproducible (`docs/operations/BENCHMARKS.md`); si no hay fuente, reformular sin número.
  4. Idioma: la skill está en inglés — mantener inglés (doc language split: docs técnicas en inglés).
  5. NO borrar secciones útiles (Quick Start, Best Practices, Troubleshooting siguen siendo válidas) — corregir, no reescribir desde cero.
- **Comandos de verificación:** los del contrato + `Test-Path skills/vantadb/scripts/install-vantadb.sh` + lectura del nuevo installer.
- **Deuda pendiente:** ninguna esperada (docs-only).

## Steps (Plan → Act → Verify)

1. **📝 DISCOVERY** — leer `skills/vantadb/SKILL.md` completo (666L) + `vantadb-python/vantadb_py/vantadb_py.pyi` (firmas reales) + `src/sdk/api.rs` (Rust SDK) + `docs/operations/BENCHMARKS.md` (números con fuente). Confirmar hallazgos del lead con `rg` de cada claim falso. Verify: lista de correcciones con `archivo:línea` objetivo.
2. **📝 EJECUCIÓN** — reescribir `skills/vantadb/SKILL.md`:
   - Versiones: 0.5.0 / Rust 1.94.1 / Py ≥3.11; dep `vantadb = "0.5.0"`.
   - Fix paths: `docs/operations/BENCHMARKS.md`, `docs/architecture/adr/`, `integrations/langchain/` (o eliminar referencia si no aplica), reemplazar `langchain_rag.py` por ejemplo existente real.
   - Mover IQL/LISP: IQL soportado (sección propia), LISP no.
   - Tokenization: stemming/stopwords SÍ soportado (tantivy default).
   - Benchmarks: actualizar con fuente de `docs/operations/BENCHMARKS.md` o reformular sin números no verificables.
   - Features nuevas: ttl_ms, put_batch kwargs, purge_expired, compact_wal, AsyncVantaDB, `:memory:`, hardware_profile.
   - Reescribir `scripts/install-vantadb.sh` como instalador real (pip install vantadb-py + cargo install alternativa, con checks) o eliminarlo si no aporta.
   - Decidir sobre `references/` y `assets/` vacíos: rellenar con contenido útil (p.ej. referencias a docs vivos) o eliminar los dirs vacíos (ponytail: si no se usa, no existe).
   - Verify: contrato mecánico (4 checks).
3. **📝 CIERRE** — crear task file actualizado (este archivo) con hallazgos/resultado + bloque RESULTADO (pipeline-full.md §7) para el lead.

## Dependencias
- Ninguna (autónoma). SKL-04 (review) depende de SKL-01/02/03.
- Fuente de verdad API: `docs/api/MCP.md` + pyi + api.rs — leer, NO editar.

## Fases explícitas — SECURITY | PERFORMANCE

- [ ] **SECURITY** — NO aplica: docs-only, sin trust boundary ni input de usuario.
- [ ] **PERFORMANCE** — NO aplica: no toca hot path; los números que se citen vienen de `docs/operations/BENCHMARKS.md` (Regla 11).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (SKL-04, wave W3).
- **Enfoque:** validar contrato mecánico (0 matches de claims falsos), versiones reales, ninguna firma API inventada.
- **Veredicto:** pendiente.

## Notas
- La escalera ponytail aplica: corregir lo roto, NO reescribir toda la skill. Si una sección sigue siendo válida, se conserva tal cual.
- El pyi (`vantadb_py.pyi`) es la fuente de verdad de firmas Python — si un método de la skill no está en el pyi, se elimina o se verifica con `rg` en el código.

## Resultado (SKL-01 — ejecutado 2026-08-17)

### Archivos modificados
- `skills/vantadb/SKILL.md` (666L → 789L): versiones 0.5.0/1.94.1/≥3.11, dep Rust `vantadb = "0.5.0"`, paths vivos, claims de features corregidos, features reales agregadas.
- `skills/vantadb/scripts/install-vantadb.sh` (6B "test" → 2109B instalador real: check Python ≥3.11 + `pip install vantadb-py` + guidance Rust SDK vía Cargo.toml).
- Eliminados dirs vacíos `skills/vantadb/references/` y `skills/vantadb/assets/` (ponytail: sin contenido, sin referencias entrantes — rg global confirmó).

### Contrato (4/4 ✅)
1. `rg "0.1.4|1.70|3\.8\+|docs/BENCHMARKS|docs/adr|packages/langchain|langchain_rag" skills/vantadb/SKILL.md` → **0 matches** ✅
2. Versiones: 0.5.0 (L48, L774), Rust 1.94.1 (L775), Python ≥3.11 (L776) ✅
3. `install-vantadb.sh`: 2109 bytes (>30), sin "test" ✅
4. Features reales documentadas: IQL (`db.query()` + sección propia + tool MCP `query_iql`), ttl_ms, purge_expired, compact_wal, AsyncVantaDB, `:memory:`, put_batch kwargs, hardware_profile ✅

### Hallazgos adicionales corregidos (DISCOVERY)
- Signatures stale: `rebuild_index(namespace)` → `rebuild_index()` (pyi sin arg), `search_batch(namespace=,query_vectors=)` → `search_batch(vectors, top_k)` (pyi), `export_namespace("ns", path)` → `export_namespace(path, "ns")` (pyi orden path-first), `list_memory()` → devuelve `VantaListResult` con `.records/.total_count/.next_cursor`.
- Integraciones: `langchain_vantadb` → `vantadb_langchain` (pip `vantadb-langchain`, `integrations/langchain/`), `llama_index.vector_stores.vantadb` → `vantadb_llamaindex` (pip `vantadb-llamaindex`, `integrations/llamaindex/`); `embedding_function=` → `embedding=` (README real).
- Claims perf: "sub-millisecond" eliminado (BENCHMARKS certifica p50 1.2ms @10K); tabla de benchmarks conservada con cita a `docs/operations/BENCHMARKS.md` §1/§2/§5/§6 (todos los números verificados contra la fuente, incluidos 68.4s y 2.43ms/4.01x).
- Tokenization: stemming/stopwords SÍ soportados (tantivy `advanced-tokenizer` = feature default, `Cargo.toml:97`); movido IQL de "Not Supported" a "Supported", LISP queda como no soportado.
- Roadmap: "Advanced tokenization (tantivy integration)" marcado como entregado y removido de Near Term.

### Pendiente
- `bash -n` no ejecutable en este host (WSL sin distro instalada) — el installer no pudo validarse con shellcheck; sintaxis simple revisada manualmente. Validar en CI/Linux antes de usarlo.
- No se hizo git add/commit ni `campaign_update_task_state` (instrucción explícita).