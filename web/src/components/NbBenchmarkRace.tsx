import { useRef, useEffect, useState } from "react";
import { gsap } from "../lib/gsap";
import "../styles/benchmark-race.css";
import { NbSection, NbSectionHeader } from "./nb";

const GROUPS = [
  {
    id: "hybrid",
    title: "Hybrid Query \u2014 p50 Latency (\u00b5s)",
    bars: [
      { label: "VantaDB", value: "1.2ms", pct: 15, amber: true },
      { label: "Chroma", value: "4.8ms", pct: 45, amber: false },
      { label: "Pinecone", value: "7.3ms", pct: 65, amber: false },
      { label: "Qdrant", value: "3.1ms", pct: 30, amber: false },
    ],
  },
  {
    id: "recall",
    title: "Recall@10 (higher is better)",
    bars: [
      { label: "VantaDB", value: "0.998", pct: 98, amber: true },
      { label: "Chroma", value: "0.945", pct: 90, amber: false },
      { label: "SQLite + vec0", value: "0.890", pct: 82, amber: false },
    ],
  },
];

export function NbBenchmarkRace() {
  const sectionRef = useRef<HTMLElement>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const el = sectionRef.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { threshold: 0.2 },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!visible) return;
    const ctx = gsap.context(() => {
      const fills = gsap.utils.toArray<HTMLElement>(".nb-bm-bar-fill");
      if (!fills.length) return;
      gsap.to(fills, {
        width: (i) => fills[i].dataset.target ?? "0%",
        duration: 0.25,
        stagger: 0.05,
        ease: "power2.out",
      });
    }, sectionRef);
    return () => ctx.revert();
  }, [visible]);

  return (
    <NbSection ref={sectionRef} ariaLabel="Benchmarks">
      <div className="nb-bm-intro">
        <NbSectionHeader monoLabel="[BENCHMARKS]" headline="VantaDB vs the field." />
      </div>

      <div className="nb-bm-body">
        <div className="nb-bm-lead">
          <span className="nb-bm-flag">VANTA DB</span>
          <div className="nb-bm-stats">
            <div className="nb-card nb-card--offset-amber">
              <span className="nb-bm-stat-val">1.2ms</span>
              <span className="nb-bm-stat-lbl">Hybrid query p50</span>
            </div>
            <div className="nb-card nb-card--offset-amber">
              <span className="nb-bm-stat-val">0.998</span>
              <span className="nb-bm-stat-lbl">Recall@10</span>
            </div>
          </div>
        </div>

        <div className="nb-vert-divider" />

        <div className="nb-bm-charts">
          {GROUPS.map((group) => (
            <div key={group.id} className="nb-bm-group">
              <span className="nb-bm-group-title">{group.title}</span>
              <div className="nb-bm-bars">
                {group.bars.map((bar) => (
                  <div key={bar.label} className="nb-bm-bar-row">
                    <span className="nb-bm-bar-label">{bar.label}</span>
                    <div className="nb-bm-bar-track">
                      <div
                        className={`nb-bm-bar-fill ${bar.amber ? "nb-bm-bar-fill--amber" : "nb-bm-bar-fill--steel"}`}
                        style={{ width: bar.amber ? `${bar.pct}%` : "0%" }}
                        data-target={`${bar.pct}%`}
                      />
                    </div>
                    <span className="nb-bm-bar-value">{bar.value}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </NbSection>
  );
}
