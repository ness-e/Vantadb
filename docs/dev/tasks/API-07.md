---
title: "API-07: W6 CLI — POSIX + `--json` global + flags simétricos"
kind: task
description: "--help × comando capturado (script, exit 0 por comando) Y --json en TODOS con salida completa (diff humano vs json: json parsea + payload íntegro sin ...) Y count sin DB → exit≠0 Y rg \\"println!\\(\\\\"{count}\\\\"\\)\\" = 0 Y lecturas sin..."
---

# API-07: W6 CLI — POSIX + `--json` global + flags simétricos

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` (§Task 7)
- **Fuente:** Backlog Phase 51 (fila API-07) vía plan api-ejecucion; ficha `docs/dev/tasks/API-STD-10.md` (C1–C4) + Gate P `docs/dev/tasks/API-STD-15.md`
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Tipo:** Rust (CLI superficie pública)
- **Turns estimados:** 25-35
- **Creado:** 2026-09-26T00:56
- **last-synced:** 2026-09-26T02:06
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 5/5 + smoke 56/56 + fmt/clippy/tests verdes; review P2-01 ronda 1 ❌ → fixes → ronda 2 ✅ APPROVE. Working tree sin commit (lo hace el LEAD; política owner).
- **Incógnitas (uphill):** 0 abiertas (ver Spec: 11/11 resueltas por Gate P + evidencia de código)
- **Pendientes (downhill):** 1 step (Step 12: re-review + cierre)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/bin/vanta-cli.rs` (dispatch), `tests/cli_tests.rs` (80+ call sites a handlers), `build.rs` + `completions/` (snapshot generado), usuarios CLI + agentes terminal + scripts docker/ops (docs), desktop sidecar (`desktop/src-tauri` spawnea `vanta-cli server --mcp`, no toca subcomandos de datos) |
| Callees | `Embedded`/`StorageEngine` (`.vanta.lock` fs2 shared/exclusive — `init.rs:161-232`), `sdk::put(MemoryInput)` → derived/text/sparse index ops (`impl_index.rs:154-240`), WAL flush, `console::Term` (TTY detect), `clap 4.4` derive |
| Implicaciones | CLI es superficie pública (Gate P fila CLI): `feat!:` breaking documentado (nadie usa los paquetes aún). Reads dejan de abrir RW (exclusive lock → shared lock; sin `ensure_indexes_current`) → requirió `put` vía SDK en el mismo cambio para que los índices estén al día al leer. Sin cambios de formato de datos en disco. Sin migración de datos. Tests existentes: 1 pin (`test_count_missing_db`) cambia por contrato; resto por firma (compilador). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/cli.rs` (492L), `src/bin/vanta-cli.rs` (287L), `src/cli_handlers/mod.rs`, `crud.rs` (570L), `search.rs` (507L), `server.rs` (357L), `data.rs` (339L), `diagnostics.rs` (588L), `index.rs` (250L), `namespace.rs` (132L), `wal.rs` (204L), `snapshot.rs` (39L), `migrate.rs` (311L), `backup.rs` (419L), `util.rs`, `fmt.rs`, `db.rs`, `tests/cli_tests.rs` (1502L), `build.rs`, `src/sdk/builder.rs:40-219`, `src/sdk/api/memory.rs` (put/get/delete), `src/sdk/api/namespaces.rs`, `src/sdk/serialization/impl_index.rs`, `src/storage/engine/init.rs:140-239`, `src/node/field.rs`, `docs/dev/tasks/API-STD-10.md`, `API-STD-15.md`, `.opencode/rules/api-contract.md`, `clean-code-clean-architecture.md` Apéndice V.
- **Archivos referenciados hacia dentro (imports/deps de los editados):** `cli.rs` ← `build.rs` (`#[path] mod cli`) + `bin/vanta-cli.rs`; handlers ← `crate::sdk`, `crate::node`, `crate::error`, `console`, `indicatif`, `web_time`; `search.rs` tests internos usan `cmd_put`.
- **Archivos que referencian a los editados (referencias entrantes):** `tests/cli_tests.rs` (handlers), `completions/_vanta-cli*` (flags), `docs/user/operations/{CONFIGURATION,SQLITE_MIGRATION_GUIDE,DISASTER_RECOVERY_RUNBOOK,DEPLOYMENT_GUIDE,PERFORMANCE_TUNING,BACKUP_RESTORE}.md` (ejemplos CLI), `vanta-mcp-local.ps1`/hooks (usan `mcp-call`/`server --mcp`, sin cambios de flags), `.github/workflows/desktop.yml` (`vanta-cli server --mcp` — intacto).
- **Veredicto impacto:** medio — superficie pública breaking controlada (`feat!`), sin cambios de persistencia; el riesgo real es lock/RO en lecturas (Regla 8, auditado abajo) y staleness de índices en DBs escritas por CLI viejo (documentado como deuda de migración).

