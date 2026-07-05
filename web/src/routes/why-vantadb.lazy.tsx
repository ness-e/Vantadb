import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

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
        eyebrow="Manifesto"
        title="Why VantaDB"
        sub="21 reasons to ship embedded vector search."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">01 / 06 — Comparison</span>
          <div className="nb-divider" />
          <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "var(--space-xl)" }}>
            {COMPARISONS.map((c) => (
              <div
                key={c.category}
                style={{
                  padding: "var(--space-lg) var(--space-xl)",
                  background: "var(--background)",
                }}
              >
                <span
                  className="nb-label nb-label--amber"
                  style={{ marginBottom: "var(--space-sm)" }}
                >
                  {c.category}
                </span>
                <div
                  style={{
                    display: "flex",
                    gap: "var(--space-md)",
                    alignItems: "center",
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-code)",
                    lineHeight: 1.5,
                  }}
                >
                  <span style={{ color: "var(--amber)", flex: 1 }}>{c.vantadb}</span>
                  <NbArrow />
                  <span style={{ color: "var(--steel)", flex: 1 }}>{c.others}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">02 / 06 — Principles</span>
          <div className="nb-divider" />
          <div
            className="nb-grid nb-grid--cols-3"
            style={{ marginTop: "var(--space-xl)" }}
          >
            {PRINCIPLES.map((p) => (
              <div
                key={p.title}
                style={{
                  padding: "var(--space-xl)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-sm)",
                  borderLeft: "2px solid var(--border-visible)",
                }}
              >
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 800,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {p.title}
                </h2>
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
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "var(--space-lg)",
              flexWrap: "wrap",
            }}
          >
            <span className="nb-label nb-label--amber">Next Step</span>
            <Link
              to="/about/company"
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-code)",
                letterSpacing: "0.12em",
                textTransform: "uppercase",
                textDecoration: "none",
                color: "var(--amber)",
                padding: "12px 28px",
                background: "var(--background)",
                boxShadow: "var(--shadow-md)",
                display: "inline-flex",
                alignItems: "center",
                gap: "0.5rem",
              }}
            >
              &gt; cd about/company
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}

function NbArrow() {
  return (
    <span
      style={{
        fontFamily: "var(--font-mono)",
        fontSize: "var(--text-title)",
        color: "var(--amber)",
        lineHeight: 1,
      }}
    >
      →
    </span>
  );
}
