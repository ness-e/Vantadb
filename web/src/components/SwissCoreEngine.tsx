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
      // Stagger reveal of rows
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
        }
      );
    },
    { scope: sectionRef }
  );

  return (
    <section
      ref={sectionRef}
      style={{
        background: "#0a0a0a",
        color: "#f0f0f0",
        position: "relative",
        padding: "120px 0 160px 0",
      }}
    >
      <div className="swiss-inner">
        {/* Title — NO eyebrow per budget */}
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "var(--text-display)",
            fontWeight: 700,
            margin: "0 0 80px 0",
            letterSpacing: "-0.04em",
            color: "#f0f0f0",
          }}
        >
          Exploded Architecture.
        </h2>

        {/* Stacked Accordion Rows */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            borderTop: "1px solid rgba(255,255,255,0.08)",
          }}
        >
          {FEATURES.map((feat) => {
            const isExpanded = expandedId === feat.id;

            return (
              <div
                key={feat.id}
                className="swiss-ce-row"
                onClick={() => setExpandedId(isExpanded ? null : feat.id)}
                style={{
                  borderBottom: "1px solid rgba(255,255,255,0.08)",
                  padding: "32px 0",
                  cursor: "pointer",
                  transition: "all 200ms cubic-bezier(0.25, 1, 0.5, 1)",
                }}
                onMouseEnter={(e) => {
                  const num = e.currentTarget.querySelector(
                    ".swiss-ce-num"
                  ) as HTMLElement;
                  if (num) num.style.color = "#ff5500";
                }}
                onMouseLeave={(e) => {
                  const num = e.currentTarget.querySelector(
                    ".swiss-ce-num"
                  ) as HTMLElement;
                  if (num && expandedId !== feat.id) num.style.color = "#555555";
                }}
              >
                {/* Row Header */}
                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "80px 1fr auto",
                    alignItems: "center",
                    gap: "24px",
                  }}
                >
                  {/* Number */}
                  <span
                    className="swiss-ce-num"
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "var(--text-label)",
                      fontWeight: 600,
                      color: isExpanded ? "#ff5500" : "#555555",
                      letterSpacing: "0.1em",
                      transition: "color 150ms",
                    }}
                  >
                    [{feat.num}]
                  </span>

                  {/* Title */}
                  <h3
                    style={{
                      margin: 0,
                      fontFamily: "var(--font-display)",
                      fontSize: "clamp(1.5rem, 3vw, 2.2rem)",
                      fontWeight: 600,
                      color: "#f0f0f0",
                      letterSpacing: "-0.02em",
                    }}
                  >
                    {feat.title}
                  </h3>

                  {/* Expand indicator */}
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "1.2rem",
                      color: "#555555",
                      transition: "transform 200ms cubic-bezier(0.25, 1, 0.5, 1)",
                      transform: isExpanded ? "rotate(45deg)" : "rotate(0deg)",
                    }}
                  >
                    +
                  </span>
                </div>

                {/* Expanded Content */}
                <div
                  style={{
                    maxHeight: isExpanded ? "200px" : "0px",
                    overflow: "hidden",
                    transition:
                      "max-height 300ms cubic-bezier(0.25, 1, 0.5, 1), opacity 200ms",
                    opacity: isExpanded ? 1 : 0,
                  }}
                >
                  <p
                    style={{
                      margin: 0,
                      paddingTop: "24px",
                      paddingLeft: "104px",
                      fontFamily: "var(--font-sans)",
                      fontSize: "1.05rem",
                      color: "#808080",
                      lineHeight: 1.65,
                      maxWidth: "600px",
                    }}
                  >
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
