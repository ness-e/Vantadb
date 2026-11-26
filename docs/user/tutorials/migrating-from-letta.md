---
title: Migrating from Letta to VantaDB
kind: tutorial
status: active
description: "Export a Letta agent file (.af), convert its memory blocks and message history with the built-in importer, and import them into VantaDB"
tags: [vantadb, tutorial, migration, letta]
---

# Migrating from Letta to VantaDB

Letta's open **Agent File** (`.af`) format serializes a stateful agent — model
configuration, memory blocks, message history, tools. VantaDB's converter maps
the two memory-bearing components: **memory blocks** (in-context segments such
as `persona` and `human`) and the **message history**, with the agent linkage
preserved. Agent configuration is not memory and is not imported (it is
counted, never dropped silently). The converter reads the file only — no Letta
server, no API key.

## 1. Export your agent file

- **ADE** — select *Export Agent* on the agent page; or
- **API / SDK** — `GET /v1/agents/{AGENT_ID}/export`, eg:

  ```python
  # vanta-skip: requires the letta-client package
  from letta_client import Letta

  client = Letta(base_url="http://localhost:8283")
  schema = client.agents.export_file(agent_id="<AGENT_ID>")

  with open("agent.af", "w", encoding="utf-8") as fh:
      fh.write(schema.model_dump_json() if hasattr(schema, "model_dump_json") else str(schema))
  ```

Secrets are exported as `null` by Letta's design — the file carries no
credentials.

## 2. Convert to the VantaDB interchange format

```rust
// vanta-skip: illustrative snippet; run inside a crate with `vantadb` as a dependency
use vantadb::sdk::importers::letta;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conversion = letta::convert_file("agent.af")?;

    println!(
        "converted {} records ({} skipped, no text content)",
        conversion.stats.converted, conversion.stats.skipped
    );
    for (field, count) in &conversion.stats.discarded {
        println!("declared discard: {count}× {field}");
    }

    conversion.write_jsonl("vantadb-memory.jsonl")?;
    Ok(())
}
```

## 3. Import into VantaDB

```bash
vanta-cli import --in vantadb-memory.jsonl
```

Or directly from Rust:

```rust
// vanta-skip: illustrative snippet; run inside a crate with `vantadb` as a dependency
use vantadb::sdk::importers::letta;
use vantadb::Embedded;

let db = Embedded::open("./vantadb_data")?;
let conversion = letta::convert_file("agent.af")?;
let report = db.import_records(conversion.into_records()?, /* quarantine */ false)?;
println!("inserted {}, updated {}, errors {}", report.inserted, report.updated, report.errors);
db.flush()?;
```

From Python, import the converted JSONL with the Python SDK:

```python
from vantadb import Client

db = Client("./vantadb_data")
print(db.import_file("vantadb-memory.jsonl"))
```

## What maps, and what is declaredly dropped

| `.af` | VantaDB record |
|-------|----------------|
| `blocks[].value` | `payload`, namespace `letta/blocks`, `key` = block id |
| `blocks[].label`, `description`, `limit`, `read_only`, `metadata` | `metadata."letta.label" | "letta.description" | "letta.limit" | "letta.read_only" | "letta.block_metadata"` |
| agent `block_ids` linkage | `metadata."letta.agents"` — every agent that references the block |
| `agents[].messages[].content` | `payload`, namespace `letta/<agent>/messages`, `key` = message id — string, or content parts (text parts joined; non-text parts counted as `message.content_parts`; empty/text-less arrays skipped) |
| message `created_at`, `role`, `type`, `model`, `in_context` | `created_at_ms` + `metadata."letta.*"` |
| any other block/message field | `metadata."letta.<field>"` |
| agent `system` prompt, `llm_config`, `embedding_config`, tool rules, tools, groups, MCP servers | **not imported** — agent configuration, counted as `agent.*` / group counters |
| `files`, `sources` | **not imported** — folder references without content; `.af` does not serialize data-source file contents upstream |
| tool-call messages without text content (`content: []` or text-less parts) | **skipped** (counted) |

Blocks and messages have no embeddings in `.af`; attach a `vector` per line
before importing if you want semantic search.

## Known limitations

- **Archival memory is not in `.af`.** Letta's file format explicitly does not
  serialize archival-memory passages (upstream roadmap), so there is nothing to
  import for them — this is a format limit, declared rather than silently
  dropped.
- **Blocks are context-window state, not chat facts.** They import as records;
  re-importing the same `.af` updates them (keys = block/message ids).
- **Multiple agents in one file**: blocks are emitted once with their agent
  linkage; message histories are namespaced per agent.

## Rollback

Keep `agent.af` and your Letta server untouched — the converter only reads the
file. To undo, delete the `letta/…` namespaces from the target database.

---

**See also:** [interchange format reference](../../api/MEMORY_INTERCHANGE_FORMAT.md) ·
[Migrating from Mem0](./migrating-from-mem0.md) ·
[Migrating from Zep / Graphiti](./migrating-from-zep.md)
