import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/cost")({
  head: () => ({
    meta: [
      { title: "VantaDB — Infrastructure Cost: Zero Runtime" },
      {
        name: "description",
        content:
          "Eliminate $200+/mo infrastructure costs. VantaDB runs in-process with zero cloud dependencies and no per-query pricing.",
      },
      { property: "og:title", content: "VantaDB — Infrastructure Cost: Zero Runtime" },
      {
        property: "og:description",
        content: "VantaDB runs in your process with zero server infrastructure.",
      },
      { property: "og:url", content: "https://vantadb.dev/cost" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/cost" }],
  }),
});
