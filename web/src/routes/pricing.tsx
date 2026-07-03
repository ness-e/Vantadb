import { createFileRoute } from "@tanstack/react-router";

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
    links: [{ rel: "canonical", href: "https://vantadb.dev/pricing" }],
  }),
});
