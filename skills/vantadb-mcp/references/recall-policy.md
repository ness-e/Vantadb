# Continuous Recall Policy (FIND-103)

> The skill (`SKILL.md`) is what the agent reads; this file is the engine
> policy it executes. Per-client hook templates that implement this policy
> live in FIND-106 — this file defines WHAT, they define HOW per client.

## 1. Hook map

| Hook | Action (VantaDB tools) | Notes |
|------|------------------------|-------|
| `SessionStart` | `memory_recall` (`scope: agent`, `top_k: 5`) → inject `prepend_context` as `additionalContext` | Pattern proven in-repo: `docs/dev/research/archive/COGNEE_EVALUATION.md:390` injects `additionalContext` on `SessionStart`. OpenCode has no `SessionStart` — it fires `session.created` via a JS plugin (`docs/dev/tasks/complete/ECO-001.md:10-17`); FIND-106 maps each client. |
| Per-message | `memory_recall` with the user message verbatim as `query` | Recall is keyed on the message, never on a paraphrase (paraphrases drift). |
| `PreCompact` | Save state: `thread_send` (open turns) and/or `scene_write` | What does not get saved before compaction is gone. |
| `Stop` / session end | Auto-capture the turn: `thread_send` (`role`, `content`) | Proxy traffic is captured automatically into `proxy-turns` (`vanta-proxy/src/capture.rs:15`); direct MCP turns need the explicit send. |

## 2. Auto-recall parameters (defaults)

- `query`: the raw user text (same shape as the auto-recall hook's `user_text`).
- `scope`: `agent` (cross-session reach without leaking other agents; `team` only when the task is explicitly shared; `session` only to restrict).
- `top_k`: `5` (`RecallConfig::default`; the MCP tool defaults to 5 and clamps to `config.max_top_k`).
- `mode`: hybrid (degrades to keyword without an embedding provider — announced via `effective_mode`, never silent).

## 3. Thresholds — when NOTHING is injected

Recall scores are RRF-fused ranks, not calibrated probabilities: there is NO
absolute score cutoff in v1. The injection rule is structural:

1. `recalled` empty → inject NOTHING. Do not paraphrase, do not fill the gap: say "no recuerdo nada sobre X" and continue with the live context.
2. `recalled` non-empty → inject `prepend_context` verbatim (up to the token budget below), never a re-summary (summaries drift; see Notion `Problema` § Deriva por resúmenes).
3. `effective_mode: "keyword"` with non-empty recall → inject, but treat ranking as weak (lexical only, no semantic signal).
4. Never invent a temporal range to narrow recall: unresolvable expressions fall back to the last `FALLBACK_LAST_DAYS` (30) days AND the agent must say so (`temporal::FALLBACK_LAST_DAYS` names the number; this policy owns the behaviour).

## 4. Temporal expressions (deterministic, never improvised)

"ayer a las 2pm" is a `[from_ms, to_ms]` interval, not a vibe:

1. Translate with `parse_temporal_expression(expr, now_ms)` (`vantadb-mcp/src/temporal.rs`, re-exported at the crate root; suite `tests/temporal_tests.rs`).
2. Execute v1 via `memory_list` pagination filtered client-side on `created_at_ms` — verified: `matches_advanced_filters` is metadata-only (`src/sdk/serialization/mod.rs:551-574`), `search_memory` filters are equality-only (`handlers/tools.rs:3209-3226`), and the only server-side time window is `graph_traverse`'s `time_range` over EDGE creation times (`:2773-2839`).
3. `None` (unresolvable or future-only) → §3 rule 4 (30 days + explicit warning).

## 5. Token budget

- Injected recall must stay under ~10% of the client's context window per message; `PreCompact` summaries are bounded by the session's compaction budget, not by this file (exact numbers per client in FIND-106).
- Prefer fewer, newer hits over many old ones: recency is a tiebreak, never a filter (old decisions stay reachable via §4).

## 6. Curation: proxy turns → threads via approval inbox

`proxy-turns` (`TURNS_NAMESPACE`, `capture.rs:15`) is raw material, not memory:
auto-promoting it would launder noise into knowledge (Notion `Propuesta` §2.4:
approve/reject on capture answers autoenvenenamiento).

1. Review: page `memory_list` on `proxy-turns` (keys `{ms}-{seq}`) and draft `thread_send` proposals.
2. Decide in the approval inbox: on approve → `thread_send`; on reject → discard. NEVER auto-promote, NEVER silently rewrite the turn text.
3. Tool status: the inbox tools (`capture_list_pending`, `capture_approve`, `capture_reject`) are FIND-107 S4 DEFER-ratificado — policy ready here, tools pending there. Until they exist, the inbox is the agent's explicit turn-by-turn confirmation, not a background job.

## 7. What this policy does NOT do (explicit non-goals)

- Per-client hook code/templates (FIND-106).
- Score calibration or learned ranking (declared non-objective in Notion `Propuesta` §2.2 Selección).
- Time-travel/bitemporal queries (Notion `Propuesta` dimension 5: [PARCIAL], queries [PROPUESTA]).
- Entity auto-resolution, conflict auto-resolution, skill auto-extraction ([PROPUESTA] v0.7/v1.0).
