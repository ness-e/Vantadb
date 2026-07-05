import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/pricing.css";

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
    tagline: "Managed cloud database. Aspirational — coming in a future release.",
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
    a: "Cloud plans are aspirational and coming in a future release. In the meantime, the self-hosted version is free and fully functional.",
  },
  {
    q: "What happens when I exceed 1M vectors in Cloud Pro?",
    a: "Cloud Pro includes up to 1M vectors. If you need more, you can upgrade to Cloud Business (10M) or Enterprise (unlimited). Self-hosted has no artificial caps — your only limit is your hardware.",
  },
];

function PricingPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="05"
        title={
          <span>
            Free to build.
            <br />
            Fair to scale.
          </span>
        }
        sub="VantaDB is open source (Apache 2.0) and free forever. Sign up for cloud databases to scale in production with SLAs, team features, and zero ops."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="pricing-plans">Plans</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2 pricing-grid">
            {tiers.slice(0, 2).map((tier) => (
              <div
                key={tier.name}
                className={`nb-card pricing-card ${tier.featured ? "pricing-card--featured" : "pricing-card--default"}`}
              >
                <div>
                  <div className="pricing-tier-name">{tier.name}</div>
                  <p className="pricing-tier-tagline">{tier.tagline}</p>
                </div>
                <div className="pricing-price-row">
                  <span className="pricing-price-value">{tier.price}</span>
                  <span className="pricing-period">{tier.period}</span>
                </div>
                <ul className="nb-list pricing-features-list">
                  {tier.features.map((f) => (
                    <li key={f} className="pricing-feature-item">
                      {f}
                    </li>
                  ))}
                </ul>
                <Link
                  to={tier.href.startsWith("/") ? (tier.href as "/") : "/about/contact"}
                  className={
                    tier.featured
                      ? "nb-btn pricing-card-cta"
                      : "nb-btn nb-btn--ghost pricing-card-cta"
                  }
                >
                  {tier.cta}
                </Link>
              </div>
            ))}
          </div>

          <div className="nb-grid nb-grid--cols-2 pricing-grid--tight">
            {tiers.slice(2).map((tier) => (
              <div key={tier.name} className="nb-card pricing-card pricing-card--default">
                <div>
                  <div className="pricing-tier-name">{tier.name}</div>
                  <p className="pricing-tier-tagline">{tier.tagline}</p>
                </div>
                <div className="pricing-price-row">
                  <span className="pricing-price-value">{tier.price}</span>
                  <span className="pricing-period">{tier.period}</span>
                </div>
                <ul className="nb-list pricing-features-list">
                  {tier.features.map((f) => (
                    <li key={f} className="pricing-feature-item">
                      {f}
                    </li>
                  ))}
                </ul>
                <Link
                  to={tier.href.startsWith("/") ? (tier.href as "/") : "/about/contact"}
                  className="nb-btn nb-btn--ghost pricing-card-cta"
                >
                  {tier.cta}
                </Link>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <div className="nb-divider" />

          <div className="nb-frame pricing-table-wrapper">
            <table className="nb-table pricing-table">
              <thead>
                <tr>
                  {comparisonColumns.map((col, idx) => {
                    const thClass =
                      idx === 2
                        ? "pricing-th--featured"
                        : idx === 0
                          ? "pricing-th--first"
                          : "pricing-th--default";
                    return (
                      <th key={col} className={thClass}>
                        {col}
                      </th>
                    );
                  })}
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row, i) => (
                  <tr
                    key={row.feature}
                    className={i % 2 === 0 ? "pricing-tr--even" : "pricing-tr--odd"}
                  >
                    <td className="pricing-td-feature">{row.feature}</td>
                    <td>{row.os}</td>
                    <td className="pricing-td--pro">{row.pro}</td>
                    <td>{row.biz}</td>
                    <td className="pricing-td--ent">{row.ent}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2 pricing-faq-grid">
            {FAQ_ITEMS.map((item) => (
              <div key={item.q} className="nb-cell pricing-faq-item">
                <h3 className="pricing-faq-question">{item.q}</h3>
                <p className="pricing-faq-answer">{item.a}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber pricing-cta-block">
            <p className="pricing-cta-heading">Start with Self-Hosted.</p>
            <p className="pricing-cta-sub">Free forever. No signup required.</p>
            <Link to="/docs" className="nb-btn nb-btn--ghost pricing-cta-btn">
              GET STARTED
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}
