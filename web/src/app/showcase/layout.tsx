import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Community Showcase · VantaDB — Proyectos de la comunidad",
    description:
    "Reference examples e integraciones con VantaDB: LangGraph, AutoGen, CrewAI, Haystack, Rust hybrid search, GraphRAG.",
  openGraph: {
    title: "Community Showcase · VantaDB",
    description:
      "Reference examples que usan VantaDB como motor local-first. Agent memory, RAG pipelines, integraciones con frameworks.",
    url: "https://vantadb.vercel.app/showcase",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/showcase",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
