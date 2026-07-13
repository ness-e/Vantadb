import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/team")({
  head: () => ({
    meta: [
      { title: "VantaDB Team — The People Behind the Database" },
      {
        name: "description",
        content:
          "Meet the team building VantaDB — open-source embedded vector database for AI agents.",
      },
      {
        property: "og:title",
        content: "VantaDB Team — The People Behind the Database",
      },
      {
        property: "og:description",
        content: "Meet the team building VantaDB.",
      },
      { property: "og:url", content: "https://vantadb.dev/about/team" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/team" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "AboutPage",
          name: "VantaDB Team — The People Behind the Database",
          description:
            "Meet the team building VantaDB — open-source embedded vector database for AI agents.",
          url: "https://vantadb.dev/about/team",
          image: "https://vantadb.dev/og/default.svg",
        }),
      },
    ],
  }),
});