## Contrato
"`--help` × comando capturado (script, exit 0 por comando) Y `--json` en TODOS con salida completa (diff humano vs json: json parsea + payload íntegro sin `...`) Y `count` sin DB → exit≠0 Y `rg \"println!\\(\\\"{count}\\\"\\)\"` = 0 Y lecturas sin `ensure_indexes_current`-RW (read-only open) o audit concurrencia (Regla 8) — evidencia §Regla 8"

## Spec (SDD — Gate P ya decidió la superficie; ítems derivados resueltos por evidencia)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | `--json` en TODOS | A: flag global en `Cli` (clap `global=true`) respetado por cada handler vs B: flag por comando (como hoy en 5) | A | ✅ Gate P: "`--json` global completo"; N/A justificado: `server`/`tui`/`completions` (sin salida de datos) y `mcp-call` (ya emite JSON-RPC verbatim) |
| 2 | Forma de `--json` | A: siempre doc JSON en stdout (objeto/array), nunca número crudo vs B: por-comando libre | A — consistencia de consumidores (agents/scripts); humano sigue en stdout por defecto, errores a stderr | ✅ evidencia: `crud.rs:551-553` (número crudo) es el outlier; resto de json existente ya emite objeto/array |
| 3 | `count --json` | A: `{"namespace","count","filter"}` vs B: mantener número crudo | A | ✅ contrato pide revisar `println!("{count}")`; breaking `feat!` aceptado (plan §Reglas globales) |
| 4 | `count` sin DB | A: `Err` → exit 1 vs B: `0`/exit 0 (actual) | A | ✅ contrato explícito: "`count` sin DB → exit≠0"; `0` es indistinguible de un count real |
| 5 | Lecturas RO | A: `open_embedded(path,true)` (shared lock, sin `ensure_indexes_current`) en search/similar/multi/all/count vs B: mantener RW + audit | A | ✅ Gate P: "lecturas RO"; viable porque #6 hace que `put` mantenga derived/text/sparse al día (`impl_index.rs:154-240`). Caveat legacy → Deuda |
| 6 | `put` vía SDK | A: `Embedded::put(MemoryInput)` + `flush()` vs B: bypass actual (UnifiedNode+`engine.insert`, `crud.rs:53-131`) | A | ✅ contrato "put vía SDK (no bypass)"; B no actualiza índices derivados → forzaba los opens RW |
| 7 | `--in`/`--out` simétricos | A: `--in` canónico import/restore (+alias oculto `--input`) + `--out` export/backup vs B: `--input` en todos | A | ✅ plan: "flags simétricos (`--in`/`--out`)"; alias conserva runbooks docs (`DISASTER_RECOVERY_RUNBOOK.md:65`) |
| 8 | `limit` vs `top-k` | A: `--limit` canónico + alias oculto `--top-k` vs B: `--top-k` | A | ✅ consistencia con `list`/`query`/`search` (3) + HTTP W2 (`limit`, API-03); SDK `top_k` es de librería, no CLI |
| 9 | Posicionales | A: el operando sujeto va posicional: `search <QUERY>` (+`--query` oculto), `search-multi/all [QUERY]`; `query <IQL>`, `migrate <target>`, `namespace info <ns>`, `snapshot create <name>` ya lo cumplen; comandos de registro siguen `--namespace/--key` vs B: todo flags | A | ✅ C1 (`Query.query` posicional como criterio) + POSIX utility syntax; alias `--query` preserva docs/scripts (`CONFIGURATION.md:406`) |
| 10 | Humano truncado | A: helper `truncate_for_term(s,max,is_term)` — trunca solo si stdout es TTY; char-safe (UTF-8) vs B: truncar siempre (actual, panics con multibyte) | A | ✅ Gate P: "humano truncado solo TTY"; B tiene panic real en `payload[..35]` con chars multibyte |
| 11 | Regla 8 (lock) | A: audit lock-order documentado (shared lock en reads, sin locks de escritura internos en RO) vs B: DEFER a vanta-chaos | A con evidencia §Regla 8; B queda como escalado si aparece deadlock | ✅ cambio rota RW (exclusive) → RO (shared) solo en 5 fns y elimina `ensure_indexes_current` del read path → menor superficie, no mayor |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `put` sigue rechazando metadata con prefijo `__vanta_`/NUL (`validate_metadata`, `serialization/mod.rs:136-150`) y JSON inválido; `put` sigue haciendo flush antes de salir (ERR-050b: RO reopen no replayea WAL); `search` en DB fresca tras `put` sigue funcionando sin `rebuild-index` (AUD-044, `search.rs:485-506`); `count` con filtro plano `{"field":value}` sigue siendo `$eq` (AUD-048); `mcp-call`/`server --mcp` intactos (WIRE-10/desktop sidecar); salidas humanas siguen en stdout y errores en stderr; exit codes existentes de `audit-index` (3) y `mcp-call` (1/2) intactos.
- **Comandos de verificación:** `cargo test --target-dir target/session-api01 -p vantadb --features cli,fjall,memmap2,fs2,roaring --test cli_tests` (suite CLI) · `cargo clippy -p vantadb --target-dir target/session-api01 --features cli,fjall,memmap2,fs2,roaring --all-targets -- -D warnings` · `cargo fmt --check` · smoke `target/session-api01/smoke-api07.ps1` (help × comando + diff humano/json + count exit).
- **Deuda pendiente:** (D1) DB escrita por CLI ≤ API-06 cuyo último write fue `put` bypass → índices derivados stale por 1 registro hasta el primer `put`/`rebuild-index` con el binario nuevo (mitigado: cualquier mutación nueva reconcilia; documentar en UPGRADE/CONFIGURATION). (D2) `cmd_server_mcp` spawn busca `vantadb-server` en PATH/cwd — preexistente, sin tocar (riesgo 2 del plan queda como FIND si vuelve a morder).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — se paga: (a) bypass de SDK en `cmd_put` (raíz de AUD-044), (b) 5× RW-open en lecturas (lock exclusive innecesario), (c) truncado con slicing UTF-8 inseguro (`payload[..35]` puede paniquear). Deuda nueva: D1 (migración legacy de índices) — documentada con mitigación, compensada por (a)+(b)+(c). Sin IDs P2 disponibles aplicables.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato § arriba 5/5 + `cli_tests` verdes + fmt/clippy verdes + smoke ok |
| **Commit** | 1 commit local `feat!(api): API-07 — ...` (lo hace el LEAD; política owner 2026-09-25, sin push) |
| **Release** | No aplica en esta tarea (release-plz decide versión/tag); verify full scoped al cierre |

