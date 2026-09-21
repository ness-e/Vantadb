import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Security · VantaDB — Crash-Safe & Local-First",
  description:
    "Seguridad de VantaDB: 6 pilares — WAL con CRC32C crash-safe, zero network, Rust memory-safe, SBOM, recovery verificable, single-file portátil. Local-first.",
  openGraph: {
    title: "Security · VantaDB — Crash-Safe & Local-First",
    description:
      "6 pilares de seguridad VantaDB: WAL CRC32C, zero network, Rust memory-safe, SBOM, recovery verificable.",
    url: "https://vantadb.vercel.app/security",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/security",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
