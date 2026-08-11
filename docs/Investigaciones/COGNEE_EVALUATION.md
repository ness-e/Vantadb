# Cognee Evaluation for VantaDB

> **Status:** Complete analysis  
> **Author:** OpenCode agent  
> **Date:** 2026-07-10  
> **Source repo:** `topoteretes/cognee` @ main  
> **Purpose:** Extract architectural patterns from Cognee for VantaDB MCP + AI context persistence

---

## 1. Executive Summary

Cognee is an **open-source AI memory platform** (27.5k stars) that gives AI agents persistent
memory across sessions. It operates in **two tiers**: a fast session cache (Redis/FS) and a
permanent knowledge graph (Neo4j/KuzuDB/Qdrant/LanceDB/ChromaDB + Postgres).

VantaDB *already* has efficient vector storage, HNSW/IVF indexes, MCP server, and CLIP/OpenAI/Ollama
embeddings — but **lacks** session lifecycle management, agent trace persistence/injection,
session→permanent sync, LLM usage tracking, and AI IDE plugin integration. These are exactly
the gaps that Cognee fills and that this document analyzes for extraction.

---

## 2. Cognee Architecture (Complete)

### 2.1 Layered Stack

```
┌──────────────────────────────────────────────────────┐
│  Claude Code / Codex Plugin                          │
│  6 lifecycle hooks + 3 skills                        │
│  ~/.cognee-plugin/claude-code/                       │
├──────────────────────────────────────────────────────┤
│  MCP Server (cognee-mcp)                             │
│  V1: cognify, search, delete, prune, list_data       │
│  V2: remember, recall, forget, improve               │
├──────────────────────────────────────────────────────┤
│  @agent_memory Decorator                             │
│  context retrieval → fn call → trace persist         │
│  Configurable: with_memory, with_session_memory,     │
│  save_session_traces, memory_query_fixed/from_method,│
│  memory_system_prompt, memory_top_k, last_n, etc.    │
├──────────────────────────────────────────────────────┤
│  Session Layer                                       │
│  SessionManager (Redis/FS cache)                     │
│  - QA entries (question/context/answer/feedback)     │
│  - Agent trace steps (origin_function, params,       │
│    return_value, status, session_feedback)           │
│  - Graph knowledge snapshots per session             │
│  - Session context entries (extracted lessons)       │
│  Session lifecycle (Postgres: SessionRecord)         │
│  - Status: running/completed/failed/abandoned        │
│  - Tokens in/out, cost USD, error count, model       │
│  - Per-model usage breakdown (SessionModelUsage)     │
├──────────────────────────────────────────────────────┤
│  improve() Pipeline                                  │
│  1. apply_feedback_weights: feedback scores→graph    │
│  2. extract_feedback_qas: QA→graph nodes             │
│  3. cognify: triplet enrichment (memify)             │
│  4. sync_graph_to_session: graph→session cache       │
├──────────────────────────────────────────────────────┤
│  Knowledge Graph Layer                               │
│  - Neo4j / KuzuDB for graph storage                 │
│  - Qdrant / LanceDB / ChromaDB for vectors          │
│  - Postgres / SQLite for metadata                   │
│  - Entity extraction + relationship detection (LLM)  │
│  - Hierarchical summaries                            │
└──────────────────────────────────────────────────────┘
```

### 2.2 Source Files Analyzed