## Herramientas necesarias
- cargo (build check/test con `--target-dir target/session-api01`, `--jobs 2` si hace falta)
- `codegraph_explore` (blast radius), `campaign_verify_cmd` por step
- `pwsh` (smoke script + captura de `--help`)

**Skills cargadas (SDP):** `incremental-implementation` (slices verticales), `test-driven-development` (RED→GREEN por contrato), `api-and-interface-design` (superficie pública CLI), `campaign-executor` + `ponytail` (base, plugin/MCP) · SDP: `source-driven-development` (verificación Clap 4.4 vía docs oficiales en Step 2), `doubt-driven-development` (review P2-01 al cierre), `context-engineering` (aplicada: jerarquía Rules→Spec→Source→Error). `frontend-ui-engineering` descartada (sin `web/`).

## Investigation Notes

### Regla 8 — Audit de lock-order en lecturas (evidencia, 2026-09-26)
- `StorageEngine::init_storage` (`src/storage/engine/init.rs:161-232`): abre `{.db}/.vanta.lock`; **read_only → `fs2::try_lock_shared`** (`:199-207`), RW → `try_lock_exclusive` (`:208-217`). Un solo lock de archivo, adquirido ANTES de cualquier lock in-process; timeout exponencial → `DatabaseBusy`.
- `Embedded::open_with_config` (`src/sdk/builder.rs:99-117`): `ensure_indexes_current()` corre SOLO si `!read_only` (`:113-114`) → el read path RO no toma locks de escritura ni toca particiones internas fuera de lecturas.
- Cambio API-07: `search`/`similar_to_key`/`search_multi`/`search_all`/`count` pasan de exclusive (RW) a shared (RO) → **reduce** alcance de lock (N lectores concurrentes OK; escritor ya no es requerido). No se introduce ningún lock nuevo ni orden de adquisición nuevo. Lock-order: `fs2 shared` → (nada más en RO). Sin inversión posible de orden porque no hay segundo lock.
- Riesgo residual (D1): staleness de índices en DB legacy (no es deadlock, es consistencia) → mitigado por #6 + documentado.
- Escalado si review detecta riesgo: `vanta-chaos` (stress 10k w/s + 1k r/s) — **no disparado** en esta tarea (cambio reduce lock scope; no hay evidencia de deadlock).

