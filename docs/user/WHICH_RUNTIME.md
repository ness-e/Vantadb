---
title: "Choose a runtime"
kind: howto
status: active
description: "Rust embedded, Python, Node native, TypeScript/WASM, raw WASM, MCP, HTTP or CLI — the criterion for each surface, what it can and cannot do, and the link to its reference"
tags: [vantadb, bindings, api]
---

# Choose a runtime

There are eight ways to reach the same engine. They differ in three things that
matter: **who embeds it**, **whether data lands on disk**, and **whether the
cognitive layer (`vanta-memory`) is exposed**. This page answers those three; the
linked reference owns every signature, so nothing here can drift from it.

## The table

| Runtime | Choose it when | Data persistence | Cognitive layer (`vanta-memory`) | Reference |
|---|---|---|---|---|
| **Rust embedded** (`vantadb` crate) | You are writing a Rust service, a CLI, or another binding. This is the full engine — every other row is a projection of it. | Disk, or in-memory, your choice | Full crate | [Embedded SDK](../api/EMBEDDED_SDK.md) |
| **Python** (`vantadb-py`) | Your stack is Python: notebooks, ML pipelines, agent frameworks, data scripts. | Disk | Minimal — capture and recall only | [Python SDK](../api/PYTHON_SDK.md) |
| **Node native** (`vantadb-node`) | A long-lived Node service, CLI or agent that needs the same durability guarantee as the Rust core. | **Disk** (WAL + fsync) | Thin facade (recall, consolidate, context assembly, reinforce) | [Node SDK](../api/NODE_SDK.md) |
| **TypeScript / WASM** (`vantadb`) | Browser, Bun, Deno, edge runtimes — or throwaway in-memory work in Node. Bundles pre-compiled WASM; no native build step. | **Browser:** OPFS/IndexedDB. **Node:** in-memory only | Not exposed | [TypeScript SDK](../api/TS_SDK.md) · [WASM persistence](../api/WASM_PERSISTENCE.md) |
| **Raw WASM** (`vantadb-wasm`) | You are writing your own JS/TS wrapper on top of the binding the TS SDK already uses. | Same as TS/WASM | Not exposed | [WASM API](../api/WASM_API.md) |
| **MCP server** | Your client is an AI IDE or agent that speaks MCP (Claude Code, Cursor, OpenCode, Windsurf). No application code at all. | Inherits the CLI's database | Exposed as MCP tool families | [MCP reference](../api/MCP.md) |
| **HTTP server** | Something that is not Rust/Python/JS has to reach the engine over the network. | Inherits the server's database | Reachable over REST — threads, conversations and skills have endpoints, and `POST /api/v2/mcp/proxy` forwards MCP | [HTTP API](../api/HTTP_API.md) |
| **CLI** (`vanta-cli`) | Operations: put/read, export, index audit and repair, certificate verification, one-off maintenance. | Disk | — | [README §Embedded CLI](../../README.md#embedded-cli) |

## The decision, in three questions

1. **Is the client an AI IDE or agent, and do you want zero code?** → MCP.
   See [AI IDEs](AI_IDES.md).
2. **Does the data have to survive a restart?** → Rust, Python or **Node native**.
   In Node specifically, this question picks the binding for you (next section).
3. **Does your host language have a binding?** Rust, Python and JS/TS do. If not,
   the HTTP server is the boundary — not a hand-rolled FFI wrapper.

If none of those matched because your code runs in a browser, take TS/WASM and
read the persistence limits before designing around them.

## The trap that actually bites: native vs WASM in Node

`vantadb` (WASM) and `vantadb-node` (native) both install from npm and both work
in Node. **Only the native one writes to disk.** The WASM build runs in memory
inside Node, so a process restart discards everything, while the same build in a
browser persists through OPFS/IndexedDB.

The ownership of that comparison — and the fairness caveat that native pays
fsync costs WASM never sees, so their throughput is not comparable — is
[Node SDK §Native vs WASM](../api/NODE_SDK.md). Do not benchmark one against the
other and conclude anything.

## Parity traps between the four bindings

Before you assume "it works the same in every language", read the parity matrix:
[Bindings namespace map](../api/BINDINGS_NAMESPACES.md). The three that cost the
most time:

| Trap | What happens |
|---|---|
| **Casing at the wire boundary** | Python keeps `snake_case` interiors; JS/TS public API is `camelCase`. A payload mid-migration keeps one casing throughout — never mixed. |
| **Score vs distance** | Bindings do not all return the same direction or field name for the same computation. The convention is declared per transport in [Bindings namespace map](../api/BINDINGS_NAMESPACES.md) and [WASM API](../api/WASM_API.md). |
| **Core-only capabilities** | Some engine features have no binding entry point yet. The matrix lists them rather than letting you discover it at runtime. |

## Where to install each one

Installation, platform support and the build prerequisites belong to each
reference; duplicating them here is how installation instructions go stale:

| Runtime | Installation lives in |
|---|---|
| Rust | [Embedded SDK](../api/EMBEDDED_SDK.md) |
| Python | [Python SDK §Installation](../api/PYTHON_SDK.md) |
| Node native | [Node SDK §Installation](../api/NODE_SDK.md) |
| TypeScript / WASM | [TypeScript SDK §Installation](../api/TS_SDK.md) |
| CLI and MCP | [Quickstart §0](QUICKSTART.md) · [README §Embedded CLI](../../README.md#embedded-cli) |
| HTTP server | [HTTP API](../api/HTTP_API.md) |

## Where to go next

| Next step | Page |
|---|---|
| Does VantaDB fit what I am building at all? | [Is VantaDB for your use case?](IS_IT_FOR_ME.md) |
| What the engine is and is not | [What VantaDB is](concepts/what-is-vantadb.md) |
| First working call in your language | [Quickstart](QUICKSTART.md) |
| The cognitive layer (L0→L3) in detail | [Vanta Memory reference](../api/VANTA_MEMORY.md) |
