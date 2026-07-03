import { useRef, useState } from "react";
import { gsap, useGSAP } from "../lib/gsap";

const FEATURES = [
  {
    id: "rust",
    num: "01",
    title: "Rust Core",
    desc: "Memory safety without garbage collection pauses. Deterministic performance under heavy multi-threaded loads.",
  },
  {
    id: "hnsw",
    num: "02",
    title: "HNSW Index",
    desc: "Hierarchical Navigable Small World graphs for sub-millisecond high-dimensional vector search.",
  },
  {
    id: "bm25",
    num: "03",
    title: "BM25 Engine",
    desc: "Full-text search engine built-in. Combine keyword accuracy with semantic recall in a single query.",
  },
  {
    id: "wal",
    num: "04",
    title: "WAL Durability",
    desc: "Write-Ahead Logging guarantees zero data loss on crashes. Automatic recovery on process restart.",
  },
  {
    id: "pyo3",
    num: "05",
    title: "PyO3 Bridge",
    desc: "Native Python bindings with zero IPC overhead. Calls directly into Rust memory space.",
  },
  {
    id: "serde",
    num: "06",
    title: "Zero-Copy Serde",
    desc: "Data serialization that bypasses the Python GIL. Zero-copy deserialization for massive throughput.",
  },
];

export function SwissCoreEngine() {
  const sectionRef = useRef<HTMLElement>(null);
  const [expandedId, setExpandedId] = useState<string | null>("rust");

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.fromTo(
          ".swiss-ce-row",
          { opacity: 0, y: 12 },
          {
            opacity: 1,
            y: 0,
            duration: 0.5,
            stagger: 0.06,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            scrollTrigger: {
              trigger: sectionRef.current,
              start: "top 70%",
            },
          },
        );
      });
    },
    { scope: sectionRef },
  );

  return (
    <section
      ref={sectionRef}
      className="swiss-section swiss-section--dark swiss-ce-section"
    >
      <div className="swiss-inner">
        {/* Title — NO eyebrow per budget */}
        <h2 className="swiss-ce-title">
          Exploded Architecture.
        </h2>

        {/* Stacked Accordion Rows */}
        <div className="swiss-ce-accordion">
          {FEATURES.map((feat) => {
            const isExpanded = expandedId === feat.id;

            return (
              <div
                key={feat.id}
                className="swiss-ce-row"
                onClick={() => setExpandedId(isExpanded ? null : feat.id)}
              >
                {/* Row Header */}
                <div className="swiss-ce-row-header">
                  {/* Title */}
                  <h3 className="swiss-ce-row-title">
                    {feat.title}
                  </h3>

                  {/* Expand indicator */}
                  <span
                    className="swiss-ce-row-expand"
                    style={{
                      transform: isExpanded ? "rotate(45deg)" : "rotate(0deg)",
                    }}
                  >
                    +
                  </span>
                </div>

                {/* Expanded Content */}
                <div
                  className="swiss-ce-row-content"
                  style={{
                    maxHeight: isExpanded ? "200px" : "0px",
                    opacity: isExpanded ? 1 : 0,
                  }}
                >
                  <p className="swiss-ce-row-desc">
                    {feat.desc}
                  </p>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