### Hallazgos C1–C4 (API-STD-10) → resolución
- C1 flags: `--in` (import) vs `restore --input` → unificados `--in` + alias. `Query.query` posicional → criterio POSIX aplicado (+ search family).
- C2 `--json` parcial (`:91,192,222,257,273`) → global + todos.
- C3 lossy: cajas + `payload[..35/80]` + json recortado → `truncate_for_term` TTY-gated + json completo.
- C4 concerns: `count` exit 0 sin DB → Err; `cmd_put` bypass → SDK; lecturas RW → RO.

### Verificación pendiente de docs oficiales (source-driven-development)
- Clap 4.4: `global = true`, `alias` (oculto), `required_unless_present`, `conflicts_with` — a validar en docs.rs/clap antes del Step 2 (fuente: `https://docs.rs/clap/4.4/clap/_derive/index.html`).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — todas resueltas en Spec (Gate P + evidencia de código) |
| Pendientes de ejecución (downhill) | 1 (Step 12: re-review + cierre) |
| % completado | ~95% (Steps 1-11 ✅ + fixes ronda 1; falta re-review) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluada: input de usuario (flags/JSON metadata) ya validado en boundary (clap + `parse_filter_json` + `validate_metadata`); sin dependencias nuevas; FFI no aplica; el cambio REDUCE locks. Sin hallazgos → no requiere `security-and-hardening` completa. (Justificación Regla: no toca auth/datos/red.)
- [x] **PERFORMANCE** — evaluada: no es hot path (CLI one-shot). Mejora medible indirecta: reads dejan de correr `ensure_indexes_current` (scan_nodes completo) en cada invocación → menos trabajo por read. Sin benchmark (Regla 9) porque no se declara claim de perf; nota, no claim.

## Steps

### Step 1: helpers de salida en `fmt.rs` (truncate TTY + JSON) — TDD
- **Archivos:** `src/cli_handlers/fmt.rs`
- **Acción:** RED: tests puros `truncate_for_term` (TTY trunca 35→`...`; pipe no trunca; multibyte no panic). GREEN: `stdout_is_term()`, `truncate_for_term(s,max,is_term)`, `field_value_to_json`, `print_json(&Value) -> Result<()>`.
- **Verify:** `cargo test --target-dir target/session-api01 -p vantadb --features cli --lib cli_handlers::fmt`
- **Estado:** ✅ DONE — RED compiló con E0425 ×10 (funciones inexistentes) → GREEN 4/4.

