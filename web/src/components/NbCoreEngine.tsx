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

const LAYERS = [
  { label: "QUERY", features: FEATURES.slice(0, 3) },
  { label: "ENGINE", features: [FEATURES[3], FEATURES[5]] },
  { label: "STORAGE", features: [FEATURES[4]] },
] as const;

export function NbCoreEngine() {
  const sectionRef = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
        if (!parts.length) return;

        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: sectionRef.current,
            start: "top 60%",
          },
        });

        parts.forEach((part) => {
          tl.fromTo(
            part,
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
    <section ref={sectionRef} className="nb-section nb-section--lg" aria-label="Core engine">
      <div className="nb-inner">
        <span className="nb-mono-label">[ARCHITECTURE]</span>
        <h2 className="nb-section-headline">Engine breakdown.</h2>

        <div className="nb-engine-diagram">
          {LAYERS.map((layer, i) => (
            <div key={layer.label}>
              {i > 0 && <div className="nb-hairline--strong" />}
              <div className="nb-engine-layer">
                <span className="nb-engine-layer-label">{layer.label}</span>
                <div className="nb-engine-components">
                  {layer.features.map((feat) => (
                    <article key={feat.num} className="nb-engine-part">
                      <h3>{feat.title}</h3>
                      <p>{feat.desc}</p>
                    </article>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
