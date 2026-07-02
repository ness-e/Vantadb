import { createFileRoute, Link } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/pricing")({
  head: () => ({
    meta: [
      { title: "VantaDB — Pricing: Open Source. Free Forever." },
      {
        name: "description",
        content:
          "VantaDB is open source (Apache 2.0) and free forever. No tiers, no hidden pricing, no per-query fees. Enterprise features coming soon.",
      },
    ],
  }),
  component: PricingPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
const tiers = [
  {
    name: "Open Source",
    tagline: "Free forever. No limits. No hidden pricing.",
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
    featured: true,
  },
];

const comparisonRows = [
  {
    feature: "Deployment",
    free: "Embedded, single-node",
    ent: "On-prem, air-gapped",
  },
  { feature: "Vector limit", free: "Unlimited", ent: "Unlimited" },
  {
    feature: "Query engines",
    free: "HNSW + BM25 + RRF",
    ent: "All engines + custom",
  },
  { feature: "Replication", free: "Single-node", ent: "Multi-node (WAL-based)" },
  { feature: "Auth", free: "None", ent: "SSO / SAML / OIDC + RBAC" },
  { feature: "Audit log", free: "None", ent: "Full + compliance export" },
  { feature: "Encryption", free: "None", ent: "AES-256-GCM" },
  {
    feature: "Support",
    free: "Community (Discord)",
    ent: "Dedicated SLA",
  },
  { feature: "License", free: "Apache 2.0", ent: "Apache 2.0 + enterprise terms" },
];

const FAQ_ITEMS = [
  {
    q: "Is VantaDB really free?",
    a: "Yes. The core engine is Apache 2.0 licensed and free forever. No hidden pricing, no per-query fees, no artificial limits.",
  },
  {
    q: "Can I use VantaDB commercially?",
    a: "Yes. The Apache 2.0 license allows unrestricted use, modification, and distribution. No royalties, no attribution required.",
  },
  {
    q: "What about Enterprise features?",
    a: "We're developing enterprise features (RBAC, SSO, encryption, multi-node) for organizations with compliance requirements. Pricing will be announced when ready. The core engine remains free forever.",
  },
  {
    q: "Do you offer managed cloud hosting?",
    a: "VantaDB is designed to be embedded — it runs in your process. For managed infrastructure, see the Enterprise tier which includes deployment support.",
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
        sub="VantaDB is open source (Apache 2.0) and free forever. No tiers, no limits. Enterprise features coming soon."
      />

      <main className="engine-main">
        {/* Section 1: Tier Cards */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 03 — Plans</span>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {tiers.map((tier) => (
              <div
                key={tier.name}
                style={{
                  background: tier.featured ? "var(--surface-raised)" : "var(--background)",
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "1.5rem",
                  borderLeft: tier.featured ? "2px solid var(--amber)" : "2px solid transparent",
                  position: "relative",
                }}
              >
                {tier.featured && (
                  <span
                    style={{
                      position: "absolute",
                      top: "1.25rem",
                      right: "1.25rem",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.55rem",
                      fontWeight: 700,
                      textTransform: "uppercase",
                      letterSpacing: "0.1em",
                      color: "var(--amber)",
                      background: "rgba(255, 85, 0, 0.1)",
                      padding: "0.2rem 0.6rem",
                    }}
                  >
                    POPULAR
                  </span>
                )}

                <div>
                  <div
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "1.4rem",
                      fontWeight: 800,
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
                      marginTop: "0.25rem",
                    }}
                  >
                    {tier.tagline}
                  </div>
                </div>

                <div style={{ display: "flex", alignItems: "baseline", gap: "0.4rem" }}>
                  <span
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "2.5rem",
                      fontWeight: 800,
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
                      textTransform: "uppercase",
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
                    gap: "0.6rem",
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
                        fontSize: "0.78rem",
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
                  style={{
                    display: "block",
                    textAlign: "center",
                    padding: "0.85rem 1.5rem",
                    fontFamily: "var(--font-display)",
                    fontSize: "0.78rem",
                    fontWeight: 700,
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                    textDecoration: "none",
                    border: tier.featured
                      ? "1.5px solid var(--amber)"
                      : "1.5px solid var(--border)",
                    background: tier.featured ? "var(--amber)" : "transparent",
                    color: tier.featured ? "#000" : "var(--foreground)",
                    transition: "all 150ms var(--ease-cut)",
                  }}
                >
                  {tier.cta}
                </Link>
              </div>
            ))}
          </div>
        </section>

        {/* Section 2: Comparison Table */}
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
                fontSize: "0.8rem",
              }}
            >
              <thead>
                <tr style={{ borderBottom: "2px solid var(--border)" }}>
                  <th
                    style={{
                      padding: "1rem 1.5rem",
                      textAlign: "left",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.6rem",
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                      color: "var(--steel)",
                      fontWeight: 600,
                    }}
                  >
                    Feature
                  </th>
                  <th
                    style={{
                      padding: "1rem 1.5rem",
                      textAlign: "center",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.6rem",
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                      color: "var(--amber)",
                      fontWeight: 600,
                    }}
                  >
                    Open Source
                  </th>
                  <th
                    style={{
                      padding: "1rem 1.5rem",
                      textAlign: "center",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.6rem",
                      textTransform: "uppercase",
                      letterSpacing: "0.08em",
                      color: "var(--steel)",
                      fontWeight: 600,
                    }}
                  >
                    Enterprise
                  </th>
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row, i) => (
                  <tr
                    key={row.feature}
                    style={{
                      borderBottom: "1px solid var(--border)",
                      background: i % 2 === 0 ? "var(--background)" : "var(--surface)",
                    }}
                  >
                    <td
                      style={{
                        padding: "0.9rem 1.5rem",
                        fontWeight: 600,
                        color: "var(--foreground)",
                      }}
                    >
                      {row.feature}
                    </td>
                    <td
                      style={{
                        padding: "0.9rem 1.5rem",
                        textAlign: "center",
                        color: "var(--foreground)",
                        fontWeight: 500,
                      }}
                    >
                      {row.free}
                    </td>
                    <td
                      style={{
                        padding: "0.9rem 1.5rem",
                        textAlign: "center",
                        color: "var(--muted)",
                      }}
                    >
                      {row.ent}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Section 3: FAQ */}
        <section className="engine-section">
          <span className="swiss-eyebrow">03 / 03 — FAQ</span>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
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
                  gap: "0.75rem",
                }}
              >
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "0.9rem",
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
                    fontSize: "0.8rem",
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
    </div>
  );
}
