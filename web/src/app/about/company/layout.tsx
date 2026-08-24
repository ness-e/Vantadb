import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Company · VantaDB",
  description:
    "Sobre VantaDB: misión local-first, motor Apache 2.0 open source, founder ness-e. Software que respeta tu data, tu hardware y tu cuenta de resultados.",
  openGraph: {
    title: "Company · VantaDB",
    description:
      "Misión VantaDB: local-first, Apache 2.0, open source. Fundado por ness-e. Respeta tu data y tu hardware.",
    url: "https://vantadb.vercel.app/about/company",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/about/company",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
