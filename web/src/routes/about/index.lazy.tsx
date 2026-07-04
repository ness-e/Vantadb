import { createLazyRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
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
  const [hoveredSection, setHoveredSection] = useState<string | null>(null);
  return (
    <div className="engine-page">
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

      <main className="engine-main">
        <section className="engine-section engine-section--bordered">
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(4, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
            }}
          >
            {STATS.map((s) => (
              <div
                key={s.label}
                style={{
                  background: "var(--background)",
                  padding: "2rem 2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.25rem",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "2.5rem",
                    fontWeight: 800,
                    letterSpacing: "-0.05em",
                    color: "var(--foreground)",
                    lineHeight: 1,
                  }}
                >
                  {s.value}
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {s.label}
                </span>
              </div>
            ))}
          </div>
        </section>

        <section className="engine-section">
          <span className="swiss-eyebrow">01 / 01 — Navigation</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {NAV_SECTIONS.map((s) => (
              <Link
                key={s.num}
                to={s.href as "/"}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                  padding: "2.5rem",
                  textDecoration: "none",
                  borderLeft: "2px solid transparent",
                  transition: "all 150ms var(--ease-cut)",
                  ...(hoveredSection === s.num
                    ? { background: "var(--surface-raised)", borderLeftColor: "var(--amber)" }
                    : { background: "var(--background)", borderLeftColor: "transparent" }),
                }}
                onMouseEnter={() => setHoveredSection(s.num)}
                onMouseLeave={() => setHoveredSection(null)}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {s.num}
                </span>
                <h2
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.4rem",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {s.title}
                </h2>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.8rem",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                    flex: 1,
                  }}
                >
                  {s.desc}
                </p>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.65rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {s.href} →
                </span>
              </Link>
            ))}
          </div>
        </section>
      </main>
    </div>
  );
}
