# D5b — distance_metric/method desconocidos → ValueError en vantadb-python

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: lógica/frontera PyO3 (worker, TDD). Sin discovery MCP (adaptador §10 plan: PROHIBIDO campaign MCP).
- Blast radius (grep + read): 4 sitios en `vantadb-python/src/lib.rs`:
  - `search:1242-1252` — `match distance_metric { Some("euclidean")=>Euclidean, Some(other)=>warn+Cosine, None=>Cosine }`
  - `explain_memory_search:2172-2182` — idéntico patrón warn+fallback
  - `SearchRequest::from_dict:2317-2327` — idéntico patrón warn+fallback (batch path)
  - `parse_search_method:2360-2375` — `None=>None, Some(ivf|scann|hnsw|flat)=>Some(..), Some(other)=>warn+None`; callers `search:1253` y `from_dict:2334-2337`
  - Bug adicional: `Some("cosine")` explícito cae en rama `warn` (no es válido explícito). Valores válidos reales: `cosine, euclidean` + `None` (default cosine); `method`: `None, ivf, scann, hnsw, flat`.
- Patrón a seguir: `parse_backend_kind:179-188` — `Some(other)=>Err(PyValueError::new_err(format!("Unknown backend ...")))` con lista de conocidos. Sin `tracing::warn` en error path.
- Test existente que consagra fallback: `vantadb-python/tests/test_sdk.py:307-309` (`method="quantum"` espera hits sin error) — debe migrar a `pytest.raises(ValueError)`.
- convert.rs/vector.rs: NO tocar (prohibición tarea; error no lo exige — parsing es local a lib.rs).
- Baseline `/cleanCA vantadb-python/src/lib.rs`: regla E1 (frontera estricta, no warn+fallback) — no se corre binario, veredicto mental.

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia: `vantadb-python/src/lib.rs` — (a) helper `parse_distance_metric(Option<&str>)->PyResult<DistanceMetric>` con `cosine/euclidean/None` válidos, resto `Err(PyValueError)`; usar en `search`, `explain_memory_search`, `SearchRequest::from_dict`; (b) `parse_search_method(Option<&str>)->PyResult<Option<IndexType>>` con `None/ivf/scann/hnsw/flat` válidos, resto `Err(PyValueError)`; callers con `?`; (c) eliminar `tracing::warn!` de esos 4 paths (mantener `clamp_top_k` warn ERR-022 intacto).
- NO cambia: valores válidos (`None/cosine/euclidean`, `None/ivf/scann/hnsw/flat`) — mismo `DistanceMetric`/`IndexType` que antes; `clamp_top_k`, `parse_backend_kind`, `convert.rs`, `vector.rs`; wire/DTO fuera de error path.
- Archivos exactos: `vantadb-python/src/lib.rs` (único prod) + `vantadb-python/tests/test_sdk.py` (migrar assert fallback → error) + test nuevo error (mismo file o `test_d5b_distance_method.py`). Task file: `docs/tasks/D5b.md` (este).
- Verify: `cargo fmt --check` + `cargo clippy -p vantadb_py --all-targets --deny warnings` (o `-p vantadb-python` según Cargo.toml) + `cargo check -p vantadb_py` + `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -k "method or distance or search"` + tests nuevos del error.

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [ ] Slice 1: `parse_distance_metric` helper + aplicar en `search:1242` (`?`) + test RED `test_search_rejects_unknown_distance_metric` (ValueError) + válidos (`None/cosine/euclidean`) siguen verdes. Verify: `cargo check` + pytest filtro.
- [ ] Slice 2: aplicar helper en `explain_memory_search:2172` + `SearchRequest::from_dict:2317` + tests `explain` y `batch` rechazan desconocido. Verify: mismo gate.
- [ ] Slice 3: `parse_search_method -> PyResult` + callers `search:1253` y `from_dict:2334` con `?` + migrar `test_sdk.py:307-309` a `pytest.raises(ValueError)` + tests `search/method` y `batch method` rechazan `quantum`. Verify: clippy + fmt + pytest completo SDK.
- [ ] Cierre: `cargo fmt --check`, clippy deny warnings, pytest SDK completo, `rg "falling back to default|falling back to engine routing" vantadb-python/src/lib.rs` debe dar 0 en esos paths. Sin commit (lo ejecuta el lead). RESULTADO + recitation.

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO exigido por despacho (✅/🟡/❌, STEPS_OK, PROXIMO_STEP, COMMIT_HASH ninguno, ARCHIVOS, VERIFY_CONTRATO, BLOQUEO, GATES_EVALUADOS, SKILLS_CARGADAS ≥10).
- `/cleanCA` mental regla E1: sin warn+fallback silencioso en frontera Python tras el fix; `clamp_top_k` warn se mantiene (ERR-022 observable, no silencioso).