| File | Purpose |
|------|---------|
| `cognee-mcp/src/server.py` | MCP server: V1 + V2 tools, background task mgmt |
| `cognee-mcp/src/cognee_client.py` | Dual-mode client (direct SDK ↔ HTTP API) |
| `cognee-mcp/src/server_utils.py` | Result formatting, validation |
| `cognee/modules/agent_memory/runtime.py` | AgentMemoryContext, memory retrieval, trace persist |
| `cognee/modules/agent_memory/decorator.py` | `@agent_memory` decorator |
| `cognee/modules/agent_memory/sanitization.py` | Value sanitization for trace storage |
| `cognee/modules/session_lifecycle/metrics.py` | SessionRecord CRUD, status transitions |
| `cognee/modules/session_lifecycle/models.py` | SessionRecord, SessionModelUsage (SQLAlchemy) |
| `cognee/modules/session_lifecycle/usage_tracking.py` | LLM token/cost tracking via ContextVar |
| `cognee/modules/agents/registry.py` | Agent connection registry + persistence |
| `cognee/modules/agents/models.py` | AgentConnection, AgentMemoryMode enum |
| `cognee/infrastructure/session/session_manager.py` | SessionManager: QA CRUD, agent traces, context |
| `cognee/infrastructure/session/session_agent_trace.py` | Per-step feedback generation (LLM + fallback) |
| `cognee/infrastructure/session/feedback_models.py` | Pydantic models: SessionTurnAnalysis, AgentTraceFeedbackSummary |
| `cognee/infrastructure/session/agent_context_extraction.py` | Live + batch agent lesson extraction |
| `cognee/infrastructure/session/session_context_builder.py` | Dedup + confidence-gated context updates |
| `cognee/infrastructure/session/session_context_models.py` | ContextProfile, CandidateUpdate models |
| `cognee/tasks/memify/apply_feedback_weights.py` | Feedback scores → graph node/edge weights |
| `cognee/tasks/memify/cognify_session.py` | Session text → graph pipeline |
| `cognee/tasks/memify/sync_graph_to_session.py` | Incremental graph→session cache sync |
| `cognee/tasks/memify/extract_agent_trace_feedbacks.py` | Trace extraction for memify |
| `cognee/tasks/memify/cognify_agent_trace_feedback.py` | Agent trace feedback → graph |
| `integrations/claude-code/README.md` | Plugin hooks, config, session strategies |

### 2.3 Data Flow

```
─── Session Lifecycle ──────────────────────────────────────

SessionStart → identity setup → ensure_and_touch_session()
                              → watcher bootstrap

UserPromptSubmit → retrieve_memory_context()
                 → session memory (recent trace feedback)
                 → cognee memory (semantic search, GRAPH_SUMMARY_COMPLETION)
                 → inject combined context into prompt

PostToolUse → add_agent_trace_step()
            → generate_agent_trace_feedback() [LLM summary]
            → store in session cache
            → _maybe_extract_agent_context() [live + batch]

Stop → add_agent_trace_step() (assistant answer)
     → optional transcript clear

SessionEnd → improve(session_ids=[...])
           → apply_feedback_weights()
           → persist QAs to graph
           → cognify (triplet enrichment)
           → sync_graph_to_session()
           → deactivate_agent_connection()

─── improve() Pipeline (detailed) ──────────────────────────

1. apply_feedback_weights():
   - Reads SessionQA entries with feedback_score
   - Maps score 1..5 to 0..1 normalized rating
   - Updates graph node/edge weights via streaming EMA: w' = w + α(r - w)
   - Marks entries as processed (MEMIFY_METADATA_FEEDBACK_WEIGHTS_APPLIED_KEY)

2. extract_feedback_qas():
   - Reads unprocessed session QA entries
   - Persists as graph nodes with question/context/answer

3. cognify (triplet enrichment):
   - Entity extraction from QAs + trace feedback
   - Relationship detection
   - Graph construction with embeddings

4. sync_graph_to_session():
   - Incremental: reads new edges via created_at checkpoint
   - Merges into existing session graph knowledge
   - Caps at max_lines (default 500)
   - Stores as dedicated key (separate from QA history)
```

---

## 3. Extractable Patterns for VantaDB MCP

### 3.1 Dual Memory Tiers

**Cognee:** Session cache (Redis/FS, volatile, fast) + Permanent graph (Neo4j/Qdrant, persistent, slow)

**VantaDB gap:** Only has permanent vector storage. Lacks the session-cache tier.

**Extraction:** Add `session_cache` mode to VantaDB MCP:
- `vantadb.put(key, vector, data, session_id="...")` → writes to in-memory/Redis cache
- `vantadb.get(key, session_id="...")` → reads from cache first, falls back to permanent
- Periodic sync from cache → permanent store

**Implementation surface:**
- `vantadb-mcp/src/session_cache.rs` — Memory-based or Redis-backed cache
- `vantadb-mcp/src/session_manager.rs` — CRUD operations for session entries
- Config: `VANTADB_SESSION_CACHE=memory|redis`, `VANTADB_REDIS_URL`

