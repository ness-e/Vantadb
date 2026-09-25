# D4a — Renames stuttering internos

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- **Tipo:** Rust rename interno (V.3 Don't Add Gratuitous Context).
- **Scope reducido y verificado hoy:** `src/agentic/thread.rs`: `create_thread` (:89) → `create`, `get_thread` (:173) → `get`, `list_threads` (:181) → `list`, `delete_thread` (:195) → `delete`.
- **Ya hecho (solo verificar, no re-renombrar):** `EntityStore::entity_*`, `SceneNodeStore::scene_node_*`, `Entity::entity_id`, `SceneNode::scene_name`, `ConnectionPool::is_saturated`. Si grep encuentra resto, incluirlo.
- **Blast radius (codegraph_explore ThreadStore 2026-09-12):** `ThreadStore` 2 callers (`src/agentic/mod.rs`, `src/sdk/builder.rs`); `get_thread` 4 callers (`thread.rs`, `vantadb-mcp/src/threads.rs`, `sdk/builder.rs`); `delete_thread` 3 callers; `list_threads` 4 callers; tests `tests/message_thread_test.rs`. Regla 0: `conversation_add` en `handlers.rs` usa `create_thread` — dejar todo compilando para B1.
- **PROHIBIDO:** tocar `src/error.rs` (es D4b); casos V.3 (Props/i18n/testids/CSS).

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- **Cambia:** solo `src/agentic/thread.rs` (4 renames) + callers necesarios para compilar (builder.rs, mcp threads.rs, handlers.rs, agentic/mod.rs) + alias serde donde haya formato persistido + docs si es pub.
- **NO cambia:** `src/error.rs`, V.3 excepciones, semántica/orden de efectos, API HTTP wire.
- **Contrato verificable:** `rg` de los 4 nombres viejos en `src/` solo pega en tests/docs históricos; `cargo check --workspace` verde; `nextest agentic` verde.
- **Verify:**
  - `rg -n "create_thread|get_thread|list_threads|delete_thread" src/`
  - `cargo check --workspace`
  - `cargo clippy -p vantadb --deny warnings` (o `--all-targets` según plan §7)
  - `cargo nextest run --profile audit -p vantadb agentic`

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- ### Step 1: Verificar `create_thread` → `create` ☐
  - **Archivos:** `src/agentic/thread.rs:89` + callers (`rg create_thread`)
  - **Acción:** Regla 0 grep callers ANTES; si ya renombrado, solo verificar; si queda resto, renombrar + actualizar callers.
  - **Verify:** `rg -n "create_thread" src/` + `cargo check --workspace`
  - **Estado:** ✅ DONE
- ### Step 2: Verificar `get_thread` → `get` ☐
  - **Archivos:** `src/agentic/thread.rs:173` + callers
  - **Acción:** idem Step 1.
  - **Verify:** `rg -n "get_thread" src/` + `cargo check --workspace`
  - **Estado:** ✅ DONE
- ### Step 3: Verificar `list_threads` → `list` ☐
  - **Archivos:** `src/agentic/thread.rs:181` + callers
  - **Acción:** idem Step 1.
  - **Verify:** `rg -n "list_threads" src/` + `cargo check --workspace`
  - **Estado:** ✅ DONE
- ### Step 4: Verificar `delete_thread` → `delete` ☐
  - **Archivos:** `src/agentic/thread.rs:195` + callers
  - **Acción:** idem Step 1.
  - **Verify:** `rg -n "delete_thread" src/` + `cargo check --workspace`
  - **Estado:** ✅ DONE
- ### Step 5: Verificar renames ya hechos (Entity/SceneNode/Pool) ☐
  - **Archivos:** `src/entity/mod.rs`, `src/entity/scene.rs`, pool file
  - **Acción:** `rg -n "entity_id|scene_name|is_saturated|entity_|scene_node_" src/`; si hay resto incluirlo.
  - **Verify:** `cargo check --workspace`
  - **Estado:** ✅ DONE
- ### Step 6: Verify final + cierre ☐
  - **Acción:** `cargo check --workspace` + `cargo clippy -p vantadb --deny warnings` + `cargo nextest run --profile audit -p vantadb agentic`
  - **Verify:** los 3 verdes
  - **Estado:** ✅ DONE

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO: ✅ (ver bloque al pie del turno 2026-09-12).
- **Hallazgo FACADE (para lead/B1):** el rename completo a `create/get/list/delete` en el facade `Embedded` (builder.rs) es INVIABLE: colisiona (E0592+E0034) con `Embedded::get/delete` (sdk/api/memory.rs) y `Embedded::list` (sdk/api/namespaces.rs). Decisión: ThreadStore renombrado (create/get/list/delete); facade conserva `*_thread` (desambiguación con carga, excepción V.3 en espíritu) + delega a los nuevos nombres. `conversation_add` (handlers.rs:1337) sigue compilando con `db.create_thread` → B1 no bloqueado.
- **Discrepancia vs briefing:** el briefing decía "YA ESTÁ HECHO"; el grep Regla 0 demostró que NO lo estaba (4 símbolos + 16 call sites). Se ejecutaron los renames.
- `src/error.rs` intacto (D4b). Sin commit (lo ejecuta el lead); cambios verificados en worktree.
