import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "AI Agents · VantaDB — Persistent Agent Memory",
  description:
    "VantaDB para AI Agents: memoria persistente con put/search híbrido, crash-safe, namespaces por agente, recuperación BM25+HNSW+RRF in-process. Olvida el cloud.",
  openGraph: {
    title: "AI Agents · VantaDB — Persistent Agent Memory",
    description:
      "Memoria persistente para AI agents con VantaDB: put/search híbrido, crash-safe, namespaces por agente.",
    url: "https://vantadb.vercel.app/solutions/ai-agents",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/solutions/ai-agents",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
