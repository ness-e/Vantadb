import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/roadmap")({
  head: () => ({
    meta: [
      { title: "VantaDB — Product Roadmap & Milestones" },
      {
        name: "description",
        content:
          "The technical and strategic roadmap for VantaDB. Phase 1-3: Core engine completed. Phase 4: Community launch. Phase 5: Enterprise readiness.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/roadmap" }],
  }),
});
