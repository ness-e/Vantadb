import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/security")({
  head: () => ({
    meta: [
      { title: "VantaDB Security Posture" },
      {
        name: "description",
        content: "Security is a first-class citizen. Learn how VantaDB protects agent memory.",
      },
      { property: "og:title", content: "VantaDB — Embedded Security Posture" },
      {
        property: "og:description",
        content:
          "Security-first embedded database for AI agents. WAL integrity, zero-network attack surface, PHI-safe local storage.",
      },
      { property: "og:url", content: "https://vantadb.dev/security" },
      { property: "og:title", content: "VantaDB — Embedded Security Posture" },
      {
        property: "og:description",
        content: "Security-first embedded database for AI agents.",
      },
      { property: "og:url", content: "https://vantadb.dev/security" },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/security" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "WebPage",
          name: "VantaDB Security Posture",
          description:
            "Security is a first-class citizen. Learn how VantaDB protects agent memory.",
        }),
      },
    ],
  }),
});
