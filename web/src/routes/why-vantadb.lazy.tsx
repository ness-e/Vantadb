import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/why-vantadb.css";

export const Route = createLazyRoute("/why-vantadb")({
  component: WhyVantaDBPage,
  pendingComponent: PendingComponent,
});

const COMPARISONS = [
  {
    category: "Architecture",
    vantadb: "Embedded, zero-infrastructure",
    others: "Requires servers, network calls",
  },
  {
    category: "Latency",
    vantadb: "&lt;1ms p50 local",
    others: "3–50ms network round-trip",
  },
  {
    category: "Embedding",
    vantadb: "Any ONNX model",
    others: "Vendor-locked models",
  },
  {
    category: "Storage",
    vantadb: "Single binary file, 2 MB",
    others: "Database cluster required",
  },
  {
    category: "Search",
    vantadb: "HNSW + BM25 hybrid fused",
    others: "Vector-only or separate text search",
  },
  {
    category: "License",
    vantadb: "Apache 2.0",
    others: "BSL or proprietary",
  },
];

const PRINCIPLES = [
  {
    title: "It's a Library, Not a Service",
    desc: "Embed VantaDB like SQLite. No daemons, no containers, no cloud bills.",
  },
  {
    title: "Hybrid by Default",
    desc: "HNSW + BM25 fused scoring. One query, ranked results, zero glue code.",
  },
  {
    title: "Built for AI Agents",
    desc: "Persistent memory, tool-native access, sub-millisecond recall. Your agent&apos;s hippocampus.",
  },
];

function WhyVantaDBPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="01"
        title="Why VantaDB"
        sub="21 reasons to ship embedded vector search."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-divider" />
          <div className="nb-grid nb-grid--cols-2 why-vantadb-compare-grid">
            {COMPARISONS.map((c) => (
              <div key={c.category} className="why-vantadb-compare-item">
                <span className="why-vantadb-compare-category">{c.category}</span>
                <div className="why-vantadb-compare-row">
                  <span className="why-vantadb-compare-vantadb">{c.vantadb}</span>
                  <NbArrow />
                  <span className="why-vantadb-compare-others">{c.others}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="why-vantadb-principles-title">Principles</h2>
          <div className="nb-divider" />
          <div className="nb-grid nb-grid--cols-3 why-vantadb-principles-grid">
            {PRINCIPLES.map((p) => (
              <div key={p.title} className="why-vantadb-principle-card">
                <h2 className="why-vantadb-principle-heading">{p.title}</h2>
                <p className="why-vantadb-principle-desc">{p.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="why-vantadb-cta-row">
            <Link to="/about/company" className="why-vantadb-cta-link">
              cd about/company
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}

function NbArrow() {
  return <span className="why-vantadb-arrow">→</span>;
}
