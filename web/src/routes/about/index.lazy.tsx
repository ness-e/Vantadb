import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../../styles/about.css";

export const Route = createLazyRoute("/about/")({
  component: AboutIndex,
  pendingComponent: PendingComponent,
});

const STATS = [
  { value: "Apache 2.0", label: "License" },
  { value: "1.2ms", label: "p50 Latency" },
  { value: "Rust", label: "Core Engine" },
  { value: "0.998", label: "Recall@10" },
];

const NAV_SECTIONS = [
  {
    num: "01",
    title: "Company",
    desc: "Who we are, our values, and why we build VantaDB.",
    href: "/about/company",
  },
  {
    num: "02",
    title: "Community",
    desc: "Join the community. Contribute, ask questions, and help shape the future of embedded AI data.",
    href: "/about/community",
  },
  {
    num: "03",
    title: "Contact",
    desc: "Enterprise inquiries, partnerships, or just to say hi.",
    href: "/about/contact",
  },
];

function AboutIndex() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="06"
        title={
          <span>
            The database that
            <br />
            thinks with you.
          </span>
        }
        sub="We're building the data infrastructure for the AI era — embedded, open-source, and engineered for sub-millisecond performance."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-grid nb-grid--cols-4">
            {STATS.map((s) => (
              <div key={s.label} className="nb-cell about-stat-card">
                <span className="about-stat-value">{s.value}</span>
                <span className="about-stat-label">{s.label}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3 about-nav-grid">
            {NAV_SECTIONS.map((s) => (
              <Link key={s.num} to={s.href as "/"} className="nb-cell about-nav-card">
                <span className="about-nav-number">{s.num}</span>
                <h2 className="about-nav-title">{s.title}</h2>
                <p className="about-nav-desc">{s.desc}</p>
                <span className="nb-arrow about-nav-arrow">{s.href}</span>
              </Link>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
