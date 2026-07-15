import { createLazyRoute, Link } from "@tanstack/react-router";
import DOMPurify from "dompurify";
import { useRef } from "react";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { useScrollReveal } from "@/hooks/useScrollReveal";
import { fadeUp } from "@/lib/motion-utils";
import { getCaseStudyBySlug } from "../../lib/case-studies";
import "../../styles/case-studies.css";

export const Route = createLazyRoute("/case-studies/$slug")({
  component: CaseStudyPage,
});

function CaseStudyPage() {
  const { slug } = Route.useParams();
  const study = getCaseStudyBySlug(slug);
  const postRef = useRef<HTMLElement>(null);

  useScrollReveal();

  useAnimationSafe(() => {
    const parts = postRef.current?.querySelectorAll<HTMLElement>(".nc-cs-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, postRef);

  if (!study) {
    return (
      <div className="nb-page">
        <NbSection ariaLabel="Case study not found">
          <div className="nb-frame nc-cs-part">
            <span className="nb-card-frame-title">Case study not found</span>
            <br />
            <Link to="/case-studies" className="nb-arrow">
              Back to case studies
            </Link>
          </div>
        </NbSection>
      </div>
    );
  }

  return (
    <div className="nb-page">
      <NbSection ref={postRef} ariaLabel="Case study">
        <div className="nc-cs-breadcrumb">
          <span>Case Studies</span>
          <span>{study.slug}</span>
        </div>

        <NbSectionHeader
          monoLabel={study.date}
          headline={study.title}
          sub={study.description || undefined}
        />

        <div className="nc-cs-meta">
          {study.author && <span className="nc-cs-author">By {study.author}</span>}
          {study.tags?.map((t) => (
            <span key={t} className="nc-cs-meta-tag">
              {t}
            </span>
          ))}
        </div>

        <div className="nb-divider" />

        <div
          className="article-body nc-cs-body-text nc-cs-part"
          dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(study.html) }}
        />

        <div className="nb-divider nc-cs-divider" />

        <nav className="nc-cs-nav">
          <Link to="/case-studies" className="nb-arrow">
            Back to case studies
          </Link>
        </nav>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="nc-cs-pending">
      <span className="nc-cs-pending-text">Loading...</span>
    </div>
  );
}
