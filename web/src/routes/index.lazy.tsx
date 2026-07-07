import { createLazyRoute } from "@tanstack/react-router";
import { NbArchPreview } from "@/components/NbArchPreview";
import { NbBenchmarkRace } from "@/components/NbBenchmarkRace";
import { NbCoreEngine } from "@/components/NbCoreEngine";
import { NbEcosystem } from "@/components/NbEcosystem";
import { NbFaqAccordion } from "@/components/NbFaqAccordion";
import { NbFeatureGrid } from "@/components/NbFeatureGrid";
import { NbMetricsBar } from "@/components/NbMetricsBar";
import { NbMonolith } from "@/components/NbMonolith";
import { NbPricingPreview } from "@/components/NbPricingPreview";
import { NbQuickstart } from "@/components/NbQuickstart";
import { NbTerminalHero } from "@/components/NbTerminalHero";
import { NbTrustBar } from "@/components/NbTrustBar";
import { NbUseCases } from "@/components/NbUseCases";
import "../styles/terminal-hero.css";
import "../styles/trust-bar.css";
import "../styles/core-engine.css";
import "../styles/quickstart.css";
import "../styles/feature-grid.css";
import "../styles/arch-preview.css";
import "../styles/benchmark-race.css";
import "../styles/use-cases.css";
import "../styles/ecosystem.css";
import "../styles/faq-accordion.css";
import "../styles/pricing-preview.css";
import "../styles/monolith.css";

export const Route = createLazyRoute("/")({
  component: IndexPage,
  pendingComponent: PendingComponent,
});

function IndexPage() {
  return (
    <main className="nb-page-content" aria-label="Landing page content">
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
