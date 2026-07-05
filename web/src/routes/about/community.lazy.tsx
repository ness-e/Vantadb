import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";

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
    title: "Report a bug",
    desc: "Found something broken? Open a GitHub issue with reproduction steps.",
  },
  {
    title: "Submit a PR",
    desc: "Check the good-first-issue label. We review PRs within 48 hours.",
  },
  {
    title: "Write docs",
    desc: "Docs are never done. Fix a typo, clarify a section, add an example.",
  },
  {
    title: "Build an integration",
    desc: "LangChain, LlamaIndex, or your own framework — we'd love to link to it.",
  },
  {
    title: "Share your project",
    desc: "Built something with VantaDB? Let us know and we'll feature it.",
  },
  {
    title: "Run a benchmark",
    desc: "Test VantaDB against your workload and share the results.",
  },
];

function CommunityPage() {
  const channelsRef = useRef<HTMLElement>(null);
  const waysRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(channelsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, channelsRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(waysRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, waysRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p17"
        title={
          <span>
            Built in the open.
            <br />
            With the community.
          </span>
        }
        sub="VantaDB is open source, and the community is at the center of everything we build. Join us on GitHub, Discord, and beyond."
      />

      <NbSection ref={channelsRef} ariaLabel="Where to find us">
        <NbSectionHeader
          monoLabel="[CHANNELS]"
          headline="Where to find us."
          sub="Join the conversation on your platform of choice."
        />

        <div className="nb-engine-part">
          <div className="nb-grid nb-grid--cols-2">
            {CHANNELS.map((ch) => (
              <a
                key={ch.name}
                href={ch.href}
                target={ch.href.startsWith("http") ? "_blank" : undefined}
                rel="noopener noreferrer"
                className="nb-cell nb-card-frame"
              >
                <span className="nb-mono-label">{ch.tag}</span>
                <h3 className="nb-card-frame-title">{ch.name}</h3>
                <p className="nb-card-frame-desc">{ch.desc}</p>
                <span className="nb-arrow nb-card-frame-header">{ch.cta}</span>
              </a>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection ref={waysRef} variant="dark" ariaLabel="Ways to contribute">
        <NbSectionHeader
          monoLabel="[CONTRIBUTE]"
          headline="Ways to get involved."
          sub="Everyone can contribute, regardless of experience."
        />

        <div className="nb-engine-part">
          <div className="nb-grid nb-grid--cols-3">
            {WAYS.map((w) => (
              <div key={w.title} className="nb-cell nb-card-frame">
                <h3 className="nb-card-frame-title">{w.title}</h3>
                <p className="nb-card-frame-desc">{w.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection ariaLabel="Contribute call to action">
        <NbBlockAmber>
          <div className="nb-text-center">
            <span className="nb-mono-label">WANT TO CONTRIBUTE?</span>
            <p className="nb-section-sub">Check out our GitHub for open issues.</p>
            <a href="https://github.com/ness-e/Vantadb" className="nb-btn nb-btn--ghost">
              GITHUB
            </a>
          </div>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="nb-pending">
      <span>Loading...</span>
    </div>
  );
}
