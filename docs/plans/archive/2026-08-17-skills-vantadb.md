# SKL Wave — Dejar perfectas las skills de VantaDB (`skills/`)

> **Estado:** planned · **Campaign ID:** skl-2026-08-17
> **Fuente:** diagnóstico del lead (2026-08-17) — revisión de `skills/vantadb/` y `skills/vantadb-mcp/` contra código real. Hallazgos con evidencia `archivo:línea` en el Backlog P21 y en cada task file.
> **FAIL_MODE:** parallel · **MAX_CONCURRENT:** 3 · **DO:** 4 (3 delegadas + 1 review gate)

## Problema (resumen del diagnóstico)

Las 2 skills del directorio `skills/` están **desactualizadas y parcialmente rotas**:

| Skill | Hallazgos críticos |
|---|---|
| `skills/vantadb/` | `install-vantadb.sh` = 6 bytes ("test", basura). Versiones viejas (0.1.4 → real 0.5.0; Rust 1.70+ → 1.94.1; Python 3.8+ → ≥3.11). Dep `vantadb = "0.1.4"`. Benchmarks viejos (62ms vector search vs 1.2ms actual). Claims falsos: IQL "not supported" pero `query_iql` existe; "no stemming/stopwords" pero tantivy `advanced-tokenizer` es feature default. Paths rotos (`docs/BENCHMARKS.md`→`docs/operations/BENCHMARKS.md`, `docs/adr/`→`docs/architecture/adr/`, `packages/langchain-vantadb/`→`integrations/langchain/`, `examples/python/langchain_rag.py` inexistente). Faltan features: ttl_ms, purge_expired, compact_wal, AsyncVantaDB, `:memory:`, IQL/graph. `references/` y `assets/` vacíos. |
| `skills/vantadb-mcp/` | Command roto `vanta-server --mcp --path` (binario real: `vanta-cli server --mcp --db` o `vantadb-server --mcp`; `--path` NO existe — `vantadb-server/src/main.rs:89-91` solo acepta `-h/--help/--mcp`). Tool `query_lisp` → renombrado a `query_iql` (AUD-004) — la skill documenta un tool inexistente. Faltan 4 tools reales (`collection_stats`, `collection_list`, `collection_delete`, `rehydrate` — 14 reales en `vantadb-mcp/src/handlers/tools.rs`). `config.json` inventado: `setup-vantadb.sh` escribe `~/.vantadb/config.json` que NADIE lee (config = env vars `VANTADB_*`, `src/config.rs`). `test-mcp.py` usa `--path` inválido + spawnea 1 proceso por test. Paths rotos (`docs/EDITOR_INTEGRATIONS.md`, `docs/MCP.md`→`docs/api/MCP.md`). Assets con command roto (`vanta-server --mcp --path`). Falta asset de configuración para **OpenCode**. |

## Waves

| Wave | Tareas | Sub-agente | Tipo |
|---|---|---|---|
| W1 | SKL-01 (skill `vantadb`), SKL-02 (skill `vantadb-mcp` SKILL.md+references) | docs / docs | Docs |
| W2 | SKL-03 (scripts + assets funcionales) | worker | Scripts |
| W3 | SKL-04 (gate P2-01 review) | review | Review |

## Routing

| Task | Archivos | Sub-agente |
|---|---|---|
| SKL-01 | `skills/vantadb/SKILL.md`, `skills/vantadb/scripts/install-vantadb.sh`, dirs vacíos `references/` `assets/` | vanta-docs |
| SKL-02 | `skills/vantadb-mcp/SKILL.md`, `skills/vantadb-mcp/references/*.md` | vanta-docs |
| SKL-03 | `skills/vantadb-mcp/scripts/*.py` `*.sh`, `skills/vantadb-mcp/assets/*.json` | vanta-worker |
| SKL-04 | revisión read-only de SKL-01/02/03 contra contrato | vanta-review |

## Archivos protegidos (TODOS los sub-agentes)

`docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/*`, `.opencode/task-system/enforcement/verify-log.jsonl`, `completions/_vanta-cli.ps1`, `docs/plans/2026-08-17-skills-vantadb.md`, `.opencode/AGENTS.md`, `.opencode/agents/*`, `docs/api/MCP.md` (fuente de verdad del contrato MCP — leer, no editar).

## Reglas

- Sub-agentes NO hacen git add/commit; NO usan campaign_update_task_state; crean su task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`; devuelven bloque RESULTADO (pipeline-full.md §7).
- El lead commitea por wave tras revisar cada RESULTADO (escalera SARL).
- Contrato de cada tarea = verificación mecánica (Test-Path / rg / comando) — ver task files.
- **API contract sync (Regla 3):** las skills documentan la API pública (MCP tools, Python SDK). El contrato de verdad es `docs/api/MCP.md` + `vantadb-mcp/src/handlers/tools.rs` + `vantadb-python/vantadb_py/vantadb_py.pyi` — nunca inventar firma.
- Cierre: plan → `docs/plans/archive/`, filas Backlog migradas por el lead, reporte final con veredictos P2-01.

## Resultado esperado

- `skills/vantadb/SKILL.md`: versiones reales (0.5.0 / Rust 1.94.1 / Py ≥3.11), paths vivos, features reales (IQL, ttl_ms, purge_expired, compact_wal, AsyncVantaDB), claims con benchmark reproducible o sin número.
- `skills/vantadb-mcp/`: command real (`vanta-cli server --mcp --db`), 14 tools documentados (incluye `query_iql` + collection_* + rehydrate), referencias vivas, asset de OpenCode.
- Scripts ejecutables y verificados: `install-vantadb.sh` real (no "test"), `test-mcp.py` con flag correcto + 1 solo proceso, `setup-vantadb.sh` sin config.json muerto.
- Assets (`claude-desktop-config.json`, `cursor-config.json`) con command correcto.
- Gate P2-01 emitido por vanta-review.