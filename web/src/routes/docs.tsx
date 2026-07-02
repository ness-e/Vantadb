import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/docs")({
  head: () => ({
    meta: [
      { title: "VantaDB — Documentation" },
      {
        name: "description",
        content:
          "Get started with VantaDB: embedded database for AI agents. Installation, quickstart, SDK reference, and guides.",
      },
    ],
  }),
  component: DocsPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
const sections = [
  {
    id: "getting-started",
    num: "01",
    title: "Getting Started",
    code: `$ pip install vantadb-py

$ python
>>> import vantadb_py as vantadb
>>> db = vantadb.VantaDB("./my_project.vdb")
>>> db.put(key="doc-1", vector=[0.12, 0.45, 0.78, 0.33], metadata={"text": "VantaDB is an embedded vector database for AI agents."})
>>> results = db.search_memory(query=[0.12, 0.45, 0.78, 0.33], top_k=5)
>>> results[0].score
0.9421`,
    desc: "Install VantaDB and run your first hybrid query in under 60 seconds. Ships as a single native binary with zero system dependencies — Python 3.10+ or Rust 1.75+ required.",
  },
  {
    id: "python-sdk",
    num: "02",
    title: "Python SDK",
    code: `import vantadb_py as vantadb

db = vantadb.VantaDB("./my_db.vdb")

# Store
db.put(key="doc-1", vector=[0.1, 0.2, 0.35], metadata={"source": "web", "tags": ["ai", "ml"], "text": "Document text here"})

# Search
results = db.search_memory(query=[0.1, 0.2, 0.35], top_k=10)

# With metadata filter
results = db.search_memory(query=[0.1, 0.2, 0.35], top_k=10, filter={"source": "web"})`,
    desc: "The Python SDK provides a pandas-friendly interface with full type hints. Supports semantic search, hybrid BM25+HNSW retrieval, metadata filtering, and WAL-backed durability out of the box.",
  },
  {
    id: "rust-sdk",
    num: "03",
    title: "Rust SDK",
    code: `use vantadb::prelude::*;

fn main() -> Result<()> {
  let mut db = VantaDB::open("./my_db.vdb")?;

  db.insert("docs", vec![
    Document::new()
      .vector(vec![0.1, 0.2, 0.3])
      .field("title", "Hello")
      .field("content", "Document body")
  ])?;

  let results = db.search_memory(
    query: vec![0.1, 0.2, 0.3],
    top_k: 10,
    mode: SearchMode::Hybrid,
  )?;

  for doc in results {
    println!("{} — {}", doc.score, doc.field::<str>("title"));
  }
  Ok(())
}`,
    desc: "Zero-cost abstractions over the core Rust engine. Embed VantaDB directly into your application with no sidecars or IPC — ideal for edge devices, CLI tools, and agent runtimes.",
  },
  {
    id: "cli-reference",
    num: "04",
    title: "CLI Reference",
    code: `$ vantadb --help
VantaDB 0.2.0 — Embedded vector database for AI agents

COMMANDS:
    init        Initialize a new database
    insert      Insert documents from JSON/CSV
    query       Run a semantic or hybrid search
    serve       Start the HTTP API server (optional)
    inspect     Inspect database stats and index
    checkpoint  Force WAL checkpoint
    repl        Connect to a primary for replication

OPTIONS:
    --db-path <PATH>      Database path [default: ./.vantadb]
    --log-level <LEVEL>   Log level [default: info]

$ vantadb init --db-path ./my_db.vdb
[INFO] Initialized database at ./my_db.vdb
[INFO] Index configured: HNSW (M=16, ef=200)`,
    desc: "The `vantadb` CLI provides full database management from the terminal — initialize, bulk-insert, query, inspect, and manage WAL checkpoints without writing code.",
  },
  {
    id: "configuration",
    num: "05",
    title: "Configuration",
    code: `# .vantadb/config.toml

[storage]
path = "./data"
sync_mode = "fsync"     # async | fsync | full
wal_flush_interval_ms = 100

[indexing.defaults]
m = 16                  # HNSW neighbors per node
ef_construction = 200   # index quality
ef_search = 50          # search breadth

[indexing.bm25]
tokenizer = "whitespace"
stemmer = "english"
k1 = 1.2
b = 0.75

[hybrid]
rrf_k = 60
weights = [0.5, 0.5]   # [bm25, vector]

[limits]
max_document_size = 10485760   # 10 MB
max_collections = 256`,
    desc: "Configure every aspect of the VantaDB engine via TOML, environment variables, or inline API calls. Tune HNSW parameters for recall/latency tradeoffs and set durability guarantees.",
  },
  {
    id: "migration-guide",
    num: "06",
    title: "Migration Guide",
    code: `# v0.1.x → v0.2.0 Migration

## Breaking changes
- Python 3.8/3.9 dropped (minimum is now 3.11)
- Rust MSRV bumped to 1.94.1
- PyO3 upgraded to 0.29 (Bound API)

## Deprecated
- Experimental governance and LISP VM feature — removed from core
- Biological terminology (neurons/synapses → UnifiedNode/Edge)

## New
- SQ8 quantization for 4x memory reduction
- Batch put with Rayon parallelism (put_batch)
- WAL compaction and TTL eviction
- MCP Server integration

## Rebuild index
$ vantadb-cli inspect ./my_db.vdb
$ vantadb-cli doctor ./my_db.vdb  # verifies index integrity`,
    desc: "Follow our migration guides to upgrade between major versions. Each guide includes breaking changes, deprecated APIs, and automated migration scripts.",
  },
];

function DocsPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="00"
        eyebrow="Documentation"
        title={
          <span>
            Start in 60
            <br />
            seconds.
          </span>
        }
        sub="Comprehensive guides, SDK references, and configuration reference for VantaDB — the embedded database for AI agents."
      />

      <main className="engine-main">
        {sections.map((s, i) => (
          <section
            key={s.id}
            id={s.id}
            className={`engine-section${i < sections.length - 1 ? " engine-section--bordered" : ""}`}
          >
            <span className="swiss-eyebrow">
              {s.num} / 06 — {s.title}
            </span>

            <div className="swiss-grid-12" style={{ alignItems: "start", marginTop: "3rem" }}>
              {/* Description left */}
              <div className="col-span-4">
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.5rem",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    color: "var(--foreground)",
                    lineHeight: 1.1,
                    marginBottom: "1.25rem",
                  }}
                >
                  {s.title}
                </h2>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.82rem",
                    color: "var(--muted)",
                    lineHeight: 1.7,
                    margin: 0,
                  }}
                >
                  {s.desc}
                </p>
              </div>

              {/* Code right */}
              <div className="col-span-8">
                <div
                  style={{
                    border: "1px solid var(--border)",
                    background: "var(--block-dark-bg)",
                  }}
                >
                  <div
                    style={{
                      padding: "0.65rem 1.25rem",
                      borderBottom: "1px solid var(--block-dark-border)",
                    }}
                  >
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.6rem",
                        color: "var(--block-dark-muted)",
                        textTransform: "uppercase",
                        letterSpacing: "0.08em",
                      }}
                    >
                      {s.id}
                    </span>
                  </div>
                  <pre
                    style={{
                      margin: 0,
                      padding: "1.5rem",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.75rem",
                      lineHeight: 1.65,
                      color: "var(--block-dark-text)",
                      overflowX: "auto",
                      whiteSpace: "pre",
                    }}
                  >
                    <code>{s.code}</code>
                  </pre>
                </div>
              </div>
            </div>
          </section>
        ))}
      </main>
    </div>
  );
}