### Step 2: superficie de args en `cli.rs` (+ docs Clap oficiales)
- **Archivos:** `src/cli.rs`
- **Acción:** atributos Clap 4.4 validados en docs.rs (raw args → `Arg::global/alias/hide/required_unless_present/conflicts_with`); `Cli.json` global; `json` per-command removido (5 sitios); `--in`+alias `--input` (import/restore); `--limit`+alias `--top-k` (similar-to-key/search-multi/search-all); posicional `QUERY` (+`--query` oculto) en search/search-multi/search-all.
- **Verify:** `cargo check … --bin vanta-cli` + `--help` capturado (39/39)
- **Estado:** ✅ DONE.

### Step 3: plumbing `bin/vanta-cli.rs` (args.json + merge QUERY)
- **Archivos:** `src/bin/vanta-cli.rs`
- **Acción:** pasar `args.json` a todos los handlers; merge positional/flag para search family con error defensivo.
- **Verify:** `cargo check … --bins` ✅ (clippy unneeded-struct-pattern → `Commands::Stats` corregido)
- **Estado:** ✅ DONE.

### Step 4: `crud.rs` — put vía SDK + count exit + JSON
- **Archivos:** `src/cli_handlers/crud.rs`
- **Acción:** `cmd_put` → `Embedded::put(MemoryInput)` + flush (validación/metadata por SDK; reserved prefix sigue rechazado); `cmd_count` → missing = Err (exit 1) + json `{namespace,count,filter}`; json en get/list/delete/delete-by-filter; truncado TTY en list; helper `is_uninitialized_db`.
- **Verify:** `cargo test … --test cli_tests test_put test_count test_delete` ✅ (88/88 suite completa luego)
- **Estado:** ✅ DONE.

### Step 5: `search.rs` — RO open + JSON completo + TTY
- **Archivos:** `src/cli_handlers/search.rs`
- **Acción:** `open_readonly_or_empty` (shared lock + fallback uninitialized→vacío) en las 4 fns; truncado TTY; json completo.
- **Verify:** regresión AUD-044/API-07 `search_on_fresh_db_after_put_works_without_manual_rebuild` ✅ + `rg "open_embedded\(db_path, false\)" search.rs` = 0 ✅
- **Estado:** ✅ DONE.

### Step 6: `data.rs` + `index.rs` + `namespace.rs` (+`export_md.rs`) — JSON
- **Archivos:** `src/cli_handlers/{data,index,namespace,export_md}.rs`
- **Acción:** json export/import/query/rebuild/repair/namespace list-infor + export_md; fix contaminación stdout en rebuild (headers/success tras json → wrap `!json_output`); query blank line gated.
- **Verify:** smoke JSON purity (8 comandos) ✅ + `test_query/test_export/test_namespace/test_rebuild/test_repair` ✅
- **Estado:** ✅ DONE.

### Step 7: `diagnostics.rs` + `server.rs` — JSON (status/stats/inspect/doctor)
- **Archivos:** `src/cli_handlers/{diagnostics,server}.rs`
- **Acción:** json en status/stats(+physical_rss)/inspect(completo, vector full)/doctor(con fixes+diagnostics); truncado TTY en inspect.
- **Verify:** smoke `status/stats/inspect/doctor --json` ✅ + tests subset ✅
- **Estado:** ✅ DONE.

### Step 8: mantenimiento — JSON (backup/restore/wal/snapshot/migrate)
- **Archivos:** `src/cli_handlers/{backup,wal,snapshot,migrate}.rs`
- **Acción:** json en backup/restore(+dry-run)/wal compact-vacuum-salvage/snapshot create-list/migrate plan-run-check; `migrate run --json` exige `--force` (nunca prompt interactivo en modo JSON).
- **Verify:** smoke (backup/restore/wal×3/snapshot×2/migrate×3) ✅
- **Estado:** ✅ DONE.

### Step 9: `tests/cli_tests.rs` — call sites + tests nuevos (contrato)
- **Archivos:** `tests/cli_tests.rs`
- **Acción:** 74 call sites actualizados (script balanceado) + `test_count_missing_db_errors` (contrato) + módulo `api07_cli_binary` (binario real: count exit≠0, json completo, positional QUERY, aliases `--query`/`--top-k`/`--in`/`--input`, human piped sin truncar).
- **Verify:** `cargo test … --test cli_tests` → 88 passed / 0 failed ✅
- **Estado:** ✅ DONE.

