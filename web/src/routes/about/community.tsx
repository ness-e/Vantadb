import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/about/community")({
  head: () => ({
    meta: [
      { title: "VantaDB — Community" },
      {
        name: "description",
        content:
          "Join the VantaDB community. Contribute on GitHub, discuss on Discord, ask questions, and help shape the future of embedded AI data infrastructure.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/community" }],
  }),
  component: CommunityPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
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
    <div className="engine-page">
      <SwissSubpageHero
        num="06"
        eyebrow="About — Community"
        title={
          <span>
            Built in the open.
            <br />
            With the community.
          </span>
        }
        sub="VantaDB is open source, and the community is at the center of everything we build. Join us on GitHub, Discord, and beyond."
      />

      <main className="engine-main">
        {/* Channels */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Where to Find Us</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {CHANNELS.map((ch) => (
              <a
                key={ch.name}
                href={ch.href}
                target={ch.href.startsWith("http") ? "_blank" : undefined}
                rel="noopener noreferrer"
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.75rem",
                  padding: "2.5rem",
                  background: "var(--background)",
                  textDecoration: "none",
                  borderLeft: "2px solid transparent",
                  transition: "all 150ms var(--ease-cut)",
                }}
                onMouseEnter={(e) => {
                  const el = e.currentTarget as HTMLAnchorElement;
                  el.style.background = "var(--surface-raised)";
                  el.style.borderLeftColor = "var(--amber)";
                }}
                onMouseLeave={(e) => {
                  const el = e.currentTarget as HTMLAnchorElement;
                  el.style.background = "var(--background)";
                  el.style.borderLeftColor = "transparent";
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.55rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {ch.tag}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.3rem",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {ch.name}
                </h3>
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
                  {ch.desc}
                </p>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.65rem",
                    color: "var(--amber)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  {ch.cta} →
                </span>
              </a>
            ))}
          </div>
        </section>

        {/* Ways to contribute */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Contribute</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(1.5rem, 3vw, 2rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            Ways to get involved.
          </h2>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
            }}
          >
            {WAYS.map((w) => (
              <div
                key={w.num}
                style={{
                  background: "var(--background)",
                  padding: "2rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.5rem",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {w.num}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "0.9rem",
                    fontWeight: 700,
                    letterSpacing: "-0.02em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {w.title}
                </h3>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.78rem",
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
        </section>
      </main>
    </div>
  );
}
