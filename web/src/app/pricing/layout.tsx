import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Pricing · VantaDB — Open Source + Team + Enterprise",
  description:
    "Pricing de VantaDB: 3 planes. Community $0 Apache 2.0 para siempre, Team $49/dev seat con soporte, Enterprise con SLA on-prem. Sin facturación por query.",
  openGraph: {
    title: "Pricing · VantaDB — Open Source + Team + Enterprise",
    description:
      "3 planes VantaDB: Community $0, Team $49/dev seat, Enterprise SLA. Sin facturación por query.",
    url: "https://vantadb.dev/pricing",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/pricing",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
