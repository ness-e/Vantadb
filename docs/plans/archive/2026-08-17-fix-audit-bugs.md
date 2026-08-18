# Plan de Ejecución: Fix audit bugs + inconsistencias + limitaciones (post-auditoría 0.5.0)

> **Campaign ID: 2f82117c-6286-4de4-a79c-ddaf8b5c573a
> **Campaign ID:** 2f82117c-6286-4de4-a79c-ddaf8b5c573a
> **Inicio:** 2026-08-17
> **Estado: completed
> **Fuente:** Auditoría profunda 0.5.0 (sesión 2026-08-17) — 4 bugs confirmados, 3 inconsistencias, 3 limitaciones
> **Modo:** FAIL_MODE=parallel — waves de 2 sub-agentes, MAX_CONCURRENT=2
> **Regla de commits:** los sub-agentes NO commitean — el lead commitea por tarea al cerrar cada wave (evita race de index.lock entre sub-agentes paralelos).

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 8 | 0 | 0 | 0 |

| ID | Título | Tipo | Sub-agente |
|----|--------|------|------------|
| AUD-044 | CLI search falla en DB fresca (text_index bm25 NotFound) | Bug | vanta-worker |
| AUD-045 | MCP memory_put ignora expires_at_ms + sparse_vector | Bug | vanta-worker |
| AUD-046 | MCP memory_put acepta dims incorrectas silenciosamente | Bug | vanta-worker |
| AUD-047 | Binario publicado sin feature `server` (HTTP inaccesible) | Bug | **vanta-lead (yo)** |
| AUD-048 | Filtros invertidos CLI vs MCP ($eq vs plano) | Inconsistencia | vanta-worker |
| AUD-049 | Naming módulo Python: `vantadb` vs `vantadb_py` | Inconsistencia | vanta-docs + vanta-worker |
| AUD-050 | `inject_context` error engañoso en thread_id | Inconsistencia | vanta-worker |
| AUD-051 | Limitaciones: CLI put sin --metadata, filtros internos, TTL MCP | Limitaciones | vanta-worker + vanta-docs |

## Archivos protegidos (NO tocar por sub-agentes)

- `docs/Backlog.md` — migración la hace el lead (skill progreso Trigger 1)
- `vantadb-mcp/src/handlers/tools.rs` — solo lo toca el sub-agente asignado a AUD-045/046/048/050 (evitar conflictos; serializar estas 4 tareas si van en paralelo)

## Antecedentes (evidencia de la auditoría 2026-08-17)

Todas las reproducciones fueron verificadas en vivo contra `vanta-cli 0.5.0`, MCP JSON-RPC (15 tools), `vantadb@0.5.0` npm y `vantadb_py` 0.5.0 editable. La DB `C:/Users/Eros/.vantadb` quedó limpia de datos de prueba; los harness viven en `%TEMP%\opencode\vanta_*.py` y `%TEMP%\vanta-audit-db`.

---

### Task 1: AUD-044 — CLI search falla en DB fresca
- **Archivos clave:** `src/cli.rs` (comando search), `src/sdk/serialization/impl_index.rs:22` (`ensure_indexes_current`)
- **Gate Justificación:** 🔴 bug real de primer contacto — `vanta-cli search` en DB nueva devuelve `Error: NotFound { kind: "text_index", id: "bm25" }` hasta correr `rebuild-index` manual
- **Contrato: inject_context con thread_id inválido devuelve error claro; 41/41 mcp_tests
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-044.md`
- **Estado:** ⏳ PENDING

### Task 2: AUD-045 — MCP memory_put ignora expires_at_ms + sparse_vector
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (handler put, schema L20-32)
- **Gate Justificación:** 🔴 gap doc/API — la skill `vantadb-mcp` documenta `expires_at_ms`/`sparse_vector` como campos del record (F11, api-reference) pero el MCP put los descarta silenciosamente (devueltos null)
- **Contrato:** `memory_put` con `expires_at_ms` devuelve el valor; `sparse_vector` persiste; `mcp_tests` 34/34 siguen verdes
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-045.md`
- **Estado:** ⏳ PENDING

