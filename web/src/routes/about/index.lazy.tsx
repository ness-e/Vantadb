import { createLazyRoute, Link } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

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
      <SwissSubpageHero
        num="06"
        eyebrow="About VantaDB"
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
              <div
                key={s.label}
                className="nb-cell"
                style={{ padding: "var(--space-lg) var(--space-xl)" }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-metric)",
                    fontWeight: 800,
                    letterSpacing: "var(--tracking-tight)",
                    color: "var(--foreground)",
                    lineHeight: 1,
                    display: "block",
                  }}
                >
                  {s.value}
                </span>
                <span
                  className="nb-label"
                  style={{ marginTop: "var(--space-3xs)", marginBottom: 0 }}
                >
                  {s.label}
                </span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">01 / 01 — Navigation</span>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3" style={{ marginTop: "var(--space-xl)" }}>
            {NAV_SECTIONS.map((s) => (
              <Link
                key={s.num}
                to={s.href as "/"}
                className="nb-cell"
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-sm)",
                  padding: "var(--space-xl)",
                  textDecoration: "none",
                  borderLeft: "2px solid transparent",
                  transition: "all 150ms var(--ease-brutal)",
                  background: "var(--background)",
                  borderLeftColor: "transparent",
                }}
              >
                <span className="nb-label nb-label--amber" style={{ marginBottom: 0 }}>
                  {s.num}
                </span>
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 800,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {s.title}
                </h2>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "var(--text-code)",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                    flex: 1,
                  }}
                >
                  {s.desc}
                </p>
                <span className="nb-arrow" style={{ marginTop: "auto" }}>
                  {s.href}
                </span>
              </Link>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
