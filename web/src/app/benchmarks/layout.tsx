import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Benchmarks · VantaDB — BENCH-01 & SIFT1M Performance",
  description:
    "Resultados BENCH-01 y SIFT1M de VantaDB: SDK 74 ops/s de ingestión, HNSW p50 1.2ms (10K), QPS peak 3,636 (SIFT1M 100K). Benchmarks reproducibles en Rust + PyO3 in-process.",
  openGraph: {
    title: "Benchmarks · VantaDB — BENCH-01 & SIFT1M Performance",
    description:
      "Benchmarks BENCH-01 y SIFT1M: 74 ops/s ingestión, HNSW p50 1.2ms (10K), QPS peak 3,636.",
    url: "https://vantadb.dev/benchmarks",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/benchmarks",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
