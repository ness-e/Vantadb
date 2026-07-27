import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  head: () => ({
    meta: [
      { title: "VantaDB — Embedded Cognitive Memory for AI Agents" },
      {
        name: "description",
        content:
          "Open-source embedded vector database. HNSW + BM25 + hybrid search in one Rust binary. Sub-millisecond hybrid queries. Zero infrastructure. Apache 2.0.",
      },
      { property: "og:title", content: "VantaDB — Embedded Cognitive Memory for AI Agents" },
      {
        property: "og:description",
        content: "HNSW + BM25 + hybrid search in one Rust binary. Sub-millisecond hybrid queries.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "SoftwareApplication",
          name: "VantaDB",
          url: "https://vantadb.dev",
          image: "https://vantadb.dev/og/default.svg",
          softwareVersion: "0.2.0",
          applicationCategory: "DatabaseApplication",
          operatingSystem: "Linux, macOS, Windows",
          description:
            "Open-source embedded vector database for AI agents. HNSW + BM25 hybrid search in one Rust binary.",
          offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
        }),
      },
    ],
  }),
});
