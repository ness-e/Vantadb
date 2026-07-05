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
          <h2 className="about-community-section-title">Where to Find Us</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-2 about-community-grid-top">
            {CHANNELS.map((ch) => (
              <a
                key={ch.name}
                href={ch.href}
                target={ch.href.startsWith("http") ? "_blank" : undefined}
                rel="noopener noreferrer"
                className="nb-cell about-community-channel-card"
              >
                <span className="about-community-channel-tag">{ch.tag}</span>
                <h3 className="about-community-channel-name">{ch.name}</h3>
                <p className="about-community-channel-desc">{ch.desc}</p>
                <span className="nb-arrow about-community-channel-cta">{ch.cta}</span>
              </a>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="about-community-section-title">Contribute</h2>
          <div className="nb-divider" />
          <p className="about-community-feature-lead">Ways to get involved.</p>

          <div className="nb-grid nb-grid--cols-3">
            {WAYS.map((w) => (
              <div key={w.num} className="nb-cell about-community-way-card">
                <span className="about-community-way-num">{w.num}</span>
                <h3 className="about-community-way-title">{w.title}</h3>
                <p className="about-community-way-desc">{w.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber about-community-cta-block">
            <span className="about-community-cta-label">WANT TO CONTRIBUTE?</span>
            <p className="about-community-cta-desc">Check out our GitHub for open issues.</p>
            <a
              href="https://github.com/ness-e/Vantadb"
              className="nb-btn nb-btn--ghost about-community-cta-btn"
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
    <div className="about-pending-container">
      <span className="about-pending-text">Loading...</span>
    </div>
  );
}
