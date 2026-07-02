import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  Outlet,
  Link,
  createRootRouteWithContext,
  useRouter,
  useMatches,
} from "@tanstack/react-router";
import { AnimatePresence, motion } from "motion/react";

import { Nav } from "../components/Nav";
import { SwissFooter } from "../components/SwissFooter";
import { SwissBackToTop } from "../components/SwissBackToTop";

function NotFoundComponent() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="max-w-md text-center">
        <h1 className="text-7xl font-bold text-foreground">404</h1>
        <h2 className="mt-4 text-xl font-semibold text-foreground">Page not found</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          The page you're looking for doesn't exist or has been moved.
        </p>
        <div className="mt-6">
          <Link
            to="/"
            className="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
          >
            Go home
          </Link>
        </div>
      </div>
    </div>
  );
}

function ErrorComponent({ error, reset }: { error: Error; reset: () => void }) {
  // Error logged
  const router = useRouter();

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="max-w-md text-center">
        <h1 className="text-xl font-semibold tracking-tight text-foreground">
          This page didn't load
        </h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Something went wrong on our end. You can try refreshing or head back home.
        </p>
        <div className="mt-6 flex flex-wrap justify-center gap-2">
          <button
            onClick={() => {
              router.invalidate();
              reset();
            }}
            className="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
          >
            Try again
          </button>
          <a
            href="/"
            className="inline-flex items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-sm font-medium text-foreground transition-colors hover:bg-accent"
          >
            Go home
          </a>
        </div>
      </div>
    </div>
  );
}

export const Route = createRootRouteWithContext<{ queryClient: QueryClient }>()({
  head: () => ({
    meta: [
      { title: "VantaDB — Embedded Vector Database for AI Agents" },
      {
        name: "description",
        content:
          "Open-source embedded vector database. HNSW + BM25 + hybrid search in one Rust binary. Apache 2.0. Sub-millisecond hybrid queries, zero infrastructure.",
      },
      { property: "og:type", content: "website" },
      { property: "og:site_name", content: "VantaDB" },
      { property: "og:image", content: "https://vantadb.dev/og/default.svg" },
      { property: "og:image:width", content: "1200" },
      { property: "og:image:height", content: "630" },
      { name: "twitter:card", content: "summary_large_image" },
      { name: "twitter:image", content: "https://vantadb.dev/og/default.svg" },
      { name: "theme-color", content: "#ff5500" },
    ],
    links: [{ rel: "icon", href: "/favicon.png", type: "image/png" }],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "SoftwareApplication",
          name: "VantaDB",
          applicationCategory: "DatabaseApplication",
          operatingSystem: "Linux, macOS, Windows",
          description:
            "Open-source embedded memory engine unifying vector search (HNSW), lexical search (BM25), and hybrid search (RRF) in a single Rust binary.",
          license: "Apache-2.0",
          offers: {
            "@type": "Offer",
            price: "0",
            priceCurrency: "USD",
          },
          author: {
            "@type": "Organization",
            name: "VantaDB",
            url: "https://vantadb.dev",
          },
        }),
      },
    ],
  }),
  component: RootComponent,
  notFoundComponent: NotFoundComponent,
  errorComponent: ErrorComponent,
});

function RootComponent() {
  const { queryClient } = Route.useRouteContext();
  const matches = useMatches();
  const routeId = matches[matches.length - 1]?.routeId;

  return (
    <QueryClientProvider client={queryClient}>
      <div className="page-container">
        <Nav />

        {/* Dynamic content with route transitions */}
        <AnimatePresence mode="wait">
          <motion.div
            key={routeId}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.2, ease: [0.25, 1, 0.5, 1] }}
          >
            <Outlet />
          </motion.div>
        </AnimatePresence>

        {/* ── Footer (Swiss OLED) ── */}
        <SwissFooter />

        {/* ── Floating Utilities ── */}
        <SwissBackToTop />
      </div>
    </QueryClientProvider>
  );
}
