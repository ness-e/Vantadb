# TASK-6: MEM-57 — Parser claude-code (classify + extract user text)

## Metadata
- **Plan file:** `docs/plans/2026-08-22-vanta-ultima-milla.md` (P33, Task 6)
- **Creado:** 2026-08-22
- **Estado:** ✅ COMPLETED (cerrado 2026-08-23 — implementado y commiteado como f76f2c23; WIP stale limpiado)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** TDAM fuente @ `97f9465`: `MemoryProxy/src/agent-adapters/{types.ts (56L), claude-code.ts (25L), ../common/cc-request-classifier.ts (70L), ../common/user-text-extractor.ts (29L)}`. Local: `vanta-proxy/src/session.rs` (328L), `vanta-proxy/src/lib.rs` (22L), `vanta-proxy/src/capture.rs` (182L), `vanta-proxy/src/mem_command.rs` (1-70), `vanta-proxy/Cargo.toml`.
- **Referencias entrantes:** `session.rs` desde `lib.rs:20`; `capture::last_user_text` consumido por handlers (write-back L0); `mem_command::extract_text` usado por `mem_command::parse` y `capture`.
- **Referencias salientes:** nuevo módulo es hoja (solo `serde_json::Value`); sin deps nuevas (serde_json ya está).
- **Veredicto:** módulo nuevo aditivo como sub-módulo de `session` (`src/session/claude_code.rs` + `pub mod claude_code;` en session.rs — layout 2018+, session.rs queda intacto). Integración mínima: `capture::last_user_text` refina su extracción de arrays delegando en `extract_last_user_text` (misma firma pública, comportamiento: último bloque text en vez de concatenar todos — los system-reminders prependidos dejan de contaminar el turno capturado). Un test de capture se ajusta al refinamiento. NO tocar `wal/vector/storage/core`, ni handlers (routing forks es Task 2), ni mem_command (path OpenAI).

## Contrato
"tests D19 port de TDAM agent-adapters/claude-code.ts: classifyCcRequest (main/fork/sidequery vía cache_control marker) + extractLastUserText (salta system-reminder blocks)"

## Herramientas
- Read/Edit/Write, codegraph, bash (cargo), campaign_verify_cmd

## Steps
### Step 1: Crear `vanta-proxy/src/session/claude_code.rs` + wiring + tests D19
- **Archivos:** `vanta-proxy/src/session/claude_code.rs` (nuevo), `vanta-proxy/src/session.rs` (+1 línea `pub mod claude_code;`)
- **Acción:** port fiel: `CcRequestKind {Main,Fork,Sidequery}`, `find_last_cache_control_index` (scan backwards por content blocks con key `cache_control`), `classify_cc_request` (marker n-1→Main, n-2→Fork, resto Main; sin marker → tools vacíos && thinking disabled → Sidequery, sino Main; body malformado → Main), `extract_last_user_text` (string→sí mismo; array→último bloque `{type:"text",text:string}` — los `<system-reminder>` van PREPENDIDOS, tomar el último los salta).
- **Verify:** `cargo test -p vanta-proxy claude_code`
- **Estado:** ✅ COMPLETED

### Step 2: Refinar integración en `capture.rs`
- **Archivos:** `vanta-proxy/src/capture.rs`
- **Acción:** `last_user_text` delega extracción de array-content en `claude_code::extract_last_user_text` (doc comment actualizado — reemplaza el inline básico anunciado como "Task 8/MEM-57"); test de multi-bloques ajustado (concat "ab" → último "b").
- **Verify:** `cargo test -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 3: Verify mecánico completo
- **Acción:** `cargo check -p vanta-proxy --all-targets` · `cargo test -p vanta-proxy` · `cargo fmt --check` · `cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings`
- **Estado:** ✅ COMPLETED

## Dependencias
- Complementa Task 1 (capture L0) y Task 2 (routing forks — consumirá `classify_cc_request`)
- Descartado: ports de codebuddy/codex/workbuddy/dsh (stubs conservativos TDAM — research P33)

## Notas
- Sin deps nuevas; sin unwrap/expect en código de producción (tests sí).
- Techo documentado (Risk Register 🟡×🟡): formato CC puede cambiar entre versiones — fixtures de tests basados en el formato actual; clasificación malformada degrada a Main (equivalente al status quo, nunca peor).
- SIN git commit (Regla del orquestador).
