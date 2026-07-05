import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/pricing")({
  head: () => ({
    meta: [
      { title: "VantaDB — Pricing: Open Source. Free Forever." },
      {
        name: "description",
        content:
          "VantaDB is open source (Apache 2.0) and free forever. Self-hosted is fully available; Cloud plans are aspirational (coming in a future release).",
      },
      { property: "og:title", content: "VantaDB — Pricing: Open Source. Free Forever." },
      {
        property: "og:description",
        content: "VantaDB is open source (Apache 2.0) and free forever.",
      },
      { property: "og:url", content: "https://vantadb.dev/pricing" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/pricing" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "Product",
          name: "VantaDB",
          applicationCategory: "DatabaseApplication",
          description: "VantaDB is open source (Apache 2.0) and free forever.",
          offers: {
            "@type": "Offer",
            price: "0",
            priceCurrency: "USD",
            availability: "https://schema.org/InStock",
          },
        }),
      },
    ],
  }),
});
