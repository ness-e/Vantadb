# OLD-14: MessageThread / GcWorker for Agentic Chat

## Metadata
- **Plan file:** N/A (direct backlog task)
- **Fuente:** `docs/Backlog.md` Phase 9 (Old Docs Rescue) línea 179
- **Esfuerzo:** 🟡 1d-1sem
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-07-26T16:00
- **last-synced:** 2026-07-26T16:00
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/builder.rs` (si se expone vía `VantaEmbedded`), `src/sdk/` (SDK layer), tests |
| Callees | `src/gc.rs` (GcWorker for TTL cleanup), `src/storage/engine` (StorageEngine for persistence), `src/node.rs` (UnifiedNode, FieldValue) |
| Implicaciones | API pública se expande (nuevo type `MessageThread`). No rompe contratos existentes (additive). Tests nuevos requeridos. |

## Contrato
"`cargo check -p vantadb` pasa, `cargo nextest run --test message_thread_test` pasa (4+ tests), y el comportamiento específico es: crear/send/read_messages/list_threads/delete funcionan con persistencia"

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt)
- rust-analyzer-mcp (diagnostics, goto def)
- codegraph_explore (blast radius)

## Investigation Notes

### Existing code
- `GcWorker` (`src/gc.rs`): TTL-based GC, `register_ttl()`, `sweep()`, `purge_ttl_for_deleted()`. Manages a `BTreeMap<u64, Vec<u128>>` for expiration timestamps. Operates on `StorageEngine`.
- No existing `MessageThread` or agentic chat types in the core crate.
- `src/sdk/builder.rs`: `VantaEmbedded` wraps `StorageEngine`, has method `graphrag_search()` — pattern to follow.

### What MessageThread needs to do
1. Store conversation threads (id, title, messages, metadata) using existing `UnifiedNode` / `VantaMemoryInput` patterns
2. Each thread = a namespace or a parent node with child message nodes
3. Messages within a thread: ordered list of `{role, content, timestamp}`
4. Optional TTL via `GcWorker::register_ttl()` for auto-expiry of threads
5. Expose via `VantaEmbedded::create_thread()`, `send_message()`, `get_thread()`, `list_threads()`, `delete_thread()`

### Design decisions
- Use the existing graph storage: thread as a node, messages as properties or child nodes with edges
- Keep it simple: no separate DB table, just `StorageEngine` + `put()`/`get()`/`delete()` on structured nodes
- Leverage `GcWorker` for thread TTL — threads can expire after inactivity
- Ponytail: no over-engineering. Start with in-memory messages in the thread struct, persist on commit.

## Steps

### Step 1: Create `src/agentic/mod.rs` + `src/agentic/thread.rs`
- **Archivos:** `src/agentic/mod.rs`, `src/agentic/thread.rs`
- **Acción:** Define `MessageThread` struct with fields: `thread_id: u128`, `title: String`, `messages: Vec<Message>`, `created_at: u64`, `updated_at: u64`, `metadata: HashMap<String, String>`. Define `Message` struct: `role: String (system/user/assistant), content: String, timestamp: u64, metadata: HashMap<String, String>`. Define `ThreadStore` with `StorageEngine` ref for CRUD.
- **Verify:** `cargo check -p vantadb`

### Step 2: Register `pub mod agentic;` in `src/lib.rs`
- **Archivos:** `src/lib.rs`
- **Acción:** Add `pub mod agentic;` after other module declarations.
- **Verify:** `cargo check -p vantadb`

### Step 3: Implement ThreadStore CRUD operations
- **Archivos:** `src/agentic/thread.rs`
- **Acción:** Implement `ThreadStore::new(engine)`, `create_thread(title, metadata) -> u128`, `send_message(thread_id, role, content) -> ()`, `get_thread(thread_id) -> Option<MessageThread>`, `list_threads(limit, offset) -> Vec<MessageThread>`, `delete_thread(thread_id) -> ()`. Store threads as nodes with namespace `_threads` and messages as properties on the thread node (JSON-serialized).
- **Verify:** `cargo check -p vantadb`

### Step 4: Wire GcWorker for thread TTL
- **Archivos:** `src/agentic/thread.rs`
- **Acción:** Add optional `ttl_secs` param to `create_thread()`. On creation, call `gc.register_ttl(thread_id, ttl_secs)`. On `send_message()`, refresh the TTL by re-registering (prevents expiry while active).
- **Verify:** `cargo check -p vantadb`

### Step 5: Expose via VantaEmbedded in `src/sdk/builder.rs`
- **Archivos:** `src/sdk/builder.rs`
- **Acción:** Add methods: `create_thread(&self, title, ttl_secs)`, `send_message(&self, thread_id, role, content)`, `get_thread(&self, thread_id)`, `list_threads(&self, limit, offset)`, `delete_thread(&self, thread_id)`. Delegate to `ThreadStore`.
- **Verify:** `cargo check -p vantadb`

### Step 6: Write tests `tests/message_thread_test.rs`
- **Archivos:** `tests/message_thread_test.rs`, `Cargo.toml` (register test target)
- **Acción:** 4+ tests: `test_create_and_send`, `test_list_threads`, `test_delete_thread`, `test_thread_ttl_expiry` (verify sweep removes expired).
- **Verify:** `cargo nextest run --test message_thread_test`

### Step 7: fmt + clippy
- **Acción:** `cargo fmt`, `cargo clippy -p vantadb -- -D warnings`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Verify:** All pass.

## Dependencias
- Ninguna

## Notas
- Ponytail: no over-engineer the thread model. Use JSON-serialized messages on a single node per thread. If message counts grow >1000 per thread, revisit with child-node layout.
- GcWorker integration is optional (only if `ttl_secs` is provided).
- Thread IDs use the same `u128` ID space as existing nodes (via `Uuid::new_v4()` or similar).