### Step 10: docs mismo-PR + snapshot completions (Regla 3)
- **Archivos:** `docs/user/operations/{CONFIGURATION,SQLITE_MIGRATION_GUIDE,DISASTER_RECOVERY_RUNBOOK,DEPLOYMENT_GUIDE,PERFORMANCE_TUNING}.md`, `completions/*`
- **Acción:** flags canónicos (`--in`, `--limit`, positional QUERY), sección `--json` + nota de upgrade (rebuild-index para DBs legacy), `namespace info <ns>` positional; completions regeneradas (`VANTA_OUT_DIR=completions`, 4 shells).
- **Verify:** `rg "vanta-cli (restore|import) --input" docs/user docs/api` = 0 ✅; `git diff --stat completions/` = 4 files/337+ líneas ✅
- **Estado:** ✅ DONE.

### Step 11: smoke `smoke-api07.ps1` (help × comando + diff humano/json + count exit)
- **Archivos:** `target/session-api01/smoke-api07.ps1` + `target/session-api01/help/*.txt` (39 capturas)
- **Acción:** 47 checks: help 39/39, JSON puro (23 comandos), payload íntegro, count exit≠0, aliases, grep contract.
- **Verify:** `pwsh -NoProfile -File target/session-api01/smoke-api07.ps1` → **47/47 OK** ✅
- **Estado:** ✅ DONE.

### Step 12: verify full scoped + review P2-01 + cierre
- **Archivos:** task file (+ plan/Backlog si aplica)
- **Acción:** fmt/clippy/lib-tests/cli-tests + OCR advisory + review agente distinto + recitation.
- **Verify:** `campaign_verify_cmd` × contrato ✅ (fmt/clippy/cli_tests/build/smoke todos exit 0); review P2-01 ✅ APPROVE ronda 2
- **Estado:** ✅ DONE

## Evidencia de contrato (2026-09-26)

| # | Cláusula | Evidencia | Resultado |
|---|----------|-----------|-----------|
| 1 | `--help` × comando capturado | `target/session-api01/help/help_*.txt` (39 comandos/subcomandos, todos exit 0 con `Usage:`) — smoke check 1 | ✅ 39/39 |
| 2 | `--json` en TODOS con salida completa (diff humano vs json) | 23 checks de JSON parse + pureza (primer char `{`/`[`) + payload 304 chars íntegro en json; humano piped también íntegro (truncado solo TTY: unit test `truncate_for_term_*`); N/A documentado: `server`/`tui`/`completions` (sin salida de datos) y `mcp-call` (JSON-RPC verbatim) | ✅ |
| 3 | `count` sin DB → exit≠0 | smoke `count sin DB → exit!=0` + test binario `count_without_db_exits_nonzero` + `test_count_missing_db_errors` | ✅ |
| 4 | `rg "println!(\"{count}\")"` revisado | smoke check `no bare println!("{count}") in crud.rs` = 0; `count --json` → objeto `{namespace,count,filter}` | ✅ |
| 5 | lecturas sin `ensure_indexes_current`-RW (o audit Regla 8) | search/similar/multi/all/count → `open_embedded(path, true)` (shared lock); evidencia §Regla 8; smoke `search.rs reads are read-only` = 0 RW opens; `put` vía SDK mantiene índices (regresión AUD-044 verde) | ✅ |

## Context Save Point (2026-09-26)

Steps 1-11 ✅. Verificaciones: `cargo fmt --all --check` = 0 · clippy (lib+bins+cli_tests, `-D warnings`) = 0 · `cargo test --lib` 2124 passed/0 failed · `cli_tests` 88 passed/0 failed · smoke 47/47. Pendiente: review P2-01 (agente distinto) + sync plan/recitation. Sin commit (política owner; lo hace el lead).

## Dependencias
- API-06 ✅ (IQL estable) — completada
- Paralela con API-08 (disjunta) — no bloquea
- nextTask: API-09

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

### Ronda 1 — ❌ CHANGES-REQUIRED (2026-09-26)

