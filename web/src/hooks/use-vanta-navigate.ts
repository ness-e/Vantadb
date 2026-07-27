"use client";

import { useCallback } from "react";
import { useRouter } from "next/navigation";
import type { View } from "@/components/vanta/vanta-data";

/**
 * useVantaNavigate — adapter that bridges the existing `onNavigate: (v: View) => void`
 * interface used by 25+ components (Hero, Footer, HomeView, etc.) to the Next.js
 * App Router. Maps a View ("home" | "benchmarks" | "docs") to a real URL path and
 * uses `router.push()` + `window.scrollTo(0,0)`.
 *
 * This lets all existing view components work unchanged in the App Router —
 * each route page just calls this hook and passes `navigate` to the view.
 */
export function useVantaNavigate() {
  const router = useRouter();

  return useCallback(
    (v: View) => {
      const path = v === "home" ? "/" : `/${v}`;
      router.push(path);
      if (typeof window !== "undefined") {
        window.scrollTo({ top: 0, behavior: "auto" });
      }
    },
    [router]
  );
}

/**
 * Routes that are "live" (have a real page.tsx) in the current phase.
 * Used by the navbar to decide between navigate vs. "coming soon" toast.
 * F1 Tier 1: /, /benchmarks, /docs.
 * F2 Tier 2: /engine, /architecture, /playground, /why-vantadb, /changelog,
 *            /pricing, /security, /use-cases, /cost, /maint,
 *            /solutions/ai-agents, /solutions/local-rag, /solutions/ai-ide-tooling
 * F4 Tier 3: /blog, /blog/[slug], /case-studies, /case-studies/[slug],
 *            /about/company, /about/team, /about/community, /about/contact
 */
export const LIVE_ROUTES = new Set([
  "/",
  "/benchmarks",
  "/docs",
  "/engine",
  "/architecture",
  "/playground",
  "/why-vantadb",
  "/changelog",
  "/pricing",
  "/security",
  "/use-cases",
  "/cost",
  "/maint",
  "/solutions/ai-agents",
  "/solutions/local-rag",
  "/solutions/ai-ide-tooling",
  "/blog",
  "/blog/introducing-vantadb",
  "/blog/how-hybrid-search-works",
  "/blog/sqlite-for-ai-agents",
  "/blog/why-i-built-vantadb-local-memory-engine",
  "/case-studies",
  "/case-studies/agent-local-memory-ollama",
  "/case-studies/rag-edge-device",
  "/case-studies/ide-semantic-search",
  "/about/company",
  "/about/team",
  "/about/community",
  "/about/contact",
  // F7 — diseño2 missing routes
  "/config",
  "/demo",
  "/docs-api",
  "/integrations",
  "/latency",
  "/showcase",
  "/storage",
]);

/**
 * Check if a path is live, supporting dynamic segments.
 * For /blog/[slug] and /case-studies/[slug], checks the prefix.
 */
export function isLiveRoute(path: string): boolean {
  if (LIVE_ROUTES.has(path)) return true;
  // Dynamic blog posts
  if (path.startsWith("/blog/")) return true;
  // Dynamic case studies
  if (path.startsWith("/case-studies/")) return true;
  return false;
}
