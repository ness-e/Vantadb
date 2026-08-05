import type { Metadata } from "next";
import { Geist, Geist_Mono, Anton, Space_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { LanguageProvider } from "@/lib/language-provider";
import { SiteShell } from "@/components/vanta/site-shell";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

const anton = Anton({
  variable: "--font-anton",
  subsets: ["latin"],
  weight: "400",
  display: "swap",
});

const spaceMono = Space_Mono({
  variable: "--font-space-mono",
  subsets: ["latin"],
  weight: ["400", "700"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "VantaDB — Embedded Rust Engine for Local-First Hybrid Retrieval",
  description:
    "VantaDB is a local-first, embedded Rust database engine for AI agents and local RAG. Persistent memory, crash-safe WAL recovery (CRC32C), and native hybrid search (BM25 + HNSW via RRF) — zero network, in-process, 1.2ms latency.",
  keywords: [
    "VantaDB",
    "vector database",
    "Rust",
    "embedded database",
    "local-first",
    "HNSW",
    "BM25",
    "RRF",
    "hybrid search",
    "RAG",
    "PyO3",
    "WAL",
  ],
  authors: [{ name: "ness-e" }],
  icons: {
    icon: "/assets/avatar_gato.png",
    apple: "/assets/avatar_gato.png",
  },
  manifest: "/manifest.json",
  openGraph: {
    title: "VantaDB — Embedded Rust Engine for Local-First Hybrid Retrieval",
    description:
      "Persistent memory + crash-safe WAL + BM25/HNSW hybrid retrieval via RRF. Zero network. In-process. 1.2ms latency.",
    url: "https://github.com/ness-e/Vantadb",
    siteName: "VantaDB",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "VantaDB — Embedded Rust Hybrid Retrieval Engine",
    description:
      "Local-first embedded Rust engine. BM25 + HNSW via RRF. WAL with CRC32C. 1.2ms in-process latency.",
  },
};

export const viewport = {
  themeColor: "#FF5500",
  width: "device-width",
  initialScale: 1,
};

// Schema.org/SoftwareApplication structured data (JSON-LD). Next.js Metadata API
// mirrors HTML tags only and does not emit JSON-LD — rendered here as a native
// <script> in the root layout's <head>, per the official Next.js JSON-LD guide.
const jsonLd = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "VantaDB",
  applicationCategory: "DatabaseApplication",
  applicationSubCategory: "Vector Database",
  operatingSystem: "Windows, macOS, Linux, WebAssembly",
  description:
    "VantaDB: An embedded persistent memory and vector retrieval engine for local-first AI applications.",
  version: "0.5.0",
  url: "https://github.com/ness-e/Vantadb",
  logo: "https://raw.githubusercontent.com/ness-e/Vantadb/main/web/public/assets/avatar_gato.png",
  offers: {
    "@type": "Offer",
    price: "0",
    priceCurrency: "USD",
  },
  softwareRequirements: "Python >= 3.11 bindings; Rust core MSRV 1.94.1; 64-bit OS",
  license: "https://www.apache.org/licenses/LICENSE-2.0",
  featureList: [
    "Embedded in-process database",
    "Crash-safe WAL recovery (CRC32C)",
    "Native hybrid search (BM25 + HNSW via RRF)",
    "PyO3 Python bindings",
    "WASM build",
    "MCP server",
  ],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="es" suppressHydrationWarning>
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(jsonLd).replace(/</g, "\\u003c"),
          }}
        />
      </head>
      <body
        className={`${geistSans.variable} ${geistMono.variable} ${anton.variable} ${spaceMono.variable} antialiased bg-background text-foreground`}
      >
          <LanguageProvider>
            <a href="#main-content" className="skip-link">
              Saltar al contenido
            </a>
            <SiteShell>{children}</SiteShell>
            <Toaster />
            <Sonner
              position="bottom-right"
              toastOptions={{
                classNames: {
                  toast:
                    "border-4 border-black bg-[#FBF9F5] text-black font-tech text-xs uppercase tracking-wider shadow-[6px_6px_0_0_#000]",
                  title: "font-bold",
                  description: "text-black/60",
                  success: "border-black bg-[#FF5500] text-black",
                },
              }}
            />
          </LanguageProvider>
      </body>
    </html>
  );
}
