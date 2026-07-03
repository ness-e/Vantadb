import { createFileRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/integrations")({
  head: () => ({
    meta: [
      { title: "VantaDB — Ecosystem & Integrations" },
      {
        name: "description",
        content:
          "Integrate VantaDB vector store and persistent memory tools natively inside LangChain, LlamaIndex and MCP runtimes.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/integrations" }],
  }),
  component: IntegrationsPage,
});

// ── Integration Data ─────────────────────────────────────────────────────────
const INTEGRATIONS = [
  {
    id: "langchain",
    label: "LangChain",
    tag: "vantadb-langchain",
    category: "RAG / Chains",
    desc: "Implement VantaDB as a lightweight persistent vectorstore within your LangChain document pipelines.",
    code: `from vantadb_langchain import VantaDBVectorStore
from langchain_openai import OpenAIEmbeddings

embeddings = OpenAIEmbeddings()
vector_store = VantaDBVectorStore(
    path="./langchain_data",
    embedding_function=embeddings,
    namespace="agent_history"
)

# Add records to local db
vector_store.add_texts(
    texts=["User prefers async python workflows"],
    metadatas=[{"priority": "high"}]
)

# Similarity query execution
docs = vector_store.similarity_search("python developer", k=3)`,
  },
  {
    id: "llamaindex",
    label: "LlamaIndex",
    tag: "llama-index-vector-stores-vantadb",
    category: "LLM Indexes",
    desc: "Connect LLM index pipelines directly to VantaDB. Leverage in-process traversals for rapid structured memory pipelines.",
    code: `from llama_index.core import SimpleDirectoryReader, VectorStoreIndex
from llama_index.vector_stores.vantadb import VantaDBVectorStore
from llama_index.core.storage.storage_context import StorageContext

# Initialize in-process store
vector_store = VantaDBVectorStore(path="./llama_data")
storage_context = StorageContext.from_defaults(vector_store=vector_store)

# Load data and build index
documents = SimpleDirectoryReader("./data").load_data()
index = VectorStoreIndex.from_documents(
    documents, storage_context=storage_context
)

# Query vector index
query_engine = index.as_query_engine()
response = query_engine.query("What is the system limits?")`,
  },
  {
    id: "mcp",
    label: "MCP Server",
    tag: "mcp-server-vantadb",
    category: "Agent Protocol",
    desc: "Expose local vector storage utilities and namespace keys dynamically to Claude Desktop or any MCP client runtime.",
    code: `{
  "mcpServers": {
    "vantadb-memory": {
      "command": "python",
      "args": ["-m", "vantadb_mcp_server", "--path", "./mcp_agent_memory"],
      "env": {
        "VANTA_DEFAULT_NAMESPACE": "agent_context"
      }
    }
  }
}`,
  },
  {
    id: "python",
    label: "Python SDK",
    tag: "vantadb-py",
    category: "Native Bindings",
    desc: "Direct Rust bindings via PyO3. Absolute speed with zero TCP networking latency floors.",
    code: `import vantadb_py as vanta

# Open database path
db = vanta.VantaDB("./vanta_memory")

# Store structured memory
db.put(
    "memories",
    "user-pref",
    "Developer is building high-end interfaces",
    vector=[0.15, 0.82, 0.44]
)

# Multi-modal retrieval
hits = db.search_memory(
    "memories",
    query_vector=[0.14, 0.85, 0.40],
    top_k=1
)`,
  },
];

// ── Additional ecosystem entries (non-interactive grid) ─────────────────────
const ECOSYSTEM_GRID = [
  { name: "FastAPI", tag: "REST API" },
  { name: "Jupyter", tag: "Notebooks" },
  { name: "Transformers", tag: "HuggingFace" },
  { name: "Sentence-Transformers", tag: "Embeddings" },
  { name: "PyO3", tag: "FFI" },
  { name: "DuckDB", tag: "Analytics compat." },
  { name: "Rust", tag: "Native crate" },
  { name: "OpenAI", tag: "Embedding API" },
  { name: "Anthropic Claude", tag: "Agent memory" },
];

