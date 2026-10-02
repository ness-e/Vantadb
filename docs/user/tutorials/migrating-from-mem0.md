---
title: Migrating from Mem0 to VantaDB
kind: tutorial
status: active
description: "Export your Mem0 memories without credentials, convert them with the built-in importer, and import them into VantaDB — metadata, timestamps and provenance preserved"
tags: [vantadb, tutorial, migration, mem0]
---

# Migrating from Mem0 to VantaDB

You can move your Mem0 memories into VantaDB without touching Mem0's API: the
core crate ships a converter from a Mem0 export file to the
[VantaDB memory interchange format](../../api/MEMORY_INTERCHANGE_FORMAT.md), and
the existing `import` path does the rest. Metadata, timestamps, categories and
scope ids survive the trip; the fields that do not survive are listed below.

## 1. Export your Mem0 memories (no credentials needed for the file)

Any of the documented Mem0 shapes works:

- **CLI** — `mem0 list --user-id alice --output json > memories.json`
  (repeat per user/agent scope; the `--agent` envelope is accepted too);
- **Python (OSS)** — serialize the `get_all()` result:

  ```python
  # vanta-skip: requires the mem0 package
  import json
  from mem0 import Memory

  m = Memory()
  data = m.get_all(filters={"user_id": "alice"}, top_k=1000)
  with open("memories.json", "w", encoding="utf-8") as fh:
      json.dump(data, fh, indent=2)
  ```

- **Platform** — *Memory Exports* in the dashboard (download the JSON).

The converter reads **the file only**; it never calls Mem0 and needs no API key.

## 2. Convert to the VantaDB interchange format

The importer lives in the core crate
(`vantadb::sdk::importers::mem0`). In any Rust project with `vantadb` added:

```rust
// vanta-skip: illustrative snippet; run inside a crate with `vantadb` as a dependency
use vantadb::sdk::importers::mem0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conversion = mem0::convert_file("memories.json")?;

    println!(
        "converted {} memories ({} skipped, no payload)",
        conversion.stats.converted, conversion.stats.skipped
    );
    for (field, count) in &conversion.stats.discarded {
        println!("declared discard: {count}× {field}");
    }

    conversion.write_jsonl("vantadb-memory.jsonl")?;
    Ok(())
}
```

Accepted input shapes: a JSON array (the `get_all()`/CLI dump), `{"results": …}`
(OSS `get_all` envelope) or `{"data": …}` (CLI `--agent` envelope).

## 3. Import into VantaDB

**CLI (one command):**

```bash
vanta-cli import --in vantadb-memory.jsonl
```

**From Rust — import directly, without the intermediate file:**

```rust
// vanta-skip: illustrative snippet; run inside a crate with `vantadb` as a dependency
use vantadb::sdk::importers::mem0;
use vantadb::Embedded;

let db = Embedded::open("./vantadb_data")?;
let conversion = mem0::convert_file("memories.json")?;
let report = db.import_records(conversion.into_records()?, /* quarantine */ false)?;
println!("inserted {}, updated {}, errors {}", report.inserted, report.updated, report.errors);
db.flush()?;
```

**From Python — import the converted JSONL with the Python SDK:**

```python
from vantadb import Client

db = Client("./vantadb_data")
report = db.import_file("vantadb-memory.jsonl")
print(report)   # inserted / updated / errors / duration_ms
```

## 4. Verify

```python
from vantadb import Client

db = Client("./vantadb_data")
page = db.memory.list("mem0/alice")       # the imported namespace
print(len(page), page[0].key, page[0].payload)
```

Each Mem0 memory becomes one record: `namespace = mem0/<scope>` (first of
`user_id`, `agent_id`, `run_id`, `app_id` present in the export), `key` = the
Mem0 memory id.

## What maps, and what is declaredly dropped

| Mem0 | VantaDB record |
|------|----------------|
| `memory` (or `text`/`content`) | `payload` |
| `id` | `key` + `metadata.source_id` |
| `created_at`, `updated_at` | `created_at_ms`, `updated_at_ms` (date-only and epoch variants accepted) |
| `metadata` object | `metadata."mem0.metadata"` (JSON string) |
| `categories` | `metadata."mem0.categories"` (filterable list) |
| `hash` | `metadata."mem0.hash"` |
| `user_id`, `agent_id`, `run_id`, `app_id` | `metadata` (same names) + namespace scope |
| any other field | `metadata."mem0.<field>"` |
| `score`, `relevance`, `selection_rank` | **discarded** (retrieval-time values); counted in `Conversion.stats.discarded` |

Memories have no embedding in the export. If you want vector search, attach
`vector` to each line before importing (`conversion.lines[i].vector =
Some(embedding)`), then `rebuild_index` if needed.

## Known limitations

- **Memories are assertion-level.** Everything imports as `Asserted` with
  confidence `1.0`; Mem0 has no equivalent of VantaDB's derived/validity
  semantics to translate.
- **Re-import is an update, not a duplicate**: keys are the Mem0 ids, so the
  same export imported twice updates records. Memories without an id get a
  deterministic content-hash key with the same property.
- The import path is **file-based**: no live sync with a running Mem0 instance
  (export, convert, import; repeat when you want a refresh).

## Rollback

Keep your original `memories.json` and your Mem0 store untouched — the
converter only reads the file, and VantaDB does not modify or delete Mem0 data.
To undo, delete the `mem0/…` namespaces from the target VantaDB database.

---

**See also:** [interchange format reference](../../api/MEMORY_INTERCHANGE_FORMAT.md) ·
[embedding integrations](./05-embedding-integrations.md) ·
[AI agent memory](./01-ai-agent-memory.md)
