import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Maintenance · VantaDB — Zero Ops",
  description:
    "Mantenimiento de VantaDB: zero ops, self-healing vía WAL, single-file portátil, sin clusters, sin upgrades dolorosos. Apache 2.0, pip install, olvídate del DBA.",
  openGraph: {
    title: "Maintenance · VantaDB — Zero Ops",
    description:
      "Zero maintenance con VantaDB: self-healing WAL, single-file, sin clusters. Apache 2.0, pip install.",
    url: "https://vantadb.vercel.app/maint",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/maint",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
