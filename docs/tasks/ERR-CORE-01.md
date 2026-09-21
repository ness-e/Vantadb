# TASK-ID: ERR-CORE-01 — VantaError::code() 10 códigos canónicos + tipa catch-alls overflow

## Metadata
- **Plan file:** `docs/plans/2026-09-02-error-observability-excellence.md` (Task 1, Wave 1)
- **Creado:** 2026-09-02 (DISCOVERY sesión vanta-worker)
- **last-synced:** 2026-09-02
- **Estado:** ✅ COMPLETED (2026-09-02, commit e1fe7ec2 — contrato 8/8, nextest 2140/2140)
- **Esfuerzo:** 🟡 1d (appetite max 1d) · **Prioridad:** 🔴 (desbloquea ERR-PY-01/TS-01/MCP-01)

## Spec (decisiones con evidencia — fuente canónica: tabla §1.1+§1.2 de `docs/api/ERROR_HANDLING.md`, commit 962831ae)

### Mapeo final 30 variantes → 10 códigos `VANTADB_*`

| Código | Variantes Rust | # vars |
|---|---|---|
| `VANTADB_VALIDATION_ERROR` | DimensionMismatch, DuplicateNode, NodeIdCollision, CycleDetected, ValidationError, InvalidInput, IqlParseError, UnsupportedOperation, NoVectorForKey, **ExecutionConflict** | 10 |
| `VANTADB_NOT_FOUND` | NodeNotFound, NotFound | 2 |
| `VANTADB_TIMEOUT` | Timeout | 1 |
| `VANTADB_BUSY` | DatabaseBusy, NotInitialized | 2 |
| `VANTADB_RESOURCE_LIMIT` | ResourceLimit + **VectorLenOverflow (nueva)** + **EdgeCountOverflow (nueva)** | 1+2 |
| `VANTADB_CORRUPT` | WALVersionMismatch, IncompatibleFormat, SerializationError, SchemaError, RestoreError, BackupError | 6 |
| `VANTADB_INVALID_ARGUMENT` | IqlError | 1 |
| `VANTADB_IO_ERROR` | IoError, WalError, BackendError, CliError, SearchError, RuntimeError | 6 |
| `VANTADB_WASM_ERROR` | Generic (fallback) + wildcard `_` (variantes futuras) | 1 |
| `VANTADB_CLOSED` | (lifecycle de handle, fuera de `VantaError` — no lo emite `code()`) | 0 |

