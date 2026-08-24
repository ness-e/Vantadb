import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Quickstart · VantaDB — Install & 5-Minute Guide",
  description:
    "Guía de instalación y quickstart de VantaDB en 5 minutos: pip install vantadb-py, ejemplos Python put/search, CLI vanta-cli y playground interactivo.",
  openGraph: {
    title: "Quickstart · VantaDB — Install & 5-Minute Guide",
    description:
      "Instala VantaDB con pip, corre put/search en Python, prueba la CLI vanta-cli y el playground. Quickstart en 5 minutos.",
    url: "https://vantadb.vercel.app/docs",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/docs",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
