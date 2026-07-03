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
      // Exploded view animation on scroll
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
    },
    { scope: sectionRef },
  );

  return (
    <section
      ref={sectionRef}
      className="swiss-section"
      style={{
        background: "var(--background)",
        paddingTop: "160px",
        paddingBottom: "160px",
      }}
    >
      <div className="swiss-inner">
        <div className="swiss-grid" style={{ gap: "24px" }}>
          {/* Texto Descriptivo - Columna 1-4 */}
          <div style={{ gridColumn: "1 / 5" }}>
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-display)",
                fontWeight: 700,
                margin: "0 0 48px 0",
                letterSpacing: "-0.04em",
                color: "var(--foreground)",
              }}
            >
              No network.
              <br />
              No latency.
            </h2>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "var(--text-body)",
                color: "var(--muted)",
                lineHeight: 1.6,
              }}
            >
              Traditional vector databases require serialization, HTTP/gRPC transport, and context
              switching.
              <br />
              <br />
              VantaDB lives in the same memory space as your application. The PyO3 bridge provides
              native zero-copy access to the Rust core.
            </p>
          </div>

          {/* Diagrama Interactivo - Columna 6-12 */}
          <div style={{ gridColumn: "6 / 13", position: "relative" }} ref={layersRef}>
            <div
              style={{ display: "flex", flexDirection: "column", gap: "24px", paddingTop: "40px" }}
            >
              {LAYERS.map((layer, index) => (
                <div
                  key={layer.id}
                  className="swiss-arch-layer"
                  style={{
                    position: "relative",
                    width: "100%",
                    height:
                      layer.size === "large" ? "140px" : layer.size === "medium" ? "100px" : "80px",
                    background: "rgba(249, 248, 246, 0.5)",
                    border: `1px solid ${layer.color}`,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    transition: "all 200ms cubic-bezier(0.25, 1, 0.5, 1)",
                    zIndex: 10 - index,
                  }}
                  onMouseEnter={(e) => {
                    const allLayers = document.querySelectorAll(".swiss-arch-layer");
                    allLayers.forEach((l) => {
                      if (l !== e.currentTarget) (l as HTMLElement).style.opacity = "0.3";
                    });
                    e.currentTarget.style.border = "1px solid var(--amber)";
                    e.currentTarget.style.background = "var(--surface)";
                  }}
                  onMouseLeave={(e) => {
                    const allLayers = document.querySelectorAll(".swiss-arch-layer");
                    allLayers.forEach((l, i) => {
                      (l as HTMLElement).style.opacity = "1";
                      (l as HTMLElement).style.border = `1px solid ${LAYERS[i].color}`;
                      (l as HTMLElement).style.background = "rgba(249, 248, 246, 0.5)";
                    });
                  }}
                >
                  <div
                    style={{
                      position: "absolute",
                      top: "-10px",
                      left: "24px",
                      background: "var(--background)",
                      padding: "0 8px",
                    }}
                  >
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.75rem",
                        color: layer.color,
                        fontWeight: 600,
                        letterSpacing: "0.1em",
                      }}
                    >
                      {layer.type}
                    </span>
                  </div>

                  <span
                    style={{
                      fontFamily: "var(--font-display)",
                      fontSize: "2rem",
                      fontWeight: 700,
                      letterSpacing: "-0.02em",
                      color: "var(--foreground)",
                    }}
                  >
                    {layer.name}
                  </span>

                  {/* Flechas conectores entre capas */}
                  {index < LAYERS.length - 1 && (
                    <svg
                      width="20"
                      height="40"
                      style={{
                        position: "absolute",
                        bottom: "-32px",
                        left: "50%",
                        transform: "translateX(-50%)",
                        zIndex: 0,
                      }}
                    >
                      <line
                        x1="10"
                        y1="0"
                        x2="10"
                        y2="40"
                        stroke="var(--border)"
                        strokeWidth="1"
                        strokeDasharray="4 4"
                      />
                      <polygon points="5,35 15,35 10,40" fill="var(--border)" />
                    </svg>
                  )}

                  {/* Dimensiones aproximadas */}
                  <div
                    style={{
                      position: "absolute",
                      right: "-40px",
                      top: "50%",
                      transform: "translateY(-50%)",
                      display: "flex",
                      alignItems: "center",
                      gap: "8px",
                    }}
                  >
                    <div
                      style={{ width: "16px", height: "1px", background: "var(--border)" }}
                    ></div>
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: "0.7rem",
                        color: "var(--steel)",
                      }}
                    >
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
