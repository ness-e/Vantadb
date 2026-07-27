import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Case Studies · VantaDB",
  description:
    "Casos de estudio reales de VantaDB: agentes con memoria local + Ollama, RAG air-gapped en edge devices, semantic code search en VS Code. Métricas y quotes.",
  openGraph: {
    title: "Case Studies · VantaDB",
    description:
      "Casos de estudio VantaDB: agentes con memoria local, RAG air-gapped en edge, semantic code search en VS Code.",
    url: "https://vantadb.dev/case-studies",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/case-studies",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
