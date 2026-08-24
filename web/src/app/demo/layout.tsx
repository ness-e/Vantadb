import type { Metadata } from "next";

/**
 * /demo — Redirects to /playground (the interactive code playground).
 * Metadata describes what actually happens: the playground is live today,
 * no beta/waitlist promises.
 */
export const metadata: Metadata = {
  title: "Try VantaDB Now · Interactive Playground",
  description:
    "Probá VantaDB en el navegador hoy: playground interactivo con BM25 + HNSW + RRF vía WebAssembly. Sin registro, sin waitlist.",
  openGraph: {
    title: "Try VantaDB Now · Interactive Playground",
    description:
      "Playground interactivo de VantaDB en el navegador: BM25 + HNSW + RRF vía WebAssembly. Disponible ahora.",
    url: "https://vantadb.vercel.app/demo",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/demo",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
