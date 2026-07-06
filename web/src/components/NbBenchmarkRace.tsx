import { useRef, useEffect, useState } from "react";
import { gsap } from "../lib/gsap";
import "../styles/benchmark-race.css";
import { NbSection, NbSectionHeader } from "./nb";

const GROUPS = [
  {
    id: "hybrid",
    title: "Hybrid Query — p50 Latency (µs)",
    unit: "ms",
    bars: [
      { label: "VantaDB", value: "1.2", pct: 15, rank: 1 },
      { label: "Qdrant", value: "3.1", pct: 30, rank: 2 },
      { label: "Chroma", value: "4.8", pct: 45, rank: 3 },
      { label: "Pinecone", value: "7.3", pct: 65, rank: 4 },
    ],
  },
  {
    id: "recall",
    title: "Recall@10 (higher is better)",
    unit: "",
    bars: [
      { label: "VantaDB", value: "0.998", pct: 98, rank: 1 },
      { label: "Chroma", value: "0.945", pct: 90, rank: 2 },
      { label: "SQLite+vec0", value: "0.890", pct: 82, rank: 3 },
    ],
  },
];

const PODIUM = [
  { label: "VantaDB", icon: "1st", color: "var(--amber)" },
  { label: "Qdrant", icon: "2nd", color: "var(--steel)" },
  { label: "Chroma", icon: "3rd", color: "var(--muted)" },
];

export function NbBenchmarkRace() {
  const sectionRef = useRef<HTMLElement>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const el = sectionRef.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => { if (entry.isIntersecting) { setVisible(true); observer.disconnect(); } },
      { threshold: 0.2 },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!visible) return;
    const ctx = gsap.context(() => {
      const fills = gsap.utils.toArray<HTMLElement>(".nb-rd-bar-fill");
      if (!fills.length) return;
      gsap.to(fills, {
        width: (i) => fills[i].dataset.target ?? "0%",
        duration: 0.4,
        stagger: 0.05,
        ease: "steps(10)",
      });
    }, sectionRef);
    return () => ctx.revert();
  }, [visible]);

  return (
    <NbSection ref={sectionRef} ariaLabel="Benchmarks">
      <NbSectionHeader monoLabel="[RACE DATA]" headline="VantaDB vs the field." sub="Real benchmarks. Reproducible. No marketing numbers." />

      <div className="nb-rd-dash">
        {/* ── Podium ── */}
        <div className="nb-rd-podium">
          <span className="nb-rd-podium-title">OVERALL STANDINGS</span>
          <div className="nb-rd-podium-steps">
            {PODIUM.map((p, i) => (
              <div key={p.label} className="nb-rd-podium-step" style={{ "--step-color": p.color } as React.CSSProperties}>
                <span className="nb-rd-podium-icon">{p.icon}</span>
                <span className="nb-rd-podium-name">{p.label}</span>
              </div>
            ))}
          </div>
        </div>

        {/* ── Charts ── */}
        <div className="nb-rd-charts">
          {GROUPS.map((group) => (
            <div key={group.id} className="nb-rd-group">
              <span className="nb-rd-group-title">{group.title}</span>
              <div className="nb-rd-bars">
                {group.bars.map((bar) => {
                  const isVanta = bar.label === "VantaDB";
                  return (
                    <div key={bar.label} className="nb-rd-bar-row">
                      <div className="nb-rd-bar-head">
                        <span className="nb-rd-bar-label">{bar.label}</span>
                        <span className="nb-rd-rank">#{bar.rank}</span>
                      </div>
                      <div className="nb-rd-bar-track">
                        <div
                          className={`nb-rd-bar-fill ${isVanta ? "nb-rd-bar-fill--amber" : "nb-rd-bar-fill--steel"}`}
                          style={{ width: isVanta ? `${bar.pct}%` : "0%" }}
                          data-target={`${bar.pct}%`}
                        />
                      </div>
                      <span className="nb-rd-bar-value">
                        {bar.value}{bar.unit}
                        {isVanta && <span className="nb-rd-bar-fast">FASTEST</span>}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      </div>

      <a
        href="https://github.com/ness-e/Vantadb/tree/main/benches"
        className="nb-mono-label nb-mono-label--steel nb-rd-src"
      >
        [ View benchmark reproduction script ]
      </a>
    </NbSection>
  );
}
