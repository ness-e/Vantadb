---
title: "Memory scopes: multi-level isolation model for hosts (MEMG-25 spec)"
kind: reference
status: proposed
description: "Spec-first scope model for memory hosts: scope levels, visibility, trust, authority, provenance, validity, retention, owner and writer; fields, defaults and wire; no enforcement"
tags: [vantadb, api, memory, scopes]
---

# Memory scopes (MEMG-25 spec)

> **Status:** `proposed` — spec-first by Backlog design (fields, defaults,
> wire) plus SDK *types*. There is **no enforcement** in this spec;
> enforcement belongs to MGR-04 / `FIND-301` and is out of scope here.
> **Source of truth for what exists today:** the symbols cited below; everything
> under "Proposed" is design, not code.

Namespaces today are flat. Sub-Ego isolation currently lives in the host
adapter, not in the engine. This spec gives hosts (Ego models its hierarchy
`User → Org → Project → Sub-Ego → Session → Task` on top) generic scope
primitives with no Ego logic inside.

## What exists today (implemented, enforced where noted)

| Piece | Symbol | Semantics |
|-------|--------|-----------|
| Recall reach | `RecallScope::{Session, Agent, Team}` (`vanta-memory/src/core/hooks/auto_recall.rs:287`, default `Agent`) | Cross-session reach of the L1 recall pool (D22) |
| Tenancy isolation | `ProfileIsolation { team_id, agent_id }` (`vanta-memory/src/core/profile/profile_sync.rs:25`, defaults `"default"`) | L2/L3 profile scope; scope string `team:{t}\|agent:{a}` |
| Injection ACL | `InjectionPolicy` (`auto_recall.rs:135`: `allow_all` / `from_prefixes` / `trust_class`) + `RecallGovernance` (`auto_recall.rs:71`: `source` / `deny`, denials never silent) | VER-04: which namespaces may be injected, with an always-populated governance summary |
| Trust classes | `InjectionPolicy::trust_class` (`auto_recall.rs:183`) | Per-namespace trust classification feeding the ACL |
| Namespace boundary | Server RBAC (`docs/api/SHARING.md`: org/project/team/asset mapping) | The top isolation boundary; SDK treats namespaces as opaque |
| Deterministic address | `memory_node_id(ns, key)` (`src/sdk/serialization/mod.rs:80`, public since DIST-19 S6) | Stable `u128` meeting point of memory and graph |

## Proposed: `MemoryScope` (spec, unenforced)

A scope is a pure descriptor attached to records and queries. The engine
stores and echoes it; it does **not** gate on it (until MGR-04/FIND-301).

```json
{
  "tenant": "acme",
  "org": "atlas",
  "project": "ego-desktop",
  "principal": "user:ada",
  "agent": "sub-ego-3",
  "session": "sess-9f2",
  "task": "task-51",
  "visibility": "team",
  "trust": "verified",
  "authority": "owner",
  "provenance": { "source": "l1/sess-9f2", "derived_from": [] },
  "validity": { "valid_at_ms": 1760572800000, "invalid_at_ms": null },
  "retention": { "ttl_ms": null, "policy": "keep" },
  "owner": "user:ada",
  "writer": "sub-ego-3"
}
```

### Fields and defaults

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `tenant` | string | `""` (unset) | Top isolation root; maps to namespace root by convention (`{tenant}/...`) |
| `org` | string | `""` | One level below tenant; empty = tenant scope |
| `project` | string | `""` | Empty = org scope |
| `principal` | string | `""` | `user:{id}` / `service:{id}`; empty = unattributed (legacy records) |
| `agent` | string | `"default"` | Matches `ProfileIsolation.agent_id` default |
| `session` | string | `""` | Matches L1 `session_key` semantics (`l1/<session>`) |
| `task` | string | `""` | Host task id; empty = outside any task |
| `visibility` | enum | `"private"` | `private` \| `team` \| `org` \| `public`. Advisory only |
| `trust` | enum | `"unverified"` | `unverified` \| `verified` \| `authoritative`. Advisory; feeds future FIND-320 ranking, never gating |
| `authority` | enum | `"writer"` | `owner` \| `writer` \| `reader`. Declares the writer's standing, not a permission |
| `provenance` | object | `{}` | `source` (namespace), `derived_from` (parent keys) — mirrors the MEMG-03 lineage edges, serialized |
| `validity` | object | `{ valid_at_ms: 0, invalid_at_ms: null }` | Mirrors the v2 record validity window |
| `retention` | object | `{ ttl_ms: null, policy: "keep" }` | `keep` \| `expire` \| `archive`. Advisory; no sweeper reads it yet |
| `owner` | string | `""` | Owning principal; empty = unowned |
| `writer` | string | `""` | Last writer; empty = unknown (legacy) |

### Wire rules

- The scope travels as a flat JSON object under the `scope` key of record
  metadata and recall/query options. Unknown keys are ignored (forward
  compatibility); missing keys take the defaults above (no sparse-map
  special cases).
- Enum values are lowercase `snake_case` wire strings; unknown enum tokens
  are rejected at parse time (never inferred) — same rule as
  `ReinforceOutcome::from_wire_str`.
- The scope does not alter `node_id`: identity stays `memory_node_id(ns, key)`.

### Explicitly out of scope

- Enforcement (gating reads/writes by scope) → MGR-04 / FIND-301.
- Migration of existing records (they read as all-defaults scope).
- Server RBAC changes (SHARING.md model untouched).
