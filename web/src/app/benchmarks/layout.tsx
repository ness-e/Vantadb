import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Benchmarks · VantaDB — BENCH-01 & SIFT1M Performance",
  description:
    "Resultados BENCH-01 y SIFT1M de VantaDB: ingestión 5,400 vec/s, latencia p50 1.2ms HNSW, hybrid search 2.10ms. Benchmarks en Rust + PyO3 in-process.",
  openGraph: {
    title: "Benchmarks · VantaDB — BENCH-01 & SIFT1M Performance",
    description:
      "Benchmarks BENCH-01 y SIFT1M: 5,400 vec/s ingestión, 1.2ms latencia HNSW, hybrid search 2.10ms p50.",
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
