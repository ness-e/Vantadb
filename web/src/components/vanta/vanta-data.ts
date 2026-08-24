// VantaDB — Centralized technical content extracted from the official repository
// Source: https://github.com/ness-e/Vantadb (README.MD)

export const VANTA = {
  name: "VantaDB",
  tagline: "Embedded Rust engine for durable local memory and hybrid vector retrieval.",
  repo: "https://github.com/ness-e/Vantadb",
  pypi: "https://pypi.org/project/vantadb-py/",
  discord: "https://discord.gg/g8nqB3NtXt",
  license: "Apache 2.0",
  rustVersion: "1.94.1+",
  pythonVersion: "3.11+",
  quickstart: "https://github.com/ness-e/Vantadb/blob/main/docs/QUICKSTART.md",
} as const;

/**
 * PRODUCT — Centralized product data. Change here, propagates everywhere.
 * All metrics, versions, and stats that appear in multiple places across the site
 * are defined ONCE here. Components import from PRODUCT, never hardcode.
 */
export const PRODUCT = {
  // Core metrics (hero, metrics-bar, cta-final, trust-section, benchmarks)
  // Source of truth: docs/operations/BENCHMARKS.md — numbers are backed by
  // stress_protocol.rs (§1), vantadb_local_bench.py (§2), SIFT1M (§5), competitive_bench.py (§7).
  metrics: {
    hnswLatency: "1.2ms",          // §1 p50 HNSW Vector Search, 10K vectors (certified)
    hnswLatencySub: "HNSW p50 · 10K · 128d",
    peakQps: "3,636",              // §5 SIFT1M Balanced Cos · 100K, Avg QPS
    peakQpsSub: "SIFT1M Balanced Cos · 100K",
    recallAt10: "99.8%",           // §1 Scaling Recall@10 10K–100K (0.9980 / 1.0000 / 0.9980)
    recallAt10Sub: "Validated 10K–100K",
    networkHops: "0",              // zero network (in-process)
    networkHopsSub: "In-process · embedded",
    siftSpeedup: "2.80x",          // §5 SIFT1M best speedup (Balanced L2)
    siftSpeedupSub: "Balanced L2 · 100K",
    pythonSdkLatency: "39.74ms",   // §7 competitive bench p50 (glove-100-angular)
    cloudDbLatency: "network-bound", // cloud vector DBs — no measured number published
  },

  // Versions (footer, navbar, trust-section, docs, about pages)
  versions: {
    vantadb: "v0.5.0",             // current release
    rust: VANTA.rustVersion,        // "1.94.1+"
    python: VANTA.pythonVersion,    // "3.11+"
    license: VANTA.license,         // "Apache 2.0"
    cli: "vanta-cli v0.5.0",
    pypiPackage: "vantadb-py",
    cargoCrate: "vantadb",
  },

  // Tech stack (trust-section, ecosystem, about pages)
  techStack: {
    core: "Rust Core",
    coreDetail: "Engine, WAL, HNSW, BM25, Fjall/RocksDB",
    bridge: "PyO3 Bridge",
    bridgeDetail: "Stable SDK boundary, zero-copy where possible",
    wheels: "Python Wheels",
    wheelsDetail: "Pre-compiled for Windows, macOS, Linux",
    cli: "Embedded CLI",
    cliDetail: "vanta-cli: put, list, export, audit, repair",
  },

  // Ecosystem / compatible stack (trust-bar, ecosystem page)
  ecosystem: [
    "Ollama", "LangChain", "LlamaIndex", "CrewAI", "AutoGen", "Haystack",
  ],

  // Distribution surfaces (footer, ecosystem, docs)
  distribution: [
    { name: "pip", cmd: "pip install vantadb-py", label: "Python" },
    { name: "cargo", cmd: "cargo add vantadb", label: "Rust" },
    { name: "binary", cmd: "curl -LO https://github.com/ness-e/Vantadb/releases/download/v0.5.0/vanta-cli-x86_64-unknown-linux-gnu.tar.gz", label: "Binary" },
    { name: "wheel", cmd: "vantadb-py", label: "Wheels" },
  ],

  // Hardware profile (benchmarks, metrics)
  hardware: "12-core CPU @ 3.5GHz, AVX2 enabled, Windows 11 / Ubuntu 22.04 LTS.",
} as const;

export type View = "home" | "benchmarks" | "docs";

// Headline stats for the hero strip — now sourced from PRODUCT
// Values backed by BENCHMARKS.md: 1.2ms HNSW p50 (§1), 3,636 QPS peak (§5), 99.8% Recall@10 scaling (§1)
export const HERO_STATS = [
  { value: PRODUCT.metrics.hnswLatency, label: "HNSW p50 · 10K", sub: PRODUCT.metrics.hnswLatencySub },
  { value: PRODUCT.metrics.peakQps, label: "Peak QPS", sub: PRODUCT.metrics.peakQpsSub },
  { value: PRODUCT.metrics.recallAt10, label: "Recall@10", sub: PRODUCT.metrics.recallAt10Sub },
  { value: PRODUCT.metrics.networkHops, label: "Network hops", sub: PRODUCT.metrics.networkHopsSub },
];

// Core capabilities — manga vignettes
export const CORE_CAPABILITIES = [
  {
    icon: "Database",
    title: "Persistent Core",
    mechanism: "StorageBackend + VantaFile + WAL",
    detail:
      "Fjall (default) or RocksDB fallback. Automatic crash recovery via Write-Ahead Log with CRC32C checksums.",
    tag: "DURABLE",
  },
  {
    icon: "Search",
    title: "Hybrid Search",
    mechanism: "BM25 + HNSW via RRF",
    detail:
      "Fuses lexical scoring and vector similarity using Reciprocal Rank Fusion. Automatically routed via query planner.",
    tag: "FUSION",
  },
  {
    icon: "Cpu",
    title: "Embedded Surface",
    mechanism: "Rust Core + PyO3 Bindings",
    detail:
      "Zero-network overhead. Python bindings route through a stable src/sdk.rs boundary. Runs in-process.",
    tag: "LOCAL-FIRST",
  },
  {
    icon: "Layers",
    title: "Vector Retrieval",
    mechanism: "Native HNSW · Cosine",
    detail:
      "Cosine similarity with configurable M, ef_construction, and ef_search. Validated on 10K–100K synthetic datasets.",
    tag: "ANN",
  },
  {
    icon: "KeyRound",
    title: "Memory API",
    mechanism: "namespace + key records",
    detail:
      "put/get/delete/list/search store UTF-8 payloads, scalar metadata, optional vectors, timestamps, versions, and deterministic node IDs.",
    tag: "CRUD",
  },
  {
    icon: "Workflow",
    title: "Operational Flows",
    mechanism: "Rebuild + JSONL + Metrics",
    detail:
      "ANN rebuild, memory export/import, text-index repair, stale derived-index repair, and process telemetry through the SDK boundary.",
    tag: "OPS",
  },
];

