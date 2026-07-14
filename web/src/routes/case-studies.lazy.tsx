import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/case-studies.css";

export const Route = createLazyRoute("/case-studies")({
  component: CaseStudiesPage,
  pendingComponent: PendingComponent,
});

function CaseStudiesPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p05"
        title={
          <span>
            Case
            <br />
            Studies
          </span>
        }
        sub="Real-world stories of teams using VantaDB for AI agents, local RAG, edge AI, and more."
      />

      <NbSection ariaLabel="Case studies">
        <NbSectionHeader
          monoLabel="[CASE STUDIES]"
          headline="Coming soon."
          sub="We're documenting real-world VantaDB deployments. Check back soon."
        />

        <div className="nc-cs-empty">
          <p className="nc-cs-empty-text">
            Have a story to share?{" "}
            <a
              href="https://github.com/ness-e/Vantadb/discussions"
              className="nb-link"
              target="_blank"
              rel="noopener noreferrer"
            >
              Tell us about it
            </a>
            .
          </p>
        </div>
      </NbSection>
    </div>
  );
}
