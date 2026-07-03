import { createLazyRoute } from "@tanstack/react-router";

import { SwissHero } from "@/components/SwissHero";
import { SwissBenchmarkGrid } from "@/components/SwissBenchmarkGrid";
import { SwissQuickstart } from "@/components/SwissQuickstart";
import { SwissCoreEngine } from "@/components/SwissCoreEngine";
import { SwissArchSection } from "@/components/SwissArchSection";
import { SwissUseCases } from "@/components/SwissUseCases";
import { SwissEcosystem } from "@/components/SwissEcosystem";
import { SwissMonolith } from "@/components/SwissMonolith";

export const Route = createLazyRoute("/")({
  component: IndexPage,
  pendingComponent: PendingComponent,
});

function IndexPage() {
  return (
    <main className="page-content" id="main-content">
      <SwissHero />
      <SwissBenchmarkGrid />
      <SwissQuickstart />
      <SwissCoreEngine />
      <SwissArchSection />
      <SwissUseCases />
      <SwissEcosystem />
      <SwissMonolith />
    </main>
  );
}

export function PendingComponent() {
  return (
    <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "60vh", color: "var(--muted)" }}>
      <div>Loading...</div>
    </div>
  );
}
