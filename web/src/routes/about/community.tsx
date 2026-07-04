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
      { property: "og:title", content: "VantaDB — Community & Contributing" },
      {
        property: "og:description",
        content: "Join the VantaDB community.",
      },
      { property: "og:url", content: "https://vantadb.dev/about/community" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/community" }],
  }),
});