### 3.2 Agent Memory Decorator / Context Injection

**Cognee:** `@agent_memory` decorator wraps any async function, retrieves relevant context
from both session memory (recent trace feedback) and permanent memory (semantic search),
injects it, then persists the trace.

**VantaDB gap:** No automated context injection. MCP tools are stateless.

**Extraction:** Create an agent middleware for `vantadb-mcp` and other MCP servers:
1. Before tool execution: retrieve relevant past entries via vector similarity
2. Inject as `additionalContext` in the MCP tool response
3. After tool execution: persist call as a new trace entry

**Implementation surface:**
- `vantadb-mcp/src/agent_memory.rs` — Middleware wrapping tool handlers
- Context injection mechanism in MCP tool responses
- Trace storage (can reuse same vector store)

### 3.3 Session Lifecycle Tracking

**Cognee:** `SessionRecord` in Postgres tracks per `(user_id, session_id)`:
- status (running/completed/failed/abandoned)
- tokens_in/out, cost_usd, error_count
- last_model, last_activity_at
- `SessionModelUsage` for per-model cost breakdown
- `track_session_usage` ContextVar scope for automatic accumulation

**VantaDB gap:** No session tracking at all.

**Extraction:** Track sessions in VantaDB's existing metadata storage:
- Session start/end/heartbeat
- LLM call accumulation (token count, model, cost)
- Abandoned session detection (30 min inactivity threshold)

**Implementation surface:**
- Add to existing `vantadb-mcp` or as separate `vantadb-session` crate
- Use VantaDB's own vector store + metadata for storage (dogfooding)
- Expose as MCP resources: `sessions://list`, `sessions://{session_id}`

### 3.4 Agent Trace Persistence

**Cognee:** Each `add_agent_trace_step()` stores:
- `origin_function` — method name
- `status` — success/error
- `method_params` — sanitized parameters
- `method_return_value` — sanitized return value
- `error_message` — error text if failed
- `session_feedback` — LLM-generated or fallback summary
- `memory_query` and `memory_context` — what was retrieved

**VantaDB gap:** No trace persistence.

**Extraction:** Persist agent tool calls as VantaDB entries with metadata:
```rust
struct AgentTrace {
    id: String,
    session_id: String,
    origin_function: String,
    status: String,
    params: HashMap<String, Value>,
    return_value: Value,
    error_message: Option<String>,
    memory_query: Option<String>,
    memory_context: Option<String>,
    timestamp: DateTime<Utc>,
}
```

### 3.5 Session→Permanent Sync (improve Pipeline)

**Cognee:** 4-stage `improve()` pipeline:
1. Feedback weights → graph node/edge weights
2. QA text → graph nodes
3. Triplet enrichment (cognify)
4. Graph changes → session cache (sync_graph_to_session)

**VantaDB gap:** No sync mechanism. Entries go to permanent store directly or not at all.

**Extraction:** Implement a 3-stage sync:
1. Score session entries by importance (frequency, recency, explicit feedback)
2. Promote high-scoring entries to permanent store
3. Evict low-scoring entries from session cache

**Implementation surface:**
- `vantadb-mcp/src/sync.rs` — Sync orchestration
- Importance scoring heuristics
- Background watcher (idle poll + periodic sync)
- Config: `VANTADB_SYNC_IDLE_POLL`, `VANTADB_SYNC_THRESHOLD`, `VANTADB_SYNC_COOLDOWN`

### 3.6 Agent-Scoped Default Dataset

**Cognee:** `_agent_scoped_default_dataset()` detects the MCP client (Cursor, VS Code)
and assigns a dedicated dataset name automatically.

**VantaDB gap:** All tools use a single `main_dataset`.

**Extraction:** Detect MCP client identity from environment/transport:
- `CURSOR=true` → `cursor_memory`
- `VSCODE_INJECTION=1` → `vscode_memory`
- `TERM_PROGRAM=vscode` → `vscode_memory`
- Default: `main_dataset`

### 3.7 Background Task Management

**Cognee:** Strong references via `_background_tasks: set[asyncio.Task]` to prevent GC;
per-dataset error ring buffers (`deque(maxlen=50)`).

