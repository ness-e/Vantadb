import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/docs-api")({
  component: DocsApiPage,
  pendingComponent: PendingComponent,
});

function DocsApiPage() {
  return (
    <div className="swiss-page">
      <SwissSubpageHero
        num="04"
        eyebrow="Developer API"
        title={
          <span>
            API &amp;
            <br />
            Documentation.
          </span>
        }
        sub="Integrate VantaDB into your local AI agent stacks using our native Python PyO3 SDK."
      />

      <main className="swiss-main">
        <section className="swiss-page-section">
          <span className="swiss-eyebrow">Quick Start</span>

          <div style={{ marginTop: "3rem" }}>
            <h3
              style={{
                margin: "0 0 1rem 0",
                color: "var(--amber)",
                fontFamily: "var(--font-mono)",
                fontSize: "1rem",
              }}
            >
              Install the Python SDK
            </h3>
            <pre
              style={{
                background: "var(--surface-raised)",
                padding: "1.5rem",
                border: "1px solid var(--border)",
                overflowX: "auto",
                fontFamily: "var(--font-mono)",
                fontSize: "0.9rem",
                color: "var(--steel)",
              }}
            >
              <code>pip install vantadb-py</code>
            </pre>

            <h3
              style={{
                margin: "3rem 0 1rem 0",
                color: "var(--amber)",
                fontFamily: "var(--font-mono)",
                fontSize: "1rem",
              }}
            >
              Basic Initialization
            </h3>
            <pre
              style={{
                background: "var(--surface-raised)",
                padding: "1.5rem",
                border: "1px solid var(--border)",
                overflowX: "auto",
                fontFamily: "var(--font-mono)",
                fontSize: "0.9rem",
                color: "var(--steel)",
                lineHeight: "1.5",
              }}
            >
              <code>
                {`import vantadb_py as vantadb

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
)`}
              </code>
            </pre>
          </div>

          <p style={{ marginTop: "3rem", color: "var(--steel)", lineHeight: "1.6" }}>
            The complete API documentation and SDK integration guides are currently hosted on our
            GitHub repository.
            <br />
            <br />
            <a
              href="https://github.com/ness-e/Vantadb"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: "var(--amber)",
                textDecoration: "none",
                borderBottom: "1px solid var(--amber)",
              }}
            >
              View GitHub Repository →
            </a>
          </p>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          pre { font-size: 0.78rem !important; overflow-x: auto !important; }
        }
      `}</style>
    </div>
  );
}
