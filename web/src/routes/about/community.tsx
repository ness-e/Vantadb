import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/community")({
  head: () => ({
    meta: [
      { title: "VantaDB — Community" },
      {
        name: "description",
        content:
          "Join the VantaDB community. Contribute on GitHub, discuss on Discord, ask questions, and help shape the future of embedded AI data infrastructure.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/community" }],
  }),
});