### Decisiones no triviales (justificación por evidencia)
1. **Firma `pub fn code(&self) -> &'static str`** (no `Option<&str>`): el match propio-crate es exhaustivo con wildcard de fallback → siempre devuelve código. Instrucción de tarea explícita; contrato grep `pub fn code` == 1. El snippet `Some(...)` de §1.2 del doc es un boceto del futuro, no firma.
2. **Split provisional VALIDATION vs INVALID_ARGUMENT se resuelve aquí** (el doc lo delegaba: "see ERR-CORE-01 for the final mapping"): `IqlParseError` → `VANTADB_VALIDATION_ERROR` (coincide con TS: `classifyWasmError` clasifica parse como VALIDATION_ERROR, doc §1.1 note); `IqlError` queda solo en `VANTADB_INVALID_ARGUMENT`. 1:1 con los 10 strings de la tabla + prefijo.
3. **`ExecutionConflict` → VALIDATION_ERROR**: la tabla §1.1 no lo lista (omisión del doc — las 10 filas cubren 29/30 vars); la familia conflictiva ya mapeada a VALIDATION es DuplicateNode/NodeIdCollision/CycleDetected → misma coherencia. Se corrige la fila del doc en Step 5.
4. **Nuevas variantes con los campos exactos de la instrucción**: `VectorLenOverflow { id: u128, len: usize, limit: u32 }`, `EdgeCountOverflow { id: u128, count: usize, limit: u16 }`. Display NUEVO (no reemplaza Display de `ResourceLimit`). `limit: u16` Display → "65535", preserva el assert `contains("65535")` del test ERR-029 en `ops.rs:395`.
5. **`is_retriable()` NO cambia**: las 2 variantes nuevas son límites duros de formato on-disk (retry no ayuda) → quedan `false`. Coherencia código↔retriable preservada para las 3 familias que la tabla declara ✅: TIMEOUT, BUSY, RESOURCE_LIMIT (ResourceLimit), IO_ERROR (BackendError, WalError). Test lo verifica.
6. **`recovery_hint()`**: nuevas variantes reciben hints accionables (split vector / reducir fan-out de edges); resto intacto.
7. **Catch-alls supervivientes** (scope discipline — NO se tipan, se documentan): `Generic(ChainedError)` (~40 callers `generic_error*` en storage/engine/*), `ResourceLimit(String)` en `governor.rs:50` (OOM soft-limit real), `vfile_mmap.rs:482` (alloc fallido), `engine/stats.rs:159` (memory pressure dinámica). Son límites de tiempo de ejecución con mensaje genuinamente variable — tiparlos no agrega match accionable.
8. **server/errors.rs**: `"code": e.code()` agregado a `vanta_error_response` (json! literal) y `query_error_response` (cambia a `json!` con los mismos campos que `QueryResponse` serializa: `node_id/nodes` son None → ya se skippean → body idéntico + campo nuevo `code`). **No se toca `QueryResponse`** (struct público con 5+ literales en handlers → agregar campo = breaking innecesario). Backward-compatible: campo aditivo. `vanta_error_status` intocado (nuevas vars caen a 500 como hoy caía ResourceLimit — sin cambio de comportamiento observable de status).

## Impacto mapeado (Regla 0)
- **Leídos completos:** `src/error.rs` (979L), `src/storage/ops.rs` (535L), `src/server/errors.rs` (182L), `docs/api/ERROR_HANDLING.md` (374L), plan Task 1, spec `docs/api/ERROR_HANDLING.md` §1.1/§1.2.
- **Referencias entrantes:** `VantaError` 64 callers in-crate + matches en crates externos (python `convert.rs:818` catch-all `_`, wasm `vanta_error_code` con wildcard, server) → `#[non_exhaustive]` hace adición de variantes minor-safe. Matches in-crate con exhaustividad potencial: `server/errors.rs` (_ presente), `bootstrap.rs:426,446`, `sdk/api.rs` tests (_ con panic), `skills/tests.rs` — todos verificados por `cargo check --all-targets`.
- **Call-sites del reemplazo (ops.rs):** 72 (Full), 86 (Binary), 100 (Turbo), 114 (SQ8), 130 (MmapFull) → `VectorLenOverflow`; 172 (edges > u16) → `EdgeCountOverflow`. Todos dentro de `write_node_to_vstore`.
- **Referencias salientes:** thiserror, std — sin deps nuevas.
- **Veredicto:** aditivo/minor. Único riesgo de ruptura: Display de call-sites reemplazados cambia prefijo "Resource limit exceeded:" → grep `exceeds u32|edge_count limit` muestra que SOLO el assert `contains("65535")` (ops.rs:395) depende del texto y se preserva. `classifyWasmError` (TS) regex-mirra prefijos de variantes EXISTENTES — ninguna tocada.
- **Premisa falsa corregida:** `src/index/search/core.rs` NO existe — revisado `src/index/search/alternate.rs` (1 uso) + módulo search. Ningún uso de ResourceLimit/Generic overflow allí.

## Archivos clave — evidencia de revisión (regla de oro del plan)
| Archivo | Evidencia | ¿Se edita? |
|---|---|---|
| src/error.rs | 979L leídos; 30 vars enumeradas; is_retriable/recovery_hint líneas 270-308 | ✅ |
| src/sdk/api.rs | rg VantaError=7 (ValidationError read_only + tests con `_`) | no |
| src/storage/ops.rs | 535L; 6 sites ResourceLimit(format!) 72/86/100/114/130/172 | ✅ |
| src/storage/vfile.rs | rg=22 usos (constructores helper/IoError) — sin overflow catch-alls | no |
| src/storage/engine/{mod,insert,get,delete,maintenance,init,ops}.rs | rg counts 8/8/4/6/14/11/4 — usan generic_error()/helpers, no construyen ResourceLimit overflow | no |
| src/index/graph.rs | rg=4 (InvalidInput, sin ResourceLimit) | no |
| src/index/ivf.rs | rg=1 | no |
| src/index/search/alternate.rs | rg=1 | no |
| src/index/search/core.rs | **no existe** — ver Spec | n/a |
| src/server/errors.rs | 182L leído; status mapping con `_` | ✅ |
| src/server/bootstrap.rs | rg=8; CliError(ChainedError::msg) — tipado, no catch-all | no |
| src/server/handlers.rs | rg=16; consume vanta_error_response/query_error_response JSON (no deserializa → campo extra seguro) | no |
| docs/api/ERROR_HANDLING.md | 374L leídos — tabla spec leída como fuente | ✅ |

## Contrato (verificación mecánica)
```bash
grep -c "pub fn code" src/error.rs                              # == 1
grep -cE "VANTADB_NOT_FOUND|VANTADB_VALIDATION|VANTADB_TIMEOUT|VANTADB_BUSY" src/error.rs   # >= 4
grep -cE "VectorLenOverflow|EdgeCountOverflow" src/error.rs     # >= 2
grep -c "Generic(String)" src/error.rs                          # == 0 (es Generic(ChainedError))
cargo check --workspace --all-targets                           # exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings   # exit 0
cargo test -p vantadb --lib error::tests::code_snapshot         # exit 0
cargo nextest run --profile audit -p vantadb                    # 0 failed
```

## SDP (Paso 0b)
`campaign_discover_skills(phase=BUILD, keywords=[VantaError, code, thiserror, ResourceLimit])` → base (campaign-executor, progreso, ponytail, writing-plans) + lifecycle BUILD (incremental-implementation, test-driven-development, context-engineering). Las 3 core + ponytail están inline en la definición del agente vanta-worker (§3a-3d) — no se re-leen (context flooding); `progreso` se carga en CIERRE (Trigger 1).

## Steps (TDD)

### Step 1: RED — tests que fallan
- **Archivos:** `src/error.rs` (mod tests)
- **Acción:** `code_snapshot` (29 mapeos actuales + 2 nuevas + wildcard), `code_prefix_consistency`, `new_variants_display_and_not_retriable`, `is_retriable_consistent_with_codes`
- **Verify:** `cargo test -p vantadb --lib error::tests::code_snapshot` → FALLA (compile error: `no method code`, variantes inexistentes) — RED legítimo
- **Estado:** ✅ DONE

### Step 2: GREEN — `code()` + 2 variantes nuevas en error.rs
- **Acción:** variantes tras `NoVectorForKey`; `pub fn code()` con match exhaustivo; hints nuevas; is_retriable intacto
- **Verify:** `cargo check -p vantadb` + test Step 1 verde
- **Estado:** ✅ DONE

### Step 3: Call-sites tipados en `src/storage/ops.rs`
- **Acción:** 5 overflow vector → `VectorLenOverflow{id,len,limit:u32::MAX}`; edges → `EdgeCountOverflow{id,count,limit:u16::MAX}` (construcción vía helper no necesaria — struct literal)
- **Verify:** `cargo test -p vantadb --lib storage::ops` (ERR-029 asserts pasan sin edición)
- **Estado:** ✅ DONE

### Step 4: `src/server/errors.rs` — `"code"` en JSON
- **Acción:** `vanta_error_response` + `query_error_response` (json! con campos equivalentes a QueryResponse serializado)
- **Verify:** test server::errors snapshot body contiene `"code":"VANTADB_NOT_FOUND"` (feature server)
- **Estado:** ✅ DONE

### Step 5: `docs/api/ERROR_HANDLING.md` — tabla canónica
- **Acción:** §1.1 → códigos `VANTADB_*` definitivos (ExecutionConflict agregado a VALIDATION row, IqlParseError fuera de INVALID_ARGUMENT, nota provisional→final, CLOSED marcado lifecycle-not-emitted, filas VectorLen/EdgeCountOverflow en RESOURCE_LIMIT); §Pending → resuelto; Changelog
- **Verify:** `grep -c VANTADB_ docs/api/ERROR_HANDLING.md` >= 12
- **Estado:** ✅ DONE

### Step 6: Verify completo + commit
- `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit -p vantadb` · contrato grep
- Commit: `feat(error): VantaError::code() 10 códigos canónicos + tipa vector/edge overflow (ERR-CORE-01)`
- NO stagear: `completions/*` (dirty pre-existente), `.opencode` submodule, `stash@{0}` intocado
- **Estado:** ✅ DONE

## Gate D
Disparado por el plan: Task 1 agrega `pub fn code()` (símbolo público nuevo). El usuario ya dio GO explícito en la instrucción de ejecución ("Decisión del plan ya tomada — NO preguntar de nuevo"). No requiere re-pregunta.

## Dependencias
- Bloquea: ERR-PY-01, ERR-TS-01, ERR-MCP-01 (consumen `code()`), ERR-DESK-01 (Domain code), ERR-OBS-01 (code tag en tracing).
- Requiere: nada (Wave 0 verde: af0bb8b8, 962831ae).
