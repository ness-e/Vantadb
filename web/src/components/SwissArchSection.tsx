import { useRef, useState } from "react";
import { gsap, useGSAP, ScrollTrigger } from "../lib/gsap";

const LAYERS = [
  {
    id: "app",
    name: "PYTHON APP",
    type: "CLIENT LAYER",
    color: "var(--foreground)",
    size: "small",
  },
  { id: "pyo3", name: "PYO3 BRIDGE", type: "FFI BOUNDARY", color: "var(--amber)", size: "medium" },
  {
    id: "rust",
    name: "RUST CORE",
    type: "ENGINE LAYER",
    color: "var(--foreground)",
    size: "large",
  },
  {
    id: "storage",
    name: "HNSW + WAL",
    type: "STORAGE LAYER",
    color: "var(--steel)",
    size: "large",
  },
];

export function SwissArchSection() {
  const sectionRef = useRef<HTMLElement>(null);
  const layersRef = useRef<HTMLDivElement>(null);
  const [hoveredLayer, setHoveredLayer] = useState<string | null>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const layers = gsap.utils.toArray<HTMLElement>(".swiss-arch-layer");
        const labels = gsap.utils.toArray<HTMLElement>(".swiss-arch-layer-name");

        gsap.fromTo(
          layers,
          { y: (i) => -(i + 1) * 60, opacity: 0.6 },
          {
            y: 0,
            opacity: 1,
            ease: "none",
            stagger: 0.15,
            scrollTrigger: {
              trigger: sectionRef.current,
              start: "top 60%",
              end: "bottom 80%",
              scrub: 1,
            },
          },
        );

        gsap.fromTo(
          labels,
          { clipPath: "inset(0 100% 0 0)", opacity: 0 },
          {
            clipPath: "inset(0)",
            opacity: 1,
            duration: 0.35,
            stagger: 0.15,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            scrollTrigger: {
              trigger: sectionRef.current,
              start: "top 55%",
              end: "bottom 75%",
              scrub: 1,
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
      className="swiss-section swiss-arch-layers"
      aria-label="Architecture layers"
    >
      <div className="swiss-inner">
        <div className="swiss-grid swiss-arch-layers-grid">
          <div className="swiss-arch-layers-text">
            <h2 className="swiss-arch-layers-title">
              No network.
              <br />
              No latency.
            </h2>
            <p className="swiss-arch-layers-body">
              Traditional vector databases require serialization, HTTP/gRPC transport, and context
              switching.
              <br />
              <br />
              VantaDB lives in the same memory space as your application. The PyO3 bridge provides
              native zero-copy access to the Rust core.
            </p>
          </div>

          <div
            className="swiss-arch-layers-diagram"
            ref={layersRef}
            role="img"
            aria-label="Architecture stack diagram"
          >
            <div
              className={`swiss-arch-layers-stack${hoveredLayer ? " swiss-arch-stack--hovered" : ""}`}
            >
              {LAYERS.map((layer, index) => (
                <article
                  key={layer.id}
                  className={`swiss-arch-layer swiss-arch-layer--${layer.size} ${layer.id === "pyo3" ? "swiss-arch-layer--accent" : ""} ${hoveredLayer === layer.id ? "swiss-arch-layer--active" : ""}`}
                  onMouseEnter={() => setHoveredLayer(layer.id)}
                  onMouseLeave={() => setHoveredLayer(null)}
                  aria-label={`${layer.type}: ${layer.name}`}
                >
                  <header className="swiss-arch-layer-badge">
                    <span
                      className={`swiss-arch-layer-badge-text ${layer.id === "pyo3" ? "swiss-arch-layer-badge-text--accent" : ""}`}
                    >
                      {layer.type}
                    </span>
                  </header>

                  <span className="swiss-arch-layer-name">{layer.name}</span>

                  {index < LAYERS.length - 1 && (
                    <svg
                      width="20"
                      height="40"
                      className="swiss-arch-layer-arrow"
                      aria-hidden="true"
                    >
                      <line x1="10" y1="0" x2="10" y2="40" />
                      <polygon points="5,35 15,35 10,40" />
                    </svg>
                  )}

                  <div className="swiss-arch-layer-dim">
                    <div className="swiss-arch-layer-dim-line"></div>
                    <span className="swiss-arch-layer-dim-text">
                      {layer.size === "large" ? "1.2ms" : layer.size === "medium" ? "0.0ms" : "2MB"}
                    </span>
                  </div>
                </article>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
