import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Cost · VantaDB — TCO vs Cloud Vector DBs",
  description:
    "Comparativa de TCO VantaDB vs cloud vector DBs: $0/mo vs $1,800/mo Pinecone + egress. Sin servidores, sin facturación por query, sin ops. Calculadora incluida.",
  openGraph: {
    title: "Cost · VantaDB — TCO vs Cloud Vector DBs",
    description:
      "TCO VantaDB vs Pinecone/Weaviate: $0/mo vs $1,800/mo. Sin servidores ni facturación por query.",
    url: "https://vantadb.dev/cost",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/cost",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