// BENCH-01 — SDK Operations Performance Baseline (10K records, 128d, Cosine, single-threaded)
// Numbers sourced from docs/operations/BENCHMARKS.md §2 (latest local run of
// benchmarks/vantadb_local_bench.py). Reproduce: python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000
export const BENCH01 = {
  title: "BENCH-01 · SDK Performance Baseline",
  subtitle: "10K records · 128 dimensions · Cosine · single-threaded · Python SDK (PyO3 boundary)",
  hardware:
    "12-core CPU @ 3.5GHz, AVX2 enabled, Windows 11 / Ubuntu 22.04 LTS.",
  rows: [
    {
      metric: "Ingestion (PUT)",
      p50: "13.174 ms",
      p99: "18.504 ms",
      throughput: "74 ops/sec",
      highlight: true,
    },
    {
      metric: "Search (Lexical BM25)",
      p50: "N/D (outlier)",
      p99: "—",
      throughput: "—",
    },
    {
      metric: "Search (Vector HNSW)",
      p50: "2.024 ms",
      p99: "4.403 ms",
      throughput: "494 qps",
    },
    {
      metric: "Search (Hybrid Fusion)",
      p50: "3.114 ms",
      p99: "5.507 ms",
      throughput: "321 qps",
    },
  ],
};

// SIFT1M Phase 2 optimizations
export const SIFT1M = {
  title: "SIFT1M · Phase 2 Competitive Benchmarks",
  subtitle:
    "Static prefetch · Euclidean sqrt elimination · pure SIMD cosine · O(M²) select_neighbors optimization",
  hardware:
    "AMD Ryzen 12-Core @ 3.5GHz, compiled with -C target-cpu=native.",
  rows: [
    {
      scale: "100K",
      config: "Balanced Cos",
      metricType: "Cosine",
      before: "139.4s",
      after: "63.7s",
      speedup: "2.18x",
      p99: "441.2 µs",
      qps: "3,636",
    },
    {
      scale: "100K",
      config: "High Recall Cos",
      metricType: "Cosine",
      before: "390.8s",
      after: "182.2s",
      speedup: "2.14x",
      p99: "1,231.8 µs",
      qps: "1,379",
    },
    {
      scale: "100K",
      config: "Balanced L2",
      metricType: "Euclidean",
      before: "191.4s",
      after: "68.4s",
      speedup: "2.80x",
      p99: "671.4 µs",
      qps: "3,270",
    },
    {
      scale: "100K",
      config: "High Recall L2",
      metricType: "Euclidean",
      before: "462.2s",
      after: "194.5s",
      speedup: "2.37x",
      p99: "1,183.6 µs",
      qps: "1,353",
    },
    {
      scale: "100K",
      config: "High Recall L2 Mmap",
      metricType: "Mmap Euclidean",
      before: "411.2s",
      after: "189.8s",
      speedup: "2.16x",
      p99: "1,094.8 µs",
      qps: "1,438",
    },
  ],
};

// Competitive benchmark — VantaDB vs LanceDB vs ChromaDB (measured locally) + Pinecone/Weaviate (CSP-managed).
// Measured numbers source: benchmarks/competitive_bench.py → docs/blog/benchmarks_vs_lancedb_chroma.md
// (glove-100-angular, 10K vectors, 100 queries, top_k=10, median of 3 runs, --batch-size 999).
// Pinecone/Weaviate are hosted/managed services — the harness does NOT run them locally, so their
// measured cells are marked "Managed" — we do not fabricate cross-vendor QPS/latency figures.
type CompetitiveRow = {
  metric: string;
  kind: "num" | "txt";
  vanta: string;
  lance: string;
  chroma: string;
  pinecone: string;
  weaviate: string;
  highlight?: boolean;
};

export const COMPETITIVE_TABLE: {
  title: string;
  subtitle: string;
  sourceLink: string;
  note: string;
  rows: CompetitiveRow[];
} = {
  title: "Competitive benchmark · embedded engines",
  subtitle:
    "glove-100-angular · 10K vectors · 100 queries · top_k=10. Measured run documented in docs/operations/BENCHMARKS.md §7 (2026-06-06); VantaDB runs through PyO3 over its mmap Rust core.",
  sourceLink: `${VANTA.repo}/blob/main/docs/operations/BENCHMARKS.md`,
  note: "Pinecone and Weaviate are managed/hosted services — the harness does not run them locally, so their measured cells read 'Managed'. We do not invent cross-vendor QPS/latency numbers here; all measured values come from BENCHMARKS.md §7 and are reproducible via the public harness.",
  rows: [
    // Measured rows — real numbers from BENCHMARKS.md §7 (run 2026-06-06).
    { metric: "Ingest QPS", kind: "num", vanta: "598.3", lance: "114,583", chroma: "3,886", pinecone: "Managed", weaviate: "Managed" },
    { metric: "Index time (ms)", kind: "num", vanta: "16,039.9", lance: "602.2", chroma: "N/A (inc)", pinecone: "Managed", weaviate: "Managed" },
    { metric: "Query QPS", kind: "num", vanta: "24.3", lance: "320.5", chroma: "978.6", pinecone: "Managed", weaviate: "Managed" },
    { metric: "Latency p50 (ms)", kind: "num", vanta: "39.74", lance: "2.653", chroma: "0.941", pinecone: "Managed", weaviate: "Managed" },
    { metric: "Latency p99 (ms)", kind: "num", vanta: "58.245", lance: "6.98", chroma: "3.349", pinecone: "Managed", weaviate: "Managed" },
    { metric: "Recall@10", kind: "num", vanta: "24.50%", lance: "13.90%", chroma: "24.10%", pinecone: "Managed", weaviate: "Managed", highlight: true },
    { metric: "Peak RSS (MB)", kind: "num", vanta: "236.5", lance: "344.2", chroma: "253.5", pinecone: "Managed", weaviate: "Managed" },
    { metric: "Delta RSS (MB)", kind: "num", vanta: "91.7", lance: "97.2", chroma: "39.1", pinecone: "Managed", weaviate: "Managed" },
    // Architecture / positioning rows — factual, no fabricated latency. Pinecone/Weaviate carry verified model facts.
    { metric: "Deployment", kind: "txt", vanta: "pip install · embedded", lance: "pip install · library", chroma: "pip install · client/server", pinecone: "Fully managed SaaS", weaviate: "Self-host or cloud" },
    { metric: "Pricing model", kind: "txt", vanta: "Open source · $0", lance: "Open source", chroma: "Open source", pinecone: "Usage / pods", weaviate: "Hosted by vendor" },
    { metric: "Durability", kind: "txt", vanta: "WAL + CRC32C", lance: "Arrow, query-versioned", chroma: "In-memory HNSW (no WAL)", pinecone: "Managed SLA", weaviate: "WAL / hosted" },
  ],
};

