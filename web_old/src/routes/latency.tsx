import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/latency")({
  head: () => ({
    meta: [
      { title: "VantaDB — Sub-Millisecond Latency" },
      {
        name: "description",
        content:
          "Rust Core 1.2ms p50 (Python SDK ~39.74ms p50) in-process latency vs 200ms+ for cloud vector databases. VantaDB eliminates network round-trips.",
      },
      { property: "og:title", content: "VantaDB — Sub-Millisecond Latency" },
      {
        property: "og:description",
        content: "1.2ms p50 query latency in Rust Core.",
      },
      { property: "og:url", content: "https://vantadb.dev/latency" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/latency" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Sub-Millisecond Latency",
          description:
            "Rust Core 1.2ms p50 (Python SDK ~39.74ms p50) in-process latency vs 200ms+ for cloud vector databases. VantaDB eliminates network round-trips.",
          url: "https://vantadb.dev/latency",
          image: "https://vantadb.dev/og/default.svg",
        }),
      },
    ],
  }),
});