**VantaDB gap:** Long-running operations (cognify, large inserts) block MCP tools.

**Extraction:** 
- Spawn long operations as background tasks with strong refs
- Return task ID immediately
- Status check tool: `task_status(task_id)`
- Error ring buffer per operation type

### 3.8 LLM Usage Tracking

**Cognee:** `track_session_usage()` ContextVar scope + `record_llm_call()`:
- Token estimation (chars/4 heuristic)
- Per-model pricing table (GPT-4o, Claude 3.5, Gemini, etc.)
- Accumulates into `SessionRecord` + `SessionModelUsage`
- Cost attribution per model per session

**VantaDB gap:** VantaDB doesn't call LLMs directly, but the MCP server could track
usage for downstream consumers.

**Extraction:** 
- Optional LLM call tracking in MCP middleware
- Usage report endpoint
- Configurable pricing table

### 3.9 Abandoned Session Detection

**Cognee:** `SESSION_ABANDON_AFTER_SECONDS` (default 1800 = 30 min):
- `get_effective_status_sql()` computes at read time without sweeper
- `running` + `last_activity_at` < threshold → `abandoned`

**VantaDB gap:** No session concept, so no abandonment.

**Extraction:** 
- Track `last_activity_at` per session
- Compute effective status at query time
- Configurable timeout

### 3.10 Agent Lesson Extraction (Self-Improvement)

**Cognee:** Two-phase agent lesson extraction:
- **Live** (no LLM): Errored trace steps → deterministic failure lessons
- **Batch** (LLM): Every N traces, propose tool_rules, success_patterns, workflow_state,
  environment_facts, failure_lessons from trace windows
- Both feed the same dedup + confidence-gated applier (`apply_candidate_updates`)

**VantaDB gap:** No self-improvement mechanism.

**Extraction:** 
- Track error patterns per agent session
- Derive "VantaDB usage optimization" suggestions from access patterns
- Surface via MCP resources

---

## 4. Claude Code Plugin Patterns

### 4.1 Plugin Architecture

```
.claude-plugin/
├── plugin.json             # Manifest
├── hooks/
│   ├── session-start.py
│   ├── user-prompt-submit.py
│   ├── post-tool-use.py
│   ├── stop.py
│   ├── pre-compact.py
│   └── session-end.py
├── skills/
│   └── cognee-memory/
│       ├── cognee-remember.md
│       ├── cognee-search.md
│       └── cognee-sync.md
├── agents/
│   └── config.yaml
├── tests/
└── scripts/
```

### 4.2 Hook Lifecycle

| Hook | When | What it does |
|------|------|-------------|
| `SessionStart` | Fresh launch | Mode select, identity, dataset readiness, watcher bootstrap |
| `UserPromptSubmit` | Each user message | Async context retrieval + prompt injection |
| `PostToolUse` | After each tool call | Async trace write to session cache |
| `Stop` | Assistant finishes | Async answer write + optional transcript clear |
| `PreCompact` | Before context compaction | Build memory anchor to preserve across resets |
| `SessionEnd` | Process exits | Detached final sync worker (improve) + unregister |

### 4.3 Session Strategies

| Strategy | Description |
|----------|-------------|
| `per-directory` | Session ID based on CWD hash (default) |
| `git-branch` | Session ID based on git branch name |
| `static` | Fixed session ID from `COGNEE_SESSION_ID` env |

### 4.4 Idle Watcher Pattern

```python
# Pseudocode from Claude Code plugin
async def idle_watcher():
    while True:
        await asyncio.sleep(COGNEE_IDLE_POLL)  # default 10s
        if seconds_since_last_activity > COGNEE_IDLE_THRESHOLD:  # default 60s
            if seconds_since_last_improve > COGNEE_IMPROVE_COOLDOWN:  # default 600s
                await improve(session_ids=[current_session])
```

### 4.5 Memory Preference Steering

The plugin injects `additionalContext` in `SessionStart` to assert Cognee as
the preferred memory over `MEMORY.md`:

```python
# From session-start.py
_additional_context = {
    "role": "system",
    "content": "Cognee is the authoritative memory system. "
               "Consult Cognee context FIRST. "
               "Prefer Cognee tools over MEMORY.md."
}
```