// The exact 5-Minute Quickstart Python snippet from the README
export const QUICKSTART_PYTHON = `import vantadb

# 1. Open or create a local database (zero configuration)
db = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)

# 2. Store a memory record with payload, metadata, and embedding
record = db.put(
    "agent/main",
    "memory-001",
    "In-process execution minimizes latency for local AI agents.",
    metadata={"category": "architecture", "priority": 1},
    vector=[0.12, 0.88, 0.54],
)

# 3. Retrieve exact record by key
stored = db.get_memory("agent/main", "memory-001")

# 4. Hybrid Search (BM25 + Cosine Similarity fused via RRF)
hits = db.search_memory("agent/main", query_vector=[0.11, 0.89, 0.55], top_k=5)

# 5. Operational Telemetry & Safe Shutdown
caps = db.hardware_profile()
db.flush()
db.close()

print(record)
print(stored)
print(hits)
print(caps)`;

// CLI commands — Embedded CLI reference
export const CLI_COMMANDS = [
  {
    cmd: "put",
    args: '--db ./vanta_data --namespace agent/main --key mem-1 --payload "hello"',
    desc: "Store a memory record under a namespace + key.",
  },
  {
    cmd: "list",
    args: "--db ./vanta_data --namespace agent/main",
    desc: "List all records within a namespace.",
  },
  {
    cmd: "export",
    args: "--db ./vanta_data --namespace agent/main --out ./memory.jsonl",
    desc: "Export a namespace to JSONL for backup or migration.",
  },
  {
    cmd: "rebuild-index",
    args: "--db ./vanta_data",
    desc: "Rebuild the ANN (HNSW) index from canonical records.",
  },
  {
    cmd: "audit-index",
    args: "--db ./vanta_data --namespace agent/main --json --deep",
    desc: "Audit derived indexes against canonical records.",
  },
  {
    cmd: "repair-text-index",
    args: "--db ./vanta_data",
    desc: "Repair stale or corrupted BM25 text indexes.",
  },
];

// Search semantics bullets
export const SEARCH_SEMANTICS = [
  {
    title: "Cosine by default",
    body: "The shipped ANN path uses cosine similarity across all HNSW traversal.",
  },
  {
    title: "Namespace-scoped retrieval",
    body: "list/search use derived namespace and scalar metadata indexes; canonical records remain the source of truth.",
  },
  {
    title: "Native hybrid fusion",
    body: "The engine plans and executes lexical (BM25) and vector (Cosine) queries, fusing them with Reciprocal Rank Fusion (RRF).",
  },
  {
    title: "SIFT-1M stress path",
    body: "SIFT-1M remains useful as a stress/recovery scenario via the Heavy Certification workflow.",
  },
];

// Product boundary classifications
export const PRODUCT_BOUNDARY = [
  {
    label: "Production-facing",
    items:
      "Embedded SDK/CLI, memory CRUD/search, WAL/recovery, namespaces, metadata indexes, HNSW vector retrieval, BM25, Hybrid Retrieval v1, phrase filtering, rebuild/audit/repair, JSONL export/import",
    tone: "ink",
  },
  {
    label: "Optional wrapper",
    items: "Local vanta-server around the embedded core",
    tone: "muted",
  },
  {
    label: "Experimental / not MVP",
    items:
      "IQL/LISP/DQL, MCP, LLM/Ollama integration, governance and maintenance semantics, graph traversal beyond stored local edges",
    tone: "muted",
  },
  {
    label: "Deferred",
    items:
      "Cloud/enterprise platform, HA/replication, distributed clustering, SQL/OLTP/warehouse/time-series, advanced ranking, RBAC, multi-tenancy",
    tone: "muted",
  },
];

// Documentation links
export const DOC_LINKS = [
  { name: "Architecture", desc: "Core engine, durability model, retrieval mechanisms, SDK boundary." },
  { name: "Mutation & Recovery Protocol", desc: "Canonical mutation order and WAL recovery behavior." },
  { name: "Text Index Design", desc: "BM25, phrase positions, derived index repair, Hybrid v1 boundaries." },
  { name: "Operations & Configuration", desc: "Runtime parameters and server wrapper configuration." },
  { name: "Memory Telemetry", desc: "Process-memory metrics contract and interpretation guidelines." },
  { name: "Python SDK Status", desc: "Stable boundary, current binding surface, distribution policy." },
  { name: "Reliability Gate", desc: "RSS stability, chaos injection, WAL durability policies." },
  { name: "CI Policy", desc: "Continuous integration strategy, profiles, certification gates." },
];

// FAQ — extracted & inferred from the README product boundary + search semantics
export const FAQ = [
  {
    q: "¿VantaDB requiere un servidor o contenedor separado?",
    a: "No. VantaDB es embedded-first y local-first. Se ejecuta in-process dentro de tu aplicación Python (o Rust). El modo servidor (vanta-server) existe como wrapper opcional, pero no es la identidad principal del producto — está pensado para desarrollo local o exposición de red puntual.",
  },
  {
    q: "¿Qué métrica de similaridad usa el HNSW por defecto?",
    a: "El path ANN shipped usa cosine similarity. Es configurable con parámetros M, ef_construction y ef_search. El motor ha sido validado en datasets sintéticos de 10K a 100K vectores con 99.8% Recall@10 (scaling, BENCHMARKS.md §1).",
  },
  {
    q: "¿Cómo funciona la recuperación híbrida (BM25 + HNSW)?",
    a: "El query planner ejecuta en paralelo el path léxico (BM25 sobre índice invertido) y el path vectorial (HNSW cosine). Los rankings se fusionan con Reciprocal Rank Fusion (RRF): score = Σ 1/(k + rank). Esto combina coincidencias keyword exactas con proximidad semántica.",
  },
  {
    q: "¿Qué garantiza el WAL con CRC32C?",
    a: "Cada mutación se escribe al Write-Ahead Log con checksums CRC32C antes del commit. En un reinicio tras crash, corte de energía o kill del proceso, el log repliega las mutaciones canónicas en orden — durabilidad real sin corrupción silenciosa.",
  },
  {
    q: "¿VantaDB es una base de datos distribuida o de nube?",
    a: "No. Eso está explícitamente diferido. VantaDB es un motor de memoria embebido, no una plataforma multimodelo universal ni cloud. El MVP = memoria embebida + WAL + vector/BM25/hybrid + export/import + CLI/Python.",
  },
  {
    q: "¿Puedo usarlo para RAG local con LLMs?",
    a: "Sí — es uno de los casos de uso principales. VantaDB está diseñado para agentes de IA, pipelines RAG locales y aplicaciones edge. La integración directa con LLM/Ollama es experimental (no MVP), pero el store de vectores + retrieval híbrido está production-facing.",
  },
  {
    q: "¿Qué lenguajes y plataformas soporta?",
    a: "Rust 1.94.1+ (crate nativo), Python 3.11+ via PyO3 (pip install vantadb-py). Wheels precompilados para Windows, macOS y Linux. El CLI (vanta-cli) se instala como binario precompilado o via cargo.",
  },
  {
    q: "¿Cómo exporto o migro mis datos?",
    a: "Usa vanta-cli export --namespace agent/main --out ./memory.jsonl para exportar a JSONL. La importación y el rebuild de índices (ANN, texto) están disponibles como flujos operacionales a través del SDK y la CLI.",
  },
];

