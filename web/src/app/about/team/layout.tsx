import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Team · VantaDB",
  description:
    "Equipo de VantaDB: founder ness-e y comunidad open source. Ingeniería de sistemas, bases de datos embebidas, Rust. Contribuye en GitHub y Discord.",
  openGraph: {
    title: "Team · VantaDB",
    description:
      "Equipo VantaDB: founder ness-e y comunidad open source. Sistemas, bases embebidas, Rust.",
    url: "https://vantadb.dev/about/team",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/about/team",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
