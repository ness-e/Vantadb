# Token budget (FIND-106, implements recall-policy §5 + §3)

Source: `../references/recall-policy.md`. Numbers here are policy arithmetic,
not benchmarks (no Regla 11 claim).

## Rule

- Injected recall per message ≤ **~10% of the client's context window**.
- `top_k` default **5** (`RecallConfig::default`; MCP clamps to `config.max_top_k`).
- `PreCompact` summaries are bounded by the **session compaction budget**, not this file.
- Prefer **fewer, newer hits**; recency is a tiebreak, never a filter.

## Per-client arithmetic (formula, not hard pin)

| Client | Window reference | ~10% recall budget | top_k | Notes |
|---|---|---|---|---|
| Claude Code | model window (e.g. 200k default class) | ~20k tokens | 5 | `additionalContext` via SessionStart/UserPromptSubmit |
| Cursor | model window (e.g. 200k default class, up to 1M max) | ~20k tokens | 5 | `additionalContext`/`followup` fields per hook schema |
| Codex | model window (e.g. GPT-5.x 272k class) | ~27k tokens | 5 | `additionalContextLimit` caps per-hook output (spill to disk over ~2.5k) |
| OpenCode | model window (depends on provider) | 10% of active model | 5 | inject via `additionalContext`; compaction via `output.context` |

If the client changes its window, the 10% recomputes — no template edit needed.

## When NOTHING is injected (structural, no score cutoff — RRF ranks aren't calibrated)

1. `recalled` empty → inject NOTHING. Say "no recuerdo nada sobre X", continue live. Never fill the gap.
2. `recalled` non-empty → inject `prepend_context` verbatim up to the budget above. Never re-summarize.
3. `effective_mode: "keyword"` + non-empty → inject, but ranking is weak (lexical only).
4. Temporal unresolvable/future-only → last **30 days** (`FALLBACK_LAST_DAYS`) + agent says so. Never invent a range.

## Estimator (reviewers)

`tokens ≈ chars/4` (en) — truncate oldest hits first when over budget, keep newest 5 max.
Tests assert: budget file exists, 10% rule present, top_k=5 present, all 4 no-inject rules present.
