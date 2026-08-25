# MCP-34a: wrapper MCP snapshot_create

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 4)
- **Fuente:** plan file Task 4 (desglose viable de MCP-34 DEFER)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Rust core SDK + MCP binding (vanta-worker)
- **Turns estimados:** 5
- **Creado:** 2026-08-25
- **Estado:** ⏳ COMPLETED-CÓDIGO (verificación full de lead pendiente: clippy full-workspace bloqueado por MOD-06 insert.rs)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/handlers/tools.rs` (handle_tools_call, handle_tools_list) |
| Callees | `src/sdk/builder.rs:253` (`VantaEmbedded::create_snapshot`), `src/storage/engine/mod.rs:507,540` (`StorageEngine::create_snapshot` → `FsSnapshot`), `src/storage/mod.rs:12` (re-export FsSnapshot) |
| Implicaciones | Tool MCP nuevo `snapshot_create` (wrapper fino sobre API pública existente). NO toca WAL/storage internals. `FsSnapshot` no Serialize → result JSON manual. `snapshot_restore` NO existe (DEFER core). No cambia API pública del core. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/mod.rs:151-159,504-584` (FsSnapshot + create_snapshot + list_snapshots), `src/sdk/builder.rs:247-263` (create_snapshot/list_snapshots SDK), `vantadb-mcp/src/handlers/tools.rs` (patrón handle_tools_call + list_snapshots:461,1467), `vantadb-mcp/src/validation.rs:11-35` (validate_identifier), `vantadb-mcp/tests/mcp_tests.rs` (setup_storage:13, recovery_call:3508, tests MCP-26), `.opencode/rules/server-mcp.md`, `.opencode/rules/durability.md`, `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME).
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `handle_tools_call` (68 callers en server.rs/lib.rs + 5 test files); `list_snapshots` (callers builder.rs + tools.rs).
- **Archivos que referencian a los editados (referencias entrantes):** `handle_tools_list` registra el tool (lista de tools); tests mcp_tests.rs/skills_tests.rs/code_tests.rs/wiki_tests.rs.
- **Veredicto impacto:** bajo — adición de un tool MCP wrapper sobre método público existente. No cambia comportamiento de tools existentes. `validate_identifier` NO bloquea separadores de path → agregar check explícito anti path-traversal (trust boundary: name → subdir de snapshots).

## Contrato
"tool `snapshot_create` (name + result `{"path","created_at"}`); `cargo test -p vantadb-mcp --test mcp_tests` pasa; docs ×2 hash SAME"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `snapshot_restore` NO se implementa (queda DEFER como feature core). No tocar `src/wal.rs`, `src/vector/`, `src/storage/` (propiedad de Arch/Engine) — solo usar la API pública `create_snapshot`. FsSnapshot no Serialize → construir JSON manualmente. Docs ×2 SKILL.md deben quedar hash SAME.
- **Comandos de verificación:** `cargo test -p vantadb-mcp --test mcp_tests`; `cargo check -p vantadb-mcp`; `cargo fmt --check`; `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`
- **Deuda pendiente:** ninguna (snapshot_restore queda como feature core DEFER, fuera de este wrapper)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — wrapper fino sobre API existente; no introduce unsafe ni clones en hot path.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Tool `snapshot_create` registrado + handler; test round-trip (create → list lo contiene → dir existe) pasa; docs ×2 hash SAME |
| **Commit** | El lead commitea (sub-agente NO commitea). Cambio: tools.rs + mcp_tests.rs + 2 SKILL.md |
| **Release** | No aplica (wrapper MCP, sin bump de crate) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — aplica parcial: `name` es input de usuario que se convierte en subdir bajo `<data_dir>/snapshots`. Trust boundary → `validate_identifier` + check explícito de separadores de path (`/`, `\`, `.`, `..`) para prevenir path traversal.
- [ ] **PERFORMANCE** — NO aplica: wrapper sobre hard-link/copy; no toca hot paths del engine.

## Steps

### Step 1: Registrar tool `snapshot_create` en handle_tools_list
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** agregar entrada `snapshot_create` (name + inputSchema con `name` required) tras `list_snapshots` (~line 464).
- **Verify:** grep `"snapshot_create"` en tools.rs → ✅
- **Estado:** ✅

### Step 2: Handler `snapshot_create` en handle_tools_call
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** agregar match arm `"snapshot_create"` tras `list_snapshots` (~line 1472). Validar name (validate_identifier + anti path-traversal), llamar `embedded.create_snapshot(name)`, construir JSON `{"path","created_at"}` manualmente (FsSnapshot no Serialize).
- **Verify:** cargo check -p vantadb-mcp → ✅ (lib compiló)
- **Estado:** ✅

### Step 3: Test round-trip en mcp_tests.rs
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs`
- **Acción:** test `test_snapshot_create_round_trip` (create → list contiene → dir existe → path traversal rechazado) tras el test MCP-26; agregar `snapshot_create` al tool-list assertion.
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests snapshot` → 2 passed (snapshot_create + list_snapshots) → ✅
- **Estado:** ✅

### Step 4: Docs ×2 hash SAME
- **Archivos:** `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md`
- **Acción:** documentar `snapshot_create` (tras `list_snapshots`), actualizar counts 72→73 / core 42→43. Mantener ambos archivos byte-idénticos (Copy-Item).
- **Verify:** `Get-FileHash` de ambos → `54346F3784051530A23C4ABA8AC2C82D64135284D66F580572CDBB0E21169C62` SAME → ✅
- **Estado:** ✅

### Step 5: Verificación contrato
- **Archivos:** —
- **Acción:** `cargo test -p vantadb-mcp --test mcp_tests` + fmt + clippy + check.
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests` → 71 passed ✅. `cargo fmt --check` sobre mis 2 archivos → limpio ✅ (fail restante solo en src/sdk/api.rs = FIND-31 paralelo). `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` → falla SOLO en src/storage/engine/insert.rs (`&mut *stats` → `&mut stats`) = MOD-06 paralelo, fuera de scope. `cargo check -p vantadb-mcp` → ✅ (lib compiló).
- **Estado:** ✅ (código verificado; clippy full-workspace bloqueado por MOD-06, ver Notas)
- **Nota:** durante la verificación, `src/storage/engine/insert.rs` (MOD-06 paralelo) estuvo roto (braces) → mi test target pasó en árbol verde; re-corrido full suite 71/71 OK tras MOD-06.

## Dependencias
- Ninguna (Wave 1, independiente)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (lead delega antes de marcar COMPLETED)
- **Enfoque:** ¿wrapper fino sin tocar core? ¿validación anti path-traversal correcta? ¿docs hash SAME?
- **Cómo se probó:** `cargo test -p vantadb-mcp --test mcp_tests` (mecánico)
- **Veredicto:** pendiente

## Notas
- `validate_identifier` (validation.rs:11) solo valida empty/max_len/null byte — NO bloquea separadores de path. Como `name` va a `<data_dir>/snapshots/<name>`, agrego check explícito de `/`, `\`, `.`, `..` (trust boundary).
- `FsSnapshot { path: PathBuf, created_at: Instant }` no deriva Serialize → `created_at` se formatea con `format!("{:?}", ...)`.
- `snapshot_restore` NO existe en core → NO se implementa (DEFER, feature core de Arch/Engine).