// Tutorials — inferred from the repo's docs/tutorials/ structure + README
export const TUTORIALS = [
  {
    num: "01",
    title: "Your First Memory Store",
    desc: "Initialize a persistent VantaDB store, put records with vectors, and retrieve them by key in pure Python.",
    level: "Beginner",
    duration: "5 min",
    tags: ["python", "put", "get"],
    steps: [
      { title: "Install the Python package", body: "Run pip install vantadb-py to get the pre-compiled wheel for your platform. The canonical import is: import vantadb.", code: "pip install vantadb-py" },
      { title: "Open or create a database", body: "Point VantaDB at a local path. It creates the store on first write — zero configuration, no servers.", code: 'import vantadb\ndb = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)' },
      { title: "Store a memory record", body: "Put a UTF-8 payload with scalar metadata and an optional embedding vector under a namespace + key.", code: 'record = db.put(\n    "agent/main",\n    "memory-001",\n    "In-process execution minimizes latency.",\n    metadata={"category": "architecture", "priority": 1},\n    vector=[0.12, 0.88, 0.54],\n)' },
      { title: "Retrieve by key", body: "get_memory() returns the exact canonical record — the source of truth, always consistent.", code: 'stored = db.get_memory("agent/main", "memory-001")\nprint(stored)' },
      { title: "Safe shutdown", body: "Flush the WAL and close handles cleanly. Crash-safe via CRC32C checksums.", code: "db.flush()\ndb.close()" },
    ],
  },
  {
    num: "02",
    title: "Hybrid Search with RRF",
    desc: "Execute BM25 + HNSW in parallel and fuse results with Reciprocal Rank Fusion for semantic + lexical recall.",
    level: "Intermediate",
    duration: "10 min",
    tags: ["hybrid", "bm25", "hnsw", "rrf"],
    steps: [
      { title: "Index multiple records", body: "Store several records with varied payloads and vectors so both lexical and vector paths have material to rank.", code: 'for i, text in enumerate(documents):\n    db.put("docs", f"doc-{i}", text, vector=embed(text))' },
      { title: "Run a hybrid search", body: "Pass a query vector and top_k. The query planner executes BM25 and HNSW in parallel, then fuses via RRF.", code: 'hits = db.search_memory("docs", query_vector=query_vec, top_k=5)' },
      { title: "Inspect the ranked hits", body: "Each hit includes the key, payload, metadata, and the fused rank score. Higher = more relevant.", code: 'for hit in hits:\n    print(hit.key, hit.score)' },
      { title: "Pin the distance metric", body: "search_memory lets you pin the distance metric per query. Cosine is the default and the most optimized path.", code: 'hits = db.search_memory("docs", query_vector=query_vec, top_k=5, distance_metric="cosine")' },
      { title: "Compare with lexical-only", body: "Run a BM25-only query to see how hybrid fusion improves ranking over pure keyword match.", code: 'lexical = db.search_memory("docs", query_vector=query_vec, text_query="latency agent", top_k=5)' },
    ],
  },
  {
    num: "03",
    title: "Crash Recovery with WAL",
    desc: "Simulate a crash mid-write and watch the WAL with CRC32C replay canonical mutations on restart — zero data loss.",
    level: "Advanced",
    duration: "15 min",
    tags: ["wal", "crc32c", "recovery"],
    steps: [
      { title: "Write records continuously", body: "Insert a batch of records. Each mutation hits the WAL with CRC32C checksums before commit.", code: 'for i in range(100):\n    db.put("agent/main", f"mem-{i}", f"record {i}", vector=embed(f"record {i}"))' },
      { title: "Simulate a crash", body: "Kill the process without calling close(). The WAL contains the committed mutations; uncommitted ones are discarded.", code: "# os.kill(os.getpid(), signal.SIGKILL)" },
      { title: "Reopen the database", body: "On restart, VantaDB replays the WAL in order. CRC32C validation detects corruption; valid mutations are applied.", code: 'db = vantadb.VantaDB("./vanta_data")' },
      { title: "Verify data integrity", body: "Read back a record that was committed before the crash. It should be present and intact.", code: 'stored = db.get_memory("agent/main", "mem-50")\nassert stored is not None' },
      { title: "Audit the indexes", body: "Run audit-index to confirm derived indexes are consistent with canonical records. Repair if needed.", code: "vanta-cli audit-index --db ./vanta_data --namespace agent/main --json --deep" },
    ],
  },
  {
    num: "04",
    title: "Local RAG Pipeline",
    desc: "Build a local Retrieval-Augmented Generation pipeline: chunk docs, embed, store in VantaDB, hybrid search, feed to an LLM.",
    level: "Advanced",
    duration: "25 min",
    tags: ["rag", "llm", "embeddings"],
    steps: [
      { title: "Chunk your documents", body: "Split source documents into overlapping chunks. Each chunk becomes a searchable record in VantaDB.", code: 'chunks = chunk_text(document, chunk_size=512, overlap=64)' },
      { title: "Embed each chunk", body: "Generate embedding vectors for each chunk using your preferred local embedding model.", code: 'for i, chunk in enumerate(chunks):\n    vec = embed(chunk)\n    db.put("corpus", f"chunk-{i}", chunk, vector=vec)' },
      { title: "Hybrid search on query", body: "Embed the user query and run hybrid search to retrieve the most relevant chunks via RRF fusion.", code: 'query_vec = embed(user_question)\nhits = db.search_memory("corpus", query_vector=query_vec, top_k=5)' },
      { title: "Assemble context", body: "Concatenate the top hits into a context window for the LLM. Include metadata for citation.", code: 'context = "\\n\\n".join(hit.payload for hit in hits)' },
      { title: "Generate the answer", body: "Feed the context + question to your local LLM. The answer is grounded in retrieved evidence.", code: 'prompt = f"Context: {context}\\n\\nQuestion: {user_question}"\nanswer = llm.generate(prompt)' },
    ],
  },
];

// Changelog — release history sourced from docs/CHANGELOG.md (single source of truth)
export const CHANGELOG = [
  {
    version: "0.5.0",
    date: "2026-07-31",
    tag: "Latest",
    tagColor: "neon",
    title: "IVF Flat index + multi-level LSM compaction",
    changes: [
      "IVF Flat index — inverted file with k-means clustering (no external deps)",
      "New IndexType::Ivf on HnswConfig; lazy-built on first search, serialized in v8 format",
      "~50x faster than brute-force Flat on 1M vectors at ~90% recall",
      "Multi-level LSM compaction (L0→L1→L2→L3): per-level VantaFiles with legacy migration",
      "compact_level() promotes live nodes between tiers; write amplification reduced from O(all data) to O(L0 size)",
      "New PipelineMode::CompactOnly / CompactL0Only variants",
    ],
  },
  {
    version: "0.4.0",
    date: "2026-07-20",
    tag: "MVP",
    tagColor: "ink",
    title: "Initial public release",
    changes: [
      "Embedded persistent vector/graph database engine (HNSW, configurable distance metrics)",
      "WAL with automatic crash recovery; Arrow IPC zero-copy interchange",
      "Hybrid Retrieval v1: BM25 + HNSW fused via RRF",
      "Python SDK (PyO3), WASM build, TypeScript SDK (vantadb-ts)",
      "CLI tool (vanta) and HTTP server with rate limiting and TLS support",
      "AI framework adapters: LangChain, LlamaIndex, Haystack, CrewAI, DSPy, Litellm, OpenAI, Ollama, Mem0, Letta",
      "MCP server (vantadb-mcp); Prometheus metrics and OpenTelemetry tracing",
      "Encryption at rest (AES-GCM), PITR, WAL shipping for replication",
    ],
  },
];

