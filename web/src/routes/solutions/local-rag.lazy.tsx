import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createLazyRoute("/solutions/local-rag")({
  component: LocalRagPage,
});

const PIPELINE = [
  {
    num: "01",
    title: "Ingest",
    desc: "Load documents (PDF, Markdown, code). Chunk and embed with a local model (all-MiniLM-L6-v2, nomic-embed-text). Zero network calls.",
  },
  {
    num: "02",
    title: "Index",
    desc: "Store embeddings + text in VantaDB. BM25 full-text index for lexical fallback. Hybrid query with RRF fusion.",
  },
  {
    num: "03",
    title: "Retrieve",
    desc: "Query with semantic + keyword search. Pass results to your local LLM. All in-process, all private.",
  },
];

const RAG_CODE = `import vantadb_py as vantadb
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("all-MiniLM-L6-v2")
db = vantadb.VantaDB("./rag_knowledge")

# Index documents — fully local
for i, doc in enumerate(documents):
    vec = model.encode(doc.text).tolist()
    db.put(key=f"doc-{i}", vector=vec, metadata={"text": doc.text})

# Search — all private, no API call
query_vec = model.encode(query_text).tolist()
results = db.search_memory(query=query_vec, top_k=5)`;

const COMPARISON = {
  problems: [
    "Pinecone/Weaviate: your document embeddings leave your network",
    "SaaS vector DB: every query crosses the wire, every result is visible",
    "Hybrid cloud: embedding API + vector DB + LLM = 3 data exposures",
    "Data residency: cloud providers may store in any region",
  ],
  solutions: [
    "Embed documents locally — your data never touches a network",
    "In-process retrieval — no API calls for vector search",
    "Works with local LLMs (Ollama, llama.cpp, MLX)",
    "Air-gap compatible — no internet connection required",
  ],
};

function LocalRagPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="02"
        eyebrow="Solution — Local RAG"
        title={
          <span>
            Your data never
            <br />
            leaves your device.
          </span>
        }
        sub="Run Retrieval-Augmented Generation entirely on-device. VantaDB embeds, indexes, and queries documents locally — no vectors in the cloud, no data leaks, no API costs."
      />

      <main className="engine-main">
        <nav aria-label="Breadcrumb" style={{ fontFamily: "var(--font-mono)", fontSize: "0.65rem", color: "var(--muted)", marginBottom: "0", padding: "0 clamp(1.5rem, 5vw, 4rem)", marginTop: "1rem" }}>
          <a href="/" style={{ color: "var(--muted)", textDecoration: "none" }}>Home</a>
          <span style={{ margin: "0 0.5rem" }}>/</span>
          <a href="/solutions" style={{ color: "var(--muted)", textDecoration: "none" }}>Solutions</a>
          <span style={{ margin: "0 0.5rem" }}>/</span>
          <span style={{ color: "var(--foreground)" }}>Local RAG</span>
        </nav>
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — Privacy Gap</span>
          <div
            className="swiss-grid-12"
            style={{ alignItems: "start", marginTop: "3rem", gap: "1px" }}
          >
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", padding: "2.5rem" }}
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
                Cloud RAG leaks data
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
                {COMPARISON.problems.map((p, i) => (
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
                        flexShrink: 0,
                      }}
                    >
                      ✗
                    </span>
                    {p}
                  </li>
                ))}
              </ul>
            </div>

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
                Local-first, zero exposure
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
                {COMPARISON.solutions.map((s, i) => (
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
                        flexShrink: 0,
                      }}
                    >
                      ✓
                    </span>
                    {s}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </section>

        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">02 / 03 — Pipeline</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {PIPELINE.map((step) => (
              <div
                key={step.num}
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
                    fontSize: "2rem",
                    fontWeight: 800,
                    color: "var(--border)",
                    lineHeight: 1,
                    letterSpacing: "-0.04em",
                  }}
                >
                  {step.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.1rem",
                    fontWeight: 700,
                    letterSpacing: "-0.03em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {step.title}
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
                  {step.desc}
                </p>
              </div>
            ))}
          </div>
        </section>

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
                local_rag.py
              </span>
            </div>
            <pre
              style={{
                margin: 0,
                padding: "2rem",
                fontFamily: "var(--font-mono)",
                fontSize: "0.78rem",
                lineHeight: 1.7,
                color: "var(--block-dark-text)",
                overflowX: "auto",
                whiteSpace: "pre",
              }}
            >
              <code>{RAG_CODE}</code>
            </pre>
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          .swiss-grid-12 > .col-span-6 { grid-column: span 12 !important; }
        }
        @media (max-width: 640px) {
          [style*="grid-template-columns: repeat(3, 1fr)"] { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "60vh", color: "var(--muted)" }}>
      <div>Loading...</div>
    </div>
  );
}
