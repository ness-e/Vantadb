import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/about/contact")({
  head: () => ({
    meta: [
      { title: "VantaDB — Contact" },
      {
        name: "description",
        content:
          "Get in touch with the VantaDB team. Enterprise inquiries, partnerships, security reports, or general questions.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/contact" }],
  }),
});
