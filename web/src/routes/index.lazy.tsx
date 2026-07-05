import { createLazyRoute } from "@tanstack/react-router";
import { NbTerminalHero } from "@/components/NbTerminalHero";
import { NbMetricsBar } from "@/components/NbMetricsBar";
import { NbCoreEngine } from "@/components/NbCoreEngine";
import { NbQuickstart } from "@/components/NbQuickstart";
import { NbArchSection } from "@/components/NbArchSection";
import { NbBenchmarkGrid } from "@/components/NbBenchmarkGrid";
import { NbUseCases } from "@/components/NbUseCases";
import { NbEcosystem } from "@/components/NbEcosystem";
import { NbMonolith } from "@/components/NbMonolith";
import "../styles/terminal-hero.css";

export const Route = createLazyRoute("/")({
  component: IndexPage,
  pendingComponent: PendingComponent,
});

function IndexPage() {
  return (
    <main className="nb-page-content">
      <NbTerminalHero />
      <NbMetricsBar />
      <NbCoreEngine />
      <NbQuickstart />
      <NbArchSection />
      <NbBenchmarkGrid />
      <NbUseCases />
      <NbEcosystem />
      <NbMonolith />
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
