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
          <h2 className="about-company-section-title">Purpose</h2>
          <div className="nb-divider" />

          <div className="nb-split-7-5 about-company-split-top">
            <h2 className="about-company-hero-title">
              Make vector-native data infrastructure invisible.
            </h2>
            <p className="about-company-hero-desc">
              Every AI agent, every RAG pipeline, every intelligent application deserves a database
              that embeds in-process but understands vectors, text, and hybrid search — without
              requiring a dedicated infrastructure team.
            </p>
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="about-company-section-title">Values</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2 about-company-grid-top">
            {VALUES.map((v) => (
              <div key={v.num} className="nb-cell about-company-value-card">
                <span className="about-company-value-num">
                  {v.num}
                </span>
                <h3 className="about-company-value-title">
                  {v.title}
                </h3>
                <p className="about-company-value-desc">
                  {v.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="about-company-section-title">Why VantaDB</h2>
          <div className="nb-divider" />
          <p className="about-company-feature-lead">
            The AI stack shouldn't need a database team.
          </p>

          <div className="about-company-compare-grid">
            <div className="nb-card about-company-compare-card-left">
              <span className="about-company-compare-label-alt">
                The alternatives
              </span>
              <ul className="nb-list">
                {COMPARISON_LEFT.map((item) => (
                  <li key={item} className="about-company-compare-item-left">
                    <span className="about-company-bullet-x">
                      ✗
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nb-card about-company-compare-card-right">
              <span className="about-company-compare-label-highlight">
                VantaDB
              </span>
              <ul className="nb-list">
                {COMPARISON_RIGHT.map((item) => (
                  <li key={item} className="about-company-compare-item-right">
                    <span className="about-company-bullet-check">
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
          <div className="nb-block-amber about-company-cta-block">
            <span className="about-company-cta-label">
              READ OUR STORY
            </span>
            <p className="about-company-cta-desc">
              Learn more about our community.
            </p>
            <Link
              to="/about/community"
              className="nb-btn nb-btn--ghost about-company-cta-btn"
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
    <div className="about-pending-container">
      <span className="about-pending-text">
        Loading...
      </span>
    </div>
  );
}
