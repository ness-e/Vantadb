import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Sub-Millisecond Latency · VantaDB — 1.2ms p50 In-Process",
  description:
    "VantaDB Rust Core: 1.2ms p50, 39.74ms Python SDK, vs 200ms+ cloud DBs. Zero network round-trips. Latencia sub-millisecond in-process.",
  openGraph: {
    title: "Sub-Millisecond Latency · VantaDB — 1.2ms p50",
    description:
      "Rust Core 1.2ms p50, Python SDK ~39.74ms p50, cloud DBs 200ms+. Zero network round-trips.",
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
