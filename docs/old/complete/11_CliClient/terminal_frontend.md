# Database Interactive Shell (CLI)
> **Status**: 🟡 In Progress — FASE 9

## 1. REPL Interface
Developers working with Local AI need a direct method to inspect the Unified Nodes stored in ConnectomeDB. 
The `connectomedb-cli` provides an interactive shell (REPL) out of the box, connecting implicitly to localhost:8080 or bypassing network for an embedded memory state.

## 2. Shell Commands
- `\connect <url>`: Connects the REPL to a remote/local TCP daemon.
- `\status`: Displays current memory limits, OOM flags, and node count inside the active database.
- `SIGUE ...`: Any unrecognized shell command is automatically tunneled to the QueryEngine's `/api/v1/query` proxy.
