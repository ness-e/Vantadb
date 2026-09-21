import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "IDE Tooling · VantaDB — Semantic Code Search",
  description:
    "VantaDB para IDE Tooling: semantic code search in-process, AST indexing, BM25 sobre símbolos, HNSW sobre embeddings. Cero servidores, 60MB RAM, starts instant.",
  openGraph: {
    title: "IDE Tooling · VantaDB — Semantic Code Search",
    description:
      "Semantic code search in-process con VantaDB: AST indexing, BM25+HNSW, cero servidores, 60MB RAM.",
    url: "https://vantadb.vercel.app/solutions/ai-ide-tooling",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/solutions/ai-ide-tooling",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
