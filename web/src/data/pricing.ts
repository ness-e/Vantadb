import type { PricingCardTier } from "@/components/PricingCard";

export const tiers: PricingCardTier[] = [
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
      "CrewAI + DSPy + Haystack + Mem0 adapters",
      "MCP Server (Model Context Protocol)",
    ],
    cta: "Get Started",
    href: "/docs",
    featured: false,
  },
  {
    name: "Enterprise",
    tagline: "Custom licensing and dedicated support available.",
    price: "Custom",
    period: "tailored pricing",
    features: [
      "Unlimited vectors, namespaces & storage",
      "On-premises deployment",
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

export const comparisonColumns = [
  "Feature",
  "Self-Hosted",
  "Enterprise",
];

interface ComparisonRow {
  feature: string;
  os: string;
  ent: string;
}

export const comparisonRows: ComparisonRow[] = [
  {
    feature: "Deployment",
    os: "Embedded, single-node",
    ent: "On-premises",
  },
  {
    feature: "Vector limit",
    os: "Limited by hardware",
    ent: "Unlimited",
  },
  {
    feature: "Storage limit",
    os: "Local disk limit",
    ent: "Unlimited",
  },
  {
    feature: "Query engines",
    os: "HNSW + BM25 + RRF",
    ent: "All + Custom hooks",
  },
  {
    feature: "Replication",
    os: "None",
    ent: "Multi-node (WAL-based)",
  },
  {
    feature: "Authentication",
    os: "None",
    ent: "SAML / OIDC + RBAC",
  },
  {
    feature: "Encryption",
    os: "Optional (user-space)",
    ent: "AES-256-GCM (Hardware)",
  },
  {
    feature: "Support",
    os: "Community (Discord)",
    ent: "Dedicated 24/7 SLA",
  },
  {
    feature: "License",
    os: "Apache 2.0",
    ent: "Enterprise terms",
  },
];

export interface FAQItem {
  q: string;
  a: string;
}

export const FAQ_ITEMS: FAQItem[] = [
  {
    q: "Is VantaDB really free?",
    a: "Yes. The core engine is Apache 2.0 licensed and free forever. No hidden pricing, no per-query fees, no artificial limits on self-hosted instances.",
  },
  {
    q: "Can I use VantaDB commercially?",
    a: "Yes. The Apache 2.0 license allows unrestricted use, modification, and distribution. No royalties, no attribution required for local self-hosted deployments.",
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
    q: "Is there a vector limit in Self-Hosted?",
    a: "Self-Hosted has no artificial caps — your only limit is your hardware. We believe in free, unrestricted access to vector search.",
  },
];
