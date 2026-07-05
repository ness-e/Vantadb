import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";

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
    <div className="nb-page">
      <NbSubpageHero
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

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-telemetry" style={{ marginBottom: "var(--space-md)" }}>
            <span>Home</span>
            <span>Solutions</span>
            <span>Local RAG</span>
          </div>

          <span className="nb-label nb-label--amber">01 / 03 — Privacy Gap</span>
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
                Cloud RAG leaks data
              </span>
              <ul className="nb-list">
                {COMPARISON.problems.map((p) => (
                  <li key={p} style={{ color: "var(--muted)" }}>
                    <span
                      style={{
                        color: "var(--danger)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                        marginRight: "var(--space-2xs)",
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
                Local-first, zero exposure
              </span>
              <ul className="nb-list">
                {COMPARISON.solutions.map((s) => (
                  <li key={s} style={{ color: "var(--foreground)" }}>
                    <span
                      style={{
                        color: "var(--amber)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                        marginRight: "var(--space-2xs)",
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
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">02 / 03 — Pipeline</span>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3" style={{ marginTop: "var(--space-xl)" }}>
            {PIPELINE.map((step) => (
              <div key={step.num} className="nb-cell" style={{ padding: "var(--space-xl)" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-metric)",
                    fontWeight: 800,
                    color: "var(--border-strong)",
                    lineHeight: 1,
                    letterSpacing: "var(--tracking-tight)",
                    display: "block",
                    marginBottom: "var(--space-sm)",
                  }}
                >
                  {step.num}
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
                  {step.title}
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
                  {step.desc}
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
            data-frame-label="local_rag.py"
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
              <code>{RAG_CODE}</code>
            </pre>
          </div>

          <div
            className="nb-block-amber"
            style={{ marginTop: "var(--space-xl)", textAlign: "center" }}
          >
            <span className="nb-label" style={{ color: "var(--text-on-amber)" }}>
              GO LOCAL
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
              Start building privacy-first RAG today.
            </p>
            <Link
              to="/docs"
              className="nb-btn nb-btn--ghost"
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