### Task 3: AUD-046 — MCP memory_put acepta dims incorrectas silenciosamente
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (put no valida dims), `src/sdk/serialization/` (validación existente en search)
- **Gate Justificación:** 🔴 corrupción silenciosa del HNSW — vector 2-dim en índice 4-dim se acepta, `vector_count` sube, el nodo nunca aparece en búsquedas
- **Contrato:** `memory_put` con dims ≠ índice devuelve error claro `Vector dimension mismatch: expected 4, got 2`; put válido sigue funcionando
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-046.md`
- **Estado:** ⏳ PENDING

### Task 4: AUD-047 — Binario publicado sin feature `server`
- **Archivos clave:** `Cargo.toml` (`default = [...]` L97 no incluye `server`), `.github/workflows/release-binaries-63.yml` (build con `$ALLOC_FEATURES` = custom-allocator/jemalloc, no server)
- **Gate Justificación:** 🔴 el HTTP server (axum, `POST /api/v2/query`) está publicado pero **inaccesible** en el binario instalado — `vanta-cli server --http` falla con "requires the 'server' feature". Verificado: `cargo build --release --features server` compila y funciona (health + VantaQL OK). Es solo packaging/features.
- **Contrato:** el binario release publicado incluye `server` en su feature set; `vanta-cli server --http` arranca y responde `/health`
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-047.md` (la implementa el lead)
- **Estado:** ⏳ PENDING

### Task 5: AUD-048 — Filtros invertidos CLI vs MCP
- **Archivos clave:** `src/cli.rs` (count/delete-by-filter exige `{"field": {"$eq": value}}`), `vantadb-mcp/src/handlers/tools.rs` (search_memory exige `{"field": value}` plano y rechaza `$eq`)
- **Gate Justificación:** 🟡 inconsistencias de formato entre interfaces — el mismo filtro expresado distinto según el canal
- **Contrato:** decisión de formato unificado documentada en ADR o task file; ambos canales aceptan el formato canónico (sin romper la API existente — evaluar backward compat)
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-048.md`
- **Estado:** ⏳ PENDING

### Task 6: AUD-049 — Naming módulo Python: `vantadb` vs `vantadb_py`
- **Archivos clave:** `vantadb-python/pyproject.toml:38` (`module-name = "vantadb_py"`), `vantadb-python/README.md` (`import vantadb_py as vdb`)
- **Gate Justificación:** 🟡 friction de primer contacto — PyPI paquete `vantadb-py`, módulo `vantadb_py`, `import vantadb` falla (mientras Rust/npm se llaman `vantadb`)
- **Contrato:** decisión tomada (alias `vantadb` en `__init__.py` O documentación explícita); quickstart sin fricción
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-049.md`
- **Estado:** ⏳ PENDING

