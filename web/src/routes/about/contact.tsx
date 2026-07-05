import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/contact")({
  head: () => ({
    meta: [
      { title: "VantaDB — Contact" },
      {
        name: "description",
        content:
          "Get in touch with the VantaDB team. Enterprise inquiries, partnerships, security reports, or general questions.",
      },
      { property: "og:title", content: "VantaDB — Contact & Support" },
      {
        property: "og:description",
        content: "Get in touch with the VantaDB team.",
      },
      { property: "og:url", content: "https://vantadb.dev/about/contact" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/contact" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "ContactPage",
          name: "Contact VantaDB",
        }),
      },
    ],
  }),
});
