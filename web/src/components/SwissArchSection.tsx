import { useState } from "react";

const LAYERS = [
  { id: "app", name: "PYTHON APP", type: "CLIENT LAYER", size: "small" },
  { id: "pyo3", name: "PYO3 BRIDGE", type: "FFI BOUNDARY", size: "medium" },
  { id: "rust", name: "RUST CORE", type: "ENGINE LAYER", size: "large" },
  { id: "storage", name: "HNSW + WAL", type: "STORAGE LAYER", size: "large" },
];

export function SwissArchSection() {
  const [hoveredLayer, setHoveredLayer] = useState<string | null>(null);

  return (
    <section className="swiss-section swiss-arch-layers" aria-label="Architecture layers">
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

          <div className="swiss-arch-layers-diagram" role="img" aria-label="Architecture stack diagram">
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