function IntegrationsPage() {
  const [selectedId, setSelectedId] = useState<string>("langchain");
  const [copied, setCopied] = useState(false);

  const active = useMemo(
    () => INTEGRATIONS.find((i) => i.id === selectedId) || INTEGRATIONS[0],
    [selectedId],
  );

  const handleCopy = () => {
    navigator.clipboard?.writeText(active.code).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  };

  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="03"
        eyebrow="Ecosystem & Integrations"
        title={
          <span>
            Fits your stack.
            <br />
            Not the other way.
          </span>
        }
        sub="Connect VantaDB directly to the frameworks you already know. Built for first-class Python and Rust ecosystems."
      />

      <main className="engine-main">
        {/* Section 1: Framework Selector + Code Panel */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Framework Connectors</span>

          <div className="swiss-grid-12" style={{ alignItems: "start", marginTop: "3rem" }}>
            {/* Selector */}
            <div className="col-span-4">
              <div
                style={{
                  border: "1px solid var(--border)",
                  display: "grid",
                  gridTemplateColumns: "1fr 1fr",
                  gap: "1px",
                  background: "var(--border)",
                }}
              >
                {INTEGRATIONS.map((int) => (
                  <button
                    key={int.id}
                    onClick={() => setSelectedId(int.id)}
                    style={{
                      background:
                        selectedId === int.id ? "var(--surface-raised)" : "var(--background)",
                      border: "none",
                      borderLeft:
                        selectedId === int.id ? "2px solid var(--amber)" : "2px solid transparent",
                      padding: "1.5rem 1.25rem",
                      cursor: "pointer",
                      textAlign: "left",
                      transition: "background-color 150ms var(--ease-cut)",
                    }}
                  >
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.55rem",
                        textTransform: "uppercase",
                        letterSpacing: "0.08em",
                        color: selectedId === int.id ? "var(--amber)" : "var(--steel)",
                        marginBottom: "0.25rem",
                      }}
                    >
                      {int.category}
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "0.9rem",
                        fontWeight: 700,
                        color: selectedId === int.id ? "var(--foreground)" : "var(--muted)",
                      }}
                    >
                      {int.label}
                    </div>
                  </button>
                ))}
              </div>

              <div
                style={{
                  marginTop: "2rem",
                  padding: "1.5rem",
                  border: "1px solid var(--border)",
                  background: "var(--surface)",
                }}
              >
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                    marginBottom: "0.5rem",
                  }}
                >
                  {active.tag}
                </div>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.82rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {active.desc}
                </p>
              </div>
            </div>

            {/* Code Panel */}
            <div className="col-span-8">
              <div
                style={{
                  border: "1px solid var(--border)",
                  background: "var(--block-dark-bg)",
                  position: "relative",
                }}
              >
                {/* Terminal header */}
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: "0.75rem 1.25rem",
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
                    {active.label} — {active.tag}
                  </span>
                  <button
                    onClick={handleCopy}
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.6rem",
                      fontWeight: 700,
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                      color: copied ? "var(--amber)" : "var(--block-dark-muted)",
                      background: "none",
                      border: "1px solid var(--block-dark-border)",
                      padding: "0.3rem 0.7rem",
                      cursor: "pointer",
                      transition: "color 150ms var(--ease-cut)",
                    }}
                  >
                    {copied ? "COPIED ✓" : "COPY"}
                  </button>
                </div>

                <pre
                  style={{
                    margin: 0,
                    padding: "2rem 1.5rem",
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.78rem",
                    lineHeight: 1.7,
                    color: "var(--block-dark-text)",
                    overflowX: "auto",
                    whiteSpace: "pre",
                  }}
                >
                  <code>{active.code}</code>
                </pre>
              </div>
            </div>
          </div>
        </section>

        {/* Section 2: Full Ecosystem Grid */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Ecosystem</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(2rem, 4vw, 3rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            Works with your stack.
          </h2>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginBottom: "3rem",
            }}
          >
            {ECOSYSTEM_GRID.map((item) => (
              <div
                key={item.name}
                style={{
                  background: "var(--background)",
                  padding: "2rem 1.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.25rem",
                  transition: "background-color 150ms var(--ease-cut)",
                  cursor: "default",
                }}
                onMouseEnter={(e) =>
                  ((e.currentTarget as HTMLDivElement).style.background = "var(--surface-raised)")
                }
                onMouseLeave={(e) =>
                  ((e.currentTarget as HTMLDivElement).style.background = "var(--background)")
                }
              >
                <div
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "0.9rem",
                    fontWeight: 700,
                    color: "var(--foreground)",
                  }}
                >
                  {item.name}
                </div>
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  {item.tag}
                </div>
              </div>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}
