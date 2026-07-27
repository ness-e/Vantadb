import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Browser Demo · VantaDB — WebAssembly Preview (Coming Soon)",
  description:
    "Prueba VantaDB en el navegador vía WebAssembly. El demo WASM está en desarrollo — únete a la beta para acceso anticipado.",
  openGraph: {
    title: "Browser Demo · VantaDB — WebAssembly Preview",
    description:
      "Demo WASM de VantaDB en el navegador. Coming soon — únete a la lista de espera de la beta.",
    url: "https://vantadb.dev/demo",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/demo",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
