import { useRef, useEffect, useState } from "react";
import { gsap } from "../lib/gsap";
import "../styles/benchmark-race.css";

const GROUPS = [
  {
    id: "hybrid",
    title: "Hybrid Query — p50 Latency (\u00b5s)",
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
      const bars = gsap.utils.toArray<HTMLElement>(".bm-race-bar-inner");
      if (!bars.length) return;
      gsap.to(bars, {
        width: (i) => bars[i].dataset.target ?? "0%",
        duration: 0.25,
        stagger: 0.05,
        ease: "power2.out",
      });
    }, sectionRef);
    return () => ctx.revert();
  }, [visible]);

  return (
    <section className="nb-section" ref={sectionRef} aria-label="Benchmarks">
      <div className="nb-inner">
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "1.25rem",
            fontWeight: 700,
            color: "var(--amber)",
          }}
        >
          Benchmarks
        </h2>
        <div className="nb-divider" />

        {GROUPS.map((g) => (
          <div key={g.id} className="bm-race-group">
            <h3 className="bm-race-group-title">{g.title}</h3>
            <div className="bm-race-bars">
              {g.bars.map((b) => (
                <div key={b.label} className="bm-race-row">
                  <span className="bm-race-label">{b.label}</span>
                  <div className="bm-race-bar-track">
                    <div
                      className={`bm-race-bar-inner ${b.amber ? "bm-race-bar--amber" : "bm-race-bar--steel"}`}
                      data-target={`${b.pct}%`}
                      style={{ width: 0 }}
                    />
                  </div>
                  <span className="bm-race-value">{b.value}</span>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
