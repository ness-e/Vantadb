---
title: Migrating from Zep / Graphiti to VantaDB
kind: tutorial
status: active
description: "Export your Zep or Graphiti graph without credentials, convert episodes and facts — valid-time windows and episode provenance included — and import them into VantaDB"
tags: [vantadb, tutorial, migration, zep]
---

# Migrating from Zep / Graphiti to VantaDB

Zep keeps two kinds of memory-bearing artifacts: **episodes** (the raw source
data you ingested) and **facts** (time-stamped edges derived from those
episodes). VantaDB's interchange format maps both: episodes become asserted
records, facts become derived records that keep their source-episode provenance
and their `valid_at`/`invalid_at` window. The converter reads a JSON dump of
your graph — it never calls Zep and needs no API key.

## 1. Export your graph (file-only)

Use the SDK bulk list methods (`Reading Data from the Graph` in the Zep docs)
and write one JSON file per scope:

```python
# vanta-skip: requires the zep-cloud package
import json
from zep_cloud.client import Zep

client = Zep(api_key=API_KEY)
USER = "emily-painter"

episodes = client.graph.episode.get_by_user_id(user_id=USER, lastn=1000)
facts = client.graph.edge.get_by_user_id(user_id=USER)

def dump(obj):
    # by_alias=True emits the REST field names (`uuid`, `valid_at`, …),
    # which is the shape the converter reads.
    return json.loads(obj.model_dump_json(by_alias=True)) if hasattr(obj, "model_dump_json") else obj

with open("zep-graph.json", "w", encoding="utf-8") as fh:
    json.dump({
        "user_id": USER,
        "episodes": [dump(e) for e in (episodes.episodes or [])],
        "facts": [dump(f) for f in facts],
    }, fh, indent=2)
```

The importer accepts `{"user_id" | "graph_id", "episodes": [...],
"facts": [...]}`; `edges` is accepted as an alias of `facts`. Graphiti (the
open-source library) uses the same episode/edge field names, so its node/edge
dumps can be shaped the same way.

## 2. Convert to the VantaDB interchange format

```rust
// vanta-skip: illustrative snippet; run inside a crate with `vantadb` as a dependency
use vantadb::sdk::importers::zep;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conversion = zep::convert_file("zep-graph.json")?;

    println!(
        "converted {} records ({} skipped)",
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
use vantadb::sdk::importers::zep;
use vantadb::Embedded;

let db = Embedded::open("./vantadb_data")?;
let conversion = zep::convert_file("zep-graph.json")?;
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

| Zep | VantaDB record |
|-----|----------------|
| episode `uuid`, `content`, `created_at` | `key`, `payload`, `created_at_ms`; `confidence_class = Asserted` |
| episode `source`, `role`, `role_type`, `thread_id`, `source_description`, `document_id`, `metadata` | `metadata."zep.*"` |
| fact `uuid`, `fact`, `created_at` | `key`, `payload`, `created_at_ms` |
| fact `valid_at` / `invalid_at` | `valid_at_ms` / `invalid_at_ms` (point-in-time queries work out of the box) |
| fact `episodes` (source episode uuids) | `derived_from` (parent keys) + `confidence_class = Derived`, `confidence = 0.9` |
| fact `name`, node uuids/names/labels, `attributes`, `scope`, `hyperedge_uuid` | `metadata."zep.*"` |
| `expired_at` | **discarded** — transaction-time invalidation has no v2 field; counted in `Conversion.stats.discarded` |
| `score`, `relevance`, `selection_rank` | **discarded** (retrieval-time values) |
| root `nodes`, `communities`, `observations`, `*_summaries` | not imported (counted) — entities/communities are graph structure, not memory records |

A fact whose `episodes` list is empty cannot be `Derived` (the format requires
at least one parent): it is imported as `Asserted` and counted as
`fact.episodes_missing`. Facts keep confidence `0.9` (the derivation discount);
episodes keep `1.0`.

## Verify the time travel

```python
from vantadb import Client

db = Client("./vantadb_data")
# The state of Emily's preferences at a point in valid time:
page = db.memory.list("zep/emily-painter", as_of_ms=1757000000000)
for record in page:
    print(record.key, record.payload, record.metadata.get("zep.name"))
```

## Known limitations

- **Fact provenance is constrained to the episodes in the export.** A fact that
  references missing or skipped episodes is filtered to its resolvable parents
  (counted as `fact.episodes_unresolved`); if none remain it imports as
  `Asserted` (`fact.episodes_missing` when the fact had no episodes at all), so
  `derived_from` never dangles.
- **`edges` when `facts` is also present** is not imported (the `facts` array
  wins) and is counted in the conversion report.
- **Transaction time is not preserved** (`expired_at`): valid time is
  (`valid_at`/`invalid_at`); when Zep learned about a change is not.
- **Re-import is an update, not a duplicate**: keys are the Zep uuids.
- **No live sync**: export, convert, import; repeat for a refresh.
- Episodes and facts share one namespace (`zep/<scope>`) so provenance links
  resolve; if you export multiple graphs, convert each file separately (each
  gets its own scope namespace).

## Rollback

Keep `zep-graph.json` and your Zep graph untouched — the converter only reads
the file. To undo, delete the `zep/…` namespaces from the target database.

---

**See also:** [interchange format reference](../../api/MEMORY_INTERCHANGE_FORMAT.md) ·
[Migrating from Mem0](./migrating-from-mem0.md) ·
[Migrating from Letta](./migrating-from-letta.md)
