import { createLazyRoute } from "@tanstack/react-router";
import { useState, useMemo, useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
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
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(connectorsRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, connectorsRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(ecosystemRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, ecosystemRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p01"
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
          sub="Drop-in integrations for the most popular AI frameworks — no glue code, no middleware."
        />

        <div className="integrations-grid">
          <div>
            <div className="nb-grid nb-grid--cols-2">
              {INTEGRATIONS.map((int) => (
                <button
                  key={int.id}
                  onClick={() => setSelectedId(int.id)}
                  className={
                    "nb-cell integrations-btn" +
                    (selectedId === int.id ? " integrations-btn--active" : "")
                  }
                >
                  <span
                    className={
                      "nb-mono-label" + (selectedId === int.id ? "" : " nb-mono-label--steel")
                    }
                  >
                    {int.category}
                  </span>
                  <div className="integrations-btn-label">{int.label}</div>
                </button>
              ))}
            </div>

            <div className="nb-card-frame integrations-card nb-engine-part">
              <div className="integrations-tag-row">
                <span className="nb-mono-label">{active.tag}</span>
                {active.experimental && (
                  <span className="integrations-tag--experimental">EXPERIMENTAL</span>
                )}
              </div>
              <p className="integrations-desc">{active.desc}</p>
            </div>
          </div>

          <div className="nb-card-frame integrations-code-frame nb-engine-part">
            <div className="integrations-code-header">
              <span className="nb-mono-label nb-mono-label--muted">{active.tag}</span>
              <button onClick={handleCopy} className="nb-btn nb-btn--ghost integrations-copy-btn">
                {copied ? "COPIED" : "COPY"}
              </button>
            </div>
            <pre className="integrations-code-pre">
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

        <div className="nb-grid integrations-ecosystem-grid nb-engine-part">
          {ECOSYSTEM_GRID.map((item) => (
            <div key={item.name} className="nb-card-frame integrations-ecosystem-cell">
              <div className="integrations-ecosystem-name">{item.name}</div>
              <span className="nb-mono-label nb-mono-label--steel">{item.tag}</span>
            </div>
          ))}
        </div>
      </NbSection>

      <NbSection ariaLabel="Build your integration">
        <NbBlockAmber className="integrations-cta-wrapper">
          <span className="nb-mono-label">BUILD YOUR INTEGRATION</span>
          <p className="integrations-cta-text">Check the docs to build your own connector.</p>
          <a href="/docs" className="nb-btn nb-btn--ghost integrations-cta-link">
            DOCS
          </a>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}
