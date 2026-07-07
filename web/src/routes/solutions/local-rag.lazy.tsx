import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import "../../styles/local-rag.css";

export const Route = createLazyRoute("/solutions/local-rag")({
  component: LocalRagPage,
});

const PIPELINE = [
  {
    title: "Ingest",
    desc: "Load documents (PDF, Markdown, code). Chunk and embed with a local model (all-MiniLM-L6-v2, nomic-embed-text). Zero network calls.",
  },
  {
    title: "Index",
    desc: "Store embeddings + text in VantaDB. BM25 full-text index for lexical fallback. Hybrid query with RRF fusion.",
  },
  {
    title: "Retrieve",
    desc: "Query with semantic + keyword search. Pass results to your local LLM. All in-process, all private.",
  },
];

const RAG_CODE = `import vantadb_py as vantadb
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("all-MiniLM-L6-v2")
db = vantadb.VantaDB("./rag_knowledge")

for i, doc in enumerate(documents):
    vec = model.encode(doc.text).tolist()
    db.put(key=f"doc-{i}", vector=vec, metadata={"text": doc.text})

query_vec = model.encode(query_text).tolist()
results = db.search_memory(query=query_vec, top_k=5)`;

const COMPARISON = {
  problems: [
    "Pinecone/Weaviate: your document embeddings leave your network",
    "SaaS vector DB: every query crosses the wire",
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
  const privacyRef = useRef<HTMLElement>(null);
  const pipelineRef = useRef<HTMLElement>(null);
  const implRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-lr-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(privacyRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, privacyRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-lr-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(pipelineRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, pipelineRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-lr-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(implRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, implRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p25"
        title={
          <span>
            Your data never
            <br />
            leaves your device.
          </span>
        }
        sub="Run Retrieval-Augmented Generation entirely on-device. VantaDB embeds, indexes, and queries documents locally."
      />

      <NbSection ref={privacyRef} ariaLabel="Privacy gap">
        <NbSectionHeader
          monoLabel="[PRIVACY GAP]"
          headline="Cloud RAG leaks data."
          sub="Every SaaS vector DB adds a data exposure point. Local-first eliminates them."
        />
        <div className="nc-lr-compare nc-lr-part">
          <div className="nc-lr-col">
            <span className="nc-lr-col-label nc-lr-col-label--danger">Cloud RAG leaks data</span>
            <ul className="nc-lr-list">
              {COMPARISON.problems.map((p) => (
                <li key={p} className="nc-lr-item">
                  <span className="nc-lr-icon nc-lr-icon--danger">✗</span>
                  {p}
                </li>
              ))}
            </ul>
          </div>
          <div className="nc-lr-col nc-lr-col--amber">
            <span className="nc-lr-col-label nc-lr-col-label--amber">Local-first, zero exposure</span>
            <ul className="nc-lr-list">
              {COMPARISON.solutions.map((s) => (
                <li key={s} className="nc-lr-item nc-lr-item--fg">
                  <span className="nc-lr-icon nc-lr-icon--amber">✓</span>
                  {s}
                </li>
              ))}
            </ul>
          </div>
        </div>
      </NbSection>

      <NbSection ref={pipelineRef} variant="lg" className="nb-bg-cross--faint" ariaLabel="Pipeline">
        <NbSectionHeader
          monoLabel="[PIPELINE]"
          headline="Ingest → Index → Retrieve."
          sub="Three stages from document to answer, entirely on-device."
        />
        <div className="nc-lr-pipeline nc-lr-part">
          {PIPELINE.map((step) => (
            <div key={step.title} className="nc-lr-step">
              <h3 className="nc-lr-step-title">{step.title}</h3>
              <p className="nc-lr-step-desc">{step.desc}</p>
            </div>
          ))}
        </div>
      </NbSection>

      <NbSection ref={implRef} ariaLabel="Implementation">
        <NbSectionHeader
          monoLabel="[IMPLEMENTATION]"
          headline="Twenty lines to local RAG."
          sub="Six imports, one database, zero cloud calls."
        />
        <div className="nc-lr-part">
          <pre className="nc-lr-code">
            <code>{RAG_CODE}</code>
          </pre>
        </div>
        <div className="nc-lr-part">
          <NbBlockAmber as="div">
            <div className="nc-lr-cta">
              <h2 className="nc-lr-cta-heading">Start building privacy-first RAG today.</h2>
              <Link to="/docs" className="nb-btn nb-btn--ghost">
                DOCS
              </Link>
            </div>
          </NbBlockAmber>
        </div>
      </NbSection>
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