### Task 7: AUD-050 — `inject_context` error engañoso en thread_id
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (inject_context handler)
- **Gate Justificación:** 🟡 string en thread_id da "Missing 'thread_id'" sin decir que el tipo debe ser numérico
- **Contrato:** error claro (`thread_id must be a numeric id, got string`); tests actualizados
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-050.md`
- **Estado:** ⏳ PENDING

### Task 8: AUD-051 — Limitaciones: CLI put sin --metadata, filtros internos, TTL vía MCP
- **Archivos clave:** `src/cli.rs` (put no tiene flag metadata), `vantadb-mcp/src/handlers/tools.rs` (TTL ya cubierto por AUD-045 — aquí: exponer ttl_ms en MCP put), docs
- **Gate Justificación:** 🟡 paridad de features entre canales (CLI/Python/MCP/TS) — el CLI no puede escribir metadata y no puede filtrar por campos internos
- **Contrato:** CLI `put --metadata '{"k":"v"}'` funcional; doc de filtros CLI aclara que aplican solo a metadata de usuario; TTL MCP (si AUD-045 no lo cubre completo)
- **Task file:** `.opencode/skills/campaign-executor/tasks/AUD-051.md`
- **Estado:** ⏳ PENDING

---

### Checkpoint: Waves

| Wave | Tareas | Sub-agente |
|------|--------|------------|
| 0 | AUD-044, AUD-045, AUD-046 | vanta-worker ×3 |
| 1 | AUD-047 (lead), AUD-048, AUD-050 | vanta-lead, vanta-worker ×2 |
| 2 | AUD-049, AUD-051 | vanta-worker + vanta-docs |

### Checkpoint final (post-Wave 2)
- [ ] `cargo check -p vantadb -p vantadb-mcp -p vantadb-server` ✅
- [ ] `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` → 34/34 (o más si se agregaron)
- [ ] Batería manual: `vanta-cli search` en DB fresca; MCP put con expires_at/sparse/dims; `vanta-cli server --http` + `/health`
- [ ] Backlog: filas AUD-044..051 actualizadas
- [ ] Review vanta-audit (GATE) antes de commitear como release

## Riesgos y Mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| AUD-045/046/048/050 tocan el mismo `tools.rs` | Medio — conflictos de edición en paralelo | Serializar: wave 0 con AUD-045 y AUD-046, wave 1 con AUD-048 y AUD-050 |
| AUD-047 cambia features del release | Alto — binario más grande / features no compilan | `cargo build --release --features server` verificado ✅; revisar tamaño final |
| AUD-048 rompe backward compat de filtros MCP | Medio — clientes existentes | Evaluar aceptar AMBOS formatos (detectar `$eq` vs plano) en vez de migración forzosa |
| AUD-049 alias `vantadb` choca con el paquete raíz | Medio | El alias es del módulo Python, no del crate — verificar `vantadb_py` editable no rompe `import vantadb` en otro lado |

## Open Questions
- AUD-048: ¿formato canónico = plano (MCP) u operadores (CLI)? Decidir en el task file con evidencia de uso.
- AUD-049: ¿alias `vantadb` en `__init__.py` o solo docs? Decidir sin romper `vantadb_py` existente.

=== RECITATION ===
Campaign ID: 972809fd-ceed-4204-9859-79addcd4c066
Objetivo activo: inject_context error engañoso
Estado: completed
Última acción: AUD-050: mensaje error distingue missing vs tipo inválido + test
Resultado: ✅
Próxima acción: skill progreso Trigger 1 + cierre campaña
Contrato: verificacion: cargo check -p vantadb ✅; cargo clippy -p vantadb --all-targets -D warnings ✅; rustfmt --check search.rs+crud.rs ✅; cargo nextest run --profile audit -p vantadb -E 'not test(test_consolidate_node_with_binary_vector)' ✅ 1893/1893. Evidencia: fix en src/cli_handlers/search.rs (4 handlers open_embedded false) y src/cli_handlers/crud.rs:499 (alta); test de regresión en src/cli_handlers/search.rs tests module (alta); verificación manual vanta-cli put+search exit 0 devuelve nodo (alta); test fallido pre-existente verificado con git stash en storage engine maintenance.rs:272 (alta). Artefactos: src/cli_handlers/search.rs, src/cli_handlers/crud.rs, .opencode/skills/campaign-executor/tasks/AUD-044.md. Invariantes: semántica de ensure_indexes_current e índices intacta; solo flag read_only en handlers CLI. Deuda: test_consolidate_node_with_binary_vector pre-existente a anotar en Backlog (no de esta tarea); fmt diffs pre-existentes en vantadb-mcp/tests/mcp_tests.rs. Queda_pendiente: lead commitea (worktree listo, sin commit hecho).
Próxima tarea si completa: ninguno — campaña completa
=== END RECITATION ===
