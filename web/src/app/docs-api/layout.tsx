import type { Metadata } from "next";

// This route permanently redirects to /docs.
export const metadata: Metadata = {
  title: "VantaDB API Docs → /docs",
  description: "Redirigiendo a la documentación de VantaDB.",
  robots: { index: false, follow: true },
  alternates: {
    canonical: "https://vantadb.vercel.app/docs",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
