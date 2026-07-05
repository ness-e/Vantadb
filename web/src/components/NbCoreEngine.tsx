import { useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";
import "../styles/core-engine.css";

const FEATURES = [
  {
    num: "01",
    title: "Rust Core",
    desc: "Memory safety without garbage collection pauses. Deterministic performance under heavy multi-threaded loads.",
  },
  {
    num: "02",
    title: "HNSW Index",
    desc: "Hierarchical Navigable Small World graphs for sub-millisecond high-dimensional vector search.",
  },
  {
    num: "03",
    title: "BM25 Engine",
    desc: "Full-text search engine built-in. Combine keyword accuracy with semantic recall in a single query.",
  },
  {
    num: "04",
    title: "WAL Durability",
    desc: "Write-Ahead Logging guarantees zero data loss on crashes. Automatic recovery on process restart.",
  },
  {
    num: "05",
    title: "PyO3 Bridge",
    desc: "Native Python bindings with zero IPC overhead. Calls directly into Rust memory space.",
  },
  {
    num: "06",
    title: "Zero-Copy Serde",
    desc: "Data serialization that bypasses the Python GIL. Zero-copy deserialization for massive throughput.",
  },
];

export function NbCoreEngine() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const cells = gsap.utils.toArray<HTMLElement>(".nb-core-cell");
        if (!cells.length) return;

        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: sectionRef.current,
            start: "top 60%",
          },
        });

        cells.forEach((cell) => {
          tl.fromTo(
            cell,
            { opacity: 0, y: 24 },
            { opacity: 1, y: 0, duration: 0.4, ease: "cubic-bezier(0.05, 0.95, 0.3, 1)" },
            "-=0.15",
          );
        });
      });
    },
    { scope: sectionRef },
  );

  return (
    <section
      ref={sectionRef}
      className="nb-section nb-section--lg"
      aria-label="Core engine architecture"
    >
      <div className="nb-inner">
        <div className="nb-section-header nb-section-header--bordered">
          <h2 className="nb-core-title">Exploded Architecture.</h2>
        </div>

        <div className="nb-core-grid" role="list">
          {FEATURES.map((feat) => (
            <article key={feat.num} className="nb-core-cell" role="listitem">
              <h3 className="nb-core-feature-title">{feat.title}</h3>
              <p className="nb-core-feature-desc">{feat.desc}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
