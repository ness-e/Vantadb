import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Not Found · VantaDB",
  description:
    "La página que buscas no existe. Explora VantaDB: hybrid search local-first, motor Rust embebido, AI agents, RAG, IDE tooling. Vuelve al inicio.",
  openGraph: {
    title: "Not Found · VantaDB",
    description:
      "La página que buscas no existe. Vuelve al inicio de VantaDB para explorar hybrid search local-first.",
    url: "https://vantadb.dev/404",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/404",
  },
  robots: { index: false, follow: false },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
