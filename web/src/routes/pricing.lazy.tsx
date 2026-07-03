import { createLazyRoute, Link } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createLazyRoute("/pricing")({
  component: PricingPage,
  pendingComponent: PendingComponent,
});

const tiers = [
  {
    name: "Self-Hosted",
    tagline: "Run locally, embed anywhere. Free forever.",
    price: "$0",
    period: "forever",
    features: [
      "HNSW vector search + BM25 full-text + hybrid RRF",
      "Unlimited vectors — no artificial caps",
      "Python SDK + Rust SDK + CLI",
      "WAL-backed durability, 3 storage backends",
      "Apache 2.0 license — unrestricted use",
      "Community support (Discord + GitHub)",
      "LangChain + LlamaIndex integrations",
      "MCP Server (Model Context Protocol)",
    ],
    cta: "Get Started",
    href: "/docs",
    featured: false,
  },
  {
    name: "Cloud Pro",
    tagline: "Managed cloud database. Currently in early access.",
    price: "$29",
    period: "per month",
    features: [
      "Fully managed serverless database",
      "Up to 1M vectors & 10GB storage",
      "Automated daily backups",
      "HTTPS client API & SDK access",
      "Priority email support",
      "99.9% uptime SLA guarantee (coming soon)",
      "Standard encryption in-transit & at-rest",
      "Single-user API token auth",
    ],
    cta: "Join Waitlist",
    href: "/about/contact",
    featured: true,
  },
  {
    name: "Cloud Business",
    tagline: "Dedicated cloud clusters. Coming soon.",
    price: "$149",
    period: "per month",
    features: [
      "Dedicated database instance hosting",
      "Up to 10M vectors & 100GB storage",
      "Continuous point-in-time recovery",
      "Multi-region replica synchronization (coming soon)",
      "Dedicated Slack channel & fast SLA support",
      "99.99% uptime SLA guarantee (coming soon)",
      "Advanced encryption at-rest (AES-256)",
      "Team authentication & SSO/SAML (coming soon)",
    ],
    cta: "Join Waitlist",
    href: "/about/contact",
    featured: false,
  },
  {
    name: "Enterprise",
    tagline: "Custom licensing and dedicated support available.",
    price: "Custom",
    period: "tailored pricing",
    features: [
      "Unlimited vectors, namespaces & storage",
      "On-premises, VPC, or hybrid cloud deployment",
      "Dedicated SLA support with 24/7/365 coverage",
      "Enterprise security: RBAC & custom keys",
      "Enterprise compliance: SOC 2 & HIPAA (coming soon)",
      "Custom query hooks & raw hardware access",
      "Dedicated systems architect support",
      "Flexible custom licensing options",
    ],
    cta: "Contact Sales",
    href: "/about/contact",
    featured: false,
  },
];

const comparisonColumns = ["Feature", "Self-Hosted", "Cloud Pro", "Cloud Business", "Enterprise"];

const comparisonRows = [
  {
    feature: "Deployment",
    os: "Embedded, single-node",
    pro: "Managed serverless",
    biz: "Managed dedicated",
    ent: "On-prem / Hybrid",
  },
  {
    feature: "Vector limit",
    os: "Limited by hardware",
    pro: "1M Vectors (scalable)",
    biz: "10M Vectors",
    ent: "Unlimited",
  },
  {
    feature: "Storage limit",
    os: "Local disk limit",
    pro: "10 GB",
    biz: "100 GB",
    ent: "Unlimited",
  },
  {
    feature: "Query engines",
    os: "HNSW + BM25 + RRF",
    pro: "HNSW + BM25 + RRF",
    biz: "HNSW + BM25 + RRF",
    ent: "All + Custom hooks",
  },
  {
    feature: "Replication",
    os: "None",
    pro: "Automated backup",
    biz: "Multi-region replica",
    ent: "Multi-node (WAL-based)",
  },
  {
    feature: "Authentication",
    os: "None",
    pro: "API Key",
    biz: "API Key + Team SSO",
    ent: "SAML / OIDC + RBAC",
  },
  {
    feature: "Encryption",
    os: "Optional (user-space)",
    pro: "At-rest & In-transit",
    biz: "At-rest & In-transit",
    ent: "AES-256-GCM (Hardware)",
  },
  {
    feature: "Support",
    os: "Community (Discord)",
    pro: "Priority Email",
    biz: "Priority Slack / SLA",
    ent: "Dedicated 24/7 SLA",
  },
  {
    feature: "License",
    os: "Apache 2.0",
    pro: "Commercial Cloud",
    biz: "Commercial Cloud",
    ent: "Enterprise terms",
  },
];