---

## 5. Implementation Roadmap for VantaDB

### Phase 1: Session Layer (estimated: 2-3 days)

1. **Session cache** — In-memory session store with optional Redis backend
   - `vantadb-mcp/src/session_cache.rs`
   - `SessionCache` trait with `MemorySessionCache`, `RedisSessionCache` impls

2. **Session lifecycle** — SessionRecord equivalent in VantaDB metadata
   - `vantadb-mcp/src/session_lifecycle.rs`
   - Status tracking, heartbeat, abandonment detection

3. **Agent trace persistence** — Store tool calls as VantaDB entries
   - `vantadb-mcp/src/agent_trace.rs`
   - Context retrieval before execution + persistence after

### Phase 2: Plugin Integration (estimated: 3-4 days)

4. **Claude Code plugin** — Installable plugin for VantaDB memory
   - `.claude-plugin/` with hooks, skills, config
   - SessionStart, UserPromptSubmit, PostToolUse, Stop, SessionEnd

5. **Cursor/VS Code detection** — `_agent_scoped_default_dataset()` equivalent
   - Auto-detect client identity
   - Per-client dataset isolation

### Phase 3: Sync & Improve (estimated: 2-3 days)

6. **Session→permanent sync** — Background sync mechanism
   - `vantadb-mcp/src/sync.rs`
   - Importance scoring, promotion, eviction
   - Idle watcher + periodic sync

7. **LLM usage tracking** — Token/cost tracking per session
   - Optional middleware for LLM call count
   - Per-model pricing table

### Phase 4: Self-Improvement (estimated: 3-5 days)

8. **Agent lesson extraction** — Error patterns → usage optimization
   - Live extraction (deterministic, no LLM)
   - Batch extraction (LLM-based pattern discovery)

9. **Context injection middleware** — MCP middleware for automated context
   - Before: retrieve relevant past entries
   - After: persist current call
   - Background: periodic sync + lesson extraction

---

## 6. VantaDB vs Cognee: Feature Comparison

| Feature | Cognee | VantaDB | Notes |
|---------|--------|---------|-------|
| Vector store | Qdrant/LanceDB/ChromaDB | **HNSW/IVF (native)** | VantaDB wins |
| Graph DB | Neo4j/KuzuDB | ❌ None | Cognee wins for relational queries |
| Session cache | Redis/FS | ❌ None | **Extractable** |
| Agent trace persistence | Full | ❌ None | **Extractable** |
| Context injection | `@agent_memory` decorator | ❌ None | **Extractable** |
| LLM usage tracking | Yes (ContextVar + DB) | ❌ None | **Extractable** |
| Session lifecycle | Full (status, cost, model) | ❌ None | **Extractable** |
| Claude Code plugin | Official | ❌ None | **Buildable** |
| MCP tools | V1 + V2 (10 tools) | V1 only (7 tools) | Comparable |
| Client detection | `_agent_scoped_default_dataset()` | ❌ None | **Extractable** |
| Self-improvement | Agent lesson extraction | ❌ None | Advanced |
| Vector search | Via Qdrant/LanceDB | **Native HNSW/IVF** | VantaDB wins (faster, no deps) |
| Embeddings | Via LLM | **CLIP/OpenAI/Ollama** | VantaDB wins (multimodal) |
| Memory tiering | Dual (cache + graph) | Single (permanent) | **Extractable** |
| Background tasks | Strong refs + error ring | ❌ None | **Extractable** |

---

## 7. Key Source References for Future Implementation

### Must-Read Before Implementation

