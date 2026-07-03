import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

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
    <div className="engine-page">
      <SwissSubpageHero
        num="06"
        eyebrow="About — Company"
        title={
          <span>
            Built for the
            <br />
            AI-native era.
          </span>
        }
        sub="VantaDB unifies vector search (HNSW), lexical search (BM25), and hybrid search (RRF) in a single Rust binary. Zero servers. Zero ops. One pip install."
      />

      <main className="engine-main">
        <nav aria-label="Breadcrumb" style={{ fontFamily: "var(--font-mono)", fontSize: "0.65rem", color: "var(--muted)", marginBottom: "0", padding: "0 clamp(1.5rem, 5vw, 4rem)", marginTop: "1rem" }}>
          <a href="/" style={{ color: "var(--muted)", textDecoration: "none" }}>Home</a>
          <span style={{ margin: "0 0.5rem" }}>/</span>
          <span style={{ color: "var(--foreground)" }}>Company</span>
        </nav>
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — Purpose</span>
          <div className="swiss-grid-12" style={{ alignItems: "start", marginTop: "3rem" }}>
            <div className="col-span-5">
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "clamp(1.5rem, 3vw, 2.25rem)",
                  fontWeight: 800,
                  letterSpacing: "-0.04em",
                  color: "var(--foreground)",
                  lineHeight: 1.1,
                  margin: 0,
                }}
              >
                Make vector-native data infrastructure invisible.
              </h2>
            </div>
            <div className="col-span-7">
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "0.9rem",
                  color: "var(--muted)",
                  lineHeight: 1.75,
                  margin: 0,
                }}
              >
                Every AI agent, every RAG pipeline, every intelligent application deserves a
                database that embeds in-process but understands vectors, text, and hybrid search —
                without requiring a dedicated infrastructure team.
              </p>
            </div>
          </div>
        </section>

        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">02 / 03 — Values</span>
          <div
            className="values-grid"
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {VALUES.map((v) => (
              <div
                key={v.num}
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
                  {v.num}
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
                  {v.title}
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
                  {v.desc}
                </p>
              </div>
            ))}
          </div>
        </section>

        <section className="engine-section">
          <span className="swiss-eyebrow">03 / 03 — Why VantaDB</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(1.5rem, 3vw, 2rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            The AI stack shouldn't need a database team.
          </h2>

          <div className="swiss-grid-12" style={{ alignItems: "start", gap: "1px" }}>
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", padding: "2.5rem" }}
            >
              <h3
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "0.85rem",
                  fontWeight: 800,
                  textTransform: "uppercase",
                  letterSpacing: "0.05em",
                  color: "var(--steel)",
                  marginBottom: "1.5rem",
                }}
              >
                The alternatives
              </h3>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.85rem",
                }}
              >
                {COMPARISON_LEFT.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--muted)",
                      lineHeight: 1.4,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--steel)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
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
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                padding: "2.5rem",
                background: "var(--surface)",
              }}
            >
              <h3
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "0.85rem",
                  fontWeight: 800,
                  textTransform: "uppercase",
                  letterSpacing: "0.05em",
                  color: "var(--amber)",
                  marginBottom: "1.5rem",
                }}
              >
                VantaDB
              </h3>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.85rem",
                }}
              >
                {COMPARISON_RIGHT.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--foreground)",
                      lineHeight: 1.4,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
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
        </section>
      </main>

      <style>{`
        @media (min-width: 641px) and (max-width: 768px) {
          .values-grid { grid-template-columns: repeat(2, 1fr) !important; }
        }
        @media (max-width: 640px) {
          .values-grid { grid-template-columns: 1fr !important; }
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
