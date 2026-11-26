---
title: "ADR-0059: Storage-path env var unification — `VANTADB_STORAGE_PATH` is the only name"
kind: adr
status: proposed
description: "The VANTA_DB / VANTADB_STORAGE_PATH coexistence decided in ADR-0012 is gone: the CLI --db flag and Config::from_env() both read VANTADB_STORAGE_PATH, and VANTA_DB is read nowhere in the tree (FIND-89 consolidation, verified 2026-10-20)"
tags: [vantadb, architecture, adr, config, env-vars, cli]
supersedes: docs/dev/architecture/adr/ADR-0012-env-var-naming.md
created: "2026-10-20"
owner: vanta-arch
---

# ADR-0059: Storage-path env var unification — `VANTADB_STORAGE_PATH` is the only name

## Status

Accepted. Supersedes [ADR-0012](./ADR-0012-env-var-naming.md) — the "coexisten
dos nombres" arrangement it decided no longer exists in the code.

**No rollback of the intent.** ADR-0012's goal was to make the storage path
unambiguous for the MCP child process (TECH-01) *without* a breaking rename.
This ADR keeps that goal and settles the debt ADR-0012 explicitly deferred:
the two names are now **one** name, and the rename that ADR-0012 refused
happened later, under the FIND-89 env-var consolidation (2026-09-14).

Verified against the 0.9.0 tree on **2026-10-20**:

| Fact | Evidence |
|---|---|
| The CLI `--db` flag reads `VANTADB_STORAGE_PATH` | `src/cli.rs:14,18` — `#[arg(short, long, env = "VANTADB_STORAGE_PATH", default_value = "./db", global = true)]` |
| `Config::from_env()` reads `VANTADB_STORAGE_PATH` | `src/config.rs:1079-1080` |
| The `vantadb-server` child sets `VANTADB_STORAGE_PATH` from `--db` | `src/cli_handlers/server.rs:331` (and `src/cli_handlers/mcp_call.rs:261`) |
| `VANTA_DB` is read **nowhere** | no `VANTA_DB` occurrence in `src/`, `scripts/`, or `vantadb-mcp/` |

## Context and problem statement

ADR-0012 (2026-08-05) found that the storage path had two independent
readers: the clap flag `--db` (env `VANTA_DB`) and the engine's
`VantaConfig::from_env()` (env `VANTADB_STORAGE_PATH`). The MCP child
inherited the parent's `--db` value through `VANTA_DB` but the engine read
`VANTADB_STORAGE_PATH`, so the two could disagree — TECH-01.

ADR-0012's decision was to keep both names and to have the child set both.
It named the debt: *"la coexistencia de dos nombres persiste"*.

Two things changed afterwards:

1. The FIND-89 consolidation (2026-09-14) unified the legacy `VANTA_*` names
   into `VANTADB_*` **without shims** — same doctrine already applied to the
   other seven vars in F3C. The `--db` flag's env was folded in with them.
2. The TECH-01 symptom fix became unnecessary in its old form: with one
   name there is nothing to disagree about.

## Considered options

### Option A — Keep both names (ADR-0012's decision)

- Pros: zero breaking change for anyone who had `VANTA_DB` in their
  environment or scripts.
- Cons: two names for one setting; the "which one wins?" question never goes
  away; every new reader has to learn the rule.

### Option B — Keep both names and add an alias `VANTADB_CLI_DB_PATH`

- Pros: no breakage.
- Cons: a *third* name, and the ambiguity it was meant to solve is the
  problem, not the solution.

### Option C — One name: `VANTADB_STORAGE_PATH` for both the flag and the config

- Pros: one setting, one env var, one precedence rule; the child-process
  handshake cannot disagree with itself.
- Cons: a breaking change for deployments that set `VANTA_DB` (mitigated by
  the same `feat!:` + `BREAKING CHANGE:` mechanism the rest of the FIND-89
  consolidation used, and by the fact that `--db` on the command line keeps
  working exactly as before).

## Decision outcome

**Option C — one name.** `VANTADB_STORAGE_PATH` is the single env var for the
storage path, read by both the global `--db` CLI flag and
`Config::from_env()`. Precedence is: **CLI flag `--db` > `VANTADB_STORAGE_PATH`
env > default** (`./db` for the CLI, `vantadb_data` for the engine).

`VANTA_DB` is **not** read anywhere and is not a supported alias. The
`vantadb-server` / MCP child sets `VANTADB_STORAGE_PATH` from `--db`, so the
flag still wins end to end — the TECH-01 property ADR-0012 wanted, achieved
without the second name.

`docs/user/operations/CONFIGURATION.md` carries the normative table
(§ Environment Variables Outside `VantaConfig` and the CLI flag table).

### Positive consequences

- One setting, one env var, one precedence rule — no "which one wins?"
  question for a future agent to get wrong.
- The TECH-01 class of bug (flag and config reading different names) is
  structurally impossible.
- Consistent with the rest of the FIND-89 consolidation: every canonical
  env var is `VANTADB_*`.

### Negative consequences

- Deployments that set `VANTA_DB` in the environment silently lose it. The
  mitigation is the `feat!:` + `BREAKING CHANGE:` changelog entry, the same
  one the other FIND-89 renames carried.
- The historical record in ADR-0012 no longer describes the shipped code —
  which is exactly why that ADR is marked superseded rather than edited.
