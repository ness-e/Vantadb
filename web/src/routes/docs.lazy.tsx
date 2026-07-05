import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { DocsSidebar } from "@/components/DocsSidebar";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/docs.css";

export const Route = createLazyRoute("/docs")({
  component: DocsPage,
  pendingComponent: PendingComponent,
});

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

db.put(key="doc-1", vector=[0.1, 0.2, 0.35], metadata={"source": "web", "tags": ["ai", "ml"], "text": "Document text here"})

results = db.search_memory(query=[0.1, 0.2, 0.35], top_k=10)

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
    code: `$ vanta-cli --help
VantaDB 0.1.5 — Embedded vector database for AI agents

COMMANDS:
    put           Insert or update a document
    get           Retrieve a document by key
    delete        Delete a document by key
    search        Search for similar vectors
    list          List all documents in a namespace
    server        Start the HTTP API server
    export        Export namespace data
    import        Import data from file
    rebuild-index Rebuild the HNSW index
    stats         Show database statistics
    compact       Compact the WAL

OPTIONS:
    --db-path <PATH>      Database path [default: ./.vantadb]
    --log-level <LEVEL>   Log level [default: info]

$ vanta-cli put --db-path ./my_db.vdb --key doc-1 --vector "0.12,0.45,0.78,0.33"
[INFO] Document 'doc-1' inserted`,
    desc: "The vanta-cli provides full database management from the terminal — put, get, delete, search, list, export, import, rebuild the index, inspect stats, and compact the WAL without writing code.",
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
    code: `# VantaDB v0.1.5 — Feature Overview

## Available
- Python SDK (pip install vantadb-py, import vantadb_py)
- Rust SDK (crates.io)
- CLI (vanta-cli — put, get, delete, search, list, server, export, import)
- HNSW vector search + BM25 full-text + hybrid RRF
- WAL-backed durability with WAL compaction
- 3 storage backends (Memory, Sled, RocksDB)
- Batch operations (put_batch)
- Graph methods (BFS, DFS, topological sort, DAG check)
- MCP Server (experimental)
- SQ8 quantization for memory reduction

## Rebuild index
$ vanta-cli rebuild-index --db-path ./my_db.vdb`,
    desc: "Overview of features available in VantaDB v0.1.5. The Python SDK, Rust SDK, and CLI (vanta-cli) are all ready for local, self-hosted use.",
  },
  {
    id: "api-reference",
    num: "07",
    title: "API Reference",
    code: `import vantadb_py as vantadb

# Initialize the engine
db = vantadb.VantaDB("./vanta_data")

# Store a memory record
db.put(
    namespace="agent/main",
    key="doc_1",
    payload="User asked to summarize the meeting notes.",
    vector=[0.1, 0.2, 0.3, 0.4],
    metadata={"source": "chat", "timestamp": 1719000000},
)

# Search memory
results = db.search_memory(
    namespace="agent/main",
    query_vector=[0.1, 0.2, 0.3, 0.4],
    top_k=5,
)`,
    desc: "The complete API reference for VantaDB — Python SDK (PyO3), Rust SDK, CLI commands, and HTTP server endpoints with code examples. Full documentation is hosted on GitHub.",
  },
];

const sidebarItems = sections.map(({ id, num, title }) => ({ id, num, title }));

function DocsPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="06"
        title={
          <span>
            Start in 60
            <br />
            seconds.
          </span>
        }
        sub="Comprehensive guides, SDK references, and configuration reference for VantaDB — the embedded database for AI agents."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <div className="docs-layout">
            <DocsSidebar items={sidebarItems} />

            <div>
              {sections.map((s) => (
                <div
                  key={s.id}
                  id={s.id}
                  className="nb-card nb-bg-cross--faint docs-card"
                >
                  <h3 className="docs-section-title">
                    {s.num} — {s.title}
                  </h3>
                  <div className="nb-divider" />
                  <p className="docs-section-desc">
                    {s.desc}
                  </p>
                  <div className="nb-frame docs-code-frame">
                    <pre className="docs-code-pre">
                      <code>{s.code}</code>
                    </pre>
                  </div>
                </div>
              ))}

              <div className="nb-block-amber docs-help-block">
                <span className="docs-help-label">
                  NEED HELP?
                </span>
                <p className="docs-help-text">
                  Join our Discord or open a GitHub discussion.
                </p>
                <a
                  href="https://github.com/ness-e/Vantadb/discussions"
                  className="nb-btn nb-btn--ghost docs-help-link"
                >
                  DISCUSSIONS
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>

      <style>{`
        @media (max-width: 768px) {
          [style*="grid-template-columns: 240px 1fr"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}
