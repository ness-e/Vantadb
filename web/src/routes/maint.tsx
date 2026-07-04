import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/maint")({
  head: () => ({
    meta: [
      { title: "VantaDB — Zero Maintenance Operations" },
      {
        name: "description",
        content:
          "No daemons to monitor, no clusters to scale, no patches to schedule. VantaDB runs as an embedded library — your app IS the database server.",
      },      { property: "og:title", content: "VantaDB — Zero Maintenance Operations" },
      {
        property: "og:description",
        content: "No daemons, no clusters, no pager duty.",
      },
      { property: "og:url", content: "https://vantadb.dev/maint" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/maint" }],
  }),
});



