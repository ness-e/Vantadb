# MEM-58 — Consolidación UI ↔ context engine real (H4-bis)

Plan: `docs/plans/2026-08-22-vanta-ultima-milla.md` Task 9 · Wave 2 · deps Task 1 (engine) ✅ + Task 8 (IPC) ✅
Contrato: "test e2e Tauri: consolidar con backend disponible usa pipeline real (report con modo/tokens); sin backend → fallback heurístico actual"
Restricciones: NO tocar core `vantadb` · NO commitear · NO editar plan file · sin deps nuevas.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `desktop/src-tauri/src/commands/memory.rs` (614L) — patrón MEM-53: DTOs wire, `offload`, `run_recall`, tests con TempDir/AppState.
- `desktop/src/components/consolidate/consolidate-core.ts` (99L, NOTA: el plan dice `src/lib/` pero la ruta real es `src/components/consolidate/`) — heurística D16a pura node-testable.
- `desktop/src/components/consolidate/ConsolidateLens.tsx` (254L) — orquesta bridge vía `../../vanta`.
- `vanta-memory/src/context_engine/engine.rs:199-271` — `assemble_with_recall` firma completa; `IntegratedContext` es `Serialize+Deserialize`.
- `vanta-memory/src/services/pipeline_worker.rs:388-480` — patrón canónico de llamada (recall → prepend/append → assemble).
- `vanta-memory/src/context_engine/mod.rs` (28L) — re-exports pub: `assemble_with_recall, AssembleConfig, IntegratedContext, ChatMessage, ChatRole, TokenEstimator`.

**Referencias entrantes (qué depende):**
- `ConsolidateLens.tsx` ← `WorkspaceShell.tsx` (solo monta la lente; contrato de props no cambia).
- `lib.rs:153-187` invoke_handler — hay que registrar el comando nuevo ahí.
- Tests TS: `desktop/src/consolidate-core.test.ts` importa desde `./components/consolidate/consolidate-core.ts` (node --test).

**Referencias salientes:** consolidate-core/lens → `desktop/src/vanta.ts` (transport.call); memory.rs → vanta-memory context_engine + core::hooks.

**Veredicto de impacto:** cambio ADITIVO. Nuevo comando `vanta_context_assemble` en commands/memory.rs (patrón MEM-53, devuelve `IntegratedContext` serializable directamente). Frontend: wrapper en vanta.ts + helpers puros en consolidate-core.ts + ruta backend-first con catch→fallback heurístico en ConsolidateLens. Ningún caller existente cambia de comportamiento. Blast radius acotado a desktop.

## Steps

### Step 1 — Rust: comando IPC `vanta_context_assemble` ✅ DONE
- [x] DTO no necesario: usar `ChatMessage`/`IntegratedContext` de `vanta_memory::context_engine` (ya serde).
- [x] Comando async: `active_embedded()` → offload → recall blocks vía `run_recall` existente cuando user_text+session_key presentes → `assemble_with_recall(estimator default, cfg default, protected_prefix 0)`.
- [x] Registrar en `lib.rs` invoke_handler.
- [x] **FIX sesión 2:** `#[tauri::command(rename_all = "snake_case")]` — el wrapper TS envía `budget_tokens`/`session_key`; el default camelCase de Tauri v2 no los matchearía ("missing required key budgetTokens"). Validado contra docs Tauri v2.
- [x] Tests: (a) backend+sesión con scene → pipeline real inyecta recall + report modo/tokens; (b) sin session_key → compaction corre pero recall_injected=false; (c) budget 0 → InvalidConfig. (en worktree de la sesión previa)

### Step 2 — Verify Rust ⬜ PENDING
- [ ] `cargo check -p vantadb-desktop --all-targets`
- [ ] `cargo nextest run -p vantadb-desktop`
- [ ] `cargo fmt --check` / clippy del crate (sin -D, grep warnings propios — lección MEM-41)

### Step 3 — Frontend: ruta real + fallback ⬜ PENDING
- [ ] `vanta.ts`: wrapper `contextAssemble(...)` via transport.call("vanta_context_assemble").
- [ ] `consolidate-core.ts`: helpers puros `toHistory(records)` + `formatAssembleReport(outcome)` + tipo `AssembledContext`.
- [ ] `ConsolidateLens.tsx`: si Tauri transport && active → intentar backend primero; catch → fallback heurístico actual (sin cambiar su comportamiento).
- [ ] Tests node --test para los helpers nuevos en consolidate-core.test.ts.

### Step 4 — Verify frontend + cierre ⬜ PENDING
- [ ] `node --test desktop/src/consolidate-core.test.ts`
- [ ] `npm run build` (tsc && vite build) en desktop/
- [ ] `campaign_update_task_state taskId=9 completed` + recitation §12. SIN commit (orden explícita).

## Context Save Point
(ninguno aún)
