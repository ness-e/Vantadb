import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "In-Process Latency · VantaDB — 1.2ms p50 HNSW",
  description:
    "VantaDB Rust Core: 1.2ms p50 (HNSW · 10K), 39.74ms Python SDK. Cloud DBs: no medidos por el harness local. Zero network round-trips. Latencia in-process.",
  openGraph: {
    title: "In-Process Latency · VantaDB — 1.2ms p50",
    description:
      "Rust Core 1.2ms p50 (HNSW · 10K), Python SDK 39.74ms p50. Cloud DBs no medidos localmente. Zero network round-trips.",
    url: "https://vantadb.dev/latency",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/latency",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