| File (Cognee) | Why |
|---------------|-----|
| `cognee-mcp/src/server.py` | Full MCP server with remember/recall/forget/improve tools + background task pattern |
| `cognee-mcp/src/cognee_client.py` | Dual-mode client — reference for VantaDB MCP client abstraction |
| `cognee/modules/agent_memory/decorator.py` | `@agent_memory` — reference for context injection middleware |
| `cognee/modules/agent_memory/runtime.py` | Memory retrieval + trace persistence — core logic |
| `cognee/modules/session_lifecycle/metrics.py` | SessionRecord CRUD — reference for session lifecycle |
| `cognee/modules/session_lifecycle/usage_tracking.py` | ContextVar-based LLM tracking pattern |
| `cognee/infrastructure/session/session_manager.py` | Full SessionManager — QA CRUD, agent traces, graph context |
| `cognee/tasks/memify/apply_feedback_weights.py` | Feedback→graph weights — streaming EMA update pattern |
| `cognee/tasks/memify/sync_graph_to_session.py` | Incremental graph→session sync with checkpointing |
| `cognee/infrastructure/session/agent_context_extraction.py` | Live + batch agent lesson extraction pattern |
| `integrations/claude-code/` | Full Claude Code plugin — hooks, skills, config, watchers |

### Key Patterns to Replicate

```python
# 1. Background task with strong reference (server.py)
_background_tasks: set[asyncio.Task] = set()
def _track_background(coro) -> asyncio.Task:
    task = asyncio.create_task(coro)
    _background_tasks.add(task)
    task.add_done_callback(_background_tasks.discard)
    return task

# 2. Session-usage tracking via ContextVar (usage_tracking.py)
_active_session: ContextVar[Optional[tuple[str, UUIDType]]] = ContextVar(...)
@asynccontextmanager
async def track_session_usage(session_id, user_id):
    token = _active_session.set((session_id, user_id))
    try:
        yield
    finally:
        _active_session.reset(token)

# 3. Config validation with dataclasses (runtime.py)
@dataclass(slots=True)
class AgentMemoryConfig:
    with_memory: bool
    with_session_memory: bool
    save_session_traces: bool
    memory_top_k: int
    # ... 15+ validated fields

# 4. Abandoned session at read time (metrics.py)
def get_effective_status_sql():
    threshold_ts = datetime.now(timezone.utc) - timedelta(seconds=1800)
    return case(
        (and_(status == "running", last_activity_at < threshold_ts), "abandoned"),
        else_=status,
    )

# 5. Sanitization for trace storage (sanitization.py)
MAX_SERIALIZED_VALUE_LENGTH = 1000
def sanitize_value(value):
    if isinstance(value, UUID): return str(value)
    if isinstance(value, str): return truncate_text(value, MAX_SERIALIZED_VALUE_LENGTH)
    if isinstance(value, (list, tuple)): return [sanitize_value(v) for v in value[:20]]
    # ... handles dicts, objects with .id, etc.

# 6. Autodetect MCP client (server.py)
def _agent_scoped_default_dataset():
    if os.getenv("CURSOR"): return "cursor_vscode_memory"
    if os.getenv("VSCODE_INJECTION"): return "cursor_vscode_memory"
    return "main_dataset"
```

---

## 8. Architecture Decision Records

### Decision 1: Rust-native session layer vs Python dependency

- **Context:** VantaDB is Rust-native; Cognee is Python-native
- **Options:** 
  a) Implement session layer in Rust (native, no Python dep)
  b) Embed Cognee as Python subprocess (fast to prototype)
  c) Standalone session service in Python/any language
- **Recommendation:** (a) — Rust-native for performance, zero dependencies

### Decision 2: Use VantaDB for session storage vs separate Redis

- **Context:** Cognee uses Redis for session cache; VantaDB is itself a store
- **Options:**
  a) Store session data in VantaDB itself (dogfooding, no extra dep)
  b) Add optional Redis backend for session cache
- **Recommendation:** (b) with (a) default — VantaDB for persistence, Redis optional for speed

### Decision 3: Plugin vs built-in for Claude Code integration

- **Context:** Cognee uses a marketplace plugin; VantaDB could embed or plugin
- **Options:**
  a) Claude Code plugin (marketplace, installable)
  b) Built-in via `vantadb-mcp` MCP tools (always available)
- **Recommendation:** Both — plugin for full lifecycle hooks, MCP tools for basic operations

---

## 9. Open Questions for Future Sessions

1. Should VantaDB's session memory use the **same embedding space** as permanent memory, or a separate lightweight embedding?
2. Is the `improve()` pipeline's feedback weight system (streaming EMA on graph nodes) relevant for VantaDB's HNSW/IVF indexes?
3. Should agent lesson extraction be LLM-based (Cognee style) or rule-based (pattern matching on error types)?
4. What MCP transport does VantaDB need for IDE plugins — SSE, streamable HTTP, or stdio?
5. Should session→permanent sync be automatic or require explicit `improve()` call?

