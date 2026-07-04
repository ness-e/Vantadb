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
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/latency" }],
  }),
});
