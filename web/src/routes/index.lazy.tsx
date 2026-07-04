import { createLazyRoute } from "@tanstack/react-router";

import { SwissHero } from "@/components/SwissHero";
import { SwissMetricsBar } from "@/components/SwissMetricsBar";
import { SwissCoreEngine } from "@/components/SwissCoreEngine";
import { SwissQuickstart } from "@/components/SwissQuickstart";
import { SwissArchSection } from "@/components/SwissArchSection";
import { SwissBenchmarkGrid } from "@/components/SwissBenchmarkGrid";
import { SwissUseCases } from "@/components/SwissUseCases";
import { SwissEcosystem } from "@/components/SwissEcosystem";
import { SwissMonolith } from "@/components/SwissMonolith";

export const Route = createLazyRoute("/")({
  component: IndexPage,
  pendingComponent: PendingComponent,
});

function IndexPage() {
  return (
    <main className="page-content">
      <SwissHero />
      <SwissMetricsBar />
      <SwissCoreEngine />
      <SwissQuickstart />
      <SwissArchSection />
      <SwissBenchmarkGrid />
      <SwissUseCases />
      <SwissEcosystem />
      <SwissMonolith />
    </main>
  );
}

export function PendingComponent() {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <div>Loading...</div>
    </div>
  );
}
