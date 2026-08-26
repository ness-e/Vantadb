# RES-03 — Session Layer for VantaDB MCP: Go/No-Go Analysis (DEC-01)

> Date: 2026-08-25 · Source task: Backlog DEC-01 / plan 2026-08-25-batch-core-fixes-research.md Task 9
> Origin of DEC-01: `docs/research/COGNEE_EVALUATION.md` §8 (ADR sketches) + §9 (5 open questions)
> Status: EVIDENCE DOC — final decision belongs to the owner (Regla 5, ADR is human-authored)

## Question

Cognee-inspired 4-phase session-layer roadmap — (1) session cache, (2) Claude Code
plugin, (3) sync/improve, (4) lesson extraction. Should we build it? What does each
phase add over what already exists?

## What already exists (verified in code)

| Capability | Where | MCP surface |
|---|---|---|
| Persistent threads w/ TTL + GC | `src/agentic/thread.rs:72-320` (`ThreadStore`) | `thread_create/send/get/list/delete/purge_expired` (`vantadb-mcp/src/threads.rs:87-102`, MCP-32); also REST `/api/v2/threads` (`src/cli_server.rs:2613-2702`) |
| Context assembly w/ recall + MMD + compaction (TDAM) | `vanta-memory/src/context_engine/engine.rs:199` (`assemble_with_recall`) | `context_assemble` (`handlers/tools.rs:1893`, MCP-31) |
| Session-scoped working memory (scenes, sandboxed CRUD) | `vanta-memory/src/core/scene/scene_tools.rs:104-200`, keyed by `session_key` | `scene_read/list/query` (MCP-30) |
| Axioms read/write/delete | `handlers/tools.rs:1134-1190` | MCP-33 |
| Per-session provenance log | `vanta-memory/src/core/memory_generation_log/store.rs:24-36` (`genlog/<session>` namespace) | queryable via memory tools; desktop Tauri `vanta_genlog_query` |
| Skills store | `vantadb-mcp/src/skills.rs` | `skill_create/skill_files_write/...` |

**Conclusion:** VantaDB already *has* a session layer. The roadmap largely re-proposes
primitives that exist.

## Phase-by-phase verdict

### Phase 1 — Session cache → ❌ NO-GO
Threads = persistent sessions with TTL/GC; scenes = hot working memory per
`session_key`; genlog = per-session audit trail. A Cognee-style "session cache"
(with optional Redis per COGNEE Decision 2) duplicates all three and adds a
dependency an embedded local-first DB doesn't need (reads are sub-ms locally).
Only worthwhile residue: a docs page documenting the recommended
session↔thread_id↔namespace convention (~1h).

### Phase 2 — Claude Code plugin → 🟡 DEFER (docs-only now)
`vantadb-mcp` over stdio already serves 60+ tools to any MCP client including
Claude Code/OpenCode — no plugin required for basic operation. Transport open
question (#4) is answered by the current spec: stdio is standard ("Clients SHOULD
support stdio whenever possible") and Streamable HTTP replaced HTTP+SSE
(deprecated since 2024-11-05). No SSE work needed today.
A marketplace plugin adds lifecycle hooks (SessionStart→recall, etc.) whose value
only materializes with external users, against a fast-moving plugin API.
**Cheapest path now:** a docs guide "connect vantadb-mcp to Claude Code" (config
snippet). Revisit plugin when there's user demand evidence.

### Phase 3 — Sync/improve → ❌ NO-GO (route as future research)
Session→permanent promotion already works manually: agent reads thread/scene →
`memory_put` into a permanent namespace (explicit, auditable via genlog).
Automatic sync requires resolving open questions #1 (embedding space) and #2
(EMA feedback weights on HNSW) — both unmeasured design decisions that would
trigger Regla 9 bench obligations. High cost, no identified consumer today.

### Phase 4 — Lesson extraction → ❌ NO-GO
Genlog already captures what happened per session; lessons/decisions memory at
the orchestration layer (`campaign_memory_write`, TSYS-15) already captures
distilled learnings. Rule-based extraction over genlog would be small but has no
consumer beyond what exists. If ever revived: rule-based first (open question #3),
one query tool over genlog namespaces.

## Open questions (COGNEE §9) — proposed resolutions

1. Same embedding space? → N/A until Phase 3 revives; if so, same space +
   namespaces (avoids second index cost). Defer.
2. EMA feedback on HNSW/IVF? → Unvalidated; defer behind benchmarks.
3. LLM vs rule-based extraction? → Rule-based first if ever built.
4. Transport? → stdio now (spec-verified); Streamable HTTP only for remote/server mode later.
5. Automatic vs explicit sync? → Explicit (manual promote via `memory_put`) until Phase 3 revisited.

## Recommendation

**Overall: DEFER the roadmap; resolve DEC-01 as "no-go-as-scoped".** Build nothing
now except two docs-only items (~half day total):
1. Docs: session conventions page (threads/scenes/genlog mapping).
2. Docs: "connect vantadb-mcp to Claude Code" config guide.
Re-open Phases 2–3 only with external-user demand evidence. Owner writes the ADR
(Regla 5) citing this doc as evidence.

## Sources

- Verified URLs: modelcontextprotocol.io spec 2025-06-18 transports (resolved 2026-08-25)
- Code: paths in table above (all read this session)
- `docs/research/COGNEE_EVALUATION.md` §8-9, App. A (in-repo source)
