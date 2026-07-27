import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/showcase.css";

export const Route = createLazyRoute("/showcase")({
  component: ShowcasePage,
  pendingComponent: PendingComponent,
});

function ShowcasePage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p07"
        title={
          <span>
            Community
            <br />
            Showcase
          </span>
        }
        sub="See what the community is building with VantaDB — AI agents, RAG pipelines, edge applications, and more."
      />

      <NbSection ariaLabel="Showcase projects">
        <NbSectionHeader
          monoLabel="[SHOWCASE]"
          headline="Built with VantaDB."
          sub="Projects, tools, and experiments from the community."
        />

        <div className="nc-showcase-empty">
          <p className="nc-showcase-empty-text">
            No projects showcased yet. Built something with VantaDB?{" "}
            <a
              href="https://github.com/ness-e/Vantadb/discussions"
              className="nb-link"
              target="_blank"
              rel="noopener noreferrer"
            >
              Share it with us
            </a>{" "}
            and we'll feature it here.
          </p>
        </div>
      </NbSection>
    </div>
  );
}
