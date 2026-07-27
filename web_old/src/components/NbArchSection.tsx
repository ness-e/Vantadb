import { useEffect, useRef } from "react";
import { animate, inView } from "motion";
import { useReducedMotion } from "../hooks/useReducedMotion";
import "../styles/architecture.css";

const PIPELINE = [
  { id: "app", label: "APP LAYER", name: "Python Application" },
  { id: "pyo3", label: "FFI BOUNDARY", name: "PyO3 Bridge", accent: true },
  { id: "query", label: "QUERY ENGINE", name: "Router + Planner" },
  { id: "index", label: "INDEX LAYER", name: "HNSW / BM25" },
  { id: "wal", label: "DURABILITY", name: "Write-Ahead Log" },
  { id: "storage", label: "STORAGE", name: "Memory-Mapped Files" },
];

export function NbArchSection() {
  const sectionRef = useRef<HTMLElement>(null);
  const lineRef = useRef<SVGPathElement>(null);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const el = sectionRef.current;
    if (!el) return;

    if (reducedMotion) return;

    const cleanup = inView(
      el,
      () => {
        const cards = el.querySelectorAll<HTMLElement>(".nb-arch-stage .nb-card");
        if (cards.length) {
          animate(
            cards,
            { opacity: [0, 1], x: [-16, 0] },
            {
              duration: 0.35,
              delay: 0.1,
              ease: [0.05, 0.95, 0.3, 1],
            },
          );
        }
        if (lineRef.current) {
          animate(
            lineRef.current,
            { strokeDashoffset: [300, 0] },
            { duration: 0.8, ease: [0.05, 0.95, 0.3, 1], delay: 0.15 },
          );
        }
      },
      { amount: 0.3 },
    );

    return () => cleanup?.();
  }, [reducedMotion]);

  return (
    <section
      ref={sectionRef}
      className="nb-section nb-section--lg"
      aria-label="Architecture pipeline"
    >
      <div className="nb-inner">
        <div className="nb-section-header nb-section-header--bordered">
          <h2 className="nb-arch-title">
            No network.
            <br />
            No latency.
          </h2>
        </div>

        <div className="nb-asymmetric">
          <div>
            <p className="nb-arch-body">
              Traditional vector databases require serialization, HTTP/gRPC transport, and context
              switching. VantaDB lives in the same memory space as your application. The PyO3 bridge
              provides native zero-copy access to the Rust core.
            </p>
          </div>

          <div>
            <div className="nb-arch-pipeline" role="list">
              {PIPELINE.map((stage, i) => (
                <div key={stage.id} className="nb-arch-stage" role="listitem">
                  <div className={`nb-card ${stage.accent ? "nb-card--amber" : ""}`}>
                    <span className="nb-arch-stage-name">{stage.name}</span>
                  </div>
                  {i < PIPELINE.length - 1 && (
                    <div className="nb-arch-connector" aria-hidden="true">
                      <span className="nb-arch-connector-arrow">&gt;&gt;&gt;</span>
                    </div>
                  )}
                </div>
              ))}
            </div>
            <svg className="nb-pipeline-line" width="100%" height="100%" aria-hidden="true">
              <path
                ref={lineRef}
                d="M0 0 L0 100%"
                stroke="var(--amber)"
                strokeWidth="1"
                fill="none"
                strokeDasharray="300"
                strokeDashoffset="300"
              />
            </svg>
          </div>
        </div>
      </div>
    </section>
  );
}
