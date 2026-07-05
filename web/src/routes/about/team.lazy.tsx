import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/about/team")({
  component: TeamPage,
  pendingComponent: PendingComponent,
});

const TEAM = [
  {
    avatarUser: "founder",
    avatarStatus: "building",
    name: "Dr. Elena Vasquez",
    role: "Founder & CEO",
    desc: "Distributed systems researcher. Former infrastructure lead at Databricks.",
  },
  {
    avatarUser: "cto",
    avatarStatus: "optimizing",
    name: "Marcus Chen",
    role: "CTO",
    desc: "Rust core team alumni. Built query engines at DuckDB and ClickHouse.",
  },
  {
    avatarUser: "head-ml",
    avatarStatus: "indexing",
    name: "Priya Sharma",
    role: "Head of ML",
    desc: "Vector search at scale. Previously ML infra at Pinecone and Weaviate.",
  },
  {
    avatarUser: "engineer",
    avatarStatus: "compiling",
    name: "Alex Kowalski",
    role: "Lead Engineer",
    desc: "Systems programmer. Contributed to SQLite, LMDB, and RocksDB.",
  },
  {
    avatarUser: "dx",
    avatarStatus: "shipping",
    name: "Yuki Tanaka",
    role: "Developer Experience",
    desc: "Python SDK architect. PyO3 maintainer and open-source advocate.",
  },
  {
    avatarUser: "pm",
    avatarStatus: "connecting",
    name: "Sarah Mitchell",
    role: "Product & Community",
    desc: "Developer relations lead. Built communities at LangChain and Hugging Face.",
  },
];

function TeamPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="04"
        title="The people behind VantaDB."
        sub="Distributed across 6 time zones. United by one mission: make vector search invisible."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="about-team-section-title">Members</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3 about-team-grid-top">
            {TEAM.map((m) => (
              <div key={m.name} className="nb-cell about-team-member-card">
                <div className="nb-dither about-team-avatar">
                  <div>{"> user: " + m.avatarUser}</div>
                  <div>{"> status: " + m.avatarStatus}</div>
                </div>

                <div className="about-team-member-info">
                  <span className="about-team-member-name">{m.name}</span>

                  <span className="about-team-member-role">[{m.role}]</span>

                  <p className="about-team-member-desc">{m.desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <div className="nb-block-amber about-team-cta-block">
            <span className="about-team-cta-label">JOIN THE TEAM</span>
            <p className="about-team-cta-desc">
              We're always looking for talented people who share our mission. Say hello.
            </p>
            <Link to="/about/contact" className="nb-btn nb-btn--ghost about-team-cta-btn">
              CONTACT US
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}
