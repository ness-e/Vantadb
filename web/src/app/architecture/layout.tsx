import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Architecture · VantaDB — Retrieval Pipeline",
  description:
    "Arquitectura del pipeline de recuperación híbrida de VantaDB: BM25 para keywords, HNSW para vectores, fusión RRF, WAL crash-safe. Diagrama y detalles técnicos.",
  openGraph: {
    title: "Architecture · VantaDB — Retrieval Pipeline",
    description:
      "Pipeline híbrido VantaDB: BM25 + HNSW + RRF, WAL crash-safe, in-process. Diagrama y arquitectura.",
    url: "https://vantadb.vercel.app/architecture",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/architecture",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
