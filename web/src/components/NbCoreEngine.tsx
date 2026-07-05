import { useRef } from "react";
import { gsap } from "../lib/gsap";
import { useAnimationSafe } from "../hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "../lib/gsap-utils";
import "../styles/core-engine.css";
import { NbSection, NbSectionHeader } from "./nb";

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

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;

    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(sectionRef.current, 60),
    });

    parts.forEach((part) => {
      tl.add(fadeUp(part, { stagger: 0 }), "-=0.15");
    });
  }, sectionRef);

  return (
    <NbSection ref={sectionRef} variant="lg" ariaLabel="Engine layers">
      <NbSectionHeader monoLabel="[LAYERS]" headline="Engine breakdown." />

      <div className="nb-engine-grid">
        {LAYERS.map((layer, i) => (
          <div key={layer.label} className="nb-engine-col">
            <div className="nb-engine-col-head">
              <span className="nb-engine-col-num">{String(i + 1).padStart(2, "0")}</span>
              <span className="nb-engine-col-label">{layer.label}</span>
            </div>
            <div className="nb-engine-col-body">
              {layer.features.map((feat) => (
                <article key={feat.num} className="nb-engine-part">
                  <span className="nb-num-marker">{feat.num}</span>
                  <h3 className="nb-engine-part-title">{feat.title}</h3>
                  <p className="nb-engine-part-desc">{feat.desc}</p>
                </article>
              ))}
            </div>
          </div>
        ))}
      </div>
    </NbSection>
  );
}
