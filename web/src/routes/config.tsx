import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/config")({
  head: () => ({
    meta: [
      { title: "VantaDB — Zero Configuration: Schema-Free" },
      {
        name: "description",
        content:
          "No YAML files, no .env secrets, no migration scripts. VantaDB requires zero configuration — just pip install and connect.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/config" }],
  }),
});
