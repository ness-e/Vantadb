import { Suspense } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  Outlet,
  Link,
  createRootRouteWithContext,
  useRouter,
  useMatches,
} from "@tanstack/react-router";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

import { NbNav } from "../components/NbNav";
import "../styles/nb-nav.css";
import { NbFooter } from "../components/NbFooter";
import { NbBackToTop } from "../components/NbBackToTop";
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

function ErrorComponent({ error, reset }: { error: Error; reset: () => void }) {
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
      { property: "og:title", content: "VantaDB — Embedded Vector Database for AI Agents" },
      {
        property: "og:description",
        content:
          "Open-source embedded vector database for AI agents. HNSW + BM25 + hybrid search in a single Rust binary. Apache 2.0. Sub-millisecond hybrid queries, zero infrastructure.",
      },
      { property: "og:url", content: "https://vantadb.dev" },
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
      { rel: "preconnect", href: "https://fonts.googleapis.com" },
      { rel: "preconnect", href: "https://fonts.gstatic.com", crossOrigin: "anonymous" },
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

  useGSAP(
    () => {
      const mm = gsap.matchMedia();

      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.from(".route-content", { opacity: 0, y: 8, duration: 0.25, ease: "power2.out" });

        const sections = gsap.utils.toArray<HTMLElement>(".nb-page-section");
        sections.forEach((section) => {
          const eyebrow = section.querySelector<HTMLElement>(".nb-eyebrow");
          const heading = section.querySelector<HTMLElement>("h2");

          if (eyebrow) {
            gsap.fromTo(
              eyebrow,
              { clipPath: "inset(0 0 100% 0)", opacity: 0 },
              {
                clipPath: "inset(0)",
                opacity: 1,
                duration: 0.3,
                ease: "cubic-bezier(0.25, 1, 0.5, 1)",
                scrollTrigger: { trigger: section, start: "top 80%" },
              },
            );
          }

          if (heading) {
            gsap.fromTo(
              heading,
              { opacity: 0, y: 12 },
              {
                opacity: 1,
                y: 0,
                duration: 0.3,
                ease: "cubic-bezier(0.25, 1, 0.5, 1)",
                scrollTrigger: { trigger: section, start: "top 80%" },
              },
            );
          }
        });
      });

      return () => {
        ScrollTrigger.getAll().forEach((t) => t.kill());
        mm.revert();
      };
    },
    { dependencies: [routeId] },
  );

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
