import { useRef } from "react";
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

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const layers = gsap.utils.toArray(".swiss-arch-layer");

        gsap.fromTo(
          layers,
          { y: (i) => i * -40, opacity: 0.8 },
          {
            y: 0,
            opacity: 1,
            ease: "none",
            scrollTrigger: {
              trigger: sectionRef.current,
              start: "top 60%",
              end: "bottom 80%",
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
    >
      <div className="swiss-inner">
        <div className="swiss-grid swiss-arch-layers-grid">
          {/* Texto Descriptivo - Columna 1-4 */}
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

          {/* Diagrama Interactivo - Columna 6-12 */}
          <div className="swiss-arch-layers-diagram" ref={layersRef}>
            <div className="swiss-arch-layers-stack">
              {LAYERS.map((layer, index) => (
                <div
                  key={layer.id}
                  className={`swiss-arch-layer swiss-arch-layer--${layer.size} ${layer.id === "pyo3" ? "swiss-arch-layer--accent" : ""}`}
                  onMouseEnter={(e) => {
                    const parent = e.currentTarget.closest(".swiss-arch-layers-stack");
                    parent?.classList.add("swiss-arch-stack--hovered");
                    e.currentTarget.classList.add("swiss-arch-layer--active");
                  }}
                  onMouseLeave={(e) => {
                    const parent = e.currentTarget.closest(".swiss-arch-layers-stack");
                    parent?.classList.remove("swiss-arch-stack--hovered");
                    e.currentTarget.classList.remove("swiss-arch-layer--active");
                  }}
                >
                  <div className="swiss-arch-layer-badge">
                    <span
                      className={`swiss-arch-layer-badge-text ${layer.id === "pyo3" ? "swiss-arch-layer-badge-text--accent" : ""}`}
                    >
                      {layer.type}
                    </span>
                  </div>

                  <span className="swiss-arch-layer-name">
                    {layer.name}
                  </span>

                  {/* Flechas connectores entre capas */}
                  {index < LAYERS.length - 1 && (
                    <svg
                      width="20"
                      height="40"
                      className="swiss-arch-layer-arrow"
                    >
                      <line
                        x1="10"
                        y1="0"
                        x2="10"
                        y2="40"
                      />
                      <polygon points="5,35 15,35 10,40" />
                    </svg>
                  )}

                  {/* Dimensiones aproximadas */}
                  <div className="swiss-arch-layer-dim">
                    <div className="swiss-arch-layer-dim-line"></div>
                    <span className="swiss-arch-layer-dim-text">
                      {layer.size === "large" ? "1.2ms" : layer.size === "medium" ? "0.0ms" : "2MB"}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
