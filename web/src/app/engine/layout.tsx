import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Core Engine · VantaDB — Rust + PyO3 Pipeline",
  description:
    "Motor core de VantaDB escrito en Rust con bindings PyO3: pipeline BM25 + HNSW fusionado vía RRF, WAL con CRC32C, in-process, sin red.",
  openGraph: {
    title: "Core Engine · VantaDB — Rust + PyO3 Pipeline",
    description:
      "Motor Rust + PyO3 de VantaDB: pipeline BM25+HNSW→RRF, WAL CRC32C, in-process.",
    url: "https://vantadb.dev/engine",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/engine",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
