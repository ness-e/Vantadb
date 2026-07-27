import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Community Showcase · VantaDB — Proyectos de la comunidad",
  description:
    "Showcase de proyectos comunitarios construidos con VantaDB: AI Agent Memory, Local RAG Chatbot, Edge Diagnostics, Code Search Tool y más.",
  openGraph: {
    title: "Community Showcase · VantaDB",
    description:
      "Proyectos comunitarios que usan VantaDB como motor local-first. Agent memory, RAG, edge diagnostics, code search.",
    url: "https://vantadb.dev/showcase",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/showcase",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
