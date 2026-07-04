import { createLazyRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/integrations")({
  component: IntegrationsPage,
  pendingComponent: PendingComponent,
});

const INTEGRATIONS = [
  {
    id: "openai",
    label: "OpenAI",
    tag: "vantadb-openai",
    category: "Memory Bridge",
    desc: "Persist OpenAI assistant conversation history and memory profiles directly in VantaDB's embedded store.",
    code: `from openai import OpenAI
import vantadb_py as vanta

client = OpenAI()
db = vanta.VantaDB("./openai_memory")

# Generate embedding via OpenAI
resp = client.embeddings.create(
    model="text-embedding-3-small",
    input="User prefers async workflows"
)
vector = resp.data[0].embedding

# Store in VantaDB
db.put("sessions", "user-001",
    "User prefers async workflows",
    vector=vector)

# Semantic search
hits = db.search("sessions",
    query_vector=vector, top_k=5)`,
  },
  {
    id: "ollama",
    label: "Ollama",
    tag: "vantadb-ollama",
    category: "Local Embeddings",
    desc: "Bridge local Ollama embeddings with VantaDB vector search for fully offline RAG pipelines.",
    code: `import requests
import vantadb_py as vanta

db = vanta.VantaDB("./ollama_store")

# Generate embedding via Ollama
resp = requests.post(
    "http://localhost:11434/api/embeddings",
    json={"model": "nomic-embed-text",
          "prompt": "Offline RAG context"}
)
vector = resp.json()["embedding"]

# Store and search locally
db.put("docs", "rag-1",
    "Offline RAG context", vector=vector)
hits = db.search("docs",
    query_vector=vector, top_k=3)`,
  },
  {
    id: "mcp",
    label: "MCP Server",
    tag: "vantadb-mcp",
    category: "Agent Protocol",
    experimental: true,
    desc: "EXPERIMENTAL — Expose VantaDB namespaces and tools to Claude Desktop or any MCP-compatible runtime via vantadb-mcp.",
    code: `{
  "mcpServers": {
    "vantadb": {
      "command": "vantadb-mcp",
      "args": ["--path", "./agent_memory"],
      "env": {
        "VANTA_NAMESPACE": "agent_context"
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
    desc: "Direct Rust bindings via PyO3. Zero TCP overhead — sync VantaDB and async AsyncVantaDB with full SDK coverage.",
    code: `import vantadb_py as vanta

# Open database path
db = vanta.VantaDB("./vanta_memory")

# Store structured memory
db.put("memories", "user-pref",
    "Developer is building high-end interfaces",
    vector=[0.15, 0.82, 0.44])

# Multi-modal retrieval
hits = db.search_memory("memories",
    query_vector=[0.14, 0.85, 0.40],
    top_k=1)

# Async wrapper
async with vanta.AsyncVantaDB("./path") as adb:
    await adb.put("ns", "k", "v", vector=[0.1]*128)
    results = await adb.search("ns", [0.1]*128)`,
  },
];

const ECOSYSTEM_GRID = [
  { name: "vantadb-openai", tag: "OpenAI Bridge" },
  { name: "vantadb-ollama", tag: "Ollama Embed" },
  { name: "vantadb-haystack", tag: "Haystack Store" },
  { name: "vantadb-dspy", tag: "DSPy Modules" },
  { name: "vantadb-litellm", tag: "LiteLLM Proxy" },
  { name: "vantadb-crewai", tag: "CrewAI Tools" },
  { name: "vantadb-mem0", tag: "Mem0 Profiles" },
  { name: "vantadb-letta", tag: "Letta Memory" },
  { name: "vantadb-mcp", tag: "EXPERIMENTAL" },
  { name: "vantadb-wasm", tag: "EXPERIMENTAL" },
  { name: "vantadb-python", tag: "PyO3 SDK" },
  { name: "vantadb-server", tag: "HTTP Server" },
];

function IntegrationsPage() {
  const [selectedId, setSelectedId] = useState<string>("openai");
  const [copied, setCopied] = useState(false);
  const [hoveredItem, setHoveredItem] = useState<string | null>(null);

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
    <div className="swiss-page">
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

      <main className="swiss-main">
        <section className="swiss-page-section swiss-page-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Framework Connectors</span>

          <div className="swiss-grid-12" style={{ alignItems: "start", marginTop: "3rem" }}>
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
                <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "0.5rem" }}>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.6rem",
                      color: "var(--amber)",
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                    }}
                  >
                    {active.tag}
                  </span>
                  {active.experimental && (
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.5rem",
                        fontWeight: 700,
                        textTransform: "uppercase",
                        letterSpacing: "0.1em",
                        color: "var(--amber)",
                        border: "1px solid var(--amber)",
                        padding: "0.15rem 0.4rem",
                        lineHeight: 1,
                      }}
                    >
                      EXPERIMENTAL
                    </span>
                  )}
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

            <div className="col-span-8">
              <div
                style={{
                  border: "1px solid var(--border)",
                  background: "var(--block-dark-bg)",
                  position: "relative",
                }}
              >
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

        <section className="swiss-page-section">
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
                  padding: "2rem 1.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.25rem",
                  transition: "background-color 150ms var(--ease-cut)",
                  cursor: "default",
                  background: hoveredItem === item.name
                    ? "var(--surface-raised)"
                    : "var(--background)",
                }}
                onMouseEnter={() => setHoveredItem(item.name)}
                onMouseLeave={() => setHoveredItem(null)}
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

      <style>{`
        @media (max-width: 768px) {
          .swiss-grid-12 { grid-template-columns: 1fr !important; }
          [style*="grid-template-columns: 1fr 1fr"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}


