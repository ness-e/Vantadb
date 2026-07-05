import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import "../../styles/local-rag.css";

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
          <h2 className="local-rag-section-title">Privacy Gap</h2>
          <div className="nb-divider" />

          <div className="local-rag-grid-2col">
            <div className="nb-cell local-rag-cell-padded">
              <span className="local-rag-label-muted">Cloud RAG leaks data</span>
              <ul className="nb-list">
                {COMPARISON.problems.map((p) => (
                  <li key={p} className="local-rag-list-item-muted">
                    <span className="local-rag-list-icon-danger">✗</span>
                    {p}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nb-cell local-rag-cell-amber-border">
              <span className="local-rag-label-amber">Local-first, zero exposure</span>
              <ul className="nb-list">
                {COMPARISON.solutions.map((s) => (
                  <li key={s} className="local-rag-list-item-foreground">
                    <span className="local-rag-list-icon-amber">✓</span>
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
          <h2 className="local-rag-section-title">Pipeline</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3 local-rag-mt-xl">
            {PIPELINE.map((step) => (
              <div key={step.num} className="nb-cell local-rag-cell-padded">
                <span className="local-rag-card-num">{step.num}</span>
                <h3 className="local-rag-card-title">{step.title}</h3>
                <p className="local-rag-card-desc">{step.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="local-rag-section-title">Implementation</h2>
          <div className="nb-divider" />

          <div className="nb-frame local-rag-mt-xl">
            <pre className="local-rag-code-block">
              <code>{RAG_CODE}</code>
            </pre>
          </div>

          <div className="nb-block-amber local-rag-cta-block">
            <span className="local-rag-cta-label">GO LOCAL</span>
            <p className="local-rag-cta-text">Start building privacy-first RAG today.</p>
            <Link to="/docs" className="nb-btn nb-btn--ghost local-rag-cta-link">
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
    <div className="local-rag-pending">
      <span className="local-rag-pending-text">Loading...</span>
    </div>
  );
}
