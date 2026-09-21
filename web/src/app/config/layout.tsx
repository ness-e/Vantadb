import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Zero Configuration · VantaDB — Schema-Free, No YAML, No .env",
  description:
    "VantaDB no requiere schema, ni YAML, ni .env, ni migraciones. Solo `pip install vantadb-py` y connect. In-process, sin configuración, sin secretos.",
  openGraph: {
    title: "Zero Configuration · VantaDB — Schema-Free",
    description:
      "Sin schema, sin YAML, sin .env, sin migraciones. pip install y connect. VantaDB es schema-free por diseño.",
    url: "https://vantadb.vercel.app/config",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.vercel.app/config",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
