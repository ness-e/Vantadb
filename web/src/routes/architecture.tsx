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
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/architecture" }],
  }),
});
