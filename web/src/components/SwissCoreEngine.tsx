import { useRef, useState } from "react";
import { gsap, useGSAP } from "../lib/gsap";

const FEATURES = [
  {
    id: "rust",
    num: "01",
    title: "Rust Core",
    desc: "Memory safety without garbage collection pauses. Deterministic performance under heavy multi-threaded loads.",
    icon: "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
  },
  {
    id: "hnsw",
    num: "02",
    title: "HNSW Index",
    desc: "Hierarchical Navigable Small World graphs for sub-millisecond high-dimensional vector search.",
    icon: "M4 4h16v16H4zM9 9h6v6H9z",
  },
  {
    id: "bm25",
    num: "03",
    title: "BM25 Engine",
    desc: "Full-text search engine built-in. Combine keyword accuracy with semantic recall in a single query.",
    icon: "M4 6h16M4 12h16M4 18h16",
  },
  {
    id: "wal",
    num: "04",
    title: "WAL Durability",
    desc: "Write-Ahead Logging guarantees zero data loss on crashes. Automatic recovery on process restart.",
    icon: "M20 12H4M12 4v16",
  },
  {
    id: "pyo3",
    num: "05",
    title: "PyO3 Bridge",
    desc: "Native Python bindings with zero IPC overhead. Calls directly into Rust memory space.",
    icon: "M8 4h8M4 8h16M4 16h16M8 20h8",
  },
  {
    id: "serde",
    num: "06",
    title: "Zero-Copy Serde",
    desc: "Data serialization that bypasses the Python GIL. Zero-copy deserialization for massive throughput.",
    icon: "M21 12a9 9 0 11-18 0 9 9 0 0118 0zM12 8v4l3 3",
  },
];

export function SwissCoreEngine() {
  const sectionRef = useRef<HTMLElement>(null);
  const [expandedId, setExpandedId] = useState<string | null>("rust");

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const rows = gsap.utils.toArray<HTMLElement>(".swiss-ce-row");
        if (!rows.length) return;

        const endPx = rows.length * 280;
        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: sectionRef.current,
            start: "top 20%",
            pin: true,
            anticipatePin: 1,
            end: `+=${endPx}`,
          },
        });

        rows.forEach((row) => {
          const icon = row.querySelector<SVGPathElement>(".swiss-ce-icon-path");
          const text = row.querySelector<HTMLElement>(".swiss-ce-row-desc");

          tl.fromTo(
            row,
            { clipPath: "inset(0 0 100% 0)", opacity: 0 },
            {
              clipPath: "inset(0)",
              opacity: 1,
              duration: 0.35,
              ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            },
          );

          if (icon) {
            tl.fromTo(
              icon,
              { strokeDashoffset: 200 },
              { strokeDashoffset: 0, duration: 0.3, ease: "none" },
              "-=0.2",
            );
          }

          if (text) {
            tl.fromTo(
              text,
              { opacity: 0, y: 8 },
              { opacity: 1, y: 0, duration: 0.2, ease: "cubic-bezier(0.25, 1, 0.5, 1)" },
              "-=0.1",
            );
          }
        });
      });
    },
    { scope: sectionRef },
  );

  return (
    <section ref={sectionRef} className="swiss-section swiss-section--dark swiss-ce-section">
      <div className="swiss-inner">
        <h2 className="swiss-ce-title">
          Exploded Architecture.
        </h2>

        <div className="swiss-ce-accordion">
          {FEATURES.map((feat) => {
            const isExpanded = expandedId === feat.id;

            return (
              <div
                key={feat.id}
                className="swiss-ce-row"
                onClick={() => setExpandedId(isExpanded ? null : feat.id)}
              >
                <div className="swiss-ce-row-header">
                  <div className="swiss-ce-row-title-group">
                    <svg
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="swiss-ce-icon"
                    >
                      <path
                        className="swiss-ce-icon-path"
                        d={feat.icon}
                      />
                    </svg>
                    <h3 className="swiss-ce-row-title">{feat.title}</h3>
                  </div>

                  <span
                    className={`swiss-ce-row-expand ${isExpanded ? "swiss-ce-row-expand--open" : ""}`}
                  >
                    +
                  </span>
                </div>

                <div
                  className={`swiss-ce-row-content ${isExpanded ? "swiss-ce-row-content--open" : ""}`}
                >
                  <p className="swiss-ce-row-desc">{feat.desc}</p>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
