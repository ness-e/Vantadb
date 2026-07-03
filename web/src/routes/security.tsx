import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/security")({
  head: () => ({
    meta: [
      { title: "VantaDB Security Posture" },
      {
        name: "description",
        content: "Security is a first-class citizen. Learn how VantaDB protects agent memory.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/security" }],
  }),
});
