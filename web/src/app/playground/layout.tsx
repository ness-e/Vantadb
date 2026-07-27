import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Code Playground · VantaDB — Try Hybrid Search",
  description:
    "Playground de código interactivo de VantaDB: prueba BM25 + HNSW + RRF en el navegador. Simulador Python en vivo con ejemplos put/search y métricas de latencia.",
  openGraph: {
    title: "Code Playground · VantaDB — Try Hybrid Search",
    description:
      "Playground interactivo de VantaDB: simula BM25+HNSW+RRF en el navegador con ejemplos Python put/search.",
    url: "https://vantadb.dev/playground",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/playground",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
