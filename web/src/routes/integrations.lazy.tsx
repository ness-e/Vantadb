import { createLazyRoute } from "@tanstack/react-router";
import { useMemo, useRef, useState } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
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

resp = client.embeddings.create(
    model="text-embedding-3-small",
    input="User prefers async workflows"
)
vector = resp.data[0].embedding

db.put("sessions", "user-001",
    "User prefers async workflows",
    vector=vector)

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

resp = requests.post(
    "http://localhost:11434/api/embeddings",
    json={"model": "nomic-embed-text",
          "prompt": "Offline RAG context"}
)
vector = resp.json()["embedding"]

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
    desc: "EXPERIMENTAL \u2014 Expose VantaDB namespaces and tools to Claude Desktop or any MCP-compatible runtime via vantadb-mcp.",
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
    desc: "Direct Rust bindings via PyO3. Zero TCP overhead \u2014 sync VantaDB and async AsyncVantaDB with full SDK coverage.",
    code: `import vantadb_py as vanta

db = vanta.VantaDB("./vanta_memory")

db.put("memories", "user-pref",
    "Developer is building high-end interfaces",
    vector=[0.15, 0.82, 0.44])

hits = db.search_memory("memories",
    query_vector=[0.14, 0.85, 0.40],
    top_k=1)

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
  const connectorsRef = useRef<HTMLElement>(null);
  const ecosystemRef = useRef<HTMLElement>(null);

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

  useAnimationSafe(() => {
        const parts = connectorsRef.current?.querySelectorAll<HTMLElement>(".");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, connectorsRef);

  useAnimationSafe(() => {
        const parts = ecosystemRef.current?.querySelectorAll<HTMLElement>(".");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, ecosystemRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p07"
        title={
          <span>
            Fits your stack.
            <br />
            Not the other way.
          </span>
        }
        sub="Connect VantaDB directly to the frameworks you already know. Built for first-class Python and Rust ecosystems."
      />

      <NbSection ref={connectorsRef} variant="lg" ariaLabel="Framework connectors">
        <NbSectionHeader
          monoLabel="[CONNECTORS]"
          headline="Framework Connectors."
          sub="Drop-in integrations for the most popular AI frameworks \u2014 no glue code, no middleware."
        />

        <div className="nc-int-grid">
          <div>
            <div className="nc-int-picker">
              {INTEGRATIONS.map((int) => (
                <button
                  key={int.id}
                  onClick={() => setSelectedId(int.id)}
                  className={`nc-int-btn${selectedId === int.id ? " nc-int-btn--active" : ""}`}
                >
                  <span
                    className={
                      selectedId === int.id
                        ? "nc-int-btn-cat"
                        : "nc-int-btn-cat nc-int-btn-cat--steel"
                    }
                  >
                    {int.category}
                  </span>
                  <div className="nc-int-btn-label">{int.label}</div>
                </button>
              ))}
            </div>

            <div className="nc-int-card nc-int-part">
              <div className="nc-int-tag-row">
                <span className="nc-int-tag">{active.tag}</span>
                {active.experimental && (
                  <span className="nc-int-tag--experimental">EXPERIMENTAL</span>
                )}
              </div>
              <p className="nc-int-desc">{active.desc}</p>
            </div>
          </div>

          <div className="nc-int-code-frame nc-int-part">
            <div className="nc-int-code-header">
              <span className="nc-int-code-label">{active.tag}</span>
              <button onClick={handleCopy} className="nb-btn nb-btn--ghost nc-int-copy-btn">
                {copied ? "COPIED" : "COPY"}
              </button>
            </div>
            <pre className="nc-int-code-pre">
              <code>{active.code}</code>
            </pre>
          </div>
        </div>
      </NbSection>

      <NbSection ref={ecosystemRef} className="nb-bg-cross--faint" ariaLabel="Ecosystem">
        <NbSectionHeader
          monoLabel="[ECOSYSTEM]"
          headline="Ecosystem."
          sub="Works with your stack."
        />

        <div className="nc-int-eco-grid nc-int-part">
          {ECOSYSTEM_GRID.map((item) => (
            <div key={item.name} className="nc-int-eco-cell">
              <div className="nc-int-eco-name">{item.name}</div>
              <span className="nc-int-eco-tag">{item.tag}</span>
            </div>
          ))}
        </div>
      </NbSection>

      <NbSection ariaLabel="Build your integration">
        <NbBlockAmber className="nc-int-cta">
          <span className="nb-mono-label">BUILD YOUR INTEGRATION</span>
          <p className="nc-int-cta-text">Check the docs to build your own connector.</p>
          <a href="/docs" className="nb-btn nb-btn--ghost nc-int-cta-link">
            DOCS
          </a>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}
