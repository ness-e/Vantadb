import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { PricingCard } from "@/components/PricingCard";
import { PricingCTA } from "@/components/PricingCTA";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
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
      "Unlimited vectors \u2014 no artificial caps",
      "Python SDK + Rust SDK + CLI",
      "WAL-backed durability, 3 storage backends",
      "Apache 2.0 license \u2014 unrestricted use",
      "Community support (Discord + GitHub)",
      "CrewAI + DSPy + Haystack + Mem0 adapters",
      "MCP Server (Model Context Protocol)",
    ],
    cta: "Get Started",
    href: "/docs",
    featured: false,
  },
  {
    name: "Cloud Pro",
    tagline: "Managed cloud database. Aspirational \u2014 coming in a future release.",
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
    a: "VantaDB is embedded \u2014 it runs in your process with zero servers. No network hop, no per-vector pricing, no ops team required. While cloud vector databases charge $70-175/mo, VantaDB is free (Apache 2.0) and runs locally.",
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
    a: "Cloud Pro includes up to 1M vectors. If you need more, you can upgrade to Cloud Business (10M) or Enterprise (unlimited). Self-hosted has no artificial caps \u2014 your only limit is your hardware.",
  },
];

function PricingPage() {
  const plansRef = useRef<HTMLElement>(null);
  const compareRef = useRef<HTMLElement>(null);
  const faqRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = plansRef.current?.querySelectorAll<HTMLElement>(".nc-price-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, plansRef);

  useAnimationSafe(() => {
    const parts = compareRef.current?.querySelectorAll<HTMLElement>(".nc-price-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, compareRef);

  useAnimationSafe(() => {
    const parts = faqRef.current?.querySelectorAll<HTMLElement>(".nc-price-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, faqRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p11"
        title={
          <span>
            Free to build.
            <br />
            Fair to scale.
          </span>
        }
        sub="VantaDB is open source (Apache 2.0) and free forever. Sign up for cloud databases to scale in production with SLAs, team features, and zero ops."
      />

      <main>
        <NbSection ref={plansRef} ariaLabel="Pricing plans">
          <NbSectionHeader
            monoLabel="[PLANS]"
            headline="Four tiers. One free."
            sub="Self-hosted is free forever under Apache 2.0. Cloud plans unlock managed infrastructure for teams that need it."
          />

          <div className="nc-price-board nc-price-part">
            {tiers.slice(0, 2).map((tier) => (
              <PricingCard key={tier.name} tier={tier} featured={tier.featured} />
            ))}
          </div>

          <div className="nc-price-board nc-price-board--tight nc-price-part">
            {tiers.slice(2).map((tier) => (
              <PricingCard key={tier.name} tier={tier} />
            ))}
          </div>
        </NbSection>

        <NbSection ref={compareRef} className="nb-bg-cross--faint" ariaLabel="Feature comparison">
          <NbSectionHeader
            monoLabel="[COMPARE]"
            headline="Side-by-side feature comparison."
            sub="Compare capabilities across all four tiers to find the right fit for your project."
          />

          <div className="nc-price-market nc-price-part">
            <table>
              <thead>
                <tr>
                  {comparisonColumns.map((col) => (
                    <th scope="col" key={col}>
                      {col}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row) => (
                  <tr key={row.feature}>
                    <td>{row.feature}</td>
                    <td>{row.os}</td>
                    <td>{row.pro}</td>
                    <td>{row.biz}</td>
                    <td>{row.ent}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </NbSection>

        <NbSection ref={faqRef} ariaLabel="FAQ">
          <NbSectionHeader
            monoLabel="[FAQ]"
            headline="Common questions."
            sub="Everything you need to know about VantaDB pricing, licensing, and cloud plans."
          />

          <div className="nc-price-faq nc-price-part">
            {FAQ_ITEMS.map((item) => (
              <div key={item.q} className="nc-price-faq-item">
                <h3 className="nc-price-faq-q">{item.q}</h3>
                <p className="nc-price-faq-a">{item.a}</p>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection ariaLabel="Get started">
          <PricingCTA
            heading="Start with Self-Hosted."
            sub="Free forever. No signup required."
            cta="GET STARTED"
            href="/docs"
          />
        </NbSection>
      </main>
    </div>
  );
}
