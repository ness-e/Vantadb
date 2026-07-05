import { createLazyRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/integrations.css";

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
    <div className="nb-page">
      <NbSubpageHero
        num="03"
        title={
          <span>
            Fits your stack.
            <br />
            Not the other way.
          </span>
        }
        sub="Connect VantaDB directly to the frameworks you already know. Built for first-class Python and Rust ecosystems."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="integrations-section-title">Framework Connectors</h2>
          <div className="nb-divider" />

          <div className="integrations-grid">
            <div>
              <div className="nb-grid nb-grid--cols-2">
                {INTEGRATIONS.map((int) => (
                  <button
                    key={int.id}
                    onClick={() => setSelectedId(int.id)}
                    className="nb-cell"
                    style={{
                      background:
                        selectedId === int.id ? "var(--surface-alt)" : "var(--background)",
                      border: "none",
                      borderLeft:
                        selectedId === int.id ? "2px solid var(--amber)" : "2px solid transparent",
                      cursor: "pointer",
                      textAlign: "left",
                      width: "100%",
                      fontFamily: "inherit",
                      color: "inherit",
                    }}
                  >
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "var(--text-micro)",
                        color: selectedId === int.id ? "var(--amber)" : "var(--steel)",
                        marginBottom: 0,
                      }}
                    >
                      {int.category}
                    </span>
                    <div
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "var(--text-title)",
                        fontWeight: 700,
                        letterSpacing: "var(--tracking-display)",
                        color: selectedId === int.id ? "var(--foreground)" : "var(--muted)",
                      }}
                    >
                      {int.label}
                    </div>
                  </button>
                ))}
              </div>

              <div className="nb-card integrations-card-margin">
                <div className="integrations-tag-row">
                  <span className="integrations-tag">{active.tag}</span>
                  {active.experimental && (
                    <span className="integrations-experimental-tag">EXPERIMENTAL</span>
                  )}
                </div>
                <p className="integrations-desc">{active.desc}</p>
              </div>
            </div>

            <div className="nb-frame integrations-code-frame">
              <div className="integrations-code-header">
                <span className="integrations-code-label">{active.tag}</span>
                <button onClick={handleCopy} className="nb-btn nb-btn--ghost integrations-copy-btn">
                  {copied ? "COPIED" : "COPY"}
                </button>
              </div>
              <pre className="integrations-code-pre">
                <code>{active.code}</code>
              </pre>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="integrations-ecosystem-title">Ecosystem</h2>
          <div className="nb-divider" />
          <p className="integrations-ecosystem-lead">Works with your stack.</p>

          <div className="nb-grid integrations-ecosystem-grid">
            {ECOSYSTEM_GRID.map((item) => (
              <div key={item.name} className="nb-cell integrations-ecosystem-cell">
                <div className="integrations-ecosystem-name">{item.name}</div>
                <span className="integrations-ecosystem-tag">{item.tag}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber integrations-cta-block">
            <span className="integrations-cta-label">BUILD YOUR INTEGRATION</span>
            <p className="integrations-cta-text">Check the docs to build your own connector.</p>
            <a href="/docs" className="nb-btn nb-btn--ghost integrations-cta-link">
              DOCS
            </a>
          </div>
        </div>
      </section>

      <style>{`
        @media (max-width: 768px) {
          [style*="grid-template-columns: 1fr 1fr"][style*="gap: var(--space-xl)"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}
