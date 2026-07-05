import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import "../../styles/ai-agents.css";

export const Route = createLazyRoute("/solutions/ai-agents")({
  component: AiAgentsPage,
});

const PROBLEMS = [
  { icon: "✗", text: "OpenAI assistants: context window limited to 128K tokens" },
  { icon: "✗", text: "Redis + embedding: two services to manage for persisted memory" },
  { icon: "✗", text: "Flat files: no semantic search, no structured querying" },
  { icon: "✗", text: "Cloud vector DB: adds 100ms+ latency per memory access" },
];

const SOLUTIONS_LIST = [
  { icon: "✓", text: "Store agent memories as typed vector + metadata records" },
  { icon: "✓", text: 'Semantic recall: "What did the user say about pricing?"' },
  { icon: "✓", text: 'Structured queries: "Show tool calls from last 24h with errors"' },
  { icon: "✓", text: "In-process: no net call, no serialization, 1.2ms recall" },
];

const PRIMITIVES = [
  {
    num: "01",
    title: "Conversation Log",
    desc: "Append-only log of every turn. Query by semantic similarity, time range, or metadata tags.",
  },
  {
    num: "02",
    title: "Tool Result Cache",
    desc: "Store tool call outputs keyed by input hash. Avoid redundant LLM invocations.",
  },
  {
    num: "03",
    title: "User Preferences",
    desc: "Persistent key-value with vector embeddings for preference matching across sessions.",
  },
  {
    num: "04",
    title: "Ephemeral State",
    desc: "In-memory WAL for active session state. Flushed to durable storage on checkpoint.",
  },
];

const MEMORY_CODE = `import vantadb_py as vantadb

db = vantadb.VantaDB("./agent_memory")

# Store a memory with vector + metadata
db.put(key="msg-1", vector=embedding, metadata={"role": "user", "session": session_id, "text": message})

# Semantic recall — no API call, no network
results = db.search_memory(query=embedding, top_k=5)`;

function AiAgentsPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="01"
        title={
          <span>
            Memory that
            <br />
            doesn't forget.
          </span>
        }
        sub="Give your AI agent persistent memory — conversation history, tool call results, learned preferences, and ephemeral state — all in one embedded database that lives inside your agent process."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="ai-agents-section-title">The Problem</h2>
          <div className="nb-divider" />

          <div className="ai-agents-grid-2col">
            <div className="nb-cell ai-agents-cell-padded">
              <span className="ai-agents-label-muted">Stateless agents</span>
              <ul className="nb-list">
                {PROBLEMS.map((p) => (
                  <li key={p.text} className="ai-agents-list-item-muted">
                    <span className="ai-agents-list-icon-danger">{p.icon}</span>
                    {p.text}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nb-cell ai-agents-cell-amber-border">
              <span className="ai-agents-label-amber">Embedded memory</span>
              <ul className="nb-list">
                {SOLUTIONS_LIST.map((s) => (
                  <li key={s.text} className="ai-agents-list-item-foreground">
                    <span className="ai-agents-list-icon-amber">{s.icon}</span>
                    {s.text}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="ai-agents-section-title">Memory Primitives</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2 ai-agents-mt-xl">
            {PRIMITIVES.map((p) => (
              <div key={p.num} className="nb-cell ai-agents-cell-padded">
                <span className="ai-agents-card-num">{p.num}</span>
                <h3 className="ai-agents-card-title">{p.title}</h3>
                <p className="ai-agents-card-desc">{p.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="ai-agents-section-title">Implementation</h2>
          <div className="nb-divider" />

          <div className="nb-frame ai-agents-mt-xl">
            <pre className="ai-agents-code-block">
              <code>{MEMORY_CODE}</code>
            </pre>
          </div>

          <div className="nb-block-amber ai-agents-cta-block">
            <span className="ai-agents-cta-label">BUILD YOUR AGENT</span>
            <p className="ai-agents-cta-text">Get started with the docs.</p>
            <Link to="/docs" className="nb-btn nb-btn--ghost ai-agents-cta-link">
              DOCS
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="ai-agents-pending">
      <span className="ai-agents-pending-text">Loading...</span>
    </div>
  );
}
