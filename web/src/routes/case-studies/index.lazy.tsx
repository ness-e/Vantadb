import { createLazyRoute, Link } from "@tanstack/react-router";
import { useMemo, useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import { getAllCaseStudies } from "@/lib/case-studies";
import "../../styles/case-studies.css";

export const Route = createLazyRoute("/case-studies/")({
  component: CaseStudiesPage,
  pendingComponent: PendingComponent,
});

function CaseStudiesPage() {
  const studies = useMemo(() => getAllCaseStudies(), []);
  const listRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = listRef.current?.querySelectorAll<HTMLElement>(".nc-cs-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, listRef);

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

      <NbSection ref={listRef} ariaLabel="Case studies">
        <NbSectionHeader
          monoLabel="[CASE STUDIES]"
          headline={`${studies.length} real-world deployments.`}
          sub="In-depth technical stories from teams using VantaDB in production."
        />

        {studies.length === 0 ? (
          <div className="nc-cs-empty nc-cs-part">
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
        ) : (
          <div className="nc-cs-list nc-cs-part">
            {studies.map((study) => (
              <Link
                key={study.slug}
                to="/case-studies/$slug"
                params={{ slug: study.slug }}
                className="nb-cell nc-cs-link"
              >
                <span className="nb-mono-label">{study.date}</span>

                <div className="nc-cs-body">
                  <h2 className="nb-card-frame-title">{study.title}</h2>
                  {study.description && <p className="nb-card-frame-desc">{study.description}</p>}
                  <div className="nc-cs-tags">
                    {study.author && (
                      <span className="nc-cs-tag nc-cs-tag--author">{study.author}</span>
                    )}
                    {study.tags?.map((t) => (
                      <span key={t} className="nc-cs-tag">
                        {t}
                      </span>
                    ))}
                  </div>
                </div>

                <span className="nb-arrow nc-cs-arrow" />
              </Link>
            ))}
          </div>
        )}
      </NbSection>
    </div>
  );
}
