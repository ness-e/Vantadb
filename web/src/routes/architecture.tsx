import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/architecture")({
  head: () => ({
    meta: [
      { title: "VantaDB — Engine Architecture & Limits" },
      {
        name: "description",
        content:
          "Behind the FFI: stable boundary FFI bindings, Fjall storage layers, concurrency models and hardware limits.",
      },
      { property: "og:title", content: "VantaDB — Engine Architecture & Limits" },
      {
        property: "og:description",
        content: "Direct compilation, shared-memory execution.",
      },
      { property: "og:url", content: "https://vantadb.dev/architecture" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/architecture" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB — Engine Architecture & Limits",
          description:
            "Behind the FFI: stable boundary FFI bindings, Fjall storage layers, concurrency models and hardware limits.",
        }),
      },
    ],
  }),
});
