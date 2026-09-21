# MEM-53: Desktop IPC commands para pipeline vanta-memory (H4)

## Metadata
- **Plan file:** docs/plans/2026-08-22-vanta-ultima-milla.md (Task 8)
- **Creado:** 2026-08-22T12:00
- **last-synced:** 2026-08-22T12:00
- **Estado:** ⬜ PENDING

## Impacto mapeado (Regla 0)
Archivos leídos completos:
- `desktop/src-tauri/src/lib.rs` (196L) — AppState, invoke_handler :148, patrón `#[tauri::command]`
- `desktop/src-tauri/Cargo.toml` (47L) — workspace aislado; dep `vantadb` path `../..` features subset
- `desktop/src-tauri/src/commands/mod.rs`, `commands/data.rs` — patrón thin-wrapper + `State<'_, AppState>`
- `desktop/src-tauri/src/connections/manager.rs` (825L) — RwLock Inner, active_id, patrón de métodos
- `desktop/src-tauri/src/connections/native.rs` (1148L) — `db: VantaEmbedded` privado, helper `blocking()`
- `desktop/src-tauri/src/connections/trait.rs` (258L) — `VantaConnection` dyn-safe, defaults Unsupported
- `desktop/src-tauri/src/error.rs` (213L) — VantaError variants (Native/Unsupported/Serialization)
- `vanta-memory`: lib.rs exports, auto_capture.rs, auto_recall.rs (RecallScope/Config/Result), persona_generator.rs (get_persona/PersonaRecord ✓Serialize), scene_index.rs (list/current_scene), scene_format.rs (SceneBlock ✓Serialize), abstractions/types.rs (SceneIndexEntry ✓Serialize), skill sink/archive (StoredSkill ✓Serialize), ingest/callback.rs (IngestProgress ✓Serialize, ProgressTracker::wiki_status)

Referencias entrantes: invoke_handler lista comandos; frontend invoca por nombre.
Referencias salientes: vanta-memory (nueva dep), vantadb SDK list/list_namespaces.
Veredicto: cambios aditivos (trait default method + manager método + módulo nuevo + registro en handler). Sin tocar WAL/vector/storage. Riesgo bajo.

## Blast Radius
Callers: invoke_handler (lib.rs:148) — agregar 7 entradas.
Callees: vanta-memory pub APIs (AutoCaptureHook, perform_auto_recall, get_persona, list_scenes/current_scene, StoredSkill via db.list, ProgressTracker::wiki_status).
Implicaciones: desktop crate tiene [workspace] propio → nueva dep path `../../vanta-memory` no toca el lockfile del repo raíz. Feature unification: vanta-memory default-features=false (sin llm-driver/tiktoken).

## Contrato
"comandos invocables desde frontend con roundtrip a DB embebida; tests Rust de cada command"
Verify: `cargo check -p vantadb-desktop --all-targets && cargo nextest run -p vantadb-desktop && cargo fmt --check && cargo clippy -p vantadb-desktop --all-targets`

## Herramientas
- codegraph, terminal (cargo), MCP campaign

## Steps
### Step 1: Dep + acceso al handle embebido
- **Archivos:** `Cargo.toml`, `connections/trait.rs`, `connections/native.rs`, `connections/manager.rs`
- **Acción:** dep vanta-memory; trait default `as_native()`; accessor `db()`; `ConnectionManager::active_embedded()`
- **Verify:** `cargo check -p vantadb-desktop` ✅
- **Estado:** ✅ DONE

### Step 2: commands/memory.rs — 7 comandos + tests
- **Archivos:** `commands/memory.rs`, `commands/mod.rs`, `lib.rs`
- **Acción:** memory_capture/recall/persona_get/scenes_list/scene_current/skills_list/wiki_status; ProgressTracker en AppState; registro en invoke_handler; tests roundtrip (12)
- **Verify:** `cargo nextest run -p vantadb-desktop` ✅ 85/85
- **Estado:** ✅ DONE

### Step 3: Verify full + cierre
- **Verify:** fmt --check ✅ · clippy --all-targets ✅ (0 warnings) · nextest ✅ · cargo audit ✅ (0 vulnerabilidades)
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (independiente)

## Notas
- RecallResult/AutoCaptureResult NO son Serialize → DTOs espejo en memory.rs.
- skills_list: no hay API "list" en vanta-memory → iterar namespaces `skills_extract/*` vía db.list_namespaces + db.list y parsear StoredSkill del payload (delegación pura, sin duplicar sanitización).
- wiki_status: ProgressTracker vive en AppState (desktop aún no corre ingests → None hasta tarea futura).
- L0 capture usa AutoCaptureHook (única entry point pública, LLM-free).

## Context Save Point
- **Fecha:** 2026-08-22T13:10
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** downcast via trait default method `as_native()` (dyn-safe, sin Any); handle `VantaEmbedded` clonado fuera del lock RwLock + spawn_blocking (patrón native.rs); skills_list delega iterando namespaces `skills_extract/*` vía SDK (sin duplicar sanitización); wiki_status pollea ProgressTracker en AppState (ingest desktop llega en tarea futura → None hoy)
- **Problemas conocidos:** h2 bumped 0.4.15→0.4.18 en lockfile desktop (RUSTSEC-2026-0258); recall L1 keyword lo cubren los 472 tests de vanta-memory — test desktop cubre ruta scene-navigation + Ok(None)
- **Próxima tarea:** Task 9 del plan P33

## Security checklist (trust boundary IPC frontend→Rust)
- Args deserializados owned por Tauri; roles filtrados y contenido sanitizado por AutoCaptureHook (vanta-memory); session keys sanitizadas por el crate al escribir (`sanitize_component`).
- Errores mapeados a VentaError serializable (sin leaks de tipos core).
- cargo audit ✅ 0 vulnerabilidades (h2 fixed).