- **Revisor:** `vanta-review` (sesión fresca `ses_f23c884f5ffeuBuIxhFvAYwvJ0` — sin participación en la implementación)
- **Enfoque:** contrato 5/5 re-ejecutado por el revisor (no auto-reporte); audit Regla 8 (shared/exclusive); pureza JSON path-por-path; aliases/consumidores; OCR spec aplicado (sin Critical/High).
- **Cómo se probó:** smoke 47/47; cli_tests 88/88; lib cli_handlers 19/19; fmt 0; `rg` cláusulas 4/5 = 0; repro en binario real: `count` sin DB exit 1 ✅; `get/list/delete/delete-by-filter` sin DB → warning humano en stdout + exit 0 (❌); `put --json -v` → tracing en stdout (❌).
- **Hallazgos:** [ALTA] pureza JSON en 4 paths missing-DB (`crud.rs` get/list/delete/delete-by-filter) · [MEDIA] `-v`+`--json` contamina stdout (`console.rs:105-132` writer=stdout) · [MEDIA] tabla de comandos docs con flags viejos como primarios (`CONFIGURATION.md:364-367`) · [BAJA] upgrade note incompleta (count shape/exit, migrate –force) · [BAJA] contadores stale del task file.
- **Veredicto:** ❌ CHANGES-REQUIRED.

### Ronda 1 — fixes aplicados (2026-09-26)

1. [ALTA] 4 paths missing-DB ahora emiten JSON vacío puro: `get` → `null`, `list` → `[]`, `delete` → `{deleted:false,…}`, `delete-by-filter` → `{deleted:0,…}` (+5 checks smoke).
2. [MEDIA] `console::init_logging` → `.with_writer(std::io::stderr)` en los 3 formatos (logs a stderr; stdout limpio para `--json` y protocolos) + `cmd_put -v` gateado con `!json_output` (+1 check smoke).
3. [MEDIA] Tabla de comandos `CONFIGURATION.md:364-367` sincronizada (QUERY posicional, `--limit`, aliases anotados).
4. [BAJA] Upgrade note ampliada (count shape/exit, `migrate run --json` requiere `--force`, rebuild-index legacy).
5. [BAJA] Contadores del task file actualizados (al cierre).

### Ronda 2 — mini re-review del delta (2026-09-26)

- **Revisor:** `vanta-review` (misma sesión de review `ses_f23c884f5ffeuBuIxhFvAYwvJ0`, contexto del veredicto ronda 1)
- **Enfoque:** mini re-review del delta de fixes ronda 1 (5/5), repro en binario real + smoke + suites
- **Cómo se probó:** repro binario missing-DB ×4 (`null`/`[]`/objetos, exit 0, stdout puro ✅); `put --json -v` → stdout JSON parseable + INFO en stderr ✅; smoke 56/56 exit 0 ✅; cli_tests 88/88 (serial; en paralelo falla por StorageFull con C: a 2.2GB — ambiental) ✅; lib cli_handlers 19/19 ✅; fmt 0 ✅. Delta acotado (`console.rs` +6, `crud.rs` +23, `CONFIGURATION.md` +12); fuera de scope intacto.
- **Veredicto:** ✅ **APPROVE** (2026-09-26) — los 5 hallazgos resueltos, sin hallazgos nuevos.

## Gate Review P2-01 — CERRADO ✅

| Ronda | Veredicto | Resumen |
|-------|-----------|---------|
| 1 | ❌ CHANGES-REQUIRED | 1 ALTA (pureza JSON missing-DB ×4) + 2 MEDIA (tracing stdout con `-v`; docs tabla) + 2 BAJA (upgrade note; contadores) |
| 2 | ✅ APPROVE | 5/5 resueltos, 0 hallazgos nuevos; contrato + suites re-verificados por el revisor |

## Notas
- ENTORNO: MCP server lockea `target/debug/vanta-cli.exe` → TODO cargo usa `--target-dir target/session-api01`; NO borrar `target/debug`.
- Política owner: commit local lo hace el LEAD al cierre; worker NO commitea ni pushea.
- Sin tocar: `vanta-memory/**`, `vantadb-mcp/**`, `vanta-proxy/**`, `src/server/**`, `src/parser/**`, `src/cli_handlers/mcp_call.rs` (WIRE-10, solo lectura).
