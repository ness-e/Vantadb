import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/changelog")({
  head: () => ({
    meta: [
      { title: "VantaDB — Changelog" },
      {
        name: "description",
        content:
          "Release notes for VantaDB. Track new features, performance improvements, bug fixes, and breaking changes across versions.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/changelog" }],
  }),
});
