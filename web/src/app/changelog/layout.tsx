import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Changelog · VantaDB — Release History",
  description:
    "Historial de releases de VantaDB: v0.5.0 índice IVF Flat y compaction LSM multi-nivel, v0.4.0 lanzamiento público inicial. Cambios, features y mejoras por versión.",
  openGraph: {
    title: "Changelog · VantaDB — Release History",
    description:
      "Releases de VantaDB: v0.5.0, v0.4.0. Features, fixes y optimizaciones por versión.",
    url: "https://vantadb.vercel.app/changelog",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/changelog",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
