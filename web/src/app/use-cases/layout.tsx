import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Use Cases · VantaDB — AI Agents, RAG, IDE",
  description:
    "Casos de uso de VantaDB: memoria persistente para AI agents, local RAG con Ollama air-gapped, IDE tooling con semantic code search. 3 dominios, 1 engine.",
  openGraph: {
    title: "Use Cases · VantaDB — AI Agents, RAG, IDE",
    description:
      "3 casos de uso de VantaDB: AI agents con memoria, local RAG con Ollama, IDE semantic code search.",
    url: "https://vantadb.dev/use-cases",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/use-cases",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
