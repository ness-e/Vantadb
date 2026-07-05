import { createLazyRoute } from "@tanstack/react-router";
import { NbTerminalHero } from "@/components/NbTerminalHero";
import { NbTrustBar } from "@/components/NbTrustBar";
import { NbMetricsBar } from "@/components/NbMetricsBar";
import { NbFeatureGrid } from "@/components/NbFeatureGrid";
import { NbCoreEngine } from "@/components/NbCoreEngine";
import { NbQuickstart } from "@/components/NbQuickstart";
import { NbArchPreview } from "@/components/NbArchPreview";
import { NbBenchmarkRace } from "@/components/NbBenchmarkRace";
import { NbUseCases } from "@/components/NbUseCases";
import { NbEcosystem } from "@/components/NbEcosystem";
import { NbFaqAccordion } from "@/components/NbFaqAccordion";
import { NbPricingPreview } from "@/components/NbPricingPreview";
import { NbMonolith } from "@/components/NbMonolith";
import "../styles/terminal-hero.css";
import "../styles/trust-bar.css";
import "../styles/feature-grid.css";
import "../styles/arch-preview.css";
import "../styles/benchmark-race.css";
import "../styles/faq-accordion.css";
import "../styles/pricing-preview.css";

export const Route = createLazyRoute("/")({
  component: IndexPage,
  pendingComponent: PendingComponent,
});

function IndexPage() {
  return (
    <main className="nb-page-content">
      <NbTerminalHero />
      <NbTrustBar />
      <NbMetricsBar />
      <NbFeatureGrid />
      <NbCoreEngine />
      <NbQuickstart />
      <NbArchPreview />
      <NbBenchmarkRace />
      <NbUseCases />
      <NbEcosystem />
      <NbFaqAccordion />
      <NbPricingPreview />
      <NbMonolith />
    </main>
  );
}

export function PendingComponent() {
  return (
    <div className="nb-pending">
      <div>Loading...</div>
    </div>
  );
}