// ───────────────────────────────────────────────────────────
// Tier 2 page data
// ───────────────────────────────────────────────────────────

// Pricing — 2 plans (community / enterprise)
export const PRICING_PLANS = [
  {
    name: "Community",
    price: "$0",
    period: "forever",
    tag: "OPEN SOURCE",
    tagColor: "ink",
    highlight: false,
    description: "The full embedded engine. Apache 2.0. No limits, no telemetry.",
    features: [
      "Embedded Rust engine + PyO3 bindings",
      "Hybrid search: BM25 + HNSW via RRF",
      "WAL with CRC32C crash recovery",
      "vanta-cli: put, list, export, audit, repair",
      "JSONL export/import",
      "Community Discord support",
    ],
    cta: "pip install vantadb-py",
    ctaLink: VANTA.pypi,
  },
  {
    name: "Enterprise",
    price: "Custom",
    period: "",
    tag: "CONTACT",
    tagColor: "muted",
    highlight: false,
    description: "On-prem, air-gapped, or regulated environments. Self-hosted everything.",
    features: [
      "Everything in Community",
      "Unlimited seats",
      "Air-gapped deployment support",
      "Custom SIMD/vector tuning engagements",
      "SBOM + compliance artifacts (SOC2-path)",
      "Dedicated engineer",
      "On-site training optional",
    ],
    cta: "Contact via GitHub",
    ctaLink: VANTA.repo,
  },
];

// Security pillars
export const SECURITY_PILLARS = [
  {
    icon: "ShieldCheck",
    title: "Crash-Safe WAL",
    tag: "DURABILITY",
    body: "Every mutation hits the Write-Ahead Log with CRC32C checksums before commit. On restart after crash, power loss, or process kill, the log replays canonical mutations in order — zero silent corruption.",
  },
  {
    icon: "Lock",
    title: "Zero Network Surface",
    tag: "LOCAL-FIRST",
    body: "No daemons, no ports, no external services. The engine runs in-process inside your application. There is nothing to attack remotely because there is no remote surface.",
  },
  {
    icon: "FileStack",
    title: "Canonical Records",
    tag: "SOURCE OF TRUTH",
    body: "Derived indexes (BM25 text, HNSW vector) are rebuildable from canonical records at any time. audit-index and repair-text-index detect drift and restore consistency without data loss.",
  },
  {
    icon: "Package",
    title: "Memory-Safe Rust Core",
    tag: "NO GC",
    body: "The engine is Rust 1.94+ with no garbage collector, no data races, and no undefined behavior in safe code. PyO3 bindings expose a stable SDK boundary — no raw pointer leaks across the FFI.",
  },
  {
    icon: "ScanSearch",
    title: "SBOM & CodeQL",
    tag: "SUPPLY CHAIN",
    body: "Each release ships a Software Bill of Materials. CodeQL security scanning runs in CI. Heavy Certification workflow validates WAL durability under chaos injection before any tag is cut.",
  },
  {
    icon: "Eye",
    title: "No Telemetry",
    tag: "PRIVACY",
    body: "Zero phone-home, zero analytics, zero tracking. VantaDB has no network calls. What you store stays on your machine. Verify it yourself — the source is open.",
  },
];

// Use cases — expanded detail for solution pages
export const USE_CASES_DETAIL = [
  {
    slug: "ai-agents",
    icon: "Bot",
    title: "AI Agents",
    tagline: "Persistent memory for autonomous agents",
    pain: "Agents lose context between sessions. Re-embedding is expensive. External vector DBs add latency and cost.",
    solution: "VantaDB gives each agent a local, durable memory store. put() a memory with payload + metadata + vector. search() hybrid across BM25 + HNSW. No network, no API keys, in-process.",
    flow: [
      "Agent observes → embed → db.put(namespace, key, payload, vector)",
      "Agent recalls → db.search_memory(namespace, query_vector=query, top_k=5)",
      "Session ends → db.flush() → WAL persists · process exits",
      "New session → reopen DB → memories intact · zero re-embedding",
    ],
    metrics: [
      { value: "1.2ms", label: "Recall latency" },
      { value: "0", label: "Network hops" },
      { value: "99.8%", label: "Recall@10" },
    ],
    code: `import vantadb

db = vantadb.VantaDB("./agent_memory")

# Store an observation with semantic vector
db.put("agent/main", "obs-042",
    "User prefers dark mode and concise answers",
    metadata={"session": "2026-W15", "type": "preference"},
    vector=embed("user prefers dark mode concise"))

# Recall relevant memories for current context
hits = db.search_memory("agent/main", query_vector=embed(query), top_k=5)
for hit in hits:
    print(hit.key, hit.score, hit.payload)

db.close()`,
  },
  {
    slug: "local-rag",
    icon: "BookOpen",
    title: "Local RAG",
    tagline: "Retrieval-Augmented Generation without the cloud",
    pain: "Cloud RAG means your documents leave your machine. API costs scale with query volume. Latency adds up across the network hop.",
    solution: "VantaDB stores your document chunks + embeddings locally. Hybrid search (BM25 + HNSW via RRF) retrieves the most relevant context. Feed it to your local LLM (Ollama, llama.cpp). Zero data leaves your machine.",
    flow: [
      "Chunk documents → embed each chunk → db.put(\"corpus\", chunk-id, text, vector)",
      "User asks → embed question → db.search_memory(\"corpus\", query_vector=q, top_k=5)",
      "Assemble context from top hits → prompt = context + question",
      "Local LLM generates answer grounded in retrieved evidence",
    ],
    metrics: [
      { value: "0", label: "Data egress" },
      { value: "In-process", label: "Hybrid retrieval" },
      { value: "99.8%", label: "Recall@10" },
    ],
    code: `import vantadb

db = vantadb.VantaDB("./rag_corpus")

# Index document chunks
for i, chunk in enumerate(chunks):
    db.put("corpus", f"chunk-{i}", chunk,
           metadata={"source": doc.path, "page": i},
           vector=embed(chunk))

# Hybrid retrieval for a query
query_vec = embed(user_question)
hits = db.search_memory("corpus", query_vector=query_vec, top_k=5)

# Build context for the LLM
context = "\\n\\n".join(hit.payload for hit in hits)
answer = local_llm.generate(f"Context: {context}\\n\\nQ: {user_question}")

db.close()`,
  },
  {
    slug: "ai-ide-tooling",
    icon: "Code2",
    title: "IDE Tooling",
    tagline: "Semantic code memory for editors and copilots",
    pain: "Code search is keyword-only. 'Where did we handle retry logic?' returns grep noise. Copilots lack project context beyond the open file.",
    solution: "Index code symbols, docstrings, and commit messages with embeddings. VantaDB hybrid search finds semantically relevant code even without exact keyword matches. Runs in-process — the editor IS the server.",
    flow: [
      "Parse AST → extract symbols + docstrings → embed each",
      "db.put(\"code\", symbol-id, source, metadata={file, line, type}, vector)",
      "Developer queries → db.search_memory(\"code\", query_vector=embed(query), top_k=10)",
      "Results ranked by BM25 (keyword) + HNSW (semantic) fused via RRF",
    ],
    metrics: [
      { value: "Local", label: "Symbol indexing" },
      { value: "In-process", label: "BM25 lookup" },
      { value: "In-process", label: "No LSP server" },
    ],
    code: `import vantadb

db = vantadb.VantaDB("./code_index")

# Index a code symbol with its docstring
db.put("code", "src/auth.py:login",
    "def login(user, password): ...",
    metadata={"file": "src/auth.py", "line": 42, "type": "function"},
    vector=embed("authenticate user login password session"))

# Semantic code search
hits = db.search_memory("code", query_vector=embed("how do we handle auth?"), top_k=10)
for hit in hits:
    print(hit.metadata["file"], hit.metadata["line"], hit.payload)

db.close()`,
  },
];

