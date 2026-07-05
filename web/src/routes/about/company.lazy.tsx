import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";

export const Route = createLazyRoute("/about/company")({
  component: CompanyPage,
});

const VALUES = [
  {
    num: "01",
    title: "Radical Simplicity",
    desc: "One binary, one pip install, zero servers. Complexity is the enemy — we eat it so developers don't have to.",
  },
  {
    num: "02",
    title: "Performance Without Compromise",
    desc: "1.2ms p50 queries at 0.998 Recall@10. Every microsecond matters when your agent is waiting.",
  },
  {
    num: "03",
    title: "Developer Empathy First",
    desc: "We ship SDKs, docs, and APIs that feel like they were built by developers for developers — because they were.",
  },
  {
    num: "04",
    title: "Open by Default",
    desc: "Open core, open benchmarks, open roadmap. Our community trusts us because we show receipts.",
  },
];

const COMPARISON_LEFT = [
  "Pinecone: $70/mo + per-vector pricing",
  "Weaviate/Qdrant: server process + ops team",
  "LanceDB: limited hybrid search",
  "LanceDB: data model is a second thought",
];

const COMPARISON_RIGHT = [
  "VantaDB: one binary, zero ops, Apache 2.0 license",
  "HNSW + BM25 + hybrid in a single query",
  "Embedded in your process — no network hop",
  "Sub-millisecond hybrid search",
];

function CompanyPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="06"
        title={
          <span>
            Built for the
            <br />
            AI-native era.
          </span>
        }
        sub="VantaDB unifies vector search (HNSW), lexical search (BM25), and hybrid search (RRF) in a single Rust binary. Zero servers. Zero ops. One pip install."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Purpose
          </h2>
          <div className="nb-divider" />

          <div className="nb-split-7-5" style={{ marginTop: "var(--space-xl)" }}>
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-display)",
                fontWeight: 800,
                letterSpacing: "var(--tracking-display)",
                color: "var(--foreground)",
                lineHeight: 1.1,
                margin: 0,
              }}
            >
              Make vector-native data infrastructure invisible.
            </h2>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "var(--text-body)",
                color: "var(--muted)",
                lineHeight: 1.75,
                margin: 0,
              }}
            >
              Every AI agent, every RAG pipeline, every intelligent application deserves a database
              that embeds in-process but understands vectors, text, and hybrid search — without
              requiring a dedicated infrastructure team.
            </p>
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Values
          </h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "var(--space-xl)" }}>
            {VALUES.map((v) => (
              <div key={v.num} className="nb-cell" style={{ padding: "var(--space-xl)" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-micro)",
                    color: "var(--amber)",
                    fontWeight: 700,
                    marginBottom: "var(--space-2xs)",
                    display: "block",
                  }}
                >
                  {v.num}
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
                  {v.title}
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
                  {v.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Why VantaDB
          </h2>
          <div className="nb-divider" />
          <p
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-display)",
              fontWeight: 800,
              letterSpacing: "var(--tracking-display)",
              margin: "var(--space-sm) 0 var(--space-xl)",
              lineHeight: 1.05,
            }}
          >
            The AI stack shouldn't need a database team.
          </p>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "1px",
              background: "var(--border-visible)",
            }}
          >
            <div
              className="nb-card"
              style={{ border: "none", boxShadow: "none", background: "var(--background)" }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-micro)",
                  color: "var(--muted)",
                  marginBottom: "var(--space-md)",
                  display: "block",
                }}
              >
                The alternatives
              </span>
              <ul className="nb-list">
                {COMPARISON_LEFT.map((item) => (
                  <li key={item} style={{ color: "var(--muted)" }}>
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
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div
              className="nb-card"
              style={{
                border: "2px solid var(--amber)",
                boxShadow: "var(--shadow-amber)",
                background: "var(--surface-alt)",
              }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-micro)",
                  color: "var(--amber)",
                  fontWeight: 700,
                  marginBottom: "var(--space-md)",
                  display: "block",
                }}
              >
                VantaDB
              </span>
              <ul className="nb-list">
                {COMPARISON_RIGHT.map((item) => (
                  <li key={item} style={{ color: "var(--foreground)" }}>
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
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber" style={{ textAlign: "center" }}>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-micro)",
                color: "var(--text-on-amber)",
                marginBottom: "var(--space-2xs)",
                display: "block",
              }}
            >
              READ OUR STORY
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
              Learn more about our community.
            </p>
            <Link
              to="/about/community"
              className="nb-btn nb-btn--ghost"
              style={{
                borderColor: "var(--text-on-amber)",
                color: "var(--text-on-amber)",
                boxShadow: "var(--shadow-brutal)",
              }}
            >
              COMMUNITY
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
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-label)",
          color: "var(--muted)",
          marginBottom: 0,
        }}
      >
        Loading...
      </span>
    </div>
  );
}
