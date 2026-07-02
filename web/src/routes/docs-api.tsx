import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/docs-api")({
  head: () => ({
    meta: [
      { title: "VantaDB API & Documentation" },
      {
        name: "description",
        content: "API reference and developer documentation for VantaDB.",
      },
    ],
  }),
  component: DocsApiPage,
});

function DocsApiPage() {
  return (
    <div className="engine-page">
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

      <main className="engine-main">
        <section className="engine-section">
          <span className="swiss-eyebrow">Quick Start</span>
          
          <div style={{ marginTop: "3rem" }}>
            <h3 style={{ margin: "0 0 1rem 0", color: "var(--amber)", fontFamily: "var(--font-mono)", fontSize: "1rem" }}>Install the Python SDK</h3>
            <pre style={{ background: "var(--surface-raised)", padding: "1.5rem", border: "1px solid var(--border)", overflowX: "auto", fontFamily: "var(--font-mono)", fontSize: "0.9rem", color: "var(--steel)" }}>
              <code>pip install vantadb-py</code>
            </pre>

            <h3 style={{ margin: "3rem 0 1rem 0", color: "var(--amber)", fontFamily: "var(--font-mono)", fontSize: "1rem" }}>Basic Initialization</h3>
            <pre style={{ background: "var(--surface-raised)", padding: "1.5rem", border: "1px solid var(--border)", overflowX: "auto", fontFamily: "var(--font-mono)", fontSize: "0.9rem", color: "var(--steel)", lineHeight: "1.5" }}>
              <code>
{`import vantadb

# Initialize the engine
db = vantadb.VantaDB("./vanta_data")

# Create a collection with 1536-dimensional vectors
db.create_collection("agent_memory", 1536)

# Insert a document
db.insert("agent_memory", {
    "id": "doc_1",
    "vector": [0.1, 0.2, ...],
    "text": "User asked to summarize the meeting notes."
})`}
              </code>
            </pre>
          </div>

          <p style={{ marginTop: "3rem", color: "var(--steel)", lineHeight: "1.6" }}>
            The complete API documentation and SDK integration guides are currently hosted on our GitHub repository.
            <br />
            <br />
            <a href="https://github.com/ness-e/Vantadb" target="_blank" rel="noopener noreferrer" style={{ color: "var(--amber)", textDecoration: "none", borderBottom: "1px solid var(--amber)" }}>
              View GitHub Repository →
            </a>
          </p>
        </section>
      </main>
    </div>
  );
}