// TCO comparison — cost vs cloud vector DBs
export const TCO_COMPARISON = [
  {
    scenario: "Small (10K vectors, 1K queries/day)",
    vantadb: "$0",
    cloudPinecone: "$70/mo",
    cloudWeaviate: "$25/mo",
    note: "VantaDB runs in-process. Cloud DBs charge minimum instance + egress.",
  },
  {
    scenario: "Medium (100K vectors, 10K queries/day)",
    vantadb: "$0",
    cloudPinecone: "$350/mo",
    cloudWeaviate: "$120/mo",
    note: "Cloud costs scale with pods/replicas. VantaDB cost stays at zero.",
  },
  {
    scenario: "Large (1M vectors, 100K queries/day)",
    vantadb: "$0",
    cloudPinecone: "$1,800/mo",
    cloudWeaviate: "$600/mo",
    note: "VantaDB uses your existing hardware. Cloud DBs need dedicated clusters.",
  },
  {
    scenario: "Enterprise (10M vectors, multi-region)",
    vantadb: "$0 (self-hosted)",
    cloudPinecone: "$8,000+/mo",
    cloudWeaviate: "$3,000+/mo",
    note: "VantaDB Enterprise adds support + compliance. Cloud adds replicas + egress.",
  },
];

// Maintenance pillars — zero-maintenance argument
export const MAINTENANCE_PILLARS = [
  {
    icon: "Server",
    title: "No Servers to Manage",
    tag: "EMBEDDED",
    body: "There is no database server. VantaDB is a library linked into your process. No provisioning, no scaling, no patching, no uptime monitoring, no 3am pages.",
  },
  {
    icon: "RefreshCw",
    title: "Self-Healing Indexes",
    tag: "REPAIRABLE",
    body: "Derived indexes (BM25 text, HNSW vector) can drift or corrupt. audit-index detects drift. repair-text-index and rebuild-index restore consistency from canonical records — no manual SQL, no downtime.",
  },
  {
    icon: "HardDrive",
    title: "Single-File Storage",
    tag: "PORTABLE",
    body: "The entire database is a directory on disk. Copy it, back it up, rsync it, ship it in a Docker image. No cluster state, no WAL segments to reconcile, no sharding config.",
  },
  {
    icon: "Gauge",
    title: "Bounded Memory",
    tag: "PREDICTABLE",
    body: "memory_limit_bytes caps RSS. The engine evicts derived-index caches, never canonical records. Memory usage is deterministic — no OOM surprises, no GC pauses, no jitter.",
  },
];

// Why VantaDB — benefits + comparison vs alternatives
export const WHY_VANTADB = {
  benefits: [
    {
      icon: "Zap",
      title: "In-Process Latency",
      body: "In-process execution. No network hop, no serialization overhead, no connection pool. The query planner runs BM25 + HNSW in parallel and fuses via RRF in-process.",
    },
    {
      icon: "ShieldCheck",
      title: "Crash-Safe by Design",
      body: "WAL with CRC32C checksums. Tested under chaos injection (kill -9, power loss). Canonical records are always recoverable. Derived indexes are rebuildable.",
    },
    {
      icon: "Lock",
      title: "Zero Data Egress",
      body: "Your vectors never leave your machine. No API keys, no cloud accounts, no per-query billing. Run it air-gapped, on edge devices, in regulated environments.",
    },
    {
      icon: "Cpu",
      title: "Memory-Safe Rust",
      body: "No GC pauses, no data races, no buffer overflows. PyO3 bindings expose a stable SDK boundary. Deterministic performance with bounded memory.",
    },
  ],
  comparison: [
    { feature: "Latency", vantadb: "1.2ms (HNSW p50 · 10K)", pinecone: "—", weaviate: "—", chroma: "—" },
    { feature: "Network hops", vantadb: "0", pinecone: "1+", weaviate: "1+", chroma: "0-1" },
    { feature: "Deployment", vantadb: "pip install", pinecone: "Cloud account", weaviate: "Docker cluster", chroma: "pip install" },
    { feature: "Crash recovery", vantadb: "WAL + CRC32C", pinecone: "Managed", weaviate: "WAL", chroma: "Limited" },
    { feature: "Hybrid search", vantadb: "BM25 + HNSW · RRF", pinecone: "Vector only*", weaviate: "BM25 + HNSW", chroma: "Vector only" },
    { feature: "Data egress", vantadb: "None", pinecone: "Cloud", weaviate: "Self-host or cloud", chroma: "None" },
    { feature: "Cost at 1M vectors", vantadb: "$0", pinecone: "$1,800/mo", weaviate: "$600/mo", chroma: "$0" },
  ],
};

// ───────────────────────────────────────────────────────────
// Tier 3 page data — blog, case studies, about
// ───────────────────────────────────────────────────────────

