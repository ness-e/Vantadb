import { createFileRoute } from "@tanstack/react-router";

import { SwissHero } from "@/components/SwissHero";
import { SwissBenchmarkGrid } from "@/components/SwissBenchmarkGrid";
import { SwissQuickstart } from "@/components/SwissQuickstart";
import { SwissCoreEngine } from "@/components/SwissCoreEngine";
import { SwissArchSection } from "@/components/SwissArchSection";
import { SwissUseCases } from "@/components/SwissUseCases";
import { SwissEcosystem } from "@/components/SwissEcosystem";
import { SwissMonolith } from "@/components/SwissMonolith";

// ── Page Route ─────────────────────────────────────────────────────────────
export const Route = createFileRoute("/")(
  {
    head: () => ({
      meta: [
        { title: "VantaDB — Embedded Cognitive Memory for AI Agents" },
        {
          name: "description",
          content:
            "Open-source embedded vector database. HNSW + BM25 + hybrid search in one Rust binary. Sub-millisecond hybrid queries. Zero infrastructure. Apache 2.0.",
        },
        { property: "og:title", content: "VantaDB — Embedded Cognitive Memory for AI Agents" },
        {
          property: "og:description",
          content:
            "HNSW + BM25 + hybrid search in one Rust binary. Sub-millisecond hybrid queries.",
        },
      ],
    }),
    component: IndexPage,
  },
);

// ── Index Page ─────────────────────────────────────────────────────────────
// Section contrast pattern from DiseñoNuevo.md §12:
//   [Warm Paper] Hero
//   [Warm Paper] VantaDB vs The Stack
//   [Warm Paper] Quickstart
//   [OLED Black] Core Engine (Exploded Architecture)
//   [Warm Paper] Architecture (Blueprint Cross-Section)
//   [Warm Paper] Use Cases
//   [Warm Paper] Ecosystem
//   [OLED Black] CTA Monolith → Footer (also OLED)
function IndexPage() {
  return (
    <main className="page-content" id="main-content">
      {/* 01 — Hero: Typographic Grid Hero */}
      <SwissHero />

      {/* 02 — VantaDB vs The Stack: Bento Benchmark Grid */}
      <SwissBenchmarkGrid />

      {/* 03 — Quickstart: Precision Terminal */}
      <SwissQuickstart />

      {/* 04 — Core Engine: Exploded Architecture (OLED Black) */}
      <SwissCoreEngine />

      {/* 05 — Architecture: Blueprint Cross-Section */}
      <SwissArchSection />

      {/* 06 — Use Cases: Case Study Cards */}
      <SwissUseCases />

      {/* 07 — Ecosystem: Integration Matrix */}
      <SwissEcosystem />

      {/* 08 — CTA Monolith (OLED Black) */}
      <SwissMonolith />
    </main>
  );
}
