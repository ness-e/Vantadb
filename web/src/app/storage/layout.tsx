import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Single-Binary Storage Architecture · VantaDB — LSM-tree + WAL + HNSW",
  description:
    "Arquitectura single-binary: LSM-tree → WAL con CRC32C → HNSW index → SDK boundary PyO3. Reemplaza Pinecone + Redis + S3 con un solo binario Rust.",
  openGraph: {
    title: "Single-Binary Storage Architecture · VantaDB",
    description:
      "LSM-tree + WAL + HNSW + SDK boundary en un solo binario Rust. Reemplaza Pinecone + Redis + S3.",
    url: "https://vantadb.dev/storage",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/storage",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
