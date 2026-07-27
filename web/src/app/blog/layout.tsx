import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Blog · VantaDB",
  description:
    "Blog de VantaDB: artículos sobre hybrid search (BM25 + HNSW + RRF), local-first software, motor Rust, WAL crash-safe y casos de uso reales con agentes y RAG.",
  openGraph: {
    title: "Blog · VantaDB",
    description:
      "Artículos VantaDB sobre hybrid search, local-first, motor Rust, WAL crash-safe y casos de uso reales.",
    url: "https://vantadb.dev/blog",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/blog",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
