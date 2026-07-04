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
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/cost" }],
  }),
});
