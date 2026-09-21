import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Pricing · VantaDB — Open Source + Enterprise",
  description:
    "Pricing de VantaDB: 2 planes. Community $0 Apache 2.0 para siempre, Enterprise con SLA on-prem. Sin facturación por query.",
  openGraph: {
    title: "Pricing · VantaDB — Open Source + Enterprise",
    description:
      "2 planes VantaDB: Community $0, Enterprise SLA. Sin facturación por query.",
    url: "https://vantadb.vercel.app/pricing",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/pricing",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
