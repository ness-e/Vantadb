import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Local RAG · VantaDB — Air-Gapped Retrieval",
  description:
    "VantaDB para Local RAG: air-gapped, funciona con Ollama, hybrid search BM25+HNSW+RRF sin red, WAL crash-safe. RAG on-device para agentes y edge, zero egress.",
  openGraph: {
    title: "Local RAG · VantaDB — Air-Gapped Retrieval",
    description:
      "Local RAG air-gapped con VantaDB + Ollama: hybrid search sin red, WAL crash-safe, zero egress.",
    url: "https://vantadb.vercel.app/solutions/local-rag",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/solutions/local-rag",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