---

## Appendix A: Analyzed Files Index

Total files analyzed: 25+ across 3 repositories (cognee main, cognee-mcp, cognee-integrations).

### cognee (core)
- `cognee-mcp/src/server.py` — MCP server with V1+V2 tools
- `cognee-mcp/src/cognee_client.py` — Dual-mode client
- `cognee-mcp/src/server_utils.py` — Result formatting
- `cognee/modules/agent_memory/__init__.py` — Agent memory exports
- `cognee/modules/agent_memory/runtime.py` — Agent memory runtime (context, retrieval, persist)
- `cognee/modules/agent_memory/decorator.py` — @agent_memory decorator
- `cognee/modules/agent_memory/sanitization.py` — Value sanitization
- `cognee/modules/session_lifecycle/__init__.py` — Session lifecycle exports
- `cognee/modules/session_lifecycle/models.py` — SessionRecord, SessionModelUsage
- `cognee/modules/session_lifecycle/metrics.py` — Session ops (CRUD, status, list, pagination)
- `cognee/modules/session_lifecycle/usage_tracking.py` — LLM token/cost tracking
- `cognee/modules/agents/registry.py` — Agent connection registry
- `cognee/infrastructure/session/session_manager.py` — Full SessionManager
- `cognee/infrastructure/session/session_agent_trace.py` — Trace feedback generation
- `cognee/infrastructure/session/feedback_models.py` — Pydantic models
- `cognee/infrastructure/session/agent_context_extraction.py` — Agent lesson extraction
- `cognee/tasks/memify/apply_feedback_weights.py` — Feedback→graph weights
- `cognee/tasks/memify/cognify_session.py` — Session→graph pipeline
- `cognee/tasks/memify/sync_graph_to_session.py` — Graph→session cache sync
- `cognee/memify_pipelines/persist_agent_trace_feedbacks_in_knowledge_graph.py` — Trace→graph

### cognee-integrations (plugins)
- `integrations/claude-code/README.md` — Plugin documentation
- `integrations/claude-code/.claude-plugin/` — Plugin manifest
- `integrations/claude-code/hooks/` — 6 lifecycle hooks
- `integrations/claude-code/skills/` — 3 skills
- `integrations/claude-agent-sdk/` — Python SDK integration

---

## Appendix B: Abandoned Session SQL Pattern

```sql
-- From cognee/modules/session_lifecycle/metrics.py
-- Computes effective status without a sweeper process.
-- A running session idle >30 min is reported as "abandoned".

CASE
    WHEN status = 'running'
         AND last_activity_at < NOW() - INTERVAL '30 minutes'
    THEN 'abandoned'
    ELSE status
END AS effective_status
```

---

## Appendix C: Key Environment Variables Reference (Cognee)

For future VantaDB implementation compatibility:

| Env var | Cognee default | Purpose |
|---------|----------------|---------|
| `COGNEE_API_KEY` | auto-minted | API auth |
| `COGNEE_BASE_URL` | unset | Remote server URL |
| `COGNEE_SESSION_ID` | auto-gen | Resume session |
| `COGNEE_SESSION_STRATEGY` | `per-directory` | Session ID strategy |
| `COGNEE_PLUGIN_DATASET` | `agent_sessions` | Dataset name |
| `COGNEE_PREFER_MEMORY` | `true` | Memory preference steer |
| `COGNEE_IDLE_POLL` | `10` | Watcher poll interval |
| `COGNEE_IDLE_THRESHOLD` | `60` | Idle seconds before sync |
| `COGNEE_IMPROVE_COOLDOWN` | `600` | Min seconds between syncs |
| `COGNEE_AUTO_IMPROVE_EVERY` | `150` | Tool calls between syncs |
| `COGNEE_REMEMBER_BACKGROUND` | `true` | Async graph build |
| `COGNEE_CLAUDE_CLEAR_AFTER_MESSAGE` | disabled | Demo auto-clear |
| `LLM_API_KEY` | unset | LLM provider key |
