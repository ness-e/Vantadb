import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";

export const Route = createLazyRoute("/about/community")({
  component: CommunityPage,
});

const CHANNELS = [
  {
    name: "GitHub",
    tag: "OPEN SOURCE",
    desc: "Star the repo, open issues, submit PRs, and follow development.",
    cta: "github.com/ness-e/Vantadb",
    href: "https://github.com/ness-e/Vantadb",
  },
  {
    name: "Discord",
    tag: "COMMUNITY",
    desc: "Real-time chat with the team and community. Ask questions, share projects, get help.",
    cta: "Join our Discord",
    href: "#",
  },
  {
    name: "Discussions",
    tag: "GITHUB",
    desc: "Long-form discussions, feature proposals, and Q&A.",
    cta: "Start a discussion",
    href: "https://github.com/ness-e/Vantadb/discussions",
  },
  {
    name: "X / Twitter",
    tag: "UPDATES",
    desc: "Follow for release announcements, benchmarks, and ecosystem news.",
    cta: "@vantadb",
    href: "#",
  },
];

const WAYS = [
  {
    num: "01",
    title: "Report a bug",
    desc: "Found something broken? Open a GitHub issue with reproduction steps.",
  },
  {
    num: "02",
    title: "Submit a PR",
    desc: "Check the good-first-issue label. We review PRs within 48 hours.",
  },
  {
    num: "03",
    title: "Write docs",
    desc: "Docs are never done. Fix a typo, clarify a section, add an example.",
  },
  {
    num: "04",
    title: "Build an integration",
    desc: "LangChain, LlamaIndex, or your own framework — we'd love to link to it.",
  },
  {
    num: "05",
    title: "Share your project",
    desc: "Built something with VantaDB? Let us know and we'll feature it.",
  },
  {
    num: "06",
    title: "Run a benchmark",
    desc: "Test VantaDB against your workload and share the results.",
  },
];

function CommunityPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="06"
        title={
          <span>
            Built in the open.
            <br />
            With the community.
          </span>
        }
        sub="VantaDB is open source, and the community is at the center of everything we build. Join us on GitHub, Discord, and beyond."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Where to Find Us
          </h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "var(--space-xl)" }}>
            {CHANNELS.map((ch) => (
              <a
                key={ch.name}
                href={ch.href}
                target={ch.href.startsWith("http") ? "_blank" : undefined}
                rel="noopener noreferrer"
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
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-micro)",
                    color: "var(--amber)",
                    fontWeight: 700,
                    marginBottom: 0,
                  }}
                >
                  {ch.tag}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 800,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {ch.name}
                </h3>
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
                  {ch.desc}
                </p>
                <span className="nb-arrow" style={{ marginTop: "auto" }}>
                  {ch.cta}
                </span>
              </a>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Contribute
          </h2>
          <div className="nb-divider" />
          <p
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-display)",
              fontWeight: 800,
              letterSpacing: "var(--tracking-display)",
              margin: "var(--space-sm) 0 var(--space-xl)",
              lineHeight: 1.05,
            }}
          >
            Ways to get involved.
          </p>

          <div className="nb-grid nb-grid--cols-3">
            {WAYS.map((w) => (
              <div key={w.num} className="nb-cell" style={{ padding: "var(--space-lg)" }}>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-micro)",
                    color: "var(--muted)",
                    marginBottom: "var(--space-2xs)",
                    display: "block",
                  }}
                >
                  {w.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 700,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: "0 0 var(--space-2xs)",
                  }}
                >
                  {w.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "var(--text-code)",
                    color: "var(--muted)",
                    lineHeight: 1.6,
                    margin: 0,
                  }}
                >
                  {w.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber" style={{ textAlign: "center" }}>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-micro)",
                color: "var(--text-on-amber)",
                marginBottom: "var(--space-2xs)",
                display: "block",
              }}
            >
              WANT TO CONTRIBUTE?
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "var(--text-body)",
                color: "var(--text-on-amber)",
                margin: "var(--space-2xs) 0",
                opacity: 0.85,
              }}
            >
              Check out our GitHub for open issues.
            </p>
            <a
              href="https://github.com/ness-e/Vantadb"
              className="nb-btn nb-btn--ghost"
              style={{
                borderColor: "var(--text-on-amber)",
                color: "var(--text-on-amber)",
                boxShadow: "var(--shadow-brutal)",
              }}
            >
              GITHUB
            </a>
          </div>
        </div>
      </section>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-label)",
          color: "var(--muted)",
          marginBottom: 0,
        }}
      >
        Loading...
      </span>
    </div>
  );
}
