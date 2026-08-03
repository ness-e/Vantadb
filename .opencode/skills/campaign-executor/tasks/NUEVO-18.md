# NUEVO-18: Sparse vectors nativos — hybrid search real (sparse + dense)

## Metadata
- **Plan file:** none (no plan activo; directamente desde Backlog.md línea 170)
- **Fuente:** docs/Backlog.md — "Sparse vectors nativos - hybrid search real. Solo mención en test"
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟠 Alto
- **Tipo:** Rust (core engine) + Mixto (serialización API pública)
- **Turns estimados:** 30-60
- **Creado:** 2026-08-02
- **last-synced:** 2026-08-02
- **Estado:** 🟡 IN PROGRESS
- **last-synced:** 2026-08-02

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/search/mod.rs::search` (híbrido), `crate::planner::fuse_rrf_with_report`, `src/index/search.rs::search_nearest`, `node.rs`, PyO3 `vantadb-python/src/types.rs`, `vantadb-ts` , `vantadb-wasm` (si se expone) |
| Callees | `DistanceMetric` (nuevo SparseDot), serialización serde en `src/sdk/serialization/vector_types.rs`, índice denso existente |
| Implicaciones | **ROMPE/EXTIENDE API pública** (VantaMemorySearchRequest, VantaMemoryInput) — GDOC `api-and-interface-design`. No requiere migración de datos (sparse es additivo). Debe coexistir con dense existente sin breaking |

## Contrato
```
"cargo nextest run --profile audit --workspace --build-jobs 2 pasa Y
 un tipo SparseVector (map<dim, f32>) puede indexarse y buscarse por dot-product,
 fusionable con los resultados dense existentes vía RRF"
```
Comportamiento específico verificable: un vector sparse `{3: 1.0, 7: 0.5}` insertado es recuperado como top-1 por una query del mismo sparse, y ambos caminos (dense-only, sparse+dense) conviven.

## Herramientas necesarias
- codegraph_explore (blast radius del engine)
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (diagnostics, goto def)

## Investigation Notes
- Estado hoy (verificado con codegraph): `VantaMemorySearchRequest.query_vector: Vec<f32>` — **solo dense**. El híbrido actual fusiona BM25 **texto** + dense (RRF). No existe representación sparse ni el tipo que la  serial.
- La única mención "sparse" en tests es un rótulo string (`TerminalReporter::sub_step("Populating sparse vector space...")`), no dato real.
- Diseño propuesto (tanivo): representación `HashMap<u32,f32>` (dim→valor), nueva métrica `SparseDotProduct` en `DistanceMetric`, campo opcional `query_sparse: Option<SparseVector>` en request, y extensión de la fusión RRF para un 3er canal OR plan dado (backlog-time, sparse + BM25 lexical son formas de búsqueda léxica — definir si sparse REEMPLAZA o complementa BM25).
- **Ambigüedad a resolver con vanta-engine**: ¿el sparse vector es input del usuario, o derivamos lexical sparse internamente (tipo SPLADE)? Backlog dice "sparse vectors nativos" → input del usuario. Confirmar antes de implementar.

## Steps

### Step 1: Diseño + ADR
- **Archivos:** `docs/architecture/adr/0NN_sparse_vectors.md`
- **Acción:** decodificar representación (map u32→f32), métrica (dot-product), y cómo convive con BM25 lexical (¿sustituye o coexiste?). Dejar escrito en ADR.
- **Verify:** existe ADR con decisión resuelta
- **Estado:** ✅ DONE

### Step 2: Tipo sparse en node.rs
- **Archivos:** `src/node.rs`
- **Acción:** agregar `SparseVector` (serde) + variante/dep del DistanceMetric requiere dot de sparse
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ DONE

### Step 3: Distancia dot product sparse
- **Archivos:** `src/index/search.rs` (o distance module)
- **Acción:** función sparse dot product entre dua sparse vectors
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ DONE

### Step 4: Extender request serialization
- **Archivos:** `src/sdk/serialization/vector_types.rs`
- **Acción:** campo `query_vector` opcional sparse en `VantaMemorySearchRequest` + en el input
- **Verify:** `cargo test -pancy test_search_request_serialization_roundtrip`
- **Estado:** ✅ DONE

### Step 5: Integración búsqueda (sparse-only + dense-only coexisten, fusion RRF 3-way si aplica)
- **Archivos:** `src/sdk/search/mod.rs`
- **Acción:** ruta sparse index/ingesta y búsqueda sparse; integrar en pipeline híbrido
- **Verify:** `cargo nextest run --profile audit -p vantadb --test`
- **Estado:** 🟡 IN PROGRESS

### Step 6: Tests de contratación (integración)
- **Archivos:** `tests/core/` (nuevo test)
- **Acción:** insert sparse-only, query sparse, assert top-1 correcto; sparse+dense coexist camino denso no roto
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2`
- **Estado:** ⏳ TODO

### Step 7: Exposición SDK (PyO3 mínima) + docs
- **Archivos:** `vantadb-python/src/types.rs`, `docs/api/EMBEDDED_SDK.md`
- **Acción:** al menos poder pasar `query_vector` sparse por Python (numpy/tydense); actualizar doc API
- **Verify:** `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v`
- **Estado:** ⏳ TODO

## Dependencias
- Ninguna (NUEVO-17 LSM Segment es ortogonal; NUEVO-16 PQ es ortogonal)

## Notas
- Este task file es el **esqueleto de discovery** armado por vanta-lead. La implementación la realiza `vanta-engine` (diseño de índice + fusión).
- **Pregunta abierta a resolver en Step 1:** ¿el sparse reemplaza o complementa el BM25 lexical existente? RRF 2-vías actual (BM25+dense) → posible 2-vías (sparse+dense) o 3-vías.
- Backlog etiqueta este como "Feature para después del lanzamiento público" — NO bloquea release 0.5.0.