const FAQ_ITEMS = [
  {
    q: "Is VantaDB really free?",
    a: "Yes. The core engine is Apache 2.0 licensed and free forever. No hidden pricing, no per-query fees, no artificial limits on self-hosted instances.",
  },
  {
    q: "Can I use VantaDB commercially?",
    a: "Yes. The Apache 2.0 license allows unrestricted use, modification, and distribution. No royalties, no attribution required for local self-hosted deployments.",
  },
  {
    q: "What is included in the Cloud plans?",
    a: "Our cloud plans provide hosted serverless and dedicated instances. By running VantaDB on our managed infrastructure, you get client-server access via HTTPS, automatic scaling, automated backups, and uptime SLAs, without managing local resources.",
  },
  {
    q: "Do you offer custom SLAs?",
    a: "Yes, our Enterprise plan includes dedicated support SLAs with up to 24/7/365 availability. We also assist with specialized hardware configuration, on-premises isolation, and security compliance (SOC 2, HIPAA).",
  },
  {
    q: "How is VantaDB different from Pinecone, Weaviate, or Qdrant?",
    a: "VantaDB is embedded — it runs in your process with zero servers. No network hop, no per-vector pricing, no ops team required. While cloud vector databases charge $70-175/mo, VantaDB is free (Apache 2.0) and runs locally.",
  },
  {
    q: "Can I migrate from Pinecone/ChromaDB to VantaDB?",
    a: "Yes. We provide migration guides from ChromaDB and LanceDB in our documentation. The process typically involves exporting your vectors and re-indexing with VantaDB's Python SDK.",
  },
  {
    q: "Do you offer a free trial for Cloud plans?",
    a: "Cloud plans are currently in early access. Join the waitlist and we'll notify you when we launch. In the meantime, the self-hosted version is free and fully functional.",
  },
  {
    q: "What happens when I exceed 1M vectors in Cloud Pro?",
    a: "Cloud Pro includes up to 1M vectors. If you need more, you can upgrade to Cloud Business (10M) or Enterprise (unlimited). Self-hosted has no artificial caps — your only limit is your hardware.",
  },
];

function PricingPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="05"
        eyebrow="Pricing"
        title={
          <span>
            Free to build.
            <br />
            Fair to scale.
          </span>
        }
        sub="VantaDB is open source (Apache 2.0) and free forever. Sign up for cloud databases to scale in production with SLAs, team features, and zero ops."
      />

      <main className="engine-main">
        {/* Plans section */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — Plans</span>

          <div
            className="pricing-grid"
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(260px, 1fr))",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {tiers.map((tier) => (
              <div
                key={tier.name}
                className="pricing-card"
                style={{
                  background: tier.featured ? "var(--surface-raised)" : "var(--background)",
                  padding: "3rem 2rem 2.5rem 2rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "1.5rem",
                  border: tier.featured ? "1px solid var(--amber)" : "1px solid transparent",
                  position: "relative",
                  transition: "all 150ms cubic-bezier(0.25, 1, 0.5, 1)",
                }}
              >
                {tier.featured && (
                  <span
                    className="pricing-badge"
                    style={{
                      position: "absolute",
                      top: "1.25rem",
                      right: "1.25rem",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.55rem",
                      fontWeight: 700,
                      textTransform: "uppercase" as const,
                      letterSpacing: "0.1em",
                      color: "var(--amber)",
                      background: "var(--amber-dim)",
                      padding: "0.2rem 0.6rem",
                    }}
                  >
                    EARLY ACCESS
                  </span>
                )}

                <div>
                  <div
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "1.4rem",
                      fontWeight: 700,
                      letterSpacing: "-0.04em",
                      color: tier.featured ? "var(--amber)" : "var(--foreground)",
                    }}
                  >
                    {tier.name}
                  </div>
                  <div
                    style={{
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.75rem",
                      color: "var(--muted)",
                      marginTop: "0.5rem",
                      lineHeight: 1.4,
                    }}
                  >
                    {tier.tagline}
                  </div>
                </div>

                <div style={{ display: "flex", alignItems: "baseline", gap: "0.4rem" }}>
                  <span
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "3rem",
                      fontWeight: 700,
                      letterSpacing: "-0.05em",
                      color: "var(--foreground)",
                    }}
                  >
                    {tier.price}
                  </span>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.65rem",
                      color: "var(--steel)",
                      textTransform: "uppercase" as const,
                      letterSpacing: "0.06em",
                    }}
                  >
                    {tier.period}
                  </span>
                </div>

                <ul
                  style={{
                    listStyle: "none",
                    margin: 0,
                    padding: 0,
                    display: "flex",
                    flexDirection: "column",
                    gap: "0.75rem",
                    flex: 1,
                  }}
                >
                  {tier.features.map((f) => (
                    <li
                      key={f}
                      style={{
                        display: "flex",
                        gap: "0.6rem",
                        fontFamily: "var(--font-sans)",
                        fontSize: "0.8rem",
                        color: "var(--muted)",
                        lineHeight: 1.4,
                      }}
                    >
                      <span
                        style={{
                          color: tier.featured ? "var(--amber)" : "var(--steel)",
                          fontFamily: "var(--font-mono)",
                          flexShrink: 0,
                        }}
                      >
                        ✓
                      </span>
                      {f}
                    </li>
                  ))}
                </ul>

                <Link
                  to={tier.href.startsWith("/") ? (tier.href as "/") : "/about/contact"}
                  className={`pricing-cta ${tier.featured ? "pricing-cta--featured" : "pricing-cta--default"}`}
                >
                  {tier.cta}
                </Link>
              </div>
            ))}
          </div>
        </section>

        {/* Feature Breakdown Section */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">02 / 03 — Feature Breakdown</span>

          <div
            style={{
              border: "1px solid var(--border)",
              marginTop: "3rem",
              overflowX: "auto",
            }}
          >
            <table
              style={{
                width: "100%",
                borderCollapse: "collapse",
                fontFamily: "var(--font-sans)",
                fontSize: "0.82rem",
                minWidth: "750px",
              }}
            >
              <thead>
                <tr
                  style={{ borderBottom: "2px solid var(--border)", background: "var(--surface)" }}
                >
                  {comparisonColumns.map((col, idx) => (
                    <th
                      key={col}
                      style={{
                        padding: "1.2rem 1.5rem",
                        textAlign: "left",
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.7rem",
                        textTransform: "uppercase" as const,
                        letterSpacing: "0.08em",
                        color:
                          idx === 2
                            ? "var(--amber)"
                            : idx === 0
                              ? "var(--foreground)"
                              : "var(--steel)",
                        fontWeight: 600,
                      }}
                    >
                      {col}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row, i) => (
                  <tr
                    key={row.feature}
                    style={{
                      borderBottom: "1px solid var(--border)",
                      background: i % 2 === 0 ? "var(--background)" : "var(--surface)",
                      transition: "background 100ms",
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background = "var(--surface-hover)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background =
                        i % 2 === 0 ? "var(--background)" : "var(--surface)";
                    }}
                  >
                    <td
                      style={{
                        padding: "1rem 1.5rem",
                        fontWeight: 600,
                        color: "var(--foreground)",
                      }}
                    >
                      {row.feature}
                    </td>
                    <td style={{ padding: "1rem 1.5rem", color: "var(--foreground)" }}>{row.os}</td>
                    <td
                      style={{
                        padding: "1rem 1.5rem",
                        color: "var(--foreground)",
                        fontWeight: 500,
                      }}
                    >
                      {row.pro}
                    </td>
                    <td style={{ padding: "1rem 1.5rem", color: "var(--foreground)" }}>
                      {row.biz}
                    </td>
                    <td style={{ padding: "1rem 1.5rem", color: "var(--muted)" }}>{row.ent}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* FAQ Section */}
        <section className="engine-section">
          <span className="swiss-eyebrow">03 / 03 — FAQ</span>

          <div
            className="faq-grid"
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(360px, 1fr))",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {FAQ_ITEMS.map((item) => (
              <div
                key={item.q}
                style={{
                  background: "var(--background)",
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "1rem",
                }}
              >
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "0.95rem",
                    fontWeight: 700,
                    letterSpacing: "-0.02em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {item.q}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.85rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {item.a}
                </p>
              </div>
            ))}
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          .faq-grid {
            grid-template-columns: 1fr !important;
          }
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
