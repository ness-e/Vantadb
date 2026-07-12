import { type QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  Link,
  Outlet,
  useMatches,
  useRouter,
} from "@tanstack/react-router";
import { Suspense, useEffect } from "react";
import { NbNav } from "../components/NbNav";
import { animate, inView } from "motion";
import { useReducedMotion } from "../hooks/useReducedMotion";
import "../styles/nb-nav.css";
import { NbBackToTop } from "../components/NbBackToTop";
import { NbFooter } from "../components/NbFooter";
import { NbToastContainer } from "../components/nb";
import { PendingComponent } from "../components/PendingComponent";
import { ScrollProgress } from "../components/ScrollProgress";

function NotFoundComponent() {
  return (
    <div className="nb-not-found-container">
      <div className="nb-not-found-card">
        <h1 className="nb-not-found-code">404</h1>
        <hr className="nb-hairline nb-not-found-divider" />
        <p className="nb-not-found-text">This page doesn't exist.</p>
        <Link to="/" className="nb-btn">
          BACK HOME
        </Link>
      </div>
    </div>
  );
}

function ErrorComponent({ error: _error, reset }: { error: Error; reset: () => void }) {
  const router = useRouter();

  return (
    <div className="nb-error-container">
      <div className="nb-error-card">
        <hr className="nb-hairline nb-error-divider" />
        <p className="nb-error-text">Something went wrong. You can try again or go home.</p>
        <div className="nb-error-actions">
          <button
            onClick={() => {
              router.invalidate();
              reset();
            }}
            className="nb-btn"
          >
            TRY AGAIN
          </button>
          <a href="/" className="nb-btn nb-btn--ghost">
            GO HOME
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
      { property: "og:locale", content: "en_US" },
      { property: "og:image", content: "https://vantadb.dev/og/default.svg" },
      { property: "og:image:width", content: "1200" },
      { property: "og:image:height", content: "630" },
      { name: "twitter:card", content: "summary_large_image" },
      { name: "twitter:site", content: "@vantadb" },
      { name: "twitter:creator", content: "@vantadb" },
      { name: "twitter:image", content: "https://vantadb.dev/og/default.svg" },
      { name: "theme-color", content: "#ff5500" },
    ],
    links: [
      { rel: "icon", href: "/favicon.svg?v=2", type: "image/svg+xml" },
      { rel: "canonical", href: "https://vantadb.dev" },
    ],
    scripts: [
      {
        type: "application/ld+json",
        children: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "SoftwareApplication",
          name: "VantaDB",
          url: "https://vantadb.dev",
          applicationCategory: "DatabaseApplication",
          operatingSystem: "Linux, macOS, Windows",
          softwareVersion: "0.2.0",
          description:
            "Open-source embedded memory engine unifying vector search (HNSW), lexical search (BM25), and hybrid search (RRF) in a single Rust binary.",
          featureList: "HNSW vector search, BM25 full-text, hybrid RRF, WAL durability",
          releaseNotes: "https://vantadb.dev/changelog",
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
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    if (reducedMotion) return;

    const routeContent = document.querySelector(".route-content");
    if (routeContent) {
      animate(
        routeContent,
        { opacity: [0, 1], y: [8, 0] },
        { duration: 0.25, ease: [0.25, 0.46, 0.45, 0.94] },
      );
    }

    const cleanups: (() => void)[] = [];
    const sections = document.querySelectorAll<HTMLElement>(".nb-page-section");
    sections.forEach((section) => {
      const eyebrow = section.querySelector<HTMLElement>(".nb-eyebrow");
      const heading = section.querySelector<HTMLElement>("h2");

      if (eyebrow) {
        const cu = inView(
          section,
          () => {
            animate(
              eyebrow,
              { clipPath: ["inset(0 0 100% 0)", "inset(0)"], opacity: [0, 1] },
              { duration: 0.3, ease: [0.25, 1, 0.5, 1] },
            );
          },
          { amount: 0.3 },
        );
        cleanups.push(cu ?? (() => {}));
      }

      if (heading) {
        const cu = inView(
          section,
          () => {
            animate(
              heading,
              { opacity: [0, 1], y: [12, 0] },
              { duration: 0.3, ease: [0.25, 1, 0.5, 1] },
            );
          },
          { amount: 0.3 },
        );
        cleanups.push(cu ?? (() => {}));
      }
    });

    return () => {
      cleanups.forEach((c) => c());
    };
  }, [routeId]);

  return (
    <QueryClientProvider client={queryClient}>
      <div className="nb-page">
        <ScrollProgress />
        <a href="#main-content" className="skip-link">
          Skip to main content
        </a>
        <NbNav />
        <Suspense fallback={<PendingComponent />}>
          <div id="main-content" className="route-content">
            <Outlet />
          </div>
        </Suspense>
        <NbFooter />
        <NbBackToTop />
        <NbToastContainer />
      </div>
    </QueryClientProvider>
  );
}