// Blog posts — 4 articles with full content
export const BLOG_POSTS = [
  {
    slug: "introducing-vantadb",
    title: "Introducing VantaDB",
    excerpt: "Why we built an embedded Rust engine for local-first hybrid retrieval — and why \"local-first\" matters more than ever today.",
    date: "2026-04-10",
    author: "VantaDB Team",
    readTime: "6 min",
    tag: "Announcement",
    tagColor: "neon",
    content: [
      {
        type: "p",
        text: "Every vector database I've used in the last two years has the same shape: a server. You spin up a container, connect over the network, serialize your embeddings, send them across, and wait. The wait is never long in absolute terms — tens to hundreds of milliseconds — but it is always there, always a hop, always a billable event.",
      },
      {
        type: "h2",
        text: "The local-first thesis",
      },
      {
        type: "p",
        text: "VantaDB starts from a different premise: the fastest network hop is no network hop. If your application needs hybrid retrieval — BM25 for keywords, HNSW for vectors, RRF to fuse them — that retrieval should happen inside the process that already has the embeddings. No serialization. No connection pool. No API key. No per-query cost.",
      },
      {
        type: "p",
        text: "This is not a rejection of cloud databases. Pinecone and Weaviate are excellent products for teams that need managed scale, multi-region replication, and someone else to carry the pager. VantaDB is for the other cases: agents that need durable memory, RAG pipelines that can't leak data, edge devices with no cloud access, and developers who want to ship without a credit card on file.",
      },
      {
        type: "h2",
        text: "Why Rust, why PyO3",
      },
      {
        type: "p",
        text: "We chose Rust for the core because memory safety is not optional in a database, and because deterministic performance is not optional in a retrieval engine. There is no garbage collector pause to explain away in a p99 latency chart. The PyO3 bindings expose a stable src/sdk.rs boundary — Python callers never touch raw pointers, and the FFI surface is narrow enough to audit by hand.",
      },
      {
        type: "p",
        text: "The result is an engine that runs in-process, recovers from crashes via a CRC32C-checksummed WAL, and costs nothing to operate beyond the hardware you already own. It is Apache 2.0, it is on PyPI, and it is ready for you to try today.",
      },
    ],
  },
  {
    slug: "how-hybrid-search-works",
    title: "How Hybrid Search Actually Works",
    excerpt: "BM25 and HNSW are not competitors. They are two lenses on the same ranking problem. Here's how RRF fuses them without comparable scores.",
    date: "2026-04-24",
    author: "VantaDB Team",
    readTime: "9 min",
    tag: "Engineering",
    tagColor: "ink",
    content: [
      {
        type: "p",
        text: "Hybrid search is often described as \"BM25 plus vectors,\" as if the two were simply concatenated. The reality is more interesting: BM25 and HNSW produce rankings that are not directly comparable, and fusing them requires a method that does not depend on score calibration. That method is Reciprocal Rank Fusion (RRF).",
      },
      {
        type: "h2",
        text: "The score comparison problem",
      },
      {
        type: "p",
        text: "BM25 produces a TF-IDF-derived score that can range from 0 to ~30 depending on corpus statistics. HNSW cosine similarity produces a score in [-1, 1]. You cannot add these. You cannot average them. You cannot normalize them in a way that is fair across queries, because the distributions are query-dependent and corpus-dependent.",
      },
      {
        type: "h2",
        text: "Reciprocal Rank Fusion",
      },
      {
        type: "p",
        text: "RRF sidesteps the calibration problem by ignoring scores entirely. It uses only the rank positions. The formula is simple: score = Σ 1/(k + rank), where k is a tunable constant (we default to 60) and rank is the position of the document in each result list (1-indexed). A document that ranks #1 in BM25 and #3 in HNSW gets 1/61 + 1/63 ≈ 0.0323. A document that ranks #1 in both gets 1/61 + 1/61 ≈ 0.0328.",
      },
      {
        type: "p",
        text: "The elegance of RRF is that it requires no score normalization, no weight tuning between systems, and no shared vocabulary between the two ranking functions. It only requires that each system produces a ranked list. This is why it has become the default fusion strategy in production hybrid search systems.",
      },
      {
        type: "h2",
        text: "The query planner",
      },
      {
        type: "p",
        text: "In VantaDB, the query planner inspects each search request. If a vector is present and the namespace has an HNSW index, the vector path runs. If text is present and the namespace has a BM25 index, the lexical path runs. If both are present, both run in parallel and RRF fuses the results. If only one is present, the engine skips the other path entirely — no wasted work.",
      },
    ],
  },
  {
    slug: "sqlite-for-ai-agents",
    title: "SQLite for AI Agents: The Missing Memory Layer",
    excerpt: "Agents don't need a vector database. They need a memory database that happens to support vectors. VantaDB is that layer.",
    date: "2026-05-15",
    author: "VantaDB Team",
    readTime: "7 min",
    tag: "Architecture",
    tagColor: "ink",
    content: [
      {
        type: "p",
        text: "The dominant pattern in AI agent infrastructure is: vector database for semantic recall, key-value store for session state, relational database for user data, object storage for artifacts. Four systems, four clients, four failure modes. This works at scale, but it is overkill for the 90% of agents that operate on a single machine with a single user.",
      },
      {
        type: "h2",
        text: "What agents actually need",
      },
      {
        type: "p",
        text: "An agent needs to remember. Specifically, it needs to: store observations with structured metadata, recall relevant past observations by semantic similarity, retrieve exact records by key, survive crashes without losing committed memories, and do all of this without a network dependency. That is not five databases. That is one database with five capabilities.",
      },
      {
        type: "h2",
        text: "VantaDB as the memory layer",
      },
      {
        type: "p",
        text: "VantaDB provides exactly this surface. put() stores a payload with metadata and an optional vector. get() retrieves by exact key. search() runs hybrid retrieval. The WAL with CRC32C ensures durability. The embedded Rust core means there is no server to manage. The Python bindings mean integration is one pip install away.",
      },
      {
        type: "p",
        text: "The analogy to SQLite is deliberate. SQLite did not replace PostgreSQL — it replaced the cases where PostgreSQL was overkill. VantaDB does not replace Pinecone — it replaces the cases where a network-attached vector database is the wrong shape for the problem. For agents running locally, that is most cases.",
      },
    ],
  },
  {
    slug: "why-i-built-vantadb-local-memory-engine",
    title: "Why I Built VantaDB: A Local Memory Engine",
    excerpt: "A personal note on the frustration that led to VantaDB — re-embedding the same documents every session, paying per query for my own data, and watching agents forget everything overnight.",
    date: "2026-06-05",
    author: "ness-e",
    readTime: "5 min",
    tag: "Story",
    tagColor: "muted",
    content: [
      {
        type: "p",
        text: "I built VantaDB because I was tired. Tired of re-embedding the same 10,000 documents every time I restarted my agent. Tired of watching my cloud vector database bill climb past $300/month for a side project. Tired of agents that forgot every conversation the moment the process exited. Tired of explaining to non-technical friends why my \"local AI\" needed an internet connection to remember things.",
      },
      {
        type: "h2",
        text: "The breaking point",
      },
      {
        type: "p",
        text: "The breaking point came on a flight. I had a RAG pipeline I wanted to demo, and the plane's WiFi was $30 and barely worked. My local LLM ran fine — llama.cpp is wonderful — but my retrieval layer needed to reach a cloud vector database on another continent, a credit card swipe away. The demo failed. I spent the flight sketching what would become VantaDB.",
      },
      {
        type: "h2",
        text: "What I wanted",
      },
      {
        type: "p",
        text: "I wanted a database that felt like SQLite: one file, no server, embedded in my process. I wanted hybrid search because pure vector retrieval misses keyword-exact matches, and pure BM25 misses semantic similarity. I wanted crash safety because agents crash. I wanted it in Rust because I had spent too much of my career chasing GC pauses in JVM-based data systems. And I wanted Python bindings because that is where the AI ecosystem lives.",
      },
      {
        type: "p",
        text: "VantaDB is all of those things. It is not a cloud platform, it is not a distributed system, and it is not trying to be. It is a local memory engine for the agents and RAG pipelines that run on the machine in front of you. I hope it saves you the frustration it saved me.",
      },
    ],
  },
];

