import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/about/team")({
  component: TeamPage,
  pendingComponent: PendingComponent,
});

const TEAM = [
  {
    avatarUser: "founder",
    avatarStatus: "building",
    name: "Dr. Elena Vasquez",
    role: "Founder & CEO",
    desc: "Distributed systems researcher. Former infrastructure lead at Databricks.",
  },
  {
    avatarUser: "cto",
    avatarStatus: "optimizing",
    name: "Marcus Chen",
    role: "CTO",
    desc: "Rust core team alumni. Built query engines at DuckDB and ClickHouse.",
  },
  {
    avatarUser: "head-ml",
    avatarStatus: "indexing",
    name: "Priya Sharma",
    role: "Head of ML",
    desc: "Vector search at scale. Previously ML infra at Pinecone and Weaviate.",
  },
  {
    avatarUser: "engineer",
    avatarStatus: "compiling",
    name: "Alex Kowalski",
    role: "Lead Engineer",
    desc: "Systems programmer. Contributed to SQLite, LMDB, and RocksDB.",
  },
  {
    avatarUser: "dx",
    avatarStatus: "shipping",
    name: "Yuki Tanaka",
    role: "Developer Experience",
    desc: "Python SDK architect. PyO3 maintainer and open-source advocate.",
  },
  {
    avatarUser: "pm",
    avatarStatus: "connecting",
    name: "Sarah Mitchell",
    role: "Product & Community",
    desc: "Developer relations lead. Built communities at LangChain and Hugging Face.",
  },
];

function TeamPage() {
  const teamRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(teamRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, teamRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p18"
        title="The people behind VantaDB."
        sub="Distributed across 6 time zones. United by one mission: make vector search invisible."
      />

      <NbSection ref={teamRef} ariaLabel="Team members">
        <NbSectionHeader
          monoLabel="[TEAM]"
          headline="The people behind VantaDB."
          sub="Distributed across 6 time zones. United by one mission."
        />

        <div className="nb-engine-part">
          <div className="nb-grid nb-grid--cols-3">
            {TEAM.map((m) => (
              <div key={m.name} className="nb-cell nb-card-frame">
                <div className="nb-dither">
                  <div>{"> user: " + m.avatarUser}</div>
                  <div>{"> status: " + m.avatarStatus}</div>
                </div>

                <div className="nb-card-frame-header">
                  <h3 className="nb-card-frame-title">{m.name}</h3>
                  <span className="nb-mono-label">[{m.role}]</span>
                </div>

                <p className="nb-card-frame-desc">{m.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection variant="dark" ariaLabel="Join the team">
        <NbBlockAmber>
          <div className="nb-text-center">
            <span className="nb-mono-label">JOIN THE TEAM</span>
            <p className="nb-section-sub">
              We're always looking for talented people who share our mission. Say hello.
            </p>
            <Link to="/about/contact" className="nb-btn nb-btn--ghost">
              CONTACT US
            </Link>
          </div>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}
