# FIND-109 — `vanta-cli wal salvage` + error tipado en MCP

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-17-seguimiento-mvp.md`
- **Campaign ID:** `64985e0b-0570-431c-a1e9-1d0d551ad54e`
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETED (4/4 steps)
- **Tipo auto-detect:** `rust` (Rust core) via `campaign_detect_task_type`
- **Workflow:** `bug-fix` (localizing→planning→implementing→testing→review→accept→close) + perfil unificado v2
- **Ruta:** vanta-worker · **Branch:** `develop` · **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta
- **Commit previsto:** `feat: FIND-109 — wal salvage opt-in + MCP WAL error tipado`
- **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design, systematic-debugging (reemplaza frontend-ui-engineering por irrelevante web/; ≤8 total)

## 1. TAREA
- **Objetivo:** Cerrar el hard-down del MCP sin vía de reparación. Hoy `verify_shard_counts` (ERR-011) aborta `open` read-write cuando un shard quedó truncado (caída real 2026-09-17: `WAL shard 1 is truncated: 39 durable records, but round-robin requires at least 40` en `C:/Users/Eros/.vantadb`); solo existen `compact`/`vacuum` y ambos abren read-write por `open_embedded(db_path,false)` así que fallan igual; solo-lectura (`doctor`, `export`) sí abre porque `config.read_only=true` saltea el replay (`src/storage/engine/init.rs:413`).
- **Contrato exacto (ley):** (a) fixture WAL truncado en Temp → `vanta-cli wal salvage` replaya lo coherente + reporte explícito de descartados (cero skip silencioso) + exit 0; (b) MCP devuelve error tipado en vez de exit 1 (o documentado por qué no con evidencia); (c) suite `wal` verde (`cargo test -p vantadb wal -j 2`); (d) `cargo clippy -p vantadb --all-targets -- -D warnings` 0 + `cargo fmt --check` 0.
- **Acceptance criteria del plan:** salvage sobre fixture corrupto recupera coherentes con reporte explícito; MCP error tipado o justificación documentada con evidencia; tests wal verdes; clippy 0. Guard ERR-011 sigue abortando por defecto — NO debilitarlo; salvage es comando explícito opt-in.
- **Stop rules:** appetite >2d → ship solo-lectura + DEFER resto; premisa invalidada (fixture no reproduce) → re-evaluar gate (usar copia del backup real en Temp).
- **Cynefin:** 🟨 complicado (WAL/round-robin). **Uphill:** definición de coherente / no debilitar guard / fixture fiel.

## 2. ARCHIVOS
### Clave (línea exacta verificada 2026-09-17)
- `src/wal_sharded.rs:69-94` — `verify_shard_counts` (ERR-011, dos reglas: max-min≤1 y no-creciente).
- `src/wal_sharded.rs:243-286` — `ShardedWal::recover` (lee por shard con `WalReader::next_record`, acumula `shard_counts`, aborta en `:281` si `num_shards>1`).
- `src/wal_sharded.rs:122-196` — `ShardedWal::new_with_buffer` (reconcilia layout on-disk vía `detect_shard_count`+`read_shard_meta`, `record_count` para `next_shard` ERR-050).
- `src/storage/engine/init.rs:413` — `if !config.read_only && config.wal_shards > 0` (solo-lectura saltea replay → por eso doctor/export abren).
- `src/storage/engine/init.rs:448-490` — replay init (lee shards, `global_seq = s + N*p`, `pending.sort`, slice-mask txn MOD-02, `verify_shard_counts` en `:486-490` y `return Err(Error::wal_error(msg))`).
- `src/cli_handlers/wal.rs:1-126` — plantilla de comando (`cmd_wal_compact` `:12`, `cmd_wal_vacuum` `:49`; ambos `open_embedded(db_path,false)` → read-write → fallan igual ante ERR-011).
- `src/cli_handlers/db.rs:18-25` — `open_embedded(path, read_only)` → `Embedded::open_with_config`.
- `src/cli_handlers/server.rs:253-309` — `cmd_server_mcp` spawnea `vantadb-server --mcp` (hereda stdio).
- `src/cli.rs:284-286` — `Wal(WalCommand)`; `src/cli.rs:410-417` — `WalCommand::{Compact,Vacuum}` (aquí se añade `Salvage`).
- `src/bin/vanta-cli.rs:237-240` — dispatch `Commands::Wal(cmd)` (aquí se añade brazo `Salvage`).
- `vantadb-server/src/main.rs:31-72` — `main()->anyhow::Result` con `.context("failed to start MCP server")` (exit 1 preservado, cadena humana en stderr); `vantadb-mcp/src/server.rs:833-906` — `run_stdio_server_auto` (`open_with_config` → `Err(e)=>Err(e)` directo, solo `DatabaseBusy` tiene fallback proxy; WAL error cae en `Err(e)=>Err(e)` `:905`).

### Relacionados (callers/callees via codegraph_explore + grep)
- `src/wal.rs:235-303` — `WalWriter::open_with_buffer` (por-shard `recover_valid_records` + `quarantine_corrupt_tail` + truncate; tolerante single-shard).
- `src/wal.rs:565-608` — `recover_valid_records` (scan-forward mid-file, retorna `(valid_end,count)`).
- `src/wal.rs:610-653` — `quarantine_corrupt_tail` + `quarantine_backup_path` (`.corrupt`, `.corrupt.N`, fail-soft).
- `src/wal.rs:663-779` — `WalReader::open` + `next_record` (scan-forward auto-healing, `Ok(None)` en EOF/torn-tail) + `replay_all`.
- `src/error.rs:140-142,345-357,385-411,416-426` — `Error::Wal(ChainedError)`, `code()` → `VANTADB_IO_ERROR`, `is_retriable()=true` para `Wal`, `recovery_hint()=None` para `Wal`, `wal_error(msg)`.
- `src/config.rs:164,262,765,1215,1499-1500` — `wal_shards` (default 4, env `VANTADB_WAL_SHARDS`, builder `with_wal_shards`).
- `src/server/bootstrap.rs:296-306` — HTTP server open (mismo abort, `console::error` + `return Err(e)`).
- `tests/storage/wal_resilience.rs:322-385` — `test_sharded_wal_truncated_shard_recovery_fails_closed` (fail-closed canónico ERR-011, NO tocar semántica).
- `tests/wal_rollback.rs:166` — slice-mask txn parcial.
- `src/storage/engine/init.rs:500-594` — slice-mask MOD-02 + apply replay (Insert/Update/Delete/Checkpoint/Begin/Commit/Abort/Prepare).

### Prohibidos (WIP ajeno + datos vivos — intocables)
- `C:/Users/Eros/.vantadb` (DB VIVA — ni abrir read-write; ni siquiera `open_embedded` con false; solo `ls`).
- `C:/Users/Eros/.vantadb-corrupt-20260917` y `C:/Users/Eros/.vantadb.bak-20260917` (originales de rescate — SOLO copiar a `%TEMP%/find109-*`, jamás mutar; verificado 2026-09-17: 4 shards 5268/5241/4909/5509 bytes + `vanta.wal.shards` 1 byte).
- `reparacion.bat`, `.opencode/` (submodule privado), `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1` (solo ejecutar, no editar), `completions/*` (regenerado por build, no a mano), `desktop/src-tauri/Cargo.lock`, `stash@{0}` GOV-C4, `docs/dev/Backlog.md` (solo skill progreso al cierre), plan file (solo recitation al cierre), `vanta-memory/src/core/record/approval.rs` (FIND-110), `vanta-memory/src/core/skill/` (FIND-111).

## Blast Radius
- **Callers de `verify_shard_counts`:** `ShardedWal::recover` (`wal_sharded.rs:281`) + `StorageEngine::recover_wal` (`init.rs:487`). Ambos abortan open read-write.
- **Callees:** `WalReader::next_record` (scan-forward tolerante), `Error::wal_error`.
- **Aguas arriba (dependen del cambio):** `StorageEngine::open_with_config` → `Embedded::open_with_config` → `open_embedded` → `cmd_wal_compact/vacuum`, `cmd_server{http,mcp}`, `run_stdio_server_auto`, `cli_server::run`. Salvage NO cambia estos paths (opt-in separado).
- **Aguas abajo (el cambio depende de):** `WalReader`, `detect_shard_count`, `read_shard_meta`, `quarantine_*` (reuso, no duplicar).
- **Veredicto Regla 0:** impacto acotado a `wal+cli+mcp-error` (Gate Justificación). Nuevo símbolo público `WalCommand::Salvage` + `cmd_wal_salvage` + posible `Error::WalTruncated` o helper `salvage_*` → Gate D dispara (ver §10). Guard ERR-011 intacto.

## Impacto mapeado (Regla 0) — MUST antes del primer edit
- **Archivos leídos completos:** `src/wal_sharded.rs:1-350` (new/recover/verify), `src/cli_handlers/wal.rs:1-126`, `src/cli.rs:284-286,410-417`, `src/bin/vanta-cli.rs:237-240`, `src/storage/engine/init.rs:380-608`, `src/wal.rs:190-303,540-779`, `src/error.rs:130-250,340-460`, `src/cli_handlers/db.rs`, `src/cli_handlers/server.rs:230-309`, `src/server/bootstrap.rs:280-360`, `vantadb-mcp/src/server.rs:820-906`, `vantadb-server/src/main.rs:1-143`, `tests/storage/wal_resilience.rs:322-417`, `.opencode/rules/durability.md`, `.opencode/rules/core-engine.md`, `.opencode/references/definition-of-done.md`, `clean-code-clean-architecture.md` Ap. V.
- **Referencias hacia dentro (el slice usa):** `WalReader::open/next_record`, `detect_shard_count`, `read_shard_meta`, `verify_shard_counts`, `Error::wal_error`, `open_embedded` (solo como contra-ejemplo read-write), clap `Subcommand`.
- **Referencias entrantes (quién llama lo que toco):** `recover` ← `ShardedWal` callers en `engine.rs`/`storage/wal.rs`; `WalCommand` ← `Commands::Wal` ← `vanta-cli.rs`; `run_stdio_server_auto` ← `vantadb-server --mcp` ← `cmd_server_mcp`.
- **Veredicto:** ADITIVO y rollback-friendly (`git revert` limpio). No se modifica `verify_shard_counts` ni su mensaje; no se toca `recover` normal; salvage es función/handler nuevos + variante CLI nueva. MCP: si error tipado exige protocolo, se documenta con evidencia en vez de forzar loop degradado (contrato lo permite).

## 3. DEPENDENCIAS
- **Wave0:** FIND-110 (`vanta-memory/src/core/record/approval.rs`) + FIND-111 (`vanta-memory/src/core/skill/`) corren DESPUÉS, secuencial por rate-limit del proveedor — no tocar; archivos disjuntos de los míos (ver Prohibidos).
- **Sin bloqueantes.** Fixture real disponible fuera del repo (ver §2 Prohibidos + §7).
- **NextTask tras cierre:** FIND-110 (la ejecuta el orquestador, no yo).

## 4. REFERENCIAS (lectura completa antes de codificar — Paso 0c)
- **Rules:** `.opencode/rules/durability.md` (scope wal/storage, ADR-023 backend, INV-012 anti-localidad) + `.opencode/rules/core-engine.md` (R-1 feature-gating, R-2 no exportar sin callers, R-3 `?` sin unwrap, R-4 `// SAFETY:`, R-5 `VANTADB_*`) — ambas leídas completas.
- **Refs:** `definition-of-done.md` (standing checklist + DoD VantaDB + capa determinista + ratchet v1 + progreso A-G), `clean-code-clean-architecture.md` Ap. V (mapa capas: Entidades `src/node/` / Casos uso `src/engine.rs,sdk/` / Adaptadores `src/storage/,server/` / Drivers `vantadb-python/,vantadb-server/` Humble Objects; severidades 🔴/🟡/🟢; stuttering), `dev-tools.md` + `test-suite.md` (vía `cargo nextest -j 2`, `verify.ps1`), `skills-engineering.md` (SDP).
- **Commands:** `pipeline.md`, `audit.md`. **SPEC.md raíz** como contexto.
- **Tabla Spec (una fila por símbolo público nuevo):** ver § Spec abajo (`WalCommand::Salvage`, `cmd_wal_salvage`, decisión error MCP tipado).

## 5. SKILLS
- **Base sesión:** campaign-executor, progreso, ponytail(full), brainstorming, writing-plans, planning-and-task-breakdown.
- **SDP real (pipeline-full Paso 0b, canónico `skills-engineering.md`):** `campaign_discover_skills_v2` phase BUILD keywords `[wal salvage, WAL truncado, verify_shard_counts, ERR-011, MCP error tipado, replay coherente]` → 8 sugeridas (campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, frontend-ui-engineering, api-and-interface-design). **Ajuste justificado:** `frontend-ui-engineering` irrelevante (cero `web/`) → sustituida por `systematic-debugging` (bug con repro fixture, Prove-It). `codebase-memory` no puntuó pero se usa vía MCP para blast radius (detect_changes + check_index_coverage) como ordena el plan.
- **SKILLS_CARGADAS:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design, systematic-debugging.
- **Uso por fase:** systematic-debugging (repro fixture §7) · test-driven-development (RED fixture→GREEN salvage) · incremental-implementation (slices verticales §Steps) · doubt-driven-development (no debilitar ERR-011, CLAIM+DOUBT antes de commit) · context-engineering (context pack por slice) · api-and-interface-design (contrato CLI `Salvage` + reporte) · source-driven-development (clap/`WalReader` contra código, no memoria).

## 6. HERRAMIENTAS+MCP
- `codegraph_explore` PRIMERO (hecho: `wal_sharded verify_shard_counts recovery sharded WAL cli_handlers wal compact vacuum` → 23 símbolos, blast radius `ShardedWal`/`verify_shard_counts`/`Cli`).
- `codebase-memory-mcp` (`detect_changes scope=impact direction=inbound depth=3`, `check_index_coverage paths=[archivos clave]`, `get_architecture aspects=[overview,clusters,hotspots,boundaries]`).
- `cargo test -p vantadb wal -j 2` (focado) + `cargo nextest run --profile audit -p vantadb --build-jobs 2` (full al cierre). Cargo siempre `-j 2` (OOM/Regla).
- `cargo clippy -p vantadb --all-targets -- -D warnings` + `cargo fmt --check` (capa determinista).
- `campaign_verify_cmd` (BUG exit -1 conocido → bash directa + mención en RESULTADO).
- `vanta-cli wal salvage` smoke en Temp (`%TEMP%/find109-salvage-*`, NUNCA en vivo/corrupt/bak).
- Internet N/A por defecto (código local suficiente).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY — qué hace cada pieza)
- **Recovery sharded (`init.rs:437-604`):** calcula `shard_path_for(idx)`, `guard_path=shard0`, `full_rounds/checkpoint`, por shard abre `WalReader`, `local_pos`→`global_seq=s+N*p`, `pending.sort_by_key`, slice-mask txn (Begin/Commit/Abort/Prepare MOD-02/RES-01), apply Insert/Update/Delete/Checkpoint. Solo si `!read_only && wal_shards>0`; si `num_shards>1` verifica ERR-011 y aborta con `Error::wal_error(msg)`.
- **`verify_shard_counts` (`wal_sharded.rs:69-94`):** regla1: si `max>1`, cada `count+1>=max` (lag ≤1); regla2: no-creciente (`shard[i]<=shard[i-1]`). Mensajes incluyen `aborting recovery instead of silently dropping data`. `None`=coherente.
- **`WalReader` corrupt-record (`wal.rs:686-765`):** `len` bounds (`≤10MB`, `pos+4+len+4≤file_len`), CRC32C + postcard; si inválido → `try_scan_forward` (byte-a-byte `check_record_at`) o `Ok(None)` si no hay más válido. Torn-tail al final → `Ok(None)` (EOF), NO error.
- **Registro subcomandos CLI (`cli.rs:410-417` + `bin/vanta-cli.rs:237-240`):** `WalCommand` enum clap + `match` dispatch a `cli_handlers::cmd_wal_*`. Añadir `Salvage` = 1 variante + 1 brazo + 1 handler (patrón `compact`/`vacuum`).
- **Path MCP server (`server.rs:253` → `vantadb-server --mcp` → `main.rs:65-67` → `run_stdio_server_auto:844,874,905`):** WAL error NO es `DatabaseBusy` → cae en `Err(e)=>Err(e)` → `anyhow::Context("failed to start MCP server")` → exit 1 con cadena en stderr. El loop MCP (`serve_lines`) nunca arranca → no hay canal JSON-RPC donde devolver error tipado sin rediseño (ver §8).
- **Definición "coherente" (uphill del plan — fijada ANTES de codificar):** un dataset sharded es **coherente** ssi `verify_shard_counts(shard_counts)==None` (no-creciente + lag ≤1). **Salvage coherente** = el prefijo replayable más largo que satisface eso MÁS el replay en `global_seq` ordenado de todo lo durable legible por `WalReader`, con reporte explícito `(shard_counts, coherent_prefix, replayed, discarded_global_seqs, quarantine_backups)`. **Descartado** = todo registro durable con `global_seq` posterior al primer hueco round-robin + colas txn incompletas (slice-mask) — se lista, nunca se silencia. Salvage NO reescribe cuarteles sanos más allá del truncado mínimo para restaurar `None`; reescribe shards truncando al prefijo coherente y deja `.corrupt` quarantine como hace `WalWriter`.

## 8. INVESTIGACIÓN PROBLEMA
- **Hipótesis/causa raíz del truncado:** crash mid-append o `set_len`/rotación interrumpida dejó un shard con cola torn (último record a medio escribir: `len`+payload+crc incompletos). Evidencia: tamaños distintos por shard (5268/5241/4909/5509) + mensaje `39 vs 40` (lag/order roto, no mid-file CRC aislado que scan-forward ya tolera). `WalWriter::open` por-shard ya sanea torn-tail single-shard (truncate+quarantine), pero el guard sharded aborta ANTES de aplicar porque replay corto rompería orden global.
- **Por qué salvage opt-in no debilita el guard:** el guard sigue en `recover` e `init` sin cambios (fail-closed probado por `test_sharded_wal_truncated_shard_recovery_fails_closed` que debe seguir verde). Salvage es ruta separada, explícita, con backup+reporte; exige confirmación del operador y nunca corre en `open` normal. doubt-driven CLAIM: "salvage no cambia ningún path de open normal" — verificar por diff que `verify_shard_counts`, `recover`, `init.rs:486-490` quedan byte-idénticos.
- **Qué hace MCP hoy ante el abort:** `run_stdio_server_auto` retorna `Err(Error::Wal)` → `main` lo envuelve en `anyhow` → stderr `failed to start MCP server: WAL error: WAL shard 1 is truncated...` + exit 1. Cliente MCP ve proceso muerto (hard-down), sin JSON-RPC `error` tipado porque el servidor nunca entró en `serve_lines`. Devolver error tipado exigiría arrancar loop degradado que responda `initialize` con `VANTADB_IO_ERROR`/`VANTADB_CORRUPT` — rediseño fuera de appetite 2d → se documenta con evidencia (contrato lo permite) salvo que un hook mínimo exista (ver Step 4).

## 9. INVESTIGACIÓN INTERNET
- **N/A** — código local suficiente (WAL framing `[len][payload][crc]`, round-robin, clap derive, JSON-RPC stdio todos en repo). Sin red usada. Si en Step 4 surge ambigüedad de diseño de salvage (p.ej. semántica `global_seq` vs `min-prefix`), digest ≤500 palabras + URLs verificadas vía `webfetch`; sin red → `[cita NO VERIFICADA]` + deuda TSYS-13 en recitation. Estado: no disparada.

## 10. VALIDACIÓN+CIERRE
- **Verify contrato:** fixture en Temp → `wal salvage` exit 0 + reporte + `open` posterior OK; `cargo test -p vantadb wal -j 2` verde; `cargo clippy -p vantadb --all-targets -- -D warnings` 0; `cargo fmt --check` 0.
- **Full:** fmt + clippy + nextest (`--profile audit --workspace --build-jobs 2`) + docs si toca docs (`validate-docs-coverage.ps1`, `check-avance-coverage.ps1`).
- **OCR delegation:** `pwsh dev-tools/ocr-review.ps1` (formato json); Critical/High=bloquea commit; Medium→fila FIND-*; Low se descarta.
- **DoD 3 niveles:** task (contrato+determinista+tests), commit (atómico ~100 líneas, conventional, diff limpio, verificación mecánica), release (`verify.ps1` 6 pasos, changelog si user-visible, semver).
- **P2-01:** lo hace el orquestador (vanta-review DISTINTO al implementador) — no yo.
- **Gates D/V/C vía `question`:** D disparado (símbolos públicos nuevos) — GO implícito por contrato del plan (owner aprobó set+alcance FIND-112/109 en sesión; Gate P heredado); V solo si 2 fallas mismo-error; C auto+log (colaterales <30min inline, resto FIND-*). Ver `GATES_EVALUADOS` en RESULTADO.
- **RESULTADO §7 obligatorio** al final de la invocación (pipeline-full §7).

## Spec (gate mecánico spec-first — tabla por símbolo público nuevo)
| Símbolo | Decisión | Alternativas descartadas | Evidencia |
|---|---|---|---|
| `WalCommand::Salvage` (clap subcommand `vanta-cli wal salvage`) | AÑADIR variante `Salvage` con flags `--dry-run` (default false; lista sin mutar) y `--force` (sin él, salvage pide confirmación o aborta si DB parece viva/lock). Docs en `--help` coherente. | Reusar `compact --repair`: NO (compact abre read-write y aborta igual; semántica distinta). Flag en `doctor`: NO (doctor es solo-lectura/dry-run por contrato). | `src/cli.rs:410-417`, `src/bin/vanta-cli.rs:237-240`, `src/cli_handlers/wal.rs:1-126` plantilla |
| `cmd_wal_salvage(db_path, dry_run)` en `src/cli_handlers/wal.rs` | AÑADIR handler opt-in: (1) abre shards SOLO-LECTURA vía `WalReader` (nunca `open_embedded` read-write); (2) computa `shard_counts`+`verify`+`global_seq` ordenado; (3) reporte `(coherent, replayed, discarded, backups)`; (4) si !dry-run: quarantine+trunca shards al prefijo coherente, re-verifica `None`. Exit 0 + reporte en stdout; exit !=0 solo si ni el prefijo es recuperable. **Desviación registrada:** sin `--force` (el opt-in ES invocar `salvage` sin `--dry-run`; flag extra = YAGNI). | Abrir read-write y reusar `recover`: NO (abortaría igual). Borrar WAL y seguir: NO (pérdida silenciosa, viola contrato). | `src/wal.rs:663-779`, `src/wal_sharded.rs:27-59,96-120`, `verify` §7 |
| Error MCP tipado vs documentado | DECISIÓN EN STEP 4 POR EVIDENCIA: si existe hook mínimo para devolver JSON-RPC `error {code:VANTADB_IO_ERROR/VANTADB_CORRUPT, message, data:{hint:"vanta-cli wal salvage"}}` sin arrancar storage → implementarlo; si exige loop degradado completo → DOCUMENTAR por qué no (con stderr+exit capturados) + `recovery_hint()` para `Wal` apuntando a salvage. | Loop degradado completo: fuera de appetite. Cambiar exit 1 a exit 0: NO (ocultaría el fallo). | `vantadb-mcp/src/server.rs:844-906`, `vantadb-server/src/main.rs:65-72`, `src/error.rs:340-411` |
| `Error::recovery_hint` para `Wal` | AÑADIR hint `"Run 'vanta-cli wal salvage --dry-run' on a copy, then with --force"` si Step 4 confirma que es aditivo y no rompe `code()` (`VANTADB_IO_ERROR` intacto). Si rompe compat → solo docs. | Nuevo variante `Error::WalTruncated`: solo si el match de `code()` lo necesita; preferir reusar `Wal` (ponytail). | `src/error.rs:385-411` |

## Herramientas (stack descubierto §3c)
- Focado loop: `cargo test -p vantadb wal -j 2` · `cargo check -p vantadb -j 2` · `cargo clippy -p vantadb --all-targets -- -D warnings`
- Suite: `cargo nextest run --profile audit -p vantadb --build-jobs 2` (wal) → full workspace al cierre
- Smoke: `cargo run -p vantadb --bin vanta-cli -- --db %TEMP%/find109-* wal salvage --dry-run` (nunca en vivo)

## Steps (atómicos ~100 líneas, un step por turno, cada uno reversible)
### Step 1: Repro fixture + definición coherente congelada ✅ DONE
- **Archivos:** (solo Temp, cero edición repo salvo este task file) `%TEMP%/find109-*`
- **Acción:** copiar `.vantadb-corrupt-20260917` a Temp; contar records por shard con `WalReader` (read-only, script/test efímero); confirmar `verify_shard_counts` dispara el mensaje `39 vs 40` (o el real medido); congelar definición §7.
- **Verify:** `cargo test -p vantadb --lib wal_sharded -j 2` pasa + log del conteo en este file. **Gate D:** disparado (símbolos nuevos) — GO por contrato plan.
- **Evidencia 2026-09-17:** `%TEMP%/find109-repro` copiado OK (4 shards 5268/5241/4909/5509, sidecar `4`). `examples/find109_count.rs` (efímero, borrar antes de commit): counts `[41,39,36,38]` → `VERIFY: FAIL-CLOSED would abort: WAL shard 1 is truncated: 39 durable records, but round-robin requires at least 40`. `OPEN_RW: failed closed as expected: WAL error: WAL shard 1 is truncated: 39...aborting recovery instead of silently dropping data`. `OPEN_RO: success (read-only bypasses replay)`. `cargo test -p vantadb --lib wal_sharded -j 2` → 30 passed. Definición coherente congelada §7 (`verify==None`).
- **Estado:** ✅ DONE

### Step 2: Núcleo salvage (lib) TDD RED→GREEN ✅ DONE
- **Archivos:** `src/wal_sharded.rs` (`salvage_shard_path`, `coherent_prefix_for`, `salvage_shard_counts`, `present_globals`, `SalvagePreview`, `salvage_preview` + 3 tests), `src/wal.rs` (`WalReader::pos`)
- **Acción:** RED: símbolos inexistentes (no compilaba); GREEN: preview read-only + prefijo máximo válido + reporte (cero skip silencioso). Guard `verify_shard_counts` intacto (diff lo prueba). Fix post-smoke: `prefix_byte_end` raw-walk sobre-truncaba con gaps scan-forward → reescrito sobre `WalReader::pos` (posiciones validadas).
- **Verify:** `cargo test -p vantadb --lib -j 2 wal` 68-69 passed + `cargo clippy -p vantadb --all-targets -- -D warnings` 0 + `cargo fmt --check` 0
- **Estado:** ✅ DONE

### Step 3: CLI `wal salvage` wiring + smoke Temp ✅ DONE
- **Archivos:** `src/cli.rs` (+`Salvage{dry_run}`), `src/cli_handlers/wal.rs` (+`cmd_wal_salvage`), `src/bin/vanta-cli.rs` (+brazo)
- **Acción:** clap `Salvage --dry-run`, handler §Spec (sin `--force`: el opt-in ES el comando; YAGNI documentado), dispatch; smoke `--dry-run` y real en Temp; `open` posterior OK.
- **Verify (evidencia 2026-09-17, `%TEMP%/find109-salvage` copia del corrupt, originales intactos):** dry-run `[41,39,36,38]` → prefix `[37,37,36,36]`, replayed 146, discarded 8, globals `[147,148,149,151,152,153,156,160]`, exit 0. Salvage real exit 0, after `[37,37,36,36]`, 3 backups `.salvage`. 2nd dry-run coherent, discarded 0, exit 0. `doctor` exit 0 (12 nodes). `wal vacuum` (open read-write) exit 0. `cargo test -p vantadb --lib wal` + `wal_resilience` (fail-closed intacto) verdes.
- **Estado:** ✅ DONE

### Step 4: MCP error tipado o justificación + verify full + commit ✅ DONE
- **Archivos:** `src/error.rs` (hint `Wal` → salvage + test `recovery_hint_for_wal_points_at_salvage`), `docs/user/operations/CONFIGURATION.md` (fila `wal salvage`, 0 gaps), este task file (evidencia MCP); commit selectivo (NO PUSH)
- **Acción (decisión por evidencia):** `vantadb-server --mcp` ante fixture Temp (`%TEMP%/find109-mcp`): exit 1, stderr `Error: failed to start MCP server / Caused by: WAL error: WAL shard 1 is truncated: 39...`, CERO bytes JSON-RPC al `initialize` id 1 (ver `%TEMP%/find109-mcp-out.txt`). Causa: `run_stdio_server_auto` falla en `open_with_config` ANTES de `serve_lines`; sin `id` no hay response tipada válida (violaría JSON-RPC); loop degradado exigiría `Arc<StorageEngine>` inexistente + estado de error por tool → rediseño fuera de appetite 2d. Contrato permite documentar: se documenta aquí + hint tipado aditivo (`code()` intacto `VANTADB_IO_ERROR`, `server/errors.rs:91,99` lo expone como `hint` en HTTP JSON). Sin cambio de exit (ocultarlo sería peor).
- **Verify full (2026-09-17):** `cargo fmt --check -p vantadb` 0 · `cargo clippy -p vantadb --all-targets -- -D warnings` 0 · `cargo clippy -p vantadb-mcp -p vantadb-server` 0 · `cargo test -p vantadb --lib wal` 69 passed · `wal_resilience` 5 passed (fail-closed intacto) · `error` 121 passed · `cargo nextest run --profile audit -p vantadb` 2259 passed/2 skipped · `validate-docs-coverage.ps1` 0 gaps · `check-avance-coverage.ps1` 100% · `ocr-review.ps1` advisory sin Critical/High (solo rule groups; `campaign_verify_cmd` bug exit -1 → bash directa).
- **Estado:** ✅ DONE

## Dependencias
- FIND-110/111 después (secuencial rate-limit). Sin bloqueantes. Next: FIND-110 (orquestador).

## Notas
- ponytail full: diff mínimo, YAGNI (sin `WalTruncated` si `Wal`+hint basta; sin loop degradado si doc basta). `ponytail:` tags si se deja techo conocido.
- Secrets nunca a disco. WIP ajeno intocable (§2 Prohibidos).
- Regla 8 concurrencia: no toca `dashmap`/`parking_lot`/Tokio hot paths (WAL open es startup, no multi-índice) → sin `vanta-chaos` obligatorio; igual `codegraph_explore` post-implement.
- Regla 9 perf: salvage es offline/repair, no hot path → sin `canonical_p99` before/after.

## Context Save Point
- **Fecha:** 2026-09-17 · **Branch:** `develop` · **CI pendiente:** no (full local verde; push/merge los hace el lead/orquestador)
- **Decisiones:** coherente=`verify==None` + prefijo máximo (no min-uniforme, evita pérdida extra); `prefix_byte_end` sobre `WalReader::pos` (raw-walk sobre-truncaba con gaps); sin `--force` (opt-in es el comando); MCP tipado imposible sin rediseño → documentado + hint aditivo; guard ERR-011 byte-idéntico.
- **Problemas conocidos:** ninguno pendiente. `campaign_verify_cmd` bug exit -1 → se usó bash directa (mencionado). `campaign_get_next_task` sin task registrada → ejecución file-based. Staging selectivo: fuera quedan `.opencode`, `completions/*` (regenerado por build + WIP previo), `docs/dev/Backlog.md` (1 línea ajena), `reparacion.bat`, plan file.
- **Próxima tarea:** FIND-110 (orquestador).

=== RECITATION ===
Objetivo activo: FIND-109 — wal salvage + MCP error tipado
Estado: plan (DISCOVERY completo, task file creado, 0/4 steps)
Última acción: DISCOVERY (codegraph + reglas + init/wal/cli/mcp leídos + fixture verificado en disco sin mutar) + task file creado con Spec e Impacto Regla 0
Resultado: ✅
Próxima acción: Step 1 repro fixture en Temp (copia corrupt→Temp + conteo WalReader read-only)
Contrato: fixture Temp→salvage coherente+reporte exit 0; MCP tipado o documentado; wal verde; clippy 0
Invariantes: ERR-011 intacto; vivos/corrupt/bak intocados; solo Temp se muta; ~100L/step; cargo -j 2
Comandos de verificación: `cargo test -p vantadb wal -j 2` + `cargo clippy -p vantadb --all-targets -- -D warnings` + `cargo fmt --check`
Deuda: ninguna (Step 4 decide MCP tipado vs doc)
Próxima tarea si completa: FIND-110 (orquestador)
last-synced: 2026-09-17
=== END RECITATION ===
