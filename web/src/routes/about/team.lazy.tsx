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
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Members
          </h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3" style={{ marginTop: "var(--space-xl)" }}>
            {TEAM.map((m) => (
              <div
                key={m.name}
                className="nb-cell"
                style={{
                  display: "flex",
                  gap: "var(--space-md)",
                  padding: "var(--space-lg)",
                  alignItems: "flex-start",
                }}
              >
                <div
                  className="nb-dither"
                  style={{
                    width: 64,
                    height: 64,
                    flexShrink: 0,
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-micro)",
                    lineHeight: 1.3,
                    background: "#0a0a0a",
                    border: "2px solid var(--border-visible)",
                    display: "flex",
                    flexDirection: "column",
                    justifyContent: "center",
                    padding: "var(--space-2xs)",
                    color: "var(--amber)",
                    whiteSpace: "pre",
                  }}
                >
                  <div>{"> user: " + m.avatarUser}</div>
                  <div>{"> status: " + m.avatarStatus}</div>
                </div>

                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "var(--space-3xs)",
                    minWidth: 0,
                  }}
                >
                  <span
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "var(--text-title)",
                      fontWeight: 700,
                      color: "var(--foreground)",
                      lineHeight: 1.2,
                    }}
                  >
                    {m.name}
                  </span>

                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "var(--text-micro)",
                      color: "var(--amber)",
                      textTransform: "uppercase",
                    }}
                  >
                    [{m.role}]
                  </span>

                  <p
                    style={{
                      fontFamily: "var(--font-sans)",
                      fontSize: "var(--text-code)",
                      color: "var(--muted)",
                      lineHeight: 1.6,
                      margin: 0,
                    }}
                  >
                    {m.desc}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
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
              JOIN THE TEAM
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
              We're always looking for talented people who share our mission. Say hello.
            </p>
            <Link
              to="/about/contact"
              className="nb-btn nb-btn--ghost"
              style={{
                borderColor: "var(--text-on-amber)",
                color: "var(--text-on-amber)",
                boxShadow: "var(--shadow-brutal)",
              }}
            >
              CONTACT US
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}
