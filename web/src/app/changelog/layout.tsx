import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Changelog · VantaDB — Release History",
  description:
    "Historial de releases de VantaDB: v0.1.0 lanzamiento inicial, v0.1.1 fixes de WAL, v0.1.2 optimizaciones SIFT1M. Cambios, features y mejoras por versión.",
  openGraph: {
    title: "Changelog · VantaDB — Release History",
    description:
      "Releases de VantaDB: v0.1.0, v0.1.1, v0.1.2. Features, fixes y optimizaciones por versión.",
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
