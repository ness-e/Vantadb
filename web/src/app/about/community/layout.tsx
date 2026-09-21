import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Comunidad · VantaDB — Discord & GitHub",
  description:
    "Comunidad VantaDB: contribuye en GitHub, chatea en Discord. Open source Apache 2.0, RFCs públicos, roadmap transparente. Local-first, community-driven.",
  openGraph: {
    title: "Comunidad · VantaDB — Discord & GitHub",
    description:
      "Comunidad VantaDB: GitHub + Discord. Apache 2.0, RFCs públicos, roadmap transparente, community-driven.",
    url: "https://vantadb.vercel.app/about/community",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/about/community",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
