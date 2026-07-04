import React from "react";

interface SwissSubpageHeroProps {
  num: string;
  eyebrow: string;
  title: React.ReactNode;
  sub?: string;
}

const SwissSubpageHero: React.FC<SwissSubpageHeroProps> = React.memo(
  ({ num, eyebrow, title, sub }) => {
    return (
      <header className="swiss-subpage-hero" role="banner">
        <div className="swiss-inner">
          <div className="swiss-subpage-hero-grid">
            <div className="swiss-subpage-hero-meta">
              <span className="swiss-subpage-hero-num" aria-hidden="true">[{num}]</span>
              <span className="swiss-subpage-hero-eyebrow">{eyebrow}</span>
            </div>
            <div>
              <h1 className="swiss-subpage-hero-title">{title}</h1>
              {sub && <p className="swiss-subpage-hero-desc">{sub}</p>}
            </div>
          </div>
        </div>
      </header>
    );
  },
);

export { SwissSubpageHero };
