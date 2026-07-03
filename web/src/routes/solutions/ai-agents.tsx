import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/solutions/ai-agents")({
  head: () => ({
    meta: [
      { title: "VantaDB — AI Agent Memory & State" },
      {
        name: "description",
        content:
          "Persistent memory for AI agents. Store conversation history, tool call results, and agent state in an embedded vector database.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/solutions/ai-agents" }],
  }),
  component: AiAgentsPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
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
    <div className="engine-page">
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

      <main className="engine-main">
        {/* Section 1: Problem vs Solution */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — The Problem</span>
          <div
            className="swiss-grid-12"
            style={{ alignItems: "start", marginTop: "3rem", gap: "1px" }}
          >
            {/* Problem */}
            <div
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                padding: "2.5rem",
              }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--steel)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                Stateless agents
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "1rem",
                }}
              >
                {PROBLEMS.map((p, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--muted)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "#ff3b30",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                      }}
                    >
                      {p.icon}
                    </span>
                    {p.text}
                  </li>
                ))}
              </ul>
            </div>

            {/* Solution */}
            <div
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                padding: "2.5rem",
                background: "var(--surface)",
              }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--amber)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                Embedded memory
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "1rem",
                }}
              >
                {SOLUTIONS_LIST.map((s, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--foreground)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
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
        </section>

        {/* Section 2: Primitives */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">02 / 03 — Memory Primitives</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {PRIMITIVES.map((p) => (
              <div
                key={p.num}
                style={{
                  background: "var(--background)",
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {p.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1rem",
                    fontWeight: 700,
                    letterSpacing: "-0.02em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {p.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
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
        </section>

        {/* Section 3: Code */}
        <section className="engine-section">
          <span className="swiss-eyebrow">03 / 03 — Implementation</span>
          <div
            style={{
              border: "1px solid var(--border)",
              background: "var(--block-dark-bg)",
              marginTop: "3rem",
            }}
          >
            <div
              style={{
                padding: "0.75rem 1.5rem",
                borderBottom: "1px solid var(--block-dark-border)",
                display: "flex",
                alignItems: "center",
                gap: "0.5rem",
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
                agent_memory.py
              </span>
            </div>
            <pre
              style={{
                margin: 0,
                padding: "2rem 2rem",
                fontFamily: "var(--font-mono)",
                fontSize: "0.78rem",
                lineHeight: 1.7,
                color: "var(--block-dark-text)",
                overflowX: "auto",
                whiteSpace: "pre",
              }}
            >
              <code>{MEMORY_CODE}</code>
            </pre>
          </div>
        </section>
      </main>
    </div>
  );
}
