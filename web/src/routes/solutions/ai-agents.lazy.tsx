import { createLazyRoute, Link } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

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
      <SwissSubpageHero
        num="01"
        eyebrow="Solution — AI Agent Memory"
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
          <div className="nb-telemetry" style={{ marginBottom: "var(--space-md)" }}>
            <span>Home</span>
            <span>Solutions</span>
            <span>AI Agents</span>
          </div>

          <span className="nb-label nb-label--amber">01 / 03 — The Problem</span>
          <div className="nb-divider" />

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "1px",
              background: "var(--border-visible)",
              marginTop: "var(--space-xl)",
            }}
          >
            <div className="nb-cell" style={{ padding: "var(--space-xl)" }}>
              <span className="nb-label" style={{ marginBottom: "var(--space-md)" }}>
                Stateless agents
              </span>
              <ul className="nb-list">
                {PROBLEMS.map((p) => (
                  <li key={p.text} style={{ color: "var(--muted)" }}>
                    <span
                      style={{
                        color: "var(--danger)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                        marginRight: "var(--space-2xs)",
                      }}
                    >
                      {p.icon}
                    </span>
                    {p.text}
                  </li>
                ))}
              </ul>
            </div>
            <div
              className="nb-cell"
              style={{
                padding: "var(--space-xl)",
                borderLeft: "2px solid var(--amber)",
                background: "var(--surface-alt)",
              }}
            >
              <span
                className="nb-label nb-label--amber"
                style={{ marginBottom: "var(--space-md)" }}
              >
                Embedded memory
              </span>
              <ul className="nb-list">
                {SOLUTIONS_LIST.map((s) => (
                  <li key={s.text} style={{ color: "var(--foreground)" }}>
                    <span
                      style={{
                        color: "var(--amber)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                        marginRight: "var(--space-2xs)",
                      }}
                    >
                      {s.icon}
                    </span>
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
          <span className="nb-label nb-label--amber">02 / 03 — Memory Primitives</span>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "var(--space-xl)" }}>
            {PRIMITIVES.map((p) => (
              <div key={p.num} className="nb-cell" style={{ padding: "var(--space-xl)" }}>
                <span
                  className="nb-label nb-label--amber"
                  style={{ marginBottom: "var(--space-2xs)" }}
                >
                  {p.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 700,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: "0 0 var(--space-2xs)",
                  }}
                >
                  {p.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "var(--text-code)",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {p.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">03 / 03 — Implementation</span>
          <div className="nb-divider" />

          <div
            className="nb-frame"
            data-frame-label="agent_memory.py"
            style={{ marginTop: "var(--space-xl)" }}
          >
            <pre
              style={{
                margin: 0,
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-code)",
                lineHeight: 1.6,
                color: "var(--foreground)",
                whiteSpace: "pre",
                overflowX: "auto",
              }}
            >
              <code>{MEMORY_CODE}</code>
            </pre>
          </div>

          <div
            className="nb-block-amber"
            style={{ marginTop: "var(--space-xl)", textAlign: "center" }}
          >
            <span className="nb-label" style={{ color: "var(--text-on-amber)" }}>
              BUILD YOUR AGENT
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "var(--text-body)",
                color: "var(--text-on-amber)",
                margin: "var(--space-2xs) 0",
                opacity: 0.85,
              }}
            >
              Get started with the docs.
            </p>
            <Link
              to="/docs"
              className="btn-ghost"
              style={{
                borderColor: "var(--text-on-amber)",
                color: "var(--text-on-amber)",
                boxShadow: "var(--shadow-brutal)",
              }}
            >
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
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <span className="nb-label" style={{ fontSize: "var(--text-label)", marginBottom: 0 }}>
        Loading...
      </span>
    </div>
  );
}
