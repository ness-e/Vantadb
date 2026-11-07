---
title: "Model Context Protocol (MCP) Server"
kind: index
status: active
description: "MCP server reference for VantaDB: tools, profiles, Brain cognitive profile, and AI IDE integration"
tags: [mcp, api, brain-profile, tools]
---

# Model Context Protocol (MCP) Server

VantaDB provides a native Model Context Protocol (MCP) server implementation (`vantadb-mcp`) allowing AI assistants in IDEs (Claude Code, Cursor, OpenCode) and autonomous agents to persist, recall, and manage long-term cognitive memory across sessions.

---

## 1. Profiles Overview

To avoid overwhelming an agent's context window with dozens of granular database primitives, `vantadb-mcp` organizes tools into structured profiles:

| Profile | Target Environment | Tools | Description |
|---|---|:---:|---|
| **`McpProfile::Brain`** (default) | AI IDEs / Coding Agents | 15 | Cognitive memory surface including the meta-dispatch router. |
| **`McpProfile::Full`** | DB Administration | 84 | Complete engine introspection, raw queries, and index maintenance. |
| Other profiles (`agent`, `dev`, `memory`) | Scoped integrations | varies | Additional profiles exist; tool counts are reported by the server at startup. |

Tool counts are build-dependent and reported by the server itself at
startup (`tools/list`). The names below were measured on the 0.9.0 build.

---

## 2. `McpProfile::Brain` Tool Surface (15 tools)

Memory: `memory_put`, `memory_get`, `memory_supersede`, `memory_recall`,
`memory_dream`, `memory_search`, `memory_forget`
Scenes & dreams: `vanta_scene`, `vanta_dream`
Graph & dispatch: `vanta_graph`, `vanta_dispatch`
Bugfix context: `bugfix_record`, `bugfix_query`
Assembly & threads: `context_assemble`, `thread_send`

## 3. `McpProfile::Full` Tool Surface (84 tools)

Structured JSON schemas are advertised per tool via `tools/list` — tool
names are listed here, schemas are not repeated to avoid drift:

```text
memory_put memory_put_batch memory_get memory_delete
memory_verify_certificate memory_delete_by_filter memory_list
memory_list_namespaces memory_versions memory_supersede memory_reinforce
query_iql memory_recall memory_search search_semantic search_with_method
search_multi get_node_neighbors graph_page_rank graph_degree_centrality
graph_traverse graph_topological_sort graph_is_dag remove_edge
inject_context read_axioms write_axiom delete_axiom collection_stats
collection_delete rehydrate purge_expired compact_wal flush
compact_layout vacuum rebuild_index audit_text_index repair_text_index
capabilities generate_snippet list_snapshots snapshot_create
snapshot_restore export import bulk_import_file bulk_import_stream
embed_texts skill_list skill_view skill_create skill_update skill_patch
skill_files_write skill_extract bugfix_record bugfix_query code_search
code_explore code_index wiki_search wiki_read wiki_list wiki_graph
wiki_ingest wiki_ingest_status context_assemble dream_list dream_load
dream_discard dream_consolidate dream_promote scene_read scene_list
scene_query scene_write scene_edit thread_create thread_send
thread_get thread_list thread_delete thread_purge_expired
```

---

## 4. Configuration in AI IDEs

### Claude Code (`claude.json` / `settings.json`)

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--profile", "brain", "--db", "./.vanta/memory.db"]
    }
  }
}
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--profile", "brain"]
    }
  }
}
```
