import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Contact · VantaDB",
  description:
    "Contacta con VantaDB: Discord para soporte y comunidad, GitHub para issues y PRs, email para enterprise. Respuesta rápida, sin formularios eternos.",
  openGraph: {
    title: "Contact · VantaDB",
    description:
      "Canales de contacto VantaDB: Discord, GitHub issues/PRs, email enterprise. Respuesta rápida.",
    url: "https://vantadb.dev/about/contact",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/about/contact",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
