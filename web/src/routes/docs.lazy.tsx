import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { DocsSidebar } from "@/components/DocsSidebar";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { gsap } from "@/lib/gsap";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
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
    desc: "Install VantaDB and run your first hybrid query in under 60 seconds. Ships as a single native binary with zero system dependencies \u2014 Python 3.11+ or Rust 1.94+ required.",
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
    println!("{} \u2014 {}", doc.score, doc.field::<str>("title"));
  }
  Ok(())
}`,
    desc: "Zero-cost abstractions over the core Rust engine. Embed VantaDB directly into your application with no sidecars or IPC \u2014 ideal for edge devices, CLI tools, and agent runtimes.",
  },
  {
    id: "cli-reference",
    num: "04",
    title: "CLI Reference",
    code: `$ vanta-cli --help
VantaDB 0.1.5 \u2014 Embedded vector database for AI agents

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
    desc: "The vanta-cli provides full database management from the terminal \u2014 put, get, delete, search, list, export, import, rebuild the index, inspect stats, and compact the WAL without writing code.",
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
    code: `# VantaDB v0.2.0 \u2014 Feature Overview

## Available
- Python SDK (pip install vantadb-py, import vantadb_py)
- Rust SDK (crates.io)
- CLI (vanta-cli)
- HNSW vector search + BM25 full-text + hybrid RRF
- WAL-backed durability with WAL compaction
- 3 storage backends (Fjall, RocksDB, InMemory)
- Batch operations (put_batch)
- Graph methods (BFS, DFS, topological sort, DAG check)
- MCP Server (experimental)
- SQ8 + TurboQuant + RaBitQ quantization
- Predictive kernel prefetching
- TTL auto-eviction with background compaction`,
    desc: "Overview of features available in VantaDB v0.2.0. The Python SDK, Rust SDK, and CLI (vanta-cli) are all ready for local, self-hosted use.",
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
    desc: "The complete API reference for VantaDB \u2014 Python SDK (PyO3), Rust SDK, CLI commands, and HTTP server endpoints with code examples. Full documentation is hosted on GitHub.",
  },
];

const sidebarItems = sections.map(({ id, num, title }) => ({ id, num, title }));

function DocsPage() {
  const docsRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-docs-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(docsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, docsRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p06"
        title={
          <span>
            Start in 60
            <br />
            seconds.
          </span>
        }
        sub="Comprehensive guides, SDK references, and configuration reference for VantaDB \u2014 the embedded database for AI agents."
      />

      <NbSection ref={docsRef} ariaLabel="Documentation">
        <NbSectionHeader
          monoLabel="[DOCUMENTATION]"
          headline="From zero to production in one import."
          sub="Comprehensive guides, SDK references, and configuration for every language and platform."
        />

        <div className="nc-docs-layout">
          <DocsSidebar items={sidebarItems} />

          <div>
            {sections.map((s) => (
              <div key={s.id} id={s.id} className="nc-docs-card nc-docs-part">
                <h3 className="nc-docs-card-title">{s.title}</h3>
                <p className="nc-docs-card-desc">{s.desc}</p>
                <div className="nc-docs-code-frame">
                  <pre className="nc-docs-code-pre">
                    <code>{s.code}</code>
                  </pre>
                </div>
              </div>
            ))}

            <div className="nc-docs-card nc-docs-part">
              <h3 className="nc-docs-card-title">When NOT to use VantaDB</h3>
              <ul className="nc-docs-limits">
                <li>
                  <strong>Multi-node HA / distributed clustering</strong> \u2014 VantaDB is an
                  embedded engine, not a distributed database. No built-in replication, sharding, or
                  consensus.
                </li>
                <li>
                  <strong>Vectors larger than available RAM</strong> \u2014 HNSW index lives in
                  memory. If your dataset exceeds physical RAM, consider a client-server vector
                  database.
                </li>
                <li>
                  <strong>High-availability writes across processes</strong> \u2014 Single-writer
                  semantics. Concurrent writes from multiple processes are not supported.
                </li>
                <li>
                  <strong>Cloud-managed / DBaaS</strong> \u2014 No hosted offering yet. Bring your
                  own infrastructure.
                </li>
                <li>
                  <strong>Real-time streaming / CDC</strong> \u2014 WAL is for crash recovery, not
                  for stream processing or change data capture.
                </li>
              </ul>
            </div>

            <div className="nc-docs-help nc-docs-part">
              <span className="nc-docs-help-label">Need Help?</span>
              <p className="nc-docs-help-text">Join our Discord or open a GitHub discussion.</p>
              <a
                href="https://github.com/ness-e/Vantadb/discussions"
                className="nb-btn nb-btn--ghost nc-docs-help-link"
              >
                DISCUSSIONS
              </a>
            </div>
          </div>
        </div>
      </NbSection>
    </div>
  );
}
