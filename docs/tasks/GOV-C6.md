# GOV-C6: Sweep bidireccional env vars — CONFIGURATION.md

## Metadata
- **Plan file:** docs/plans/2026-08-22-doc-governance-plan.md (NO editar)
- **Creado:** 2026-08-22T00:00
- **last-synced:** 2026-08-22T00:00
- **Estado:** ✅ COMPLETED

## Contrato
"rate_limit_rpm=600 en doc; sweep bidireccional completo; spot-check ≥10 defaults; markdownlint exit 0"

## Steps

### Step 1: Corregir rate_limit_rpm default
- **Archivos:** docs/operations/CONFIGURATION.md
- **Acción:** default 100 → 600 (config.rs:659 `parse_env_or("VANTADB_RATE_LIMIT_RPM", 600u32)`); añadido "(0 = disabled)" verificado en cli_server.rs:263 (`if rpm > 0`)
- **Estado:** ✅

### Step 2: Sweep bidireccional de env vars
- **Método:** `rg -o 'VANTA[A-Z_]*|VANTADB[A-Z_]*' src/` (44 vars únicas) vs tabla CONFIGURATION.md
- **Resultado:** ver tabla abajo
- **Estado:** ✅

### Step 3: Spot-check defaults ≥10 contra config.rs
- **Estado:** ✅ (14 verificados; 1 mismatch adicional corregido: flush_threshold)

### Step 4: Verify markdownlint
- **Comando:** `npx markdownlint-cli2 "docs/operations/CONFIGURATION.md"`
- **Resultado:** 0 issues, exit 0
- **Estado:** ✅

## Tabla resumen del sweep

| Métrica | N |
|---------|---|
| Vars únicas en código (`rg -o` sobre src/) | 44 |
| Vars documentadas en CONFIGURATION.md | 40 (38 VantaConfig + VANTA_DB + 2 legacy LOG_JSON) |
| Añadidas al doc | 5 |
| Eliminadas del doc | 0 |
| Fantasmas verificados (existentes pero reales) | 2 |

**Añadidas** (sección "Environment Variables Outside `VantaConfig`"):
| Var | Evidencia código |
|-----|------------------|
| `VANTA_EMBEDDING_PROVIDER` | src/llm.rs:40 (default `ollama`) |
| `VANTA_OPENAI_API_KEY` | src/llm.rs:145 (panic si falta con provider=openai) |
| `VANTA_OPENAI_MODEL` | src/llm.rs:147 (default `text-embedding-3-small`) |
| `VANTA_BACKUP_DIR` | src/storage/engine/maintenance.rs:658 (default `./vantadb_snapshots`) |
| `VANTADB_REPORTED_VERSION` | src/metadata.rs:22 (override semver para banners/MCP) |

**Fantasmas auditados (del brief):**
| Ghost claim | Veredicto | Evidencia |
|-------------|-----------|-----------|
| Fallback `HOST` en fila host | REAL — se conserva | src/config.rs:512 `.or_else(env::var("HOST"))` |
| Fallback `PORT` en fila port | No existe en estado actual del doc (línea 22 no lo cita) — nada que eliminar |
| `flush_interval_ms` | No está en CONFIGURATION.md; solo en mdBook generado (`docs/book/book/operations/DURABILITY_GUARANTEES.html`) — artefacto generado, fuera de scope |

**Spot-check de defaults (14 vars, config.rs):**
| Var | Doc | Código | Veredicto |
|-----|-----|--------|-----------|
| `storage_path=vantadb_data` | ✓ | config.rs:506 | OK |
| `host=127.0.0.1` | ✓ | config.rs:513 | OK |
| `port=8080` | ✓ | config.rs:518 | OK |
| `llm_url=http://localhost:11434` | ✓ | config.rs:524 | OK |
| `llm_model=all-minilm` | ✓ | config.rs:529 | OK |
| `llm_summarize_model=llama3` | ✓ | config.rs:535 | OK |
| `max_connections=max_blocking*2` | ✓ | config.rs:618 | OK |
| `pool_acquire_timeout_ms=5000` | ✓ | config.rs:623 | OK |
| `circuit_breaker_failure_threshold=5` | ✓ | config.rs:628 | OK |
| `circuit_breaker_open_timeout_secs=30` | ✓ | config.rs:633 | OK |
| `insert_lock_timeout_ms=5000` / `file_lock_timeout_ms=1000` | ✓ | config.rs:639,644 | OK |
| `version_history_limit=Some(32)` | ✓ | config.rs:703 | OK |
| `wal_shards=4`, `flat_threshold=10000` | ✓ | config.rs:780,782 | OK |
| `rate_limit_rpm=100` vs `600` | ❌→✅ | config.rs:659 | CORREGIDO |
| `flush_threshold=10000` vs `None` | ❌→✅ | config.rs:727 (0 filtrado → None) | CORREGIDO |

Notas: `wal_buffer_size=65536` doc es correcto como default efectivo (`src/storage/wal.rs:16` `unwrap_or(64 * 1024)` aunque from_env da None). `batch_size=None (1000)` y `bulk_commit_interval=None (10000)` correctos como defaults efectivos (src/sdk/api.rs:263,1635). La ocurrencia suelta `"VANTADB"` (sin sufijo) en el rg es un prefijo de string, no una env var.

## Dependencias
- GOV-B2 (limpieza previa restore --from :330) — trabajó sobre este archivo

## Notas
- PROHIBIDO git (instrucción del orquestador): sin commit. Cambio queda en worktree.
- Plan file NO editado (instrucción explícita).

## Context Save Point
- **Fecha:** 2026-08-22
- **Branch:** develop (worktree sucio, sin commit por orden del orquestador)
- **CI pendiente:** sí (commit pendiente)
- **Decisiones:** defaults efectivos (unwrap_or downstream) aceptados como válidos cuando from_env da None pero el runtime aplica fallback documentado
- **Problemas conocidos:** ninguno
- **Próxima tarea:** según plan file
