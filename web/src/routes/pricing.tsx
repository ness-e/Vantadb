import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/pricing")({
  head: () => ({
    meta: [
      { title: "VantaDB — Pricing: Open Source. Free Forever." },
      {
        name: "description",
        content:
          "VantaDB is open source (Apache 2.0) and free forever. Self-hosted is fully available; Cloud plans are in early access.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/pricing" }],
  }),
});