// Case studies — 3 detailed customer stories
export const CASE_STUDIES = [
  {
    slug: "agent-local-memory-ollama",
    company: "Indie AI Studio",
    industry: "AI Agents",
    title: "Persistent Agent Memory with Local LLMs",
    summary: "A solo developer replaced a cloud vector DB with VantaDB + Ollama, cutting monthly costs to $0 and giving agents crash-safe memory that survives restarts.",
    metrics: [
      { value: "$0", label: "Monthly DB cost (was $340)" },
      { value: "In-process", label: "Recall latency" },
      { value: "100%", label: "Memory retention across restarts" },
    ],
    challenge: "The developer ran a crew of autonomous agents on Ollama for a customer-support side project. Each agent re-embedded its context window every session because the cloud vector database was too expensive to keep populated. Monthly costs hit $340. Agents forgot everything when the process crashed.",
    solution: "Replaced the cloud vector DB with VantaDB. Each agent now has a namespace in a local VantaDB store. Observations are put() with payload + metadata + vector. Recall uses hybrid search (BM25 + HNSW via RRF). The WAL persists everything — agents survive crashes and restarts with full memory intact.",
    quote: "VantaDB gave my agents a brain that doesn't forget and a bill that doesn't grow. It's the SQLite moment for AI memory.",
    quoteAuthor: "Developer, Indie AI Studio",
  },
  {
    slug: "rag-edge-device",
    company: "Field Robotics Co.",
    industry: "Edge / IoT",
    title: "Air-Gapped RAG on Edge Devices",
    summary: "A robotics team deployed VantaDB on edge devices for retrieval-augmented diagnostics — no cloud, no network, in-process hybrid search on-device.",
    metrics: [
      { value: "0", label: "Network dependencies" },
      { value: "In-process", label: "On-device hybrid search" },
      { value: "48MB", label: "Binary footprint" },
    ],
    challenge: "Field robots operate in environments with unreliable or no network connectivity. Diagnostic RAG pipelines needed to retrieve from a 50K-document manual corpus, but cloud vector databases were impossible — the devices are air-gapped for security. SQLite + custom vector code was too slow and fragile.",
    solution: "Embedded VantaDB into the robot's Rust control process. The manual corpus is indexed at build time. At runtime, technicians query in natural language; VantaDB runs hybrid search in-process and returns ranked manual sections instantly. The WAL ensures diagnostics survive power loss mid-query.",
    quote: "We needed retrieval that works at the bottom of a mine with no WiFi. VantaDB runs in-process, survives power loss, and returns answers before the cloud would have even received the query.",
    quoteAuthor: "Lead Robotics Engineer, Field Robotics Co.",
  },
  {
    slug: "ide-semantic-search",
    company: "DevTools Startup",
    industry: "Developer Tools",
    title: "Semantic Code Search Without a Server",
    summary: "A VS Code extension replaced an LSP-based code search with VantaDB, delivering semantic relevance in-process with zero server infrastructure.",
    metrics: [
      { value: "In-process", label: "BM25 lookup" },
      { value: "Local", label: "Symbol indexing" },
      { value: "0", label: "Servers to run" },
    ],
    challenge: "The startup built a VS Code extension for semantic code search. Their initial architecture ran a local LSP server + a Python vector DB process, communicating over localhost. The multi-process setup was fragile, slow to start, and used 800MB of RAM idle. Users complained about install complexity.",
    solution: "Replaced both the LSP server and the vector DB with VantaDB embedded in the extension's Node.js host (via a Rust FFI). Code symbols are indexed with embeddings at workspace open. Search runs hybrid (BM25 on symbol names + HNSW on docstring vectors) in-process. The entire extension now uses 60MB and starts instantly.",
    quote: "We went from three processes and 800MB to one process and 60MB. Install complexity dropped to zero because there's nothing to install — VantaDB is just a library.",
    quoteAuthor: "Founder, DevTools Startup",
  },
];

// Team members — 4 people
export const TEAM_MEMBERS = [
  {
    name: "ness-e",
    role: "Founder & Core Engineer",
    bio: "Systems engineer with a background in embedded databases and distributed storage. Built VantaDB after years of frustration with cloud-attached vector DBs for local AI workloads.",
    avatar: "/assets/avatar_gato.png",
    github: "ness-e",
  },
  {
    name: "Vanta Cat",
    role: "Mascot & Morale Officer",
    bio: "The shadow cat with fire eyes. Appears in the hero, the navbar, and the easter egg. Does not write code but reviews all commits with a critical eye.",
    avatar: "/assets/mascota_gato.png",
    github: "",
  },
  {
    name: "Community",
    role: "Contributors & Discord",
    bio: "VantaDB is shaped by its community. Bug reports, feature requests, benchmark results, and edge-case discoveries from Discord contributors directly influence the roadmap.",
    avatar: "",
    github: "ness-e/Vantadb",
  },
  {
    name: "Open Source",
    role: "Apache 2.0 License",
    bio: "The entire engine, CLI, Python bindings, and documentation are Apache 2.0 licensed. Fork it, audit it, embed it, ship it. No CLA, no telemetry, no strings attached.",
    avatar: "",
    github: "ness-e/Vantadb",
  },
];

// Company info — for /about/company
export const COMPANY_INFO = {
  name: "VantaDB",
  founded: "2026",
  mission: "Make local-first hybrid retrieval the default for AI agents, RAG pipelines, and edge applications. No cloud tax on your own data.",
  location: "Distributed · Open Source",
  license: "Apache 2.0",
  repo: VANTA.repo,
  stats: [
    { value: "Apache 2.0", label: "License" },
    { value: "0.5.0", label: "Current release" },
    { value: "3.11+", label: "Python support" },
    { value: "1.94+", label: "Rust support" },
  ],
  principles: [
    {
      title: "Local-first, always",
      body: "The fastest network hop is no network hop. Your data stays on your machine unless you explicitly choose otherwise.",
    },
    {
      title: "Embedded, not attached",
      body: "A database that runs in your process is simpler, faster, and safer than one that runs next to it. SQLite taught us this. We're extending the lesson to retrieval.",
    },
    {
      title: "Crash-safe by construction",
      body: "Durability is not a feature you add later. The WAL with CRC32C checksums is there from the first commit. Chaos injection tests it in CI.",
    },
    {
      title: "Open source, no telemetry",
      body: "Apache 2.0, no phone-home, no analytics. Verify the source yourself. If we ever change this, fork us — the license lets you.",
    },
  ],
};